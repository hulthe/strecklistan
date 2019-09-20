use crate::app::{Msg, StateReady};
use crate::generated::css_classes::C;
use crate::views::view_tillgodo;
use seed::prelude::*;
use seed::*;

pub fn deposition_page(data: &StateReady) -> Node<Msg> {
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
            if data.deposition_credit_account.is_some() {
                class![C.debit_selected]
            } else {
                class![]
            },
            attrs! {At::Value => data.deposition_search_string},
            {
                let s = if let Some(acc_id) = data.deposition_credit_account {
                    data.book_accounts
                        .get(&acc_id)
                        .map(|acc| acc.name.as_str())
                        .unwrap_or("[MISSING]")
                } else {
                    "Välj Tillgodokonto"
                };
                attrs! {At::Placeholder => s}
            },
            input_ev(Ev::Input, Msg::DepositSearchDebit),
            keyboard_ev(Ev::KeyDown, Msg::DepositCreditKeyDown),
        ],
        div![
            class![C.flex, C.flex_row],
            button![
                if !data.deposition_use_cash {
                    class![C.debit_selected]
                } else {
                    class![]
                },
                class![C.select_debit_button, C.border_on_focus, C.rounded_bl_lg],
                simple_ev(Ev::Click, Msg::DepositUseCash(false),),
                "Swish",
            ],
            if !data.deposition_search_string.is_empty() {
                div![
                    class![C.tillgodo_drop_down],
                    data.deposition_search
                        .iter()
                        .map(|(_, _, acc, mem)| view_tillgodo(
                            acc,
                            mem,
                            Msg::DepositCreditSelect(acc.id)
                        ))
                        .collect::<Vec<_>>(),
                ]
            } else {
                div![]
            },
            button![
                if data.deposition_use_cash {
                    class![C.debit_selected]
                } else {
                    class![]
                },
                class![C.select_debit_button, C.border_on_focus, C.rounded_br_lg],
                simple_ev(Ev::Click, Msg::DepositUseCash(true),),
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
                C.bg_grey_light
            ],
            attrs! {At::Value => data.deposition_amount.to_string()},
            input_ev(Ev::Input, Msg::DepositSetAmount),
        ],
        button![
            class![C.wide_button],
            simple_ev(Ev::Click, Msg::Deposit),
            "Sätt in"
        ]
    ]
}
