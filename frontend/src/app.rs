use crate::fuzzy_search::FuzzySearch;
use crate::generated::css_classes::C;
use crate::models::{event::Event, http::StatusJson, user::Credentials};
use crate::page::Page;
use chrono::NaiveDateTime;
use futures::Future;
use laggit_api::{
    book_account::{BookAccount, MasterAccounts},
    currency::Currency,
    inventory::{InventoryBundle, InventoryItemStock as InventoryItem},
    member::Member,
    transaction::{NewTransaction, Transaction, TransactionBundle},
};
use seed::prelude::*;
use seed::{fetch::FetchObject, *};
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::rc::Rc;
use web_sys;

#[derive(Clone)]
pub enum StoreItem {
    Item(Rc<InventoryItem>),
    Bundle(Rc<InventoryBundle>),
}

impl FuzzySearch for StoreItem {
    fn get_search_string(&self) -> &str {
        match self {
            StoreItem::Item(item) => &item.name,
            StoreItem::Bundle(bundle) => &bundle.name,
        }
    }
}

impl FuzzySearch for BookAccount {
    fn get_search_string(&self) -> &str {
        &self.name
    }
}

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
    pub transaction: NewTransaction,

    pub store_search_string: String,
    pub override_transaction_total: bool,
    pub inventory_search: Vec<(i32, Vec<(usize, usize)>, StoreItem)>,

    pub inventory: HashMap<i32, Rc<InventoryItem>>,
    pub bundles: HashMap<i32, Rc<InventoryBundle>>,

    pub deposition_credit_account: Option<i32>,
    pub deposition_use_cash: bool,
    pub deposition_search_string: String,
    pub deposition_search: Vec<(i32, Vec<(usize, usize)>, Rc<BookAccount>, Rc<Member>)>,
    pub deposition_amount: Currency,

    pub tillgodolista_search_string: String,
    pub tillgodolista_search: Vec<(i32, Vec<(usize, usize)>, Rc<BookAccount>, Rc<Member>)>,
}

#[derive(Clone)]
pub enum State {
    Loading(StateLoading),
    Ready(StateReady),
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
pub enum L {}

#[derive(Clone)]
pub enum Msg {
    ChangePage(Page),

    Login,
    //UserDataFetched(fetch::FetchObject<Commit>),
    ToggleLoginBox,

    FetchEvents,

    Fetched(FetchMsg),

    KeyPressed(web_sys::KeyboardEvent),

    UsernameInput(String),
    PasswordInput(String),
    LoginResponse(FetchObject<StatusJson>),

    DepositSearchDebit(String),
    DepositCreditKeyDown(web_sys::KeyboardEvent),
    DepositCreditSelect(i32),
    DepositUseCash(bool),
    DepositSetAmount(String),
    Deposit,
    DepositSent(FetchObject<i32>),

    StoreSearchDebit(String),
    StoreDebitKeyDown(web_sys::KeyboardEvent),
    StoreDebitSelect(i32),

    StoreSearchInput(String),
    StoreSearchKeyDown(web_sys::KeyboardEvent),
    StoreConfirmPurchase,
    StorePurchaseSent(FetchObject<i32>),

    NewTransactionTotalInput(String),
    AddItemToNewTransaction(i32, i32),
    AddBundleToNewTransaction(i32, i32),
    SetNewTransactionBundleChange { bundle_index: usize, change: i32 },

    DeleteTransaction(i32),
    TransactionDeleted(FetchObject<i32>),

    ReloadData,
}

pub fn routes(url: Url) -> Msg {
    if url.path.is_empty() {
        Msg::ChangePage(Page::Root)
    } else {
        match url.path[0].as_ref() {
            "store" => Msg::ChangePage(Page::Store),
            "deposit" => Msg::ChangePage(Page::Deposit),
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
/*
fn sort_tillgodolista_search(data: &mut StateReady) {
    let search = &data.tillgodolista_search_string;
    for (score, matches, acc, _) in data.tillgodolista_search.iter_mut() {
        let (s, m) = acc.compare_fuzzy(search);
        *score = s;
        *matches = m;
    }
    data.tillgodolista_search
        .sort_by(|(scr_a, _, acc_a, _), (scr_b, _, acc_b, _)| {
            scr_b.cmp(scr_a).then(acc_a.id.cmp(&acc_b.id))
        });
}
*/

fn rebuild_store_list(data: &mut StateReady) {
    let items = data
        .inventory
        .values()
        // Don't show items without a default price in the store view
        .filter(|item| item.price.is_some())
        .map(|item| (0, vec![], StoreItem::Item(item.clone())));

    let bundles = data
        .bundles
        .values()
        .map(|bundle| (0, vec![], StoreItem::Bundle(bundle.clone())));

    data.inventory_search = bundles.chain(items).collect();

    sort_store_list(data);
}

fn sort_store_list(data: &mut StateReady) {
    for (score, matches, item) in data.inventory_search.iter_mut() {
        let (s, m) = item.compare_fuzzy(&data.store_search_string);
        *score = s;
        *matches = m;
    }
    data.inventory_search.sort_by(|(sa, _, ia), (sb, _, ib)| {
        sb.cmp(sa)
            .then(ia.get_search_string().cmp(&ib.get_search_string()))
    });
}

fn recompute_new_transaction_total(data: &mut StateReady) {
    if !data.override_transaction_total {
        data.transaction.amount = data
            .transaction
            .bundles
            .iter()
            .map(|bundle| -bundle.change * bundle.price.map(|p| p.into()).unwrap_or(0i32))
            .sum::<i32>()
            .into();
    }
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

        Msg::Login => {
            orders
                .skip()
                .perform_cmd(send_credentials(&model.credentials));
        }
        //Msg::UserDataFetched(fetch_object) => {/*TODO*/}
        Msg::ToggleLoginBox => model.show_login_box = !model.show_login_box,

        Msg::FetchEvents => {
            orders.skip().perform_cmd(fetch_events(-1, 2));
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
                        let mut data = StateReady {
                            book_accounts: book_accounts.clone(),
                            master_accounts: master_accounts.clone(),
                            transaction_history: transaction_history.clone(),
                            inventory: inventory.clone(),
                            bundles: bundles.clone(),
                            events: events.clone(),
                            members: members.clone(),
                            inventory_search: vec![],
                            override_transaction_total: false,
                            store_search_string: String::new(),
                            transaction: NewTransaction {
                                description: Some("Försäljning".to_string()),
                                bundles: vec![],
                                amount: 0.into(),
                                debited_account: master_accounts.bank_account_id,
                                credited_account: master_accounts.sales_account_id,
                            },
                            deposition_credit_account: None,
                            deposition_use_cash: false,
                            deposition_search: accounts_search_list.clone(),
                            deposition_search_string: String::new(),
                            deposition_amount: 0.into(),
                            tillgodolista_search: accounts_search_list,
                            tillgodolista_search_string: String::new(),
                        };
                        rebuild_store_list(&mut data);
                        State::Ready(data)
                    }
                    still_loading => State::Loading(still_loading.clone()),
                };
            } else {
                error!("Incorrect state for loading data.")
            }
        }

        Msg::KeyPressed(ev) => {
            match ev.key().as_str() {
                "Escape" => {
                    model.show_login_box = false;
                    if let State::Ready(data) = &mut model.state {
                        data.transaction.amount = 0.into();
                        data.transaction.bundles = vec![];
                        data.transaction.description = Some("Försäljning".into());
                    }
                }
                //key => log!(key),
                _key => {}
            }
        }

        Msg::UsernameInput(input) => model.credentials.name = input,
        Msg::PasswordInput(input) => model.credentials.pass = input,
        Msg::LoginResponse(fetch_object) => match fetch_object.response() {
            Ok(response) => {
                model.token = response.raw.headers().get("Authorization").unwrap_or(None);
                model.show_login_box = false;
                log!("Got JWT", response.data, model.token);
                orders.perform_cmd(
                    Request::new("/api/inventory/items")
                        .fetch_json(|data| Msg::Fetched(FetchMsg::Inventory(data))),
                );
            }
            Err(e) => {
                error!("Login request failed", e);
            }
        },

        Msg::DepositSearchDebit(search) => {
            if let State::Ready(data) = &mut model.state {
                sort_tillgodolista_search(&search, &mut data.deposition_search);
                data.deposition_search_string = search;
            }
        }
        Msg::DepositCreditKeyDown(ev) => match ev.key().as_str() {
            "Enter" => {
                if let State::Ready(data) = &mut model.state {
                    // TODO: Apply debit account
                    if let Some((_, _, acc, _)) = data.deposition_search.first() {
                        update(Msg::DepositCreditSelect(acc.id), model, orders)
                    }
                }
            }
            _ => {}
        },
        Msg::DepositCreditSelect(acc_id) => {
            if let State::Ready(data) = &mut model.state {
                data.deposition_search_string = String::new();
                data.deposition_credit_account = Some(acc_id);
            }
        }
        Msg::DepositUseCash(use_cash) => {
            if let State::Ready(data) = &mut model.state {
                data.deposition_use_cash = use_cash;
            }
        }
        Msg::DepositSetAmount(amount) => {
            if let State::Ready(data) = &mut model.state {
                data.deposition_amount = amount.parse().unwrap_or(0.into());
            }
        }
        Msg::Deposit => {
            if let State::Ready(data) = &mut model.state {
                if let Some(credit_acc) = data.deposition_credit_account {
                    let transaction = NewTransaction {
                        description: Some("Insättning".into()),
                        amount: data.deposition_amount,
                        credited_account: credit_acc,
                        debited_account: if data.deposition_use_cash {
                            data.master_accounts.cash_account_id
                        } else {
                            data.master_accounts.bank_account_id
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
            if let State::Ready(data) = &mut model.state {
                match fetch_object.response() {
                    Ok(response) => {
                        log!("ID: ", response.data);
                        data.deposition_amount = 0.into();
                        data.deposition_credit_account = None;
                        update(Msg::ReloadData, model, orders);
                    }
                    Err(e) => {
                        error!("Failed to post deposit", e);
                    }
                }
            }
        }

        Msg::StoreSearchDebit(search) => {
            if let State::Ready(data) = &mut model.state {
                sort_tillgodolista_search(&search, &mut data.tillgodolista_search);
                data.tillgodolista_search_string = search;
            }
        }
        Msg::StoreDebitKeyDown(ev) => match ev.key().as_str() {
            "Enter" => {
                if let State::Ready(data) = &mut model.state {
                    // TODO: Apply debit account
                    if let Some((_, _, acc, _)) = data.tillgodolista_search.first() {
                        update(Msg::StoreDebitSelect(acc.id), model, orders)
                    }
                }
            }
            _ => {}
        },
        Msg::StoreDebitSelect(acc_id) => {
            if let State::Ready(data) = &mut model.state {
                data.tillgodolista_search_string = String::new();
                data.transaction.debited_account = acc_id;
            }
        }

        Msg::StoreSearchInput(input) => {
            if let State::Ready(data) = &mut model.state {
                data.store_search_string = input;
                sort_store_list(data);
            }
        }
        Msg::StoreSearchKeyDown(ev) => match ev.key().as_str() {
            "Enter" => {
                if let State::Ready(data) = &mut model.state {
                    match data.inventory_search.first() {
                        Some((_, _, StoreItem::Item(item))) => {
                            update(Msg::AddItemToNewTransaction(item.id, 1), model, orders);
                        }
                        Some((_, _, StoreItem::Bundle(bundle))) => {
                            update(Msg::AddBundleToNewTransaction(bundle.id, 1), model, orders);
                        }
                        None => {}
                    }
                }
            }
            _ => {}
        },
        Msg::StoreConfirmPurchase => {
            if let State::Ready(data) = &mut model.state {
                data.transaction.bundles.retain(|bundle| bundle.change != 0);
                orders.perform_cmd(
                    Request::new("/api/transaction")
                        .method(Method::Post)
                        .send_json(&data.transaction)
                        .fetch_json(Msg::StorePurchaseSent),
                );
                orders.skip();
            }
        }
        Msg::StorePurchaseSent(fetch_object) => {
            if let State::Ready(data) = &mut model.state {
                match fetch_object.response() {
                    Ok(response) => {
                        log!("ID: ", response.data);
                        data.transaction.amount = 0.into();
                        data.transaction.bundles = vec![];
                        data.transaction.description = Some("Försäljning".into());
                        update(Msg::ReloadData, model, orders);
                    }
                    Err(e) => {
                        error!("Failed to post purchase", e);
                    }
                }
            }
        }
        Msg::NewTransactionTotalInput(input) => {
            if let State::Ready(data) = &mut model.state {
                log!("Input", input);
                if input == "" {
                    data.override_transaction_total = false;
                    recompute_new_transaction_total(data);
                } else {
                    data.override_transaction_total = true;
                    data.transaction.amount = input.parse().unwrap_or(0.into());
                    log!(format!("{}:-", data.transaction.amount));
                }
            }
        }

        Msg::AddItemToNewTransaction(item_id, amount) => {
            if let State::Ready(data) = &mut model.state {
                let item = data
                    .inventory
                    .get(&item_id)
                    .unwrap_or_else(|| panic!("No inventory item with that id exists"))
                    .clone();

                let mut item_ids = HashMap::new();
                item_ids.insert(item.id, 1);

                let bundle = TransactionBundle {
                    description: None,
                    // TODO: Handle case where price is null
                    price: Some(item.price.unwrap_or(0).into()),
                    change: -amount,
                    item_ids,
                };

                if let Some(b) =
                    data.transaction.bundles.iter_mut().find(|b| {
                        b.item_ids == bundle.item_ids && b.description == bundle.description
                    })
                {
                    b.change -= amount;
                } else {
                    log!("Pushing bundle", bundle);
                    data.transaction.bundles.push(bundle);
                }

                recompute_new_transaction_total(data);
            }
        }
        Msg::AddBundleToNewTransaction(bundle_id, amount) => {
            if let State::Ready(data) = &mut model.state {
                let bundle = data
                    .bundles
                    .get(&bundle_id)
                    .unwrap_or_else(|| panic!("No inventory bundle with that id exists"))
                    .clone();

                let mut item_ids = HashMap::new();
                for &id in bundle.item_ids.iter() {
                    *item_ids.entry(id).or_default() += 1;
                }

                let bundle = TransactionBundle {
                    description: Some(bundle.name.clone()),
                    price: Some(bundle.price),
                    change: -amount,
                    item_ids,
                };

                if let Some(b) =
                    data.transaction.bundles.iter_mut().find(|b| {
                        b.item_ids == bundle.item_ids && b.description == bundle.description
                    })
                {
                    b.change -= amount;
                } else {
                    log!("Pushing bundle", bundle);
                    data.transaction.bundles.push(bundle);
                }

                recompute_new_transaction_total(data);
            }
        }

        Msg::SetNewTransactionBundleChange {
            bundle_index,
            change,
        } => {
            if let State::Ready(data) = &mut model.state {
                let bundle = &mut data.transaction.bundles[bundle_index];
                if !data.override_transaction_total {
                    let diff = bundle.change - change;
                    data.transaction.amount +=
                        (bundle.price.map(|p| p.into()).unwrap_or(0i32) * diff).into();
                }
                bundle.change = change;
            }
        }

        Msg::DeleteTransaction(id) => {
            orders.perform_cmd(
                Request::new(format!("/api/transaction/{}", id))
                    .method(Method::Delete)
                    .fetch_json(Msg::TransactionDeleted),
            );
        }
        Msg::TransactionDeleted(fetch_object) => match fetch_object.response() {
            Ok(response) => {
                log!(format!("Transaction {} deleted", response.data));
                update(Msg::ReloadData, model, orders);
            }
            Err(e) => {
                error!("Failed to delete transaction", e);
            }
        },

        Msg::ReloadData => {
            model.state = State::Loading(Default::default());
            fetch_data(orders);
            //orders.skip();
        }
    }
}

pub fn view(model: &Model) -> Vec<Node<Msg>> {
    vec![div![
        #[cfg(debug_assertions)]
        div![class!["debug_banner"],"DEBUG"],
        div![
            class![C.header],
            a!["home", class![C.header_link], attrs! {At::Href => "/"}],
            a![
                "store",
                class![C.header_link],
                attrs! {At::Href => "/store"}
            ],
            a![
                "tillgodo",
                class![C.header_link],
                attrs! {At::Href => "/deposit"}
            ],
            a![
                "transactions",
                class![C.header_link],
                attrs! {At::Href => "/transactions"}
            ],
        ],
        match &model.state {
            State::Ready(data) => model.page.view(data),
            State::Loading(_) => p!["Loading..."],
        },
    ]]
}

fn send_credentials(credentials: &Credentials) -> impl Future<Item = Msg, Error = Msg> {
    Request::new("/api/login")
        .method(Method::Post)
        //.mode(web_sys::RequestMode::Cors)
        .send_json(credentials)
        .fetch_json(Msg::LoginResponse)
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

pub fn fetch_transactions() -> impl Future<Item = Msg, Error = Msg> {
    Request::new("/api/transactions").fetch_json(|data| Msg::Fetched(FetchMsg::Transactions(data)))
}

//fn fetch_user_data() -> impl Future<Item = Msg, Error = Msg> {
//    let url = "/api/me";
//    Request::new(url.into()).fetch_json(Msg::UserDataFetched)
//}

pub fn window_events(_model: &Model) -> Vec<events::Listener<Msg>> {
    vec![keyboard_ev("keydown", |ev| Msg::KeyPressed(ev))]
}
