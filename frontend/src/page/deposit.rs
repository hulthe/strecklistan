use crate::app::{Msg, StateReady};
use crate::generated::css_classes::C;
use crate::util::sort_tillgodolista_search;
use crate::views::view_tillgodo;
use laggit_api::{
    book_account::{BookAccount, BookAccountId},
    currency::Currency,
    member::Member,
    transaction::{NewTransaction, TransactionId},
};
use seed::prelude::*;
use seed::{fetch::FetchObject, *};
use std::rc::Rc;

#[derive(Clone)]
pub enum DepositionMsg {
    SearchDebit(String),
    CreditKeyDown(web_sys::KeyboardEvent),
    CreditSelect(BookAccountId),
    SetUseCash(bool),
    SetAmount(String),
    Deposit,
    DepositSent(FetchObject<TransactionId>),
}

#[derive(Clone)]
pub struct DepositionPage {
    use_cash: bool,
    credit_account: Option<BookAccountId>,
    search_string: String,
    accs_search: Vec<(i32, Vec<(usize, usize)>, Rc<BookAccount>, Rc<Member>)>,
    amount: Currency,
}

impl DepositionPage {
    pub fn new(global: &StateReady) -> Self {
        DepositionPage {
            use_cash: false,
            credit_account: None,
            search_string: String::new(),
            accs_search: global
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
            amount: 0.into(),
        }
    }

    pub fn update(
        &mut self,
        msg: DepositionMsg,
        global: &mut StateReady,
        orders: &mut impl Orders<Msg>,
    ) {
        let mut orders_local = orders.proxy(|msg| Msg::DepositionMsg(msg));
        match msg {
            DepositionMsg::SearchDebit(input) => {
                sort_tillgodolista_search(&input, &mut self.accs_search);
                self.search_string = input;
            }
            DepositionMsg::CreditKeyDown(ev) => match ev.key().as_str() {
                "Enter" => {
                    if let Some((_, _, acc, _)) = self.accs_search.first() {
                        orders_local.send_msg(DepositionMsg::CreditSelect(acc.id));
                    }
                }
                _ => {}
            },
            DepositionMsg::CreditSelect(acc_id) => {
                self.search_string = String::new();
                self.credit_account = Some(acc_id);
            }
            DepositionMsg::SetUseCash(use_cash) => {
                self.use_cash = use_cash;
            }
            DepositionMsg::SetAmount(input) => {
                self.amount = input.parse().unwrap_or(0.into());
            }
            DepositionMsg::Deposit => {
                if let Some(credit_acc) = self.credit_account {
                    let transaction = NewTransaction {
                        description: Some("Insättning".into()),
                        amount: self.amount,
                        credited_account: credit_acc,
                        debited_account: if self.use_cash {
                            global.master_accounts.cash_account_id
                        } else {
                            global.master_accounts.bank_account_id
                        },
                        bundles: vec![],
                    };

                    orders_local.perform_cmd(
                        Request::new("/api/transaction")
                            .method(Method::Post)
                            .send_json(&transaction)
                            .fetch_json(DepositionMsg::DepositSent),
                    );
                }
            }

            DepositionMsg::DepositSent(fetch_object) => match fetch_object.response() {
                Ok(response) => {
                    log!("ID: ", response.data);
                    self.amount = 0.into();
                    self.credit_account = None;
                    orders.send_msg(Msg::ReloadData);
                }
                Err(e) => {
                    error!("Failed to post deposit", e);
                }
            },
        }
    }

    pub fn view(&self, global: &StateReady) -> Node<Msg> {
        div![
            class![C.tillgodolista],
            input![
                class![
                    C.tillgodolista_search_field,
                    C.rounded_lg,
                    C.mb_2,
                    C.px_2,
                    C.h_12,
                    C.border_on_focus,
                ],
                if self.credit_account.is_some() {
                    class![C.debit_selected]
                } else {
                    class![]
                },
                attrs! {At::Value => self.search_string},
                {
                    let s = if let Some(acc_id) = self.credit_account {
                        global
                            .book_accounts
                            .get(&acc_id)
                            .map(|acc| acc.name.as_str())
                            .unwrap_or("[MISSING]")
                    } else {
                        "Välj Tillgodokonto"
                    };
                    attrs! {At::Placeholder => s}
                },
                input_ev(Ev::Input, DepositionMsg::SearchDebit),
                keyboard_ev(Ev::KeyDown, DepositionMsg::CreditKeyDown),
            ],
            div![
                class![C.flex, C.flex_row],
                button![
                    if !self.use_cash {
                        class![C.debit_selected]
                    } else {
                        class![]
                    },
                    class![C.select_debit_button, C.border_on_focus, C.rounded_l_lg],
                    simple_ev(Ev::Click, DepositionMsg::SetUseCash(false)),
                    "Swish",
                ],
                if !self.search_string.is_empty() {
                    div![
                        class![C.tillgodo_drop_down],
                        self.accs_search
                            .iter()
                            .map(|(_, _, acc, mem)| view_tillgodo(
                                acc,
                                mem,
                                DepositionMsg::CreditSelect(acc.id)
                            ))
                            .collect::<Vec<_>>(),
                    ]
                } else {
                    empty![]
                },
                button![
                    if self.use_cash {
                        class![C.debit_selected]
                    } else {
                        class![]
                    },
                    class![C.select_debit_button, C.border_on_focus, C.rounded_r_lg],
                    simple_ev(Ev::Click, DepositionMsg::SetUseCash(true),),
                    "Kontant",
                ],
            ],
            input![
                class![
                    C.rounded,
                    C.px_2,
                    C.my_2,
                    //C.h_12,
                    C.border_on_focus,
                    C.bg_gray_300,
                ],
                attrs! {At::Value => self.amount.to_string()},
                input_ev(Ev::Input, DepositionMsg::SetAmount),
            ],
            button![
                class![C.wide_button],
                simple_ev(Ev::Click, DepositionMsg::Deposit),
                "Sätt in"
            ]
        ]
        .map_message(|msg| Msg::DepositionMsg(msg))
    }
}
