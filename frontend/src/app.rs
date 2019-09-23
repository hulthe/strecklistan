use crate::fuzzy_search::FuzzySearch;
use crate::generated::css_classes::C;
use crate::models::{event::Event, user::Credentials};
use crate::page::{
    accounting::{AccountingMsg, AccountingPage},
    store::{StoreMsg, StorePage},
    transactions::{TransactionsMsg, TransactionsPage},
    Page,
};
use chrono::NaiveDateTime;
use futures::Future;
use laggit_api::{
    book_account::{BookAccount, MasterAccounts},
    currency::Currency,
    inventory::{InventoryBundle, InventoryItemStock as InventoryItem},
    member::Member,
    transaction::{NewTransaction, Transaction},
};
use seed::prelude::*;
use seed::{fetch::FetchObject, *};
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::rc::Rc;
use web_sys;

#[derive(Clone, Default)]
pub struct StateLoading {
    pub book_accounts: Option<HashMap<i32, Rc<BookAccount>>>,
    pub master_accounts: Option<MasterAccounts>,
    pub transaction_history: Option<Vec<Transaction>>,
    pub inventory: Option<HashMap<i32, Rc<InventoryItem>>>,
    pub bundles: Option<HashMap<i32, Rc<InventoryBundle>>>,
    pub events: Option<BTreeMap<NaiveDateTime, Vec<Event>>>,
    pub members: Option<HashMap<i32, Rc<Member>>>,
}

#[derive(Clone)]
pub struct StateReady {
    pub events: BTreeMap<NaiveDateTime, Vec<Event>>,

    pub book_accounts: HashMap<i32, Rc<BookAccount>>,
    pub master_accounts: MasterAccounts,

    pub members: HashMap<i32, Rc<Member>>,

    pub transaction_history: Vec<Transaction>,

    pub inventory: HashMap<i32, Rc<InventoryItem>>,
    pub bundles: HashMap<i32, Rc<InventoryBundle>>,

    pub deposition_credit_account: Option<i32>,
    pub deposition_use_cash: bool,
    pub deposition_search_string: String,
    pub deposition_search: Vec<(i32, Vec<(usize, usize)>, Rc<BookAccount>, Rc<Member>)>,
    pub deposition_amount: Currency,
}

#[derive(Clone)]
pub enum State {
    Loading(StateLoading),
    Ready {
        accounting_page: AccountingPage,
        transactions_page: TransactionsPage,
        store_page: StorePage,
        state: StateReady,
    },
}

pub struct Model {
    pub page: Page,
    pub show_login_box: bool,
    pub token: Option<String>,
    pub credentials: Credentials,
    pub state: State,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            page: Page::Root,
            show_login_box: false,
            token: None,
            credentials: Credentials {
                name: String::new(),
                pass: String::new(),
            },
            state: State::Loading(Default::default()),
        }
    }
}

#[derive(Clone)]
pub enum FetchMsg {
    Events(FetchObject<Vec<Event>>),
    Inventory(FetchObject<Vec<InventoryItem>>),
    Bundles(FetchObject<Vec<InventoryBundle>>),
    Transactions(FetchObject<Vec<Transaction>>),
    BookAccounts(FetchObject<Vec<BookAccount>>),
    MasterAccounts(FetchObject<MasterAccounts>),
    Members(FetchObject<Vec<Member>>),
}

#[derive(Clone)]
pub enum Msg {
    ChangePage(Page),

    Fetched(FetchMsg),

    KeyPressed(web_sys::KeyboardEvent),

    DepositSearchDebit(String),
    DepositCreditKeyDown(web_sys::KeyboardEvent),
    DepositCreditSelect(i32),
    DepositUseCash(bool),
    DepositSetAmount(String),
    Deposit,
    DepositSent(FetchObject<i32>),

    AccountingMsg(AccountingMsg),
    TransactionsMsg(TransactionsMsg),
    StoreMsg(StoreMsg),

    ReloadData,
}

pub fn routes(url: Url) -> Msg {
    if url.path.is_empty() {
        Msg::ChangePage(Page::Root)
    } else {
        match url.path[0].as_ref() {
            "accounting" => Msg::ChangePage(Page::Accounting),
            "deposit" => Msg::ChangePage(Page::Deposit),
            "store" => Msg::ChangePage(Page::Store),
            "transactions" => Msg::ChangePage(Page::TransactionHistory),
            _ => Msg::ChangePage(Page::NotFound),
        }
    }
}

fn sort_tillgodolista_search(
    search: &str,
    list: &mut Vec<(i32, Vec<(usize, usize)>, Rc<BookAccount>, Rc<Member>)>,
) {
    for (score, matches, acc, _) in list.iter_mut() {
        let (s, m) = acc.compare_fuzzy(search);
        *score = s;
        *matches = m;
    }
    list.sort_by(|(scr_a, _, acc_a, _), (scr_b, _, acc_b, _)| {
        scr_b.cmp(scr_a).then(acc_a.id.cmp(&acc_b.id))
    });
}

// Vec<Item> -> HashMap<Item::id, Rc<Item>>
macro_rules! vec_id_to_map {
    ($vec:expr) => {
        $vec.into_iter()
            .map(|elem| (elem.id, Rc::new(elem)))
            .collect()
    };
}

pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::ChangePage(page) => {
            model.page = page;
        }

        Msg::Fetched(msg) => {
            if let State::Loading(data) = &mut model.state {
                match msg {
                    FetchMsg::Events(fetch_object) => match fetch_object.response() {
                        Ok(response) => {
                            let mut events: BTreeMap<NaiveDateTime, Vec<Event>> = BTreeMap::new();
                            for event in response.data {
                                events.entry(event.start_time).or_default().push(event)
                            }
                            data.events = Some(events);
                        }
                        Err(e) => {
                            error!("Failed to fetch events", e);
                        }
                    },
                    FetchMsg::Inventory(fetch_object) => match fetch_object.response() {
                        Ok(response) => {
                            data.inventory = Some(vec_id_to_map!(response.data));
                        }
                        Err(e) => {
                            error!("Failed to fetch inventory", e);
                        }
                    },
                    FetchMsg::Bundles(fetch_object) => match fetch_object.response() {
                        Ok(response) => {
                            data.bundles = Some(vec_id_to_map!(response.data));
                        }
                        Err(e) => {
                            error!("Failed to fetch transaction history", e);
                        }
                    },
                    FetchMsg::Transactions(fetch_object) => match fetch_object.response() {
                        Ok(response) => {
                            data.transaction_history = Some(response.data);
                        }
                        Err(e) => {
                            error!("Failed to fetch transaction history", e);
                        }
                    },
                    FetchMsg::BookAccounts(fetch_object) => match fetch_object.response() {
                        Ok(response) => {
                            data.book_accounts = Some(vec_id_to_map!(response.data));
                        }
                        Err(e) => {
                            error!("Failed to fetch book-accounts", e);
                        }
                    },
                    FetchMsg::MasterAccounts(fetch_object) => match fetch_object.response() {
                        Ok(response) => {
                            data.master_accounts = Some(response.data);
                        }
                        Err(e) => {
                            error!("Failed to fetch master book-accounts", e);
                        }
                    },
                    FetchMsg::Members(fetch_object) => match fetch_object.response() {
                        Ok(response) => {
                            data.members = Some(vec_id_to_map!(response.data));
                        }
                        Err(e) => {
                            error!("Failed to fetch master book-accounts", e);
                        }
                    },
                }

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
                        let accounts_search_list: Vec<_> = book_accounts
                            .values()
                            .filter_map(|acc| acc.creditor.map(|id| (acc, id)))
                            .filter_map(|(acc, id)| {
                                members.get(&id).cloned().map(|creditor| (acc, creditor))
                            })
                            .map(|(acc, creditor)| (0, vec![], acc.clone(), creditor))
                            .collect();
                        let data = StateReady {
                            book_accounts: book_accounts.clone(),
                            master_accounts: master_accounts.clone(),
                            transaction_history: transaction_history.clone(),
                            inventory: inventory.clone(),
                            bundles: bundles.clone(),
                            events: events.clone(),
                            members: members.clone(),
                            deposition_credit_account: None,
                            deposition_use_cash: false,
                            deposition_search: accounts_search_list.clone(),
                            deposition_search_string: String::new(),
                            deposition_amount: 0.into(),
                        };
                        State::Ready {
                            accounting_page: AccountingPage::new(&data),
                            transactions_page: TransactionsPage::new(&data),
                            store_page: StorePage::new(&data),
                            state: data,
                        }
                    }
                    still_loading => State::Loading(still_loading.clone()),
                };
            } else {
                error!("Incorrect state for loading data.")
            }
        }

        Msg::AccountingMsg(msg) => {
            if let State::Ready {
                state,
                accounting_page,
                ..
            } = &mut model.state
            {
                accounting_page.update(msg, state, orders);
            }
        }

        Msg::TransactionsMsg(msg) => {
            if let State::Ready {
                state,
                transactions_page,
                ..
            } = &mut model.state
            {
                transactions_page.update(msg, state, orders);
            }
        }

        Msg::StoreMsg(msg) => {
            if let State::Ready {
                state, store_page, ..
            } = &mut model.state
            {
                store_page.update(msg, state, orders);
            }
        }

        Msg::KeyPressed(ev) => {
            match ev.key().as_str() {
                "Escape" => {
                    model.show_login_box = false;
                }
                //key => log!(key),
                _key => {}
            }
        }

        Msg::DepositSearchDebit(search) => {
            if let State::Ready { state, .. } = &mut model.state {
                sort_tillgodolista_search(&search, &mut state.deposition_search);
                state.deposition_search_string = search;
            }
        }
        Msg::DepositCreditKeyDown(ev) => match ev.key().as_str() {
            "Enter" => {
                if let State::Ready { state, .. } = &mut model.state {
                    if let Some((_, _, acc, _)) = state.deposition_search.first() {
                        update(Msg::DepositCreditSelect(acc.id), model, orders)
                    }
                }
            }
            _ => {}
        },
        Msg::DepositCreditSelect(acc_id) => {
            if let State::Ready { state, .. } = &mut model.state {
                state.deposition_search_string = String::new();
                state.deposition_credit_account = Some(acc_id);
            }
        }
        Msg::DepositUseCash(use_cash) => {
            if let State::Ready { state, .. } = &mut model.state {
                state.deposition_use_cash = use_cash;
            }
        }
        Msg::DepositSetAmount(amount) => {
            if let State::Ready { state, .. } = &mut model.state {
                state.deposition_amount = amount.parse().unwrap_or(0.into());
            }
        }
        Msg::Deposit => {
            if let State::Ready { state, .. } = &mut model.state {
                if let Some(credit_acc) = state.deposition_credit_account {
                    let transaction = NewTransaction {
                        description: Some("Insättning".into()),
                        amount: state.deposition_amount,
                        credited_account: credit_acc,
                        debited_account: if state.deposition_use_cash {
                            state.master_accounts.cash_account_id
                        } else {
                            state.master_accounts.bank_account_id
                        },
                        bundles: vec![],
                    };

                    orders.perform_cmd(
                        Request::new("/api/transaction")
                            .method(Method::Post)
                            .send_json(&transaction)
                            .fetch_json(Msg::DepositSent),
                    );
                }
            }
        }
        Msg::DepositSent(fetch_object) => {
            if let State::Ready { state, .. } = &mut model.state {
                match fetch_object.response() {
                    Ok(response) => {
                        log!("ID: ", response.data);
                        state.deposition_amount = 0.into();
                        state.deposition_credit_account = None;
                        update(Msg::ReloadData, model, orders);
                    }
                    Err(e) => {
                        error!("Failed to post deposit", e);
                    }
                }
            }
        }

        Msg::ReloadData => {
            model.state = State::Loading(Default::default());
            fetch_data(orders);
            //orders.skip();
        }
    }
}

pub fn view(model: &Model) -> Vec<Node<Msg>> {
    use crate::page::deposit::deposition_page;
    vec![div![
        if cfg!(debug_assertions) {
            div![class!["debug_banner"], "DEBUG"]
        } else {
            empty![]
        },
        div![
            class![C.header],
            //a!["hem", class![C.header_link], attrs! {At::Href => "/"}],
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
        match &model.state {
            State::Ready {
                accounting_page,
                transactions_page,
                store_page,
                state,
            } => match model.page {
                Page::Accounting => accounting_page.view(state),
                Page::Store => store_page.view(state),
                Page::Deposit => deposition_page(state),
                Page::TransactionHistory => transactions_page.view(state),
                Page::Root | Page::NotFound => {
                    div![class![C.not_found_message, C.unselectable], "404"]
                }
            },
            State::Loading(_) => p!["Loading..."],
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

fn fetch_events(low: i64, high: i64) -> impl Future<Item = Msg, Error = Msg> {
    let url = format!("/api/events?low={}&high={}", low, high);
    Request::new(url).fetch_json(|data| Msg::Fetched(FetchMsg::Events(data)))
}

pub fn window_events(_model: &Model) -> Vec<events::Listener<Msg>> {
    vec![keyboard_ev("keydown", |ev| Msg::KeyPressed(ev))]
}
