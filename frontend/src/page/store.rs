use crate::app::{Msg, StateReady};
use crate::fuzzy_search::FuzzySearch;
use crate::generated::css_classes::C;
use crate::notification_manager::{Notification, NotificationMessage};
use crate::util::{compare_fuzzy, sort_tillgodolista_search};
use crate::views::{
    view_inventory_bundle, view_inventory_item, view_new_transaction, view_tillgodo, ParsedInput,
    ParsedInputMsg,
};
use seed::app::cmds::timeout;
use seed::prelude::*;
use seed::*;
use std::collections::HashMap;
use std::rc::Rc;
use strecklistan_api::{
    book_account::{BookAccount, BookAccountId},
    currency::Currency,
    inventory::{
        InventoryBundle, InventoryBundleId, InventoryItemId, InventoryItemStock as InventoryItem,
    },
    izettle::ClientPollResult,
    member::Member,
    transaction::{NewTransaction, TransactionBundle, TransactionId},
};

#[derive(Clone, Debug)]
pub enum StoreItem {
    Item(Rc<InventoryItem>),
    Bundle(Rc<InventoryBundle>),
}

impl StoreItem {
    pub fn get_name(&self) -> &str {
        match self {
            StoreItem::Item(item) => &item.name,
            StoreItem::Bundle(bundle) => &bundle.name,
        }
    }
}

impl FuzzySearch for StoreItem {
    fn compare_fuzzy(&self, search: &str) -> (i32, Vec<(usize, usize)>) {
        compare_fuzzy(self.get_name().chars(), search.chars())
    }
}

impl FuzzySearch for Member {
    fn compare_fuzzy(&self, search: &str) -> (i32, Vec<(usize, usize)>) {
        match &self.nickname {
            Some(nick) => compare_fuzzy(nick.chars(), search.chars()),
            None => compare_fuzzy(
                self.first_name
                    .chars()
                    .chain(std::iter::once(' '))
                    .chain(self.last_name.chars()),
                search.chars(),
            ),
        }
    }
}

#[derive(Clone, Debug)]
pub enum StoreMsg {
    SearchDebit(String),
    DebitKeyDown(web_sys::KeyboardEvent),
    DebitSelect(BookAccountId),

    DebitSelectIZettle,
    PollForPendingIZettleTransaction(i32),
    CancelIZettle {
        message_title: String,
        message_body: Option<String>,
    },

    SearchInput(String),
    SearchKeyDown(web_sys::KeyboardEvent),
    ConfirmPurchase,
    PurchaseSent(TransactionId),

    NewTransactionTotalInput(ParsedInputMsg),
    AddItemToNewTransaction(InventoryItemId, i32),
    AddBundleToNewTransaction(InventoryBundleId, i32),
    SetNewTransactionBundleChange {
        bundle_index: usize,
        change: i32,
    },
}

#[derive(Clone)]
pub struct StorePage {
    pub transaction: NewTransaction,

    pub transaction_total: ParsedInput<Currency>,
    pub override_transaction_total: bool,

    pub inventory_search_string: String,
    pub inventory_search: Vec<(i32, Vec<(usize, usize)>, StoreItem)>,

    pub tillgodolista_search_string: String,
    pub tillgodolista_search: Vec<(i32, Vec<(usize, usize)>, Rc<BookAccount>, Rc<Member>)>,

    pub izettle: bool,
}

impl StorePage {
    pub fn new(global: &StateReady) -> Self {
        let mut p = StorePage {
            transaction_total: ParsedInput::new("0", "text", "ogiltig summa"),
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

            izettle: false,
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
                    if let Some((_, _, acc, _)) = self.tillgodolista_search.first() {
                        let msg = StoreMsg::DebitSelect(acc.id);
                        self.update(msg, global, orders)
                    }
                }
                _ => {}
            },
            StoreMsg::DebitSelect(acc_id) => {
                self.izettle = false;
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
                global.request_in_progress = true;
                let msg = self.transaction.clone();

                if self.izettle {
                    orders.perform_cmd(async move {
                        let result = async {
                            Request::new("/api/izettle/client/transaction")
                                .method(Method::Post)
                                .json(&msg)?
                                .fetch()
                                .await?
                                .json()
                                .await
                        }
                        .await;
                        match result {
                            Ok(reference) => Some(Msg::StoreMsg(
                                StoreMsg::PollForPendingIZettleTransaction(reference),
                            )),
                            Err(e) => {
                                error!("Failed to post purchase", e);
                                None
                            }
                        }
                    });
                } else {
                    orders.perform_cmd(async move {
                        let result = async {
                            Request::new("/api/transaction")
                                .method(Method::Post)
                                .json(&msg)?
                                .fetch()
                                .await?
                                .json()
                                .await
                        }
                        .await;
                        match result {
                            Ok(id) => Some(Msg::StoreMsg(StoreMsg::PurchaseSent(id))),
                            Err(e) => {
                                error!("Failed to post purchase", e);
                                None
                            }
                        }
                    });
                }
            }

            StoreMsg::PurchaseSent(id) => {
                orders.send_msg(Msg::NotificationMessage(
                    NotificationMessage::ShowNotification {
                        duration_ms: 5000,
                        notification: Notification {
                            title: "Purchase complete".to_string(),
                            body: Some(format!("Total: {}:-", self.transaction.amount)),
                        },
                    },
                ));

                global.request_in_progress = false;
                log!("ID: ", id);
                self.transaction.amount = 0.into();
                self.transaction.bundles = vec![];
                self.transaction.description = Some("Försäljning".into());
                orders.send_msg(Msg::ReloadData);
            }

            StoreMsg::NewTransactionTotalInput(msg) => {
                match &msg {
                    ParsedInputMsg::FocusOut => {
                        if self.transaction_total.get_value().is_none() {
                            self.override_transaction_total = false;
                            self.recompute_new_transaction_total();
                        }
                    }
                    ParsedInputMsg::Input(_) => {
                        self.override_transaction_total = true;
                    }
                    _ => {}
                }
                self.transaction_total.update(msg);

                if self.override_transaction_total {
                    let new_total = self.transaction_total.get_value().copied();
                    log!(format!("new transaction total: {:?}", new_total));
                    self.transaction.amount = new_total.unwrap_or(0.into());
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

            StoreMsg::PollForPendingIZettleTransaction(reference) => {
                orders.perform_cmd(async move {
                    let result = async {
                        Request::new(&format!("/api/izettle/client/poll/{}", reference))
                            .method(Method::Get)
                            //.json(&msg)?
                            .fetch()
                            .await?
                            .json()
                            .await
                    }
                    .await;
                    match result {
                        Ok(ClientPollResult::NotPaid) => {
                            timeout(1000u32, || ()).await;
                            Some(Msg::StoreMsg(StoreMsg::PollForPendingIZettleTransaction(
                                reference,
                            )))
                        }
                        Ok(ClientPollResult::Paid { transaction_id }) => {
                            Some(Msg::StoreMsg(StoreMsg::PurchaseSent(transaction_id)))
                        }
                        Ok(ClientPollResult::NoTransaction(error)) => {
                            Some(Msg::StoreMsg(StoreMsg::CancelIZettle {
                                message_title: "Server Error".to_string(),
                                message_body: Some(format!(
                                    "No pending transaction: {}",
                                    error.message
                                )),
                            }))
                        }
                        Ok(ClientPollResult::Canceled) => {
                            Some(Msg::StoreMsg(StoreMsg::CancelIZettle {
                                message_title: "Payment canceled".to_string(),
                                message_body: None,
                            }))
                        }
                        Ok(ClientPollResult::Failed(error)) => {
                            Some(Msg::StoreMsg(StoreMsg::CancelIZettle {
                                message_title: "Payment failed".to_string(),
                                message_body: Some(error.message),
                            }))
                        }
                        Err(e) => {
                            error!("Failed to post purchase", e);
                            None
                        }
                    }
                });
            }

            StoreMsg::DebitSelectIZettle => {
                self.update(
                    StoreMsg::DebitSelect(global.master_accounts.bank_account_id),
                    global,
                    orders,
                );
                self.izettle = true;
            }

            StoreMsg::CancelIZettle {
                message_title,
                message_body,
            } => {
                global.request_in_progress = false;
                orders.send_msg(Msg::NotificationMessage(
                    NotificationMessage::ShowNotification {
                        duration_ms: 10000,
                        notification: Notification {
                            title: message_title,
                            body: message_body,
                        },
                    },
                ));
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
            self.transaction_total.set_value(self.transaction.amount);
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
        self.inventory_search
            .sort_by(|(sa, _, ia), (sb, _, ib)| sb.cmp(sa).then(ia.get_name().cmp(&ib.get_name())));
    }

    pub fn view(&self, global: &StateReady) -> Node<Msg> {
        #[derive(PartialEq)]
        enum SelectedDebit {
            IZettleEPay,
            OtherEPay,
            Cash,
            Tillgodo,
        }

        let selected_debit = if self.izettle {
            SelectedDebit::IZettleEPay
        } else if self.transaction.debited_account == global.master_accounts.bank_account_id {
            SelectedDebit::OtherEPay
        } else if self.transaction.debited_account == global.master_accounts.cash_account_id {
            SelectedDebit::Cash
        } else {
            SelectedDebit::Tillgodo
        };

        let apply_selection_class_on = |matching_debit| {
            if selected_debit == matching_debit {
                class![C.debit_selected]
            } else {
                class![]
            }
        };

        div![
            class![C.store_page],
            div![
                class![C.store_top_box],
                div![
                    class![C.pay_method_select_box, C.margin_hcenter],
                    input![
                        class![
                            C.tillgodolista_search_field,
                            C.rounded_t_lg,
                            C.px_2,
                            C.h_12,
                            C.border_on_focus,
                        ],
                        apply_selection_class_on(SelectedDebit::Tillgodo),
                        attrs! {At::Value => self.tillgodolista_search_string},
                        {
                            attrs! {
                                At::Placeholder => match selected_debit {
                                    SelectedDebit::Tillgodo => global
                                        .book_accounts
                                        .get(&self.transaction.debited_account)
                                        .map(|acc| format!("{}: {}:-", acc.name, acc.balance))
                                        .unwrap_or("[MISSING]".into()),
                                    _ => "Tillgodolista".into(),
                                },
                            }
                        },
                        input_ev(Ev::Input, |input| Msg::StoreMsg(StoreMsg::SearchDebit(
                            input
                        ))),
                        keyboard_ev(Ev::KeyDown, |ev| Msg::StoreMsg(StoreMsg::DebitKeyDown(ev))),
                    ],
                    div![
                        class![C.flex, C.flex_row],
                        if !self.tillgodolista_search_string.is_empty() {
                            div![
                                class![C.tillgodo_drop_down],
                                div![
                                    class![C.tillgodo_list],
                                    self.tillgodolista_search
                                        .iter()
                                        .map(|(_, _, acc, mem)| view_tillgodo(
                                            acc,
                                            mem,
                                            Msg::StoreMsg(StoreMsg::DebitSelect(acc.id)),
                                        ))
                                        .collect::<Vec<_>>(),
                                ],
                            ]
                        } else {
                            empty![]
                        },
                        button![
                            apply_selection_class_on(SelectedDebit::OtherEPay),
                            class![C.select_debit_button, C.border_on_focus, C.rounded_bl_lg],
                            simple_ev(
                                Ev::Click,
                                Msg::StoreMsg(StoreMsg::DebitSelect(
                                    global.master_accounts.bank_account_id
                                )),
                            ),
                            "Swish",
                        ],
                        button![
                            apply_selection_class_on(SelectedDebit::IZettleEPay),
                            class![C.select_debit_button, C.border_on_focus, C.rounded_br_lg],
                            simple_ev(Ev::Click, Msg::StoreMsg(StoreMsg::DebitSelectIZettle)),
                            "iZettle",
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
                !global.request_in_progress,
                match selected_debit {
                    SelectedDebit::IZettleEPay if global.request_in_progress =>
                        Some("Waiting for payment"),
                    _ => None,
                },
                &global.inventory,
                |bundle_index, change| Msg::StoreMsg(StoreMsg::SetNewTransactionBundleChange {
                    bundle_index,
                    change,
                }),
                &self.transaction_total,
                |input| Msg::StoreMsg(StoreMsg::NewTransactionTotalInput(input)),
                Msg::StoreMsg(StoreMsg::ConfirmPurchase),
            ),
        ]
    }
}
