use crate::app::{Msg, StateReady};
use crate::fuzzy_search::FuzzySearch;
use crate::generated::css_classes::C;
use crate::views::{
    view_inventory_bundle, view_inventory_item, view_new_transaction, view_tillgodo,
};
use laggit_api::{
    book_account::BookAccount,
    inventory::InventoryBundle,
    inventory::InventoryItemStock as InventoryItem,
    member::Member,
    transaction::{NewTransaction, TransactionBundle},
};
use seed::prelude::*;
use seed::{fetch::FetchObject, *};
use std::collections::HashMap;
use std::rc::Rc;

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

#[derive(Clone)]
pub enum StoreMsg {
    SearchDebit(String),
    DebitKeyDown(web_sys::KeyboardEvent),
    DebitSelect(i32),

    SearchInput(String),
    SearchKeyDown(web_sys::KeyboardEvent),
    ConfirmPurchase,
    PurchaseSent(FetchObject<i32>),

    NewTransactionTotalInput(String),
    AddItemToNewTransaction(i32, i32),
    AddBundleToNewTransaction(i32, i32),
    SetNewTransactionBundleChange { bundle_index: usize, change: i32 },
}

#[derive(Clone)]
pub struct StorePage {
    pub transaction: NewTransaction,

    pub override_transaction_total: bool,

    pub inventory_search_string: String,
    pub inventory_search: Vec<(i32, Vec<(usize, usize)>, StoreItem)>,

    pub tillgodolista_search_string: String,
    pub tillgodolista_search: Vec<(i32, Vec<(usize, usize)>, Rc<BookAccount>, Rc<Member>)>,
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

impl StorePage {
    pub fn new(global: &StateReady) -> Self {
        let mut p = StorePage {
            override_transaction_total: false,

            inventory_search_string: String::new(),
            inventory_search: vec![],

            tillgodolista_search_string: String::new(),
            tillgodolista_search: global
                .book_accounts
                .values()
                .filter_map(|acc| acc.creditor.map(|id| (acc, id)))
                .filter_map(|(acc, id)| {
                    global
                        .members
                        .get(&id)
                        .cloned()
                        .map(|creditor| (acc, creditor))
                })
                .map(|(acc, creditor)| (0, vec![], acc.clone(), creditor))
                .collect(),

            transaction: NewTransaction {
                description: Some("Försäljning".to_string()),
                bundles: vec![],
                amount: 0.into(),
                debited_account: global.master_accounts.bank_account_id,
                credited_account: global.master_accounts.sales_account_id,
            },
        };
        p.rebuild_store_list(global);
        p
    }

    pub fn update(
        &mut self,
        msg: StoreMsg,
        global: &mut StateReady,
        orders: &mut impl Orders<Msg>,
    ) {
        match msg {
            StoreMsg::SearchDebit(input) => {
                sort_tillgodolista_search(&input, &mut self.tillgodolista_search);
                self.tillgodolista_search_string = input;
            }
            StoreMsg::DebitKeyDown(ev) => match ev.key().as_str() {
                "Enter" => {
                    // TODO: Apply debit account
                    if let Some((_, _, acc, _)) = self.tillgodolista_search.first() {
                        let msg = StoreMsg::DebitSelect(acc.id);
                        self.update(msg, global, orders)
                    }
                }
                _ => {}
            },
            StoreMsg::DebitSelect(acc_id) => {
                self.tillgodolista_search_string = String::new();
                self.transaction.debited_account = acc_id;
            }

            StoreMsg::SearchInput(input) => {
                self.inventory_search_string = input;
                self.sort_store_list();
            }
            StoreMsg::SearchKeyDown(ev) => match ev.key().as_str() {
                "Enter" => match self.inventory_search.first() {
                    Some((_, _, StoreItem::Item(item))) => {
                        let msg = StoreMsg::AddItemToNewTransaction(item.id, 1);
                        self.update(msg, global, orders);
                    }
                    Some((_, _, StoreItem::Bundle(bundle))) => {
                        let msg = StoreMsg::AddBundleToNewTransaction(bundle.id, 1);
                        self.update(msg, global, orders);
                    }
                    None => {}
                },
                _ => {}
            },
            StoreMsg::ConfirmPurchase => {
                self.transaction.bundles.retain(|bundle| bundle.change != 0);
                orders.perform_cmd(
                    Request::new("/api/transaction")
                        .method(Method::Post)
                        .send_json(&self.transaction)
                        .fetch_json(|fetch| Msg::StoreMsg(StoreMsg::PurchaseSent(fetch))),
                );
                orders.skip();
            }
            StoreMsg::PurchaseSent(fetch_object) => match fetch_object.response() {
                Ok(response) => {
                    log!("ID: ", response.data);
                    self.transaction.amount = 0.into();
                    self.transaction.bundles = vec![];
                    self.transaction.description = Some("Försäljning".into());
                    orders.send_msg(Msg::ReloadData);
                }
                Err(e) => {
                    error!("Failed to post purchase", e);
                }
            },

            StoreMsg::NewTransactionTotalInput(input) => {
                log!("Input", input);
                if input == "" {
                    self.override_transaction_total = false;
                    self.recompute_new_transaction_total();
                } else {
                    self.override_transaction_total = true;
                    self.transaction.amount = input.parse().unwrap_or(0.into());
                    log!(format!("{}:-", self.transaction.amount));
                }
            }

            StoreMsg::AddItemToNewTransaction(item_id, amount) => {
                let item = global
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
                    self.transaction.bundles.iter_mut().find(|b| {
                        b.item_ids == bundle.item_ids && b.description == bundle.description
                    })
                {
                    b.change -= amount;
                } else {
                    log!("Pushing bundle", bundle);
                    self.transaction.bundles.push(bundle);
                }

                self.recompute_new_transaction_total();
            }
            StoreMsg::AddBundleToNewTransaction(bundle_id, amount) => {
                let bundle = global
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
                    self.transaction.bundles.iter_mut().find(|b| {
                        b.item_ids == bundle.item_ids && b.description == bundle.description
                    })
                {
                    b.change -= amount;
                } else {
                    log!("Pushing bundle", bundle);
                    self.transaction.bundles.push(bundle);
                }

                self.recompute_new_transaction_total();
            }

            StoreMsg::SetNewTransactionBundleChange {
                bundle_index,
                change,
            } => {
                let bundle = &mut self.transaction.bundles[bundle_index];
                if !self.override_transaction_total {
                    let diff = bundle.change - change;
                    self.transaction.amount +=
                        (bundle.price.map(|p| p.into()).unwrap_or(0i32) * diff).into();
                }
                bundle.change = change;
            }
        }
    }

    fn recompute_new_transaction_total(&mut self) {
        if !self.override_transaction_total {
            self.transaction.amount = self
                .transaction
                .bundles
                .iter()
                .map(|bundle| -bundle.change * bundle.price.map(|p| p.into()).unwrap_or(0i32))
                .sum::<i32>()
                .into();
        }
    }

    fn rebuild_store_list(&mut self, global: &StateReady) {
        let items = global
            .inventory
            .values()
            // Don't show items without a default price in the store view
            .filter(|item| item.price.is_some())
            .map(|item| (0, vec![], StoreItem::Item(item.clone())));

        let bundles = global
            .bundles
            .values()
            .map(|bundle| (0, vec![], StoreItem::Bundle(bundle.clone())));

        self.inventory_search = bundles.chain(items).collect();

        self.sort_store_list();
    }

    fn sort_store_list(&mut self) {
        for (score, matches, item) in self.inventory_search.iter_mut() {
            let (s, m) = item.compare_fuzzy(&self.inventory_search_string);
            *score = s;
            *matches = m;
        }
        self.inventory_search.sort_by(|(sa, _, ia), (sb, _, ib)| {
            sb.cmp(sa)
                .then(ia.get_search_string().cmp(&ib.get_search_string()))
        });
    }

    pub fn view(&self, global: &StateReady) -> Node<Msg> {
        let selected_bank_account =
            global.master_accounts.bank_account_id == self.transaction.debited_account;
        let selected_cash_account =
            global.master_accounts.cash_account_id == self.transaction.debited_account;

        div![
            class![C.store_page],
            div![
                class![C.store_top_box],
                div![
                    class![C.tillgodolista],
                    input![
                        class![
                            C.tillgodolista_search_field,
                            C.rounded_t_lg,
                            C.px_2,
                            C.h_12,
                            C.border_on_focus,
                        ],
                        if !(selected_bank_account || selected_cash_account) {
                            class![C.debit_selected]
                        } else {
                            class![]
                        },
                        attrs! {At::Value => self.tillgodolista_search_string},
                        {
                            let s = if selected_cash_account || selected_bank_account {
                                "Tillgodolista".into()
                            } else {
                                global
                                    .book_accounts
                                    .get(&self.transaction.debited_account)
                                    .map(|acc| format!("{}: {}:-", acc.name, acc.balance))
                                    .unwrap_or("[MISSING]".into())
                            };
                            attrs! {At::Placeholder => s}
                        },
                        input_ev(Ev::Input, |input| Msg::StoreMsg(StoreMsg::SearchDebit(
                            input
                        ))),
                        keyboard_ev(Ev::KeyDown, |ev| Msg::StoreMsg(StoreMsg::DebitKeyDown(ev))),
                    ],
                    div![
                        class![C.flex, C.flex_row],
                        button![
                            if selected_bank_account {
                                class![C.debit_selected]
                            } else {
                                class![]
                            },
                            class![C.select_debit_button, C.border_on_focus, C.rounded_bl_lg],
                            simple_ev(
                                Ev::Click,
                                Msg::StoreMsg(StoreMsg::DebitSelect(
                                    global.master_accounts.bank_account_id
                                )),
                            ),
                            "Swish",
                        ],
                        if !self.tillgodolista_search_string.is_empty() {
                            div![
                                class![C.tillgodo_drop_down],
                                self.tillgodolista_search
                                    .iter()
                                    .map(|(_, _, acc, mem)| view_tillgodo(
                                        acc,
                                        mem,
                                        Msg::StoreMsg(StoreMsg::DebitSelect(acc.id)),
                                    ))
                                    .collect::<Vec<_>>(),
                            ]
                        } else {
                            div![]
                        },
                        button![
                            if selected_cash_account {
                                class![C.debit_selected]
                            } else {
                                class![]
                            },
                            class![C.select_debit_button, C.border_on_focus, C.rounded_br_lg],
                            simple_ev(
                                Ev::Click,
                                Msg::StoreMsg(StoreMsg::DebitSelect(
                                    global.master_accounts.cash_account_id
                                )),
                            ),
                            "Kontant",
                        ],
                    ]
                ],
                input![
                    class![
                        C.inventory_search_field,
                        C.rounded,
                        C.px_2,
                        C.h_12,
                        C.border_on_focus,
                    ],
                    attrs! {At::Value => self.inventory_search_string},
                    attrs! {At::Placeholder => "sök varor"},
                    input_ev(Ev::Input, |input| Msg::StoreMsg(StoreMsg::SearchInput(
                        input
                    ))),
                    keyboard_ev(Ev::KeyDown, |ev| Msg::StoreMsg(StoreMsg::SearchKeyDown(ev))),
                ],
            ],
            div![
                class![C.inventory_view],
                self.inventory_search
                    .iter()
                    .map(|(_, matches, element)| match element {
                        StoreItem::Item(item) => view_inventory_item(
                            &item,
                            matches.iter().map(|&(_, i)| i),
                            |id, amount| Msg::StoreMsg(StoreMsg::AddItemToNewTransaction(
                                id, amount
                            ))
                        ),
                        StoreItem::Bundle(bundle) => view_inventory_bundle(
                            &bundle,
                            matches.iter().map(|&(_, i)| i),
                            |id, amount| Msg::StoreMsg(StoreMsg::AddBundleToNewTransaction(
                                id, amount
                            ))
                        ),
                    })
                    .collect::<Vec<_>>(),
            ],
            view_new_transaction(
                &self.transaction,
                self.override_transaction_total,
                &global.inventory,
                |bundle_index, change| Msg::StoreMsg(StoreMsg::SetNewTransactionBundleChange {
                    bundle_index,
                    change,
                }),
                |input| Msg::StoreMsg(StoreMsg::NewTransactionTotalInput(input)),
                Msg::StoreMsg(StoreMsg::ConfirmPurchase),
            ),
        ]
    }
}