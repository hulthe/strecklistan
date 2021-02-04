use crate::app::{Msg, StateReady};
use crate::components::checkout::{Checkout, CheckoutMsg};
use crate::fuzzy_search::FuzzySearch;
use crate::generated::css_classes::C;
use crate::notification_manager::{Notification, NotificationMessage};
use crate::util::{compare_fuzzy, sort_tillgodolista_search};
use crate::views::{view_inventory_bundle, view_inventory_item, view_tillgodo};
use seed::app::cmds::timeout;
use seed::prelude::*;
use seed::*;
use std::rc::Rc;
use strecklistan_api::{
    book_account::{BookAccount, BookAccountId},
    inventory::{InventoryBundle, InventoryItemStock as InventoryItem},
    izettle::ClientPollResult,
    member::Member,
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

    CheckoutMsg(CheckoutMsg),
}

#[derive(Clone)]
pub struct StorePage {
    pub checkout: Checkout,

    pub inventory_search_string: String,
    pub inventory_search: Vec<(i32, Vec<(usize, usize)>, StoreItem)>,

    pub tillgodolista_search_string: String,
    pub tillgodolista_search: Vec<(i32, Vec<(usize, usize)>, Rc<BookAccount>, Rc<Member>)>,

    pub izettle: bool,
}

impl StorePage {
    pub fn new(global: &StateReady) -> Self {
        let mut p = StorePage {
            checkout: Checkout::new(global),

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
                self.checkout.set_debited(acc_id);
            }

            StoreMsg::SearchInput(input) => {
                self.inventory_search_string = input;
                self.sort_store_list();
            }
            StoreMsg::SearchKeyDown(ev) => match ev.key().as_str() {
                "Enter" => match self.inventory_search.first() {
                    Some((_, _, StoreItem::Item(item))) => {
                        let msg = StoreMsg::CheckoutMsg(CheckoutMsg::AddItem {
                            item_id: item.id,
                            amount: 1,
                        });
                        self.update(msg, global, orders);
                    }
                    Some((_, _, StoreItem::Bundle(bundle))) => {
                        let msg = StoreMsg::CheckoutMsg(CheckoutMsg::AddBundle {
                            bundle_id: bundle.id,
                            amount: 1,
                        });
                        self.update(msg, global, orders);
                    }
                    None => {}
                },
                _ => {}
            },
            StoreMsg::PollForPendingIZettleTransaction(reference) => {
                orders.perform_cmd(async move {
                    let result = async {
                        Request::new(&format!("/api/izettle/client/poll/{}", reference))
                            .method(Method::Get)
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
                        Ok(ClientPollResult::Paid { transaction_id }) => Some(Msg::StoreMsg(
                            StoreMsg::CheckoutMsg(CheckoutMsg::PurchaseSent { transaction_id }),
                        )),
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
                self.checkout.confirm_button_message = None;
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

            StoreMsg::CheckoutMsg(msg) => {
                let forward_msg = match msg {
                    // if iZettle integration is enabled we intercept and handle the purchase here
                    CheckoutMsg::ConfirmPurchase if self.izettle => {
                        global.request_in_progress = true;
                        self.checkout.remove_cleared_items();
                        self.checkout.confirm_button_message = Some("Väntar på betalning");
                        let transaction = self.checkout.transaction().clone();
                        orders.perform_cmd(async move {
                            let result = async {
                                Request::new("/api/izettle/client/transaction")
                                    .method(Method::Post)
                                    .json(&transaction)?
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
                        None // don't forward the message
                    }
                    // show a notification & reload the app when a purchase completes
                    msg @ CheckoutMsg::PurchaseSent { .. } => {
                        orders.send_msg(Msg::NotificationMessage(
                            NotificationMessage::ShowNotification {
                                duration_ms: 5000,
                                notification: Notification {
                                    title: "Purchase complete".to_string(),
                                    body: Some(format!(
                                        "Total: {}:-",
                                        self.checkout.transaction().amount
                                    )),
                                },
                            },
                        ));
                        self.checkout.confirm_button_message = None;
                        orders.send_msg(Msg::ReloadData);
                        Some(msg)
                    }
                    msg => Some(msg),
                };

                forward_msg.map(|msg| {
                    self.checkout.update(
                        msg,
                        global,
                        &mut orders.proxy(Msg::StoreMsg).proxy(StoreMsg::CheckoutMsg),
                    )
                });
            }
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
        } else if self.checkout.transaction().debited_account
            == global.master_accounts.bank_account_id
        {
            SelectedDebit::OtherEPay
        } else if self.checkout.transaction().debited_account
            == global.master_accounts.cash_account_id
        {
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
                                        .get(&self.checkout.transaction().debited_account)
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
                            |item_id, amount| Msg::StoreMsg(StoreMsg::CheckoutMsg(
                                CheckoutMsg::AddItem { item_id, amount }
                            ))
                        ),
                        StoreItem::Bundle(bundle) => view_inventory_bundle(
                            &bundle,
                            matches.iter().map(|&(_, i)| i),
                            |bundle_id, amount| Msg::StoreMsg(StoreMsg::CheckoutMsg(
                                CheckoutMsg::AddBundle { bundle_id, amount }
                            ))
                        ),
                    })
                    .collect::<Vec<_>>(),
            ],
            self.checkout
                .view(global)
                .map_msg(StoreMsg::CheckoutMsg)
                .map_msg(Msg::StoreMsg),
        ]
    }
}
