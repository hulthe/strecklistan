use crate::generated::css_classes::C;
use crate::models::event::Event;
use crate::page::{
    accounting::{AccountingMsg, AccountingPage},
    deposit::{DepositionMsg, DepositionPage},
    store::{StoreMsg, StorePage},
    transactions::{TransactionsMsg, TransactionsPage},
    Page,
};
use chrono::NaiveDateTime;
use laggit_api::{
    book_account::{BookAccount, BookAccountId, MasterAccounts},
    inventory::{
        InventoryBundle, InventoryBundleId, InventoryItemId, InventoryItemStock as InventoryItem,
    },
    member::{Member, MemberId},
    transaction::Transaction,
};
use seed::browser::service::fetch::FetchObject;
use seed::prelude::*;
use seed::*;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;
use web_sys;

#[derive(Clone, Default)]
pub struct StateLoading {
    pub book_accounts: Option<HashMap<BookAccountId, Rc<BookAccount>>>,
    pub master_accounts: Option<MasterAccounts>,
    pub transaction_history: Option<Vec<Transaction>>,
    pub inventory: Option<HashMap<InventoryItemId, Rc<InventoryItem>>>,
    pub bundles: Option<HashMap<InventoryBundleId, Rc<InventoryBundle>>>,
    pub events: Option<BTreeMap<NaiveDateTime, Vec<Event>>>,
    pub members: Option<HashMap<MemberId, Rc<Member>>>,
}

#[derive(Clone)]
pub struct StateReady {
    pub events: BTreeMap<NaiveDateTime, Vec<Event>>,

    pub book_accounts: HashMap<BookAccountId, Rc<BookAccount>>,
    pub master_accounts: MasterAccounts,

    pub members: HashMap<MemberId, Rc<Member>>,

    pub transaction_history: Vec<Transaction>,

    pub inventory: HashMap<InventoryItemId, Rc<InventoryItem>>,
    pub bundles: HashMap<InventoryBundleId, Rc<InventoryBundle>>,
}

#[derive(Clone)]
pub enum State {
    Loading(StateLoading),
    Ready {
        accounting_page: AccountingPage,
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
}

impl Default for Model {
    fn default() -> Self {
        Self {
            page: Page::Root,
            state: State::Loading(Default::default()),
        }
    }
}

#[derive(Clone, Debug)]
pub enum FetchMsg {
    Events(FetchObject<Vec<Event>>),
    Inventory(FetchObject<Vec<InventoryItem>>),
    Bundles(FetchObject<Vec<InventoryBundle>>),
    Transactions(FetchObject<Vec<Transaction>>),
    BookAccounts(FetchObject<Vec<BookAccount>>),
    MasterAccounts(FetchObject<MasterAccounts>),
    Members(FetchObject<Vec<Member>>),
}

#[derive(Clone, Debug)]
pub enum Msg {
    ChangePage(Page),

    Fetched(FetchMsg),

    KeyPressed(web_sys::KeyboardEvent),

    AccountingMsg(AccountingMsg),
    DepositionMsg(DepositionMsg),
    TransactionsMsg(TransactionsMsg),
    StoreMsg(StoreMsg),

    ReloadData,
}

pub fn routes(url: Url) -> Option<Msg> {
    Some(if url.path.is_empty() {
        Msg::ChangePage(Page::Root)
    } else {
        match url.path[0].as_ref() {
            "accounting" => Msg::ChangePage(Page::Accounting),
            "deposit" => Msg::ChangePage(Page::Deposit),
            "" | "store" => Msg::ChangePage(Page::Store),
            "transactions" => Msg::ChangePage(Page::TransactionHistory),
            _ => Msg::ChangePage(Page::NotFound),
        }
    })
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

pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::ChangePage(page) => {
            model.page = page;
        }

        Msg::Fetched(msg) => {
            fn handle_fetch<T: Debug>(
                model: &mut Model,
                fetch: FetchObject<T>,
                errid: &str,
                handler: impl FnOnce(T, &mut StateLoading),
            ) {
                match fetch.response() {
                    Ok(response) => {
                        match &mut model.state {
                            State::Loading(data) => {
                                handler(response.data, data);
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
                                        };
                                        State::Ready {
                                            accounting_page: AccountingPage::new(&data),
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
                    Err(e) => {
                        error!(format!("Failed to fetch {}", errid), e);
                        model.state = State::LoadingFailed(
                            format!("Failed to fetch {}.", errid),
                            format!("{:#?}", e),
                        );
                    }
                }
            }

            use FetchMsg::*;
            match msg {
                Events(fetch) => handle_fetch(model, fetch, "event", |data, s| {
                    let mut events: BTreeMap<NaiveDateTime, Vec<Event>> = BTreeMap::new();
                    for event in data {
                        events.entry(event.start_time).or_default().push(event)
                    }
                    s.events = Some(events);
                }),
                Inventory(fetch) => handle_fetch(model, fetch, "inventory", |data, s| {
                    s.inventory = Some(vec_id_to_map!(data));
                }),
                Bundles(fetch) => handle_fetch(model, fetch, "bundles", |data, s| {
                    s.bundles = Some(vec_id_to_map!(data));
                }),
                Transactions(fetch) => handle_fetch(model, fetch, "transactions", |data, s| {
                    s.transaction_history = Some(data);
                }),
                BookAccounts(fetch) => handle_fetch(model, fetch, "book-accounts", |data, s| {
                    s.book_accounts = Some(vec_id_to_map!(data));
                }),
                MasterAccounts(fetch) => {
                    handle_fetch(model, fetch, "master book-accounts", |data, s| {
                        s.master_accounts = Some(data);
                    })
                }
                Members(fetch) => handle_fetch(model, fetch, "members", |data, s| {
                    s.members = Some(vec_id_to_map!(data));
                }),
            }
        }

        Msg::DepositionMsg(msg) => fwd_child_msg!(model.state, deposition_page, msg, orders),
        Msg::AccountingMsg(msg) => fwd_child_msg!(model.state, accounting_page, msg, orders),
        Msg::TransactionsMsg(msg) => fwd_child_msg!(model.state, transactions_page, msg, orders),
        Msg::StoreMsg(msg) => fwd_child_msg!(model.state, store_page, msg, orders),

        Msg::KeyPressed(ev) => {
            match ev.key().as_str() {
                //key => log!(key),
                _key => {}
            }
        }

        Msg::ReloadData => {
            model.state = State::Loading(Default::default());
            fetch_data(orders);
        }
    }
}

pub fn view(model: &Model) -> Vec<Node<Msg>> {
    vec![div![
        div![
            class![C.header],
            if cfg!(debug_assertions) {
                div![class![C.debug_banner], "DEBUG"]
            } else {
                empty![]
            },
            div![
                // links
                //a!["hem", class![C.header_link], attrs! {At::Href => "/"}],
                class![C.header_link_box],
                a![
                    "försäljning",
                    class![C.header_link],
                    attrs! {At::Href => "/store"}
                ],
                a![
                    "tillgodo",
                    class![C.header_link],
                    attrs! {At::Href => "/deposit"}
                ],
                a![
                    "transaktioner",
                    class![C.header_link],
                    attrs! {At::Href => "/transactions"}
                ],
                a![
                    "bokföring",
                    class![C.header_link],
                    attrs! {At::Href => "/accounting"}
                ],
            ],
        ],
        if cfg!(debug_assertions) {
            div![raw!["&nbsp;"]]
        } else {
            empty![]
        },
        div![class![C.header_margin], raw!["&nbsp;"]],
        match &model.state {
            State::Ready {
                accounting_page,
                deposition_page,
                transactions_page,
                store_page,
                state,
            } => match model.page {
                Page::Accounting => accounting_page.view(state),
                Page::Store => store_page.view(state),
                Page::Deposit => deposition_page.view(state),
                Page::TransactionHistory => transactions_page.view(state),
                Page::Root | Page::NotFound => {
                    div![class![C.not_found_message, C.unselectable], "404"]
                }
            },
            State::Loading(_) => div![class!["text-center"], div![class!["lds-heart"], div![]]],
            State::LoadingFailed(msg, error) => div![
                class![C.flex, C.flex_col],
                p!["An has error occured."],
                p![msg],
                textarea![
                    class!["code_box"],
                    attrs! { At::ReadOnly => true, },
                    attrs! { At::Rows => error.lines().count(), },
                    error,
                ],
            ],
        },
    ]]
}

pub fn fetch_data(orders: &mut impl Orders<Msg>) {
    orders.perform_cmd(
        Request::new("/api/book_accounts")
            .fetch_json(|data| Msg::Fetched(FetchMsg::BookAccounts(data))),
    );
    orders.perform_cmd(
        Request::new("/api/book_accounts/masters")
            .fetch_json(|data| Msg::Fetched(FetchMsg::MasterAccounts(data))),
    );
    orders.perform_cmd(
        Request::new("/api/members").fetch_json(|data| Msg::Fetched(FetchMsg::Members(data))),
    );
    orders.perform_cmd(fetch_events(-1, 2));
    orders.perform_cmd(
        Request::new("/api/inventory/items")
            .fetch_json(|data| Msg::Fetched(FetchMsg::Inventory(data))),
    );
    orders.perform_cmd(
        Request::new("/api/inventory/bundles")
            .fetch_json(|data| Msg::Fetched(FetchMsg::Bundles(data))),
    );
    orders.perform_cmd(
        Request::new("/api/transactions")
            .fetch_json(|data| Msg::Fetched(FetchMsg::Transactions(data))),
    );
}

async fn fetch_events(low: i64, high: i64) -> Result<Msg, Msg> {
    let url = format!("/api/events?low={}&high={}", low, high);
    Request::new(url).fetch_json(|data| Msg::Fetched(FetchMsg::Events(data))).await
}

pub fn window_events(_model: &Model) -> Vec<Listener<Msg>> {
    vec![keyboard_ev("keydown", |ev| Msg::KeyPressed(ev))]
}
