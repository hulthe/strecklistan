use crate::app::{Msg, StateReady};
use crate::generated::css_classes::C;
use crate::views::filter_menu::{FilterMenu, FilterMenuMsg};
use laggit_api::book_account::BookAccountId;
use laggit_api::currency::Currency;
use laggit_api::inventory::InventoryItemStock as InventoryItem;
use laggit_api::transaction::{Transaction, TransactionId};
use seed::prelude::*;
use seed::{browser::service::fetch::FetchObject, *};
use std::collections::HashMap;
use std::ops::Deref;

const VIEW_COUNT_CHUNK: usize = 50;

#[derive(Clone, Debug)]
pub enum TransactionsMsg {
    DeleteTransaction(TransactionId),
    TransactionDeleted(FetchObject<TransactionId>),
    SetShowDelete(bool),
    SetShowLeftPanel(bool),
    FilterMenuMsg(FilterMenuMsg),
    IncreaseViewLimit,
}

#[derive(Clone)]
pub struct TransactionsPage {
    show_delete: bool,
    show_left_panel: bool,
    view_limit: usize,
    filter_menu: FilterMenu,

    // Indexes into global.transaction_history
    filtered_transactions: Vec<usize>,

    // The balance of all accounts based on the filtered transactions
    accounts_balance: HashMap<BookAccountId, Currency>,
}

impl TransactionsPage {
    pub fn new(global: &StateReady) -> Self {
        let mut page = TransactionsPage {
            show_delete: false,
            show_left_panel: false,
            view_limit: VIEW_COUNT_CHUNK,
            filter_menu: FilterMenu::new(vec!["datum", "klockslag", "summa", "debet", "kredit"]),
            filtered_transactions: vec![],
            accounts_balance: HashMap::new(),
        };
        page.filter_transactions(global);
        page
    }

    /// Rebuild self.filtered_transactions
    fn filter_transactions(&mut self, global: &StateReady) {
        self.filtered_transactions = global
            .transaction_history
            .iter()
            .enumerate()
            .filter(|(_, tr)| {
                self.filter_menu.filter(&[
                    &tr.time.with_timezone(&global.timezone).format("%Y-%m-%d"), // datum
                    &tr.time.with_timezone(&global.timezone).format("%H:%M:%S"), // klockslag
                    &tr.amount,                                                  // summa
                    &global.book_accounts.get(&tr.debited_account).unwrap().name, // debet
                    &global.book_accounts.get(&tr.credited_account).unwrap().name, // kredit
                ])
            })
            .map(|(i, _)| i)
            .collect();

        self.accounts_balance.clear();
        for tr in self
            .filtered_transactions
            .iter()
            .map(|&i| &global.transaction_history[i])
        {
            if let Some(acc) = global.book_accounts.get(&tr.debited_account) {
                *self.accounts_balance.entry(tr.debited_account).or_default() +=
                    acc.debit_diff(tr.amount);
            }
            if let Some(acc) = global.book_accounts.get(&tr.credited_account) {
                *self
                    .accounts_balance
                    .entry(tr.credited_account)
                    .or_default() += acc.credit_diff(tr.amount);
            }
        }
    }

    pub fn update(
        &mut self,
        msg: TransactionsMsg,
        global: &mut StateReady,
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
            TransactionsMsg::SetShowLeftPanel(show_left_panel) => {
                self.show_left_panel = show_left_panel;
            }
            TransactionsMsg::FilterMenuMsg(msg) => {
                self.filter_menu.update(
                    msg,
                    &mut orders_local.proxy(|msg| TransactionsMsg::FilterMenuMsg(msg)),
                );
                self.view_limit = VIEW_COUNT_CHUNK; // reset view limit
                self.filter_transactions(global);
            }
            TransactionsMsg::IncreaseViewLimit => {
                self.view_limit += VIEW_COUNT_CHUNK;
                self.filter_transactions(global);
            }
        }
    }

    pub fn view(&self, global: &StateReady) -> Node<Msg> {
        let show_acc_entry = |name: &str, balance: Currency| {
            div![
                class![C.balance_entry],
                span![name],
                span![": "],
                span![class![C.flex_grow, C.flex_shrink, C.w_8]],
                span![format!("{}:-", balance)],
            ]
        };
        let show_acc = |id: &BookAccountId| {
            show_acc_entry(
                global
                    .book_accounts
                    .get(id)
                    .map(|acc| acc.name.as_str())
                    .unwrap_or("[missing]"),
                self.accounts_balance
                    .get(id)
                    .map(|&c| c)
                    .unwrap_or(0.into()),
            )
        };

        let transaction_list: Vec<_> = self
            .filtered_transactions
            .iter()
            .take(self.view_limit)
            .map(|&i| &global.transaction_history[i])
            .map(|tr| view_transaction(global, tr, self.show_delete))
            .collect();

        div![
            class![C.transactions_page],
            div![
                class![C.left_panel, C.px_4],
                if self.show_left_panel {
                    class![C.left_panel_showing]
                } else {
                    class![]
                },
                div![
                    class![C.flex, C.flex_row, C.text_3xl, C.font_bold],
                    h2![class![C.mx_auto, C.my_2], "Balansräkning"],
                ],
                div![
                    class![C.balance_sheet, C.margin_hcenter],
                    show_acc(&global.master_accounts.bank_account_id),
                    show_acc(&global.master_accounts.cash_account_id),
                    show_acc(&global.master_accounts.sales_account_id),
                    show_acc(&global.master_accounts.purchases_account_id),
                    show_acc_entry(
                        "Tillgodo Totalt",
                        self.accounts_balance
                            .iter()
                            .filter_map(|(id, balance)| global
                                .book_accounts
                                .get(id)
                                .map(|acc| (acc, balance)))
                            .filter(|(acc, _)| acc.creditor.is_some())
                            .map(|(_, balance)| *balance)
                            .fold(0.into(), |a: Currency, b| a + b)
                    ),
                ],
                hr![class![C.my_2]],
                div![
                    class![C.flex, C.flex_row, C.text_3xl, C.font_bold],
                    h2![class![C.mx_auto, C.my_2], "Filtrera (WIP)"],
                ],
                self.filter_menu
                    .view()
                    .map_msg(|msg| TransactionsMsg::FilterMenuMsg(msg)),
            ],
            button![
                class![C.left_panel_button,],
                simple_ev(
                    Ev::Click,
                    TransactionsMsg::SetShowLeftPanel(!self.show_left_panel),
                ),
                "⚙"
            ],
            div![if self.show_left_panel {
                class![C.left_panel_sub_spacer]
            } else {
                class![C.left_panel_sub_spacer, C.left_panel_sub_spacer_hidden]
            },],
            div![
                class![C.transactions_list],
                button![
                    class![C.transaction_page_show_delete],
                    "Radera transaktioner?",
                    simple_ev(Ev::Click, TransactionsMsg::SetShowDelete(!self.show_delete)),
                ],
                transaction_list,
                if self.view_limit < self.filtered_transactions.len() {
                    button![
                        class![C.wide_button],
                        "Visa fler",
                        simple_ev(Ev::Click, TransactionsMsg::IncreaseViewLimit),
                    ]
                } else {
                    empty![]
                },
            ],
        ]
        .map_msg(|msg| Msg::TransactionsMsg(msg))
    }
}

fn view_transaction(
    global: &StateReady,
    transaction: &Transaction,
    show_delete: bool,
) -> Node<TransactionsMsg> {
    div![
        class![C.transaction_view],
        p![
            span![format!("#{} ", transaction.id)],
            span![transaction
                .description
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or("Transaktion")],
            if show_delete {
                button![
                    class![C.transaction_view_delete_button],
                    simple_ev(
                        Ev::Click,
                        TransactionsMsg::DeleteTransaction(transaction.id)
                    ),
                    "✖",
                ]
            } else {
                empty![]
            }
        ],
        p![format!(
            "{}",
            transaction
                .time
                .with_timezone(&global.timezone)
                .format("%Y-%m-%d %H:%M:%S %Z"),
        )],
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
}
