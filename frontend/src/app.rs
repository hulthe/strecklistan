use crate::generated::css_classes::C;
use crate::models::event::Event;
use crate::notification_manager::{NotificationManager, NotificationMessage};
use crate::page::{
    analytics::{AnalyticsMsg, AnalyticsPage},
    deposit::{DepositionMsg, DepositionPage},
    store::{StoreMsg, StorePage},
    transactions::{TransactionsMsg, TransactionsPage},
    Page,
};
use crate::util::compare_semver;
use chrono::{DateTime, FixedOffset, Local, Utc};
use seed::prelude::*;
use seed::*;
use semver::Version;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::rc::Rc;
use strecklistan_api::{
    book_account::{BookAccount, BookAccountId, MasterAccounts},
    inventory::{
        InventoryBundle, InventoryBundleId, InventoryItemId, InventoryItemStock as InventoryItem,
    },
    member::{Member, MemberId},
    transaction::Transaction,
};

const PKG_VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Default)]
pub struct StateLoading {
    pub book_accounts: Option<HashMap<BookAccountId, Rc<BookAccount>>>,
    pub master_accounts: Option<MasterAccounts>,
    pub transaction_history: Option<Vec<Transaction>>,
    pub inventory: Option<HashMap<InventoryItemId, Rc<InventoryItem>>>,
    pub bundles: Option<HashMap<InventoryBundleId, Rc<InventoryBundle>>>,
    pub events: Option<BTreeMap<DateTime<Utc>, Vec<Event>>>,
    pub members: Option<HashMap<MemberId, Rc<Member>>>,
}

#[derive(Clone)]
pub struct StateReady {
    pub events: BTreeMap<DateTime<Utc>, Vec<Event>>,

    pub book_accounts: HashMap<BookAccountId, Rc<BookAccount>>,
    pub master_accounts: MasterAccounts,

    pub members: HashMap<MemberId, Rc<Member>>,

    pub transaction_history: Vec<Transaction>,

    pub inventory: HashMap<InventoryItemId, Rc<InventoryItem>>,
    pub bundles: HashMap<InventoryBundleId, Rc<InventoryBundle>>,

    pub timezone: FixedOffset,

    pub request_in_progress: bool,
}

#[derive(Clone)]
pub enum State {
    Loading(StateLoading),
    Ready {
        analytics_page: AnalyticsPage,
        deposition_page: DepositionPage,
        transactions_page: TransactionsPage,
        store_page: StorePage,
        state: StateReady,
    },
    LoadingFailed(String, String),
}

pub struct Model {
    pub page: Page,
    pub state: State,
    pub notifications: NotificationManager,
}

#[derive(Clone, Debug)]
pub enum FetchMsg {
    ApiVersion(String),
    Events(Vec<Event>),
    Inventory(Vec<InventoryItem>),
    Bundles(Vec<InventoryBundle>),
    Transactions(Vec<Transaction>),
    BookAccounts(Vec<BookAccount>),
    MasterAccounts(MasterAccounts),
    Members(Vec<Member>),
}

#[derive(Clone, Debug)]
pub enum Msg {
    ChangePage(Page),

    Fetched(FetchMsg),

    ShowError { header: String, dump: String },

    AnalyticsMsg(AnalyticsMsg),
    DepositionMsg(DepositionMsg),
    TransactionsMsg(TransactionsMsg),
    StoreMsg(StoreMsg),

    NotificationMessage(NotificationMessage),

    ReloadData,
}

// Vec<Item> -> HashMap<Item::id, Rc<Item>>
macro_rules! vec_id_to_map {
    ($vec:expr) => {
        $vec.into_iter()
            .map(|elem| (elem.id, Rc::new(elem)))
            .collect()
    };
}

macro_rules! fwd_child_msg {
    ($state:expr, $page:ident, $msg:expr, $orders:expr) => {
        if let State::Ready { state, $page, .. } = &mut $state {
            $page.update($msg, state, $orders);
        }
    };
}

pub fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.send_msg(Msg::ReloadData);

    orders
        .subscribe(|subs::UrlChanged(mut url)| {
            let page = match url.remaining_path_parts().as_slice() {
                [] | [""] | ["store"] => (Page::Store),
                ["transactions"] => (Page::TransactionHistory),
                ["analytics"] => (Page::Analytics),
                ["deposit"] => (Page::Deposit),
                _ => (Page::NotFound),
            };

            Msg::ChangePage(page)
        })
        .notify(subs::UrlChanged(url.clone()));

    Model {
        page: Page::Root,
        state: State::Loading(Default::default()),
        notifications: Default::default(),
    }
}

pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::ChangePage(page) => {
            model.page = page;
        }

        Msg::ShowError { header, dump } => {
            model.state = State::LoadingFailed(header, dump);
        }

        Msg::Fetched(msg) => {
            fn prepare_state<T: Debug>(
                model: &mut Model,
                fetched: T,
                handler: impl FnOnce(T, &mut StateLoading),
            ) {
                match &mut model.state {
                    State::Loading(data) => {
                        handler(fetched, data);
                        model.state = match data {
                            StateLoading {
                                book_accounts: Some(book_accounts),
                                master_accounts: Some(master_accounts),
                                transaction_history: Some(transaction_history),
                                inventory: Some(inventory),
                                bundles: Some(bundles),
                                events: Some(events),
                                members: Some(members),
                            } => {
                                let data = StateReady {
                                    book_accounts: book_accounts.clone(),
                                    master_accounts: master_accounts.clone(),
                                    transaction_history: transaction_history.clone(),
                                    inventory: inventory.clone(),
                                    bundles: bundles.clone(),
                                    events: events.clone(),
                                    members: members.clone(),
                                    timezone: *Local::now().offset(),
                                    request_in_progress: false,
                                };
                                State::Ready {
                                    analytics_page: AnalyticsPage::new(&data),
                                    deposition_page: DepositionPage::new(&data),
                                    transactions_page: TransactionsPage::new(&data),
                                    store_page: StorePage::new(&data),
                                    state: data,
                                }
                            }
                            // TODO: Remove clone
                            still_loading => State::Loading(still_loading.clone()),
                        };
                    }
                    State::LoadingFailed(_, _) => {}
                    State::Ready { .. } => {
                        error!("Tried to load an aspect of the page while already loaded");
                    }
                }
            }

            use FetchMsg::*;
            match msg {
                ApiVersion(response) => {
                    if let Ok(api_version) = Version::parse(&response) {
                        let frontend_version = Version::parse(PKG_VERSION).unwrap();

                        log!("API version:", response);
                        log!("Application version:", PKG_VERSION);

                        if !compare_semver(frontend_version, api_version) {
                            model.state = State::LoadingFailed(
                                "Mismatching api version.".to_string(),
                                format!(
                                    "API version: {}\nApplication version: {}",
                                    response, PKG_VERSION
                                ),
                            );
                        }
                    } else {
                        model.state = State::LoadingFailed(
                            "Failed to parse server api version.".to_string(),
                            response,
                        );
                    }
                }
                Events(fetch) => prepare_state(model, fetch, |data, s| {
                    let mut events: BTreeMap<DateTime<Utc>, Vec<Event>> = BTreeMap::new();
                    for event in data {
                        events.entry(event.start_time).or_default().push(event)
                    }
                    s.events = Some(events);
                }),
                Inventory(fetch) => prepare_state(model, fetch, |data, s| {
                    s.inventory = Some(vec_id_to_map!(data));
                }),
                Bundles(fetch) => prepare_state(model, fetch, |data, s| {
                    s.bundles = Some(vec_id_to_map!(data));
                }),
                Transactions(fetch) => prepare_state(model, fetch, |data, s| {
                    s.transaction_history = Some(data);
                }),
                BookAccounts(fetch) => prepare_state(model, fetch, |data, s| {
                    s.book_accounts = Some(vec_id_to_map!(data));
                }),
                MasterAccounts(fetch) => prepare_state(model, fetch, |data, s| {
                    s.master_accounts = Some(data);
                }),
                Members(fetch) => prepare_state(model, fetch, |data, s| {
                    s.members = Some(vec_id_to_map!(data));
                }),
            }
        }

        Msg::DepositionMsg(msg) => fwd_child_msg!(model.state, deposition_page, msg, orders),
        Msg::AnalyticsMsg(msg) => fwd_child_msg!(model.state, analytics_page, msg, orders),
        Msg::TransactionsMsg(msg) => fwd_child_msg!(model.state, transactions_page, msg, orders),
        Msg::StoreMsg(msg) => fwd_child_msg!(model.state, store_page, msg, orders),

        Msg::NotificationMessage(msg) => model.notifications.update(msg, orders),

        Msg::ReloadData => {
            model.state = State::Loading(Default::default());
            fetch_data(orders);
        }
    }
}

pub fn view(model: &Model) -> Vec<Node<Msg>> {
    vec![
        model.notifications.view(),
        div![
            div![
                C![C.header],
                if cfg!(debug_assertions) {
                    div![C![C.debug_banner], "DEBUG"]
                } else {
                    empty![]
                },
                div![
                    // links
                    //a!["hem", C![C.header_link], attrs! {At::Href => "/"}],
                    C![C.header_link_box],
                    a![
                        "försäljning",
                        C![C.header_link],
                        attrs! {At::Href => "/store"}
                    ],
                    a![
                        "tillgodo",
                        C![C.header_link],
                        attrs! {At::Href => "/deposit"}
                    ],
                    a![
                        "transaktioner",
                        C![C.header_link],
                        attrs! {At::Href => "/transactions"}
                    ],
                    a![
                        "analys",
                        C![C.header_link],
                        attrs! {At::Href => "/analytics"}
                    ],
                ],
            ],
            if cfg!(debug_assertions) {
                div![raw!["&nbsp;"]]
            } else {
                empty![]
            },
            div![C![C.header_margin], raw!["&nbsp;"]],
            match &model.state {
                State::Ready {
                    analytics_page,
                    deposition_page,
                    transactions_page,
                    store_page,
                    state,
                } => match model.page {
                    Page::Analytics => analytics_page.view(state),
                    Page::Store => store_page.view(state),
                    Page::Deposit => deposition_page.view(state),
                    Page::TransactionHistory => transactions_page.view(state),
                    Page::Root | Page::NotFound => {
                        div![C![C.not_found_message, C.unselectable], "404"]
                    }
                },

                State::Loading(_) => div![C![C.text_center, C.mt_2], div![C![C.lds_heart], div![]]],
                State::LoadingFailed(msg, error) => div![
                    C![C.flex, C.flex_col],
                    p!["An has error occured."],
                    p![msg],
                    textarea![
                        C![C.code_box],
                        attrs! { At::ReadOnly => true, },
                        attrs! { At::Rows => error.lines().count(), },
                        error,
                    ],
                ],
            },
        ],
    ]
}

fn handle_fetch(
    orders: &mut impl Orders<Msg>,
    label: &'static str,
    request: impl Future<Output = Result<Msg, FetchError>> + 'static,
) {
    orders.perform_cmd(async move {
        match request.await {
            Ok(response) => response,
            Err(fetch_error) => Msg::ShowError {
                header: format!("Failed to fetch network resource: \"{}\"", label),
                dump: format!("{:?}", fetch_error),
            },
        }
    });
}

pub fn fetch_data(orders: &mut impl Orders<Msg>) {
    handle_fetch(orders, "api_version", async {
        let data = fetch("/api/version").await?.text().await?;
        Ok(Msg::Fetched(FetchMsg::ApiVersion(data)))
    });
    handle_fetch(orders, "book_accounts", async {
        let data = fetch("/api/book_accounts").await?.json().await?;
        Ok(Msg::Fetched(FetchMsg::BookAccounts(data)))
    });
    handle_fetch(orders, "book_account_masters", async {
        let data = fetch("/api/book_accounts/masters").await?.json().await?;
        Ok(Msg::Fetched(FetchMsg::MasterAccounts(data)))
    });
    handle_fetch(orders, "members", async {
        let data = fetch("/api/members").await?.json().await?;
        Ok(Msg::Fetched(FetchMsg::Members(data)))
    });
    fetch_events(orders, -1, 2);
    handle_fetch(orders, "inventory_items", async {
        let data = fetch("/api/inventory/items").await?.json().await?;
        Ok(Msg::Fetched(FetchMsg::Inventory(data)))
    });
    handle_fetch(orders, "inventory_bundles", async {
        let data = fetch("/api/inventory/bundles").await?.json().await?;
        Ok(Msg::Fetched(FetchMsg::Bundles(data)))
    });
    handle_fetch(orders, "transactions", async {
        let data = fetch("/api/transactions").await?.json().await?;
        Ok(Msg::Fetched(FetchMsg::Transactions(data)))
    });
}

fn fetch_events(orders: &mut impl Orders<Msg>, low: i64, high: i64) {
    let url = format!("/api/events?low={}&high={}", low, high);
    handle_fetch(orders, "events", async {
        let data = fetch(url).await?.json().await?;
        Ok(Msg::Fetched(FetchMsg::Events(data)))
    });
}
