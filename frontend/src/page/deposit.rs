use crate::app::{Msg, StateReady};
use crate::components::izettle_pay::{IZettlePay, IZettlePayErr, IZettlePayMsg};
use crate::components::parsed_input::{ParsedInput, ParsedInputMsg};
use crate::generated::css_classes::C;
use crate::notification_manager::{Notification, NotificationMessage};
use crate::strings;
use crate::util::sort_tillgodolista_search;
use crate::views::view_tillgodo;
use seed::prelude::*;
use seed::*;
use std::rc::Rc;
use strecklistan_api::{
    book_account::{BookAccount, BookAccountId},
    currency::Currency,
    member::{Member, MemberId, NewMember},
    transaction::{NewTransaction, TransactionId},
};

#[derive(Clone)]
pub struct DepositionPage {
    debit: DebitOption,
    izettle_pay: IZettlePay,
    credit_account: Option<BookAccountId>,
    search_string: String,
    accs_search: Vec<(i32, Vec<(usize, usize)>, Rc<BookAccount>, Rc<Member>)>,
    amount_input: ParsedInput<Currency>,
    new_member: Option<(String, String, String, Option<String>)>,
}

#[derive(Clone, Debug)]
pub enum DepositionMsg {
    SearchDebit(String),

    CreditKeyDown(web_sys::KeyboardEvent),
    CreditSelect(BookAccountId),
    SelectDebit(DebitOption),

    AmountInputMsg(ParsedInputMsg),

    Deposit,
    DepositSent {
        transaction_id: TransactionId,
    },
    DepositFailed {
        message_title: String,
        message_body: Option<String>,
    },

    IZettlePay(IZettlePayMsg),

    ShowNewMemberMenu,
    NewMember(NewMemberMsg),
    NewMemberCreated((MemberId, BookAccountId)),
}

#[derive(Clone, Debug)]
pub enum NewMemberMsg {
    FirstNameInput(String),
    LastNameInput(String),
    NicknameInput(String),
    AccNameInput(String),
    Create,
    HideMenu,
}

#[derive(Clone, Debug)]
pub enum DebitOption {
    IZettleEPay,
    OtherEPay,
    #[allow(dead_code)]
    Cash,
}

impl DepositionPage {
    pub fn new(global: &StateReady) -> Self {
        DepositionPage {
            debit: DebitOption::IZettleEPay,
            izettle_pay: IZettlePay::new(global),
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
            amount_input: ParsedInput::new("0")
                .with_error_message(strings::INVALID_MONEY_MESSAGE_LONG),
            new_member: None,
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
            DepositionMsg::SelectDebit(debit) => {
                self.debit = debit;
            }
            DepositionMsg::AmountInputMsg(msg) => {
                self.amount_input.update(msg);
            }
            DepositionMsg::Deposit => {
                if let Some((credit_acc, &amount)) =
                    self.credit_account.zip(self.amount_input.get_value())
                {
                    let transaction = NewTransaction {
                        description: Some(strings::TRANSACTION_DEPOSIT.to_string()),
                        amount,
                        credited_account: credit_acc,
                        debited_account: match self.debit {
                            DebitOption::Cash => global.master_accounts.cash_account_id,
                            DebitOption::IZettleEPay | DebitOption::OtherEPay => {
                                global.master_accounts.bank_account_id
                            }
                        },
                        bundles: vec![],
                    };

                    global.request_in_progress = true;

                    if let DebitOption::IZettleEPay = self.debit {
                        self.izettle_pay
                            .pay(transaction, orders_local.proxy(DepositionMsg::IZettlePay));
                    } else {
                        orders_local.perform_cmd(async move {
                            let result = async {
                                Request::new("/api/transaction")
                                    .method(Method::Post)
                                    .json(&transaction)?
                                    .fetch()
                                    .await?
                                    .json()
                                    .await
                            }
                            .await;
                            match result {
                                Ok(transaction_id) => {
                                    Some(DepositionMsg::DepositSent { transaction_id })
                                }
                                Err(e) => {
                                    error!("Failed to post transaction", e);
                                    Some(DepositionMsg::DepositFailed {
                                        message_title: strings::SERVER_ERROR.to_string(),
                                        message_body: Some(
                                            strings::POSTING_TRANSACTION_FAILED.to_string(),
                                        ),
                                    })
                                }
                            }
                        });
                    }
                }
            }

            DepositionMsg::DepositSent { .. } => {
                orders.send_msg(Msg::NotificationMessage(
                    NotificationMessage::ShowNotification {
                        duration_ms: 5000,
                        notification: Notification {
                            title: strings::DEPOSIT_COMPLETE.to_string(),
                            body: self
                                .amount_input
                                .get_value()
                                .map(|value| format!("{}:-", value)),
                        },
                    },
                ));

                global.request_in_progress = false;
                self.amount_input.set_value(0.into());
                self.credit_account = None;
                orders.send_msg(Msg::ReloadData);
            }

            DepositionMsg::DepositFailed {
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

            DepositionMsg::IZettlePay(msg) => {
                let reaction = match &msg {
                    &IZettlePayMsg::PaymentCompleted { transaction_id } => {
                        Some(DepositionMsg::DepositSent { transaction_id })
                    }
                    IZettlePayMsg::PaymentCancelled => Some(DepositionMsg::DepositFailed {
                        message_title: strings::PAYMENT_CANCELLED.to_string(),
                        message_body: None,
                    }),
                    IZettlePayMsg::Error(IZettlePayErr::PaymentFailed { reason, .. }) => {
                        Some(DepositionMsg::DepositFailed {
                            message_title: strings::PAYMENT_FAILED.to_string(),
                            message_body: Some(reason.clone()),
                        })
                    }
                    IZettlePayMsg::Error(IZettlePayErr::NoTransaction { .. }) => {
                        Some(DepositionMsg::DepositFailed {
                            message_title: strings::SERVER_ERROR.to_string(),
                            message_body: Some(strings::NO_PENDING_TRANSACTION.to_string()),
                        })
                    }
                    IZettlePayMsg::Error(IZettlePayErr::NetworkError { reason }) => {
                        Some(DepositionMsg::DepositFailed {
                            message_title: strings::SERVER_ERROR.to_string(),
                            message_body: Some(reason.clone()),
                        })
                    }
                    IZettlePayMsg::PollPendingPayment(_) => None,
                };

                if let Some(msg) = reaction {
                    orders_local.send_msg(msg);
                }

                self.izettle_pay
                    .update(msg, global, orders_local.proxy(DepositionMsg::IZettlePay));
            }

            DepositionMsg::ShowNewMemberMenu => {
                self.new_member = Some((String::new(), String::new(), String::new(), None));
            }

            DepositionMsg::NewMember(msg) => {
                if let Some((first_name, last_name, nickname, acc_name)) = &mut self.new_member {
                    match msg {
                        NewMemberMsg::FirstNameInput(input) => *first_name = input,
                        NewMemberMsg::LastNameInput(input) => *last_name = input,
                        NewMemberMsg::NicknameInput(input) => *nickname = input,
                        NewMemberMsg::AccNameInput(input) => {
                            *acc_name = if input == "" { None } else { Some(input) }
                        }
                        NewMemberMsg::HideMenu => {
                            self.new_member = None;
                        }
                        NewMemberMsg::Create => {
                            if first_name == "" || last_name == "" {
                                log!("Missing fields: `first_name` and `last_name` required");
                            } else {
                                let msg = (
                                    NewMember {
                                        first_name: first_name.clone(),
                                        last_name: last_name.clone(),
                                        nickname: match nickname.as_str() {
                                            "" => None,
                                            nickname => Some(nickname.to_string()),
                                        },
                                    },
                                    acc_name.clone().unwrap_or(generate_tillgodo_acc_name(
                                        first_name, nickname,
                                    )),
                                );
                                orders_local.perform_cmd(async move {
                                    let response = async {
                                        Request::new("/api/add_member_with_book_account")
                                            .method(Method::Post)
                                            .json(&msg)?
                                            .fetch()
                                            .await?
                                            .json()
                                            .await
                                    }
                                    .await;

                                    match response {
                                        Ok(data) => Some(DepositionMsg::NewMemberCreated(data)),
                                        Err(e) => {
                                            error!("Failed to create new member", e);
                                            None
                                        }
                                    }
                                });
                            }
                        }
                    }
                } else {
                    error!("Tried to edit new member fields while in incorrect state.");
                }
            }
            DepositionMsg::NewMemberCreated((member_id, book_account_id)) => {
                log!("New member ID: ", member_id);
                log!("New book account ID: ", book_account_id);
                self.new_member = None;
                orders.send_msg(Msg::ReloadData);
            }
        }
    }

    pub fn view(&self, global: &StateReady) -> Node<Msg> {
        if let Some((first_name, last_name, nickname, acc_name)) = &self.new_member {
            div![
                class![C.new_member_view],
                button![
                    class![C.border_on_focus, C.wide_button, C.my_2],
                    simple_ev(Ev::Click, NewMemberMsg::HideMenu),
                    strings::ABORT,
                ],
                input![
                    class![C.border_on_focus, C.mb_2],
                    attrs! {At::Placeholder => strings::FIRST_NAME},
                    attrs! {At::Value => first_name},
                    input_ev(Ev::Input, NewMemberMsg::FirstNameInput),
                ],
                input![
                    class![C.border_on_focus, C.mb_2],
                    attrs! {At::Placeholder => strings::LAST_NAME},
                    attrs! {At::Value => last_name},
                    input_ev(Ev::Input, NewMemberMsg::LastNameInput),
                ],
                input![
                    class![C.border_on_focus, C.mb_2],
                    attrs! {At::Placeholder => strings::NICKNAME},
                    attrs! {At::Value => nickname},
                    input_ev(Ev::Input, NewMemberMsg::NicknameInput),
                ],
                input![
                    class![C.border_on_focus, C.mb_2],
                    attrs! {At::Placeholder => strings::ACCOUNT_NAME},
                    attrs! {At::Value => match acc_name {
                        Some(name) => name.to_string(),
                        None => generate_tillgodo_acc_name(first_name, nickname),
                    }},
                    input_ev(Ev::Input, NewMemberMsg::AccNameInput),
                ],
                button![
                    class![C.border_on_focus, C.wide_button],
                    if first_name == "" || last_name == "" {
                        attrs! {At::Disabled => true}
                    } else {
                        attrs! {}
                    },
                    simple_ev(Ev::Click, NewMemberMsg::Create),
                    strings::CONFIRM,
                ],
            ]
            .map_msg(|msg| DepositionMsg::NewMember(msg))
        } else {
            div![
                class![C.deposit_page],
                div![
                    class![C.tillgodo_list, C.m_2],
                    input![
                        class![
                            C.tillgodolista_search_field,
                            C.rounded_lg,
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
                                strings::CHOOSE_TILLGODO_ACC
                            };
                            attrs! {At::Placeholder => s}
                        },
                        input_ev(Ev::Input, DepositionMsg::SearchDebit),
                        keyboard_ev(Ev::KeyDown, DepositionMsg::CreditKeyDown),
                    ],
                    button![
                        class![C.wide_button, C.border_on_focus, C.my_2],
                        simple_ev(Ev::Click, DepositionMsg::ShowNewMemberMenu),
                        "+",
                    ],
                    self.accs_search
                        .iter()
                        .map(|(_, _, acc, mem)| div![
                            if self.credit_account == Some(acc.id) {
                                class![C.border_highlight]
                            } else {
                                class![]
                            },
                            view_tillgodo(acc, mem, DepositionMsg::CreditSelect(acc.id)),
                        ])
                        .collect::<Vec<_>>(),
                ],
                div![
                    class![C.pay_method_select_box, C.m_2],
                    div![
                        class![C.flex, C.flex_row],
                        button![
                            if let DebitOption::IZettleEPay = self.debit {
                                class![C.debit_selected]
                            } else {
                                class![]
                            },
                            class![C.select_debit_button, C.border_on_focus, C.rounded_l_lg],
                            simple_ev(
                                Ev::Click,
                                DepositionMsg::SelectDebit(DebitOption::IZettleEPay)
                            ),
                            strings::IZETTLE,
                        ],
                        button![
                            if let DebitOption::OtherEPay = self.debit {
                                class![C.debit_selected]
                            } else {
                                class![]
                            },
                            class![C.select_debit_button, C.border_on_focus, C.rounded_r_lg],
                            simple_ev(
                                Ev::Click,
                                DepositionMsg::SelectDebit(DebitOption::OtherEPay),
                            ),
                            strings::OTHER_EPAY,
                        ],
                    ],
                    self.amount_input
                        .view(class![
                            C.rounded,
                            C.px_2,
                            C.my_2,
                            C.border_on_focus,
                            C.bg_gray_300,
                        ])
                        .map_msg(DepositionMsg::AmountInputMsg),
                    if global.request_in_progress {
                        button![
                            class![C.wide_button, C.border_on_focus],
                            attrs! {At::Disabled => true},
                            div![
                                class![C.lds_ripple],
                                attrs! { At::Style => "position: fixed; margin-top: -20px;" },
                                div![],
                                div![],
                            ],
                            strings::DEPOSIT,
                        ]
                    } else {
                        button![
                            class![C.wide_button, C.border_on_focus],
                            {
                                let disabled = match self.amount_input.get_value().copied() {
                                    None => true,
                                    Some(x) if x == 0.into() => true,
                                    Some(_) if self.credit_account.is_none() => true,
                                    Some(_) => false,
                                };

                                if disabled {
                                    attrs! { At::Disabled => true }
                                } else {
                                    attrs! {}
                                }
                            },
                            simple_ev(Ev::Click, DepositionMsg::Deposit),
                            strings::DEPOSIT,
                        ]
                    },
                    if let Some(_) = &self.izettle_pay.pending() {
                        div![class![C.wide_button_message], strings::WAITING_FOR_PAYMENT]
                    } else {
                        empty![]
                    },
                ],
            ]
        }
        .map_msg(|msg| Msg::DepositionMsg(msg))
    }
}

fn generate_tillgodo_acc_name(first_name: &str, nickname: &str) -> String {
    format!(
        "{}/{}",
        strings::TRANSACTION_TILLGODO,
        match nickname {
            "" => first_name,
            nn => nn,
        }
    )
}
