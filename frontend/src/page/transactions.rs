use crate::app::{Msg, StateReady};
use crate::generated::css_classes::C;
use laggit_api::inventory::InventoryItemStock as InventoryItem;
use laggit_api::transaction::Transaction;
use seed::prelude::*;
use seed::{fetch::FetchObject, *};
use std::ops::Deref;

#[derive(Clone)]
pub enum TransactionsMsg {
    DeleteTransaction(i32),
    TransactionDeleted(FetchObject<i32>),
    SetShowDelete(bool),
}

#[derive(Clone)]
pub struct TransactionsPage {
    show_delete: bool,
}

impl TransactionsPage {
    pub fn new(_global: &StateReady) -> Self {
        TransactionsPage { show_delete: false }
    }

    pub fn update(
        &mut self,
        msg: TransactionsMsg,
        _global: &mut StateReady,
        orders: &mut impl Orders<Msg>,
    ) {
        let mut orders_local = orders.proxy(|msg| Msg::TransactionsMsg(msg));
        match msg {
            TransactionsMsg::DeleteTransaction(id) => {
                orders_local.perform_cmd(
                    Request::new(format!("/api/transaction/{}", id))
                        .method(Method::Delete)
                        .fetch_json(TransactionsMsg::TransactionDeleted),
                );
            }

            TransactionsMsg::TransactionDeleted(fetch_object) => match fetch_object.response() {
                Ok(response) => {
                    log!(format!("Transaction {} deleted", response.data));
                    orders.send_msg(Msg::ReloadData);
                }
                Err(e) => {
                    error!("Failed to delete transaction", e);
                }
            },

            TransactionsMsg::SetShowDelete(show_delete) => {
                self.show_delete = show_delete;
            }
        }
    }

    pub fn view(&self, global: &StateReady) -> Node<Msg> {
        let view_transaction = |transaction: &Transaction| {
            div![
                class![C.transaction_view],
                p![
                    span![format!("#{} ", transaction.id)],
                    span![transaction
                        .description
                        .as_ref()
                        .map(|s| s.as_str())
                        .unwrap_or("Transaktion")],
                    if self.show_delete {
                        button![
                            class![C.transaction_view_delete_button],
                            simple_ev(
                                Ev::Click,
                                TransactionsMsg::DeleteTransaction(transaction.id)
                            ),
                            "X"
                        ]
                    } else {
                        empty![]
                    }
                ],
                p![
                    class![C.mt_2],
                    span!["Debet: "],
                    span![
                        class![C.font_bold],
                        global
                            .book_accounts
                            .get(&transaction.debited_account)
                            .map(|acc| acc.name.as_str())
                            .unwrap_or("[MISSING]")
                    ],
                ],
                p![
                    class![C.mt_2, C.mb_2],
                    span!["Kredit: "],
                    span![
                        class![C.font_bold],
                        global
                            .book_accounts
                            .get(&transaction.credited_account)
                            .map(|acc| acc.name.as_str())
                            .unwrap_or("[MISSING]")
                    ],
                ],
                transaction
                    .bundles
                    .iter()
                    .map(|bundle| {
                        let mut items = bundle.item_ids.keys().map(|id| &global.inventory[id]);

                        // TODO: Properly display more complicated bundles

                        let (item_name, item_price) = match items.next().map(|rc| rc.deref()) {
                            None => (None, 0),
                            Some(InventoryItem { name, price, .. }) => {
                                (Some(name.as_str()), price.unwrap_or(0))
                            }
                        };

                        let name = bundle
                            .description
                            .as_ref()
                            .map(|s| s.as_str())
                            .unwrap_or(item_name.unwrap_or("[NAMN SAKNAS]"));
                        let price = bundle.price.unwrap_or(item_price.into());
                        p![
                            class![C.transaction_entry],
                            span![
                                class![C.transaction_entry_item_name],
                                format!("{}x {}", -bundle.change, name),
                            ],
                            span![
                                class![C.transaction_entry_item_price],
                                format!("{}:-", price),
                            ],
                        ]
                    })
                    .collect::<Vec<_>>(),
                p![
                    span!["Totalt: "],
                    span![
                        class![C.transaction_entry_item_price],
                        format!("{}:-", transaction.amount),
                    ],
                ],
            ]
        };

        div![
            class![C.transactions_page],
            button![
                class!["transaction_page_show_delete"],
                "Radera transaktioner?",
                simple_ev(Ev::Click, TransactionsMsg::SetShowDelete(!self.show_delete)),
            ],
            global
                .transaction_history
                .iter()
                .map(|t| view_transaction(t))
                .collect::<Vec<_>>(),
        ]
        .map_message(|msg| Msg::TransactionsMsg(msg))
    }
}
