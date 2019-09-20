use crate::app::{Msg, StateReady, StoreItem};
use crate::generated::css_classes::C;
use crate::views::{
    view_inventory_bundle, view_inventory_item, view_new_transaction, view_tillgodo,
};
use seed::prelude::*;
use seed::*;

pub fn store_page(data: &StateReady) -> Node<Msg> {
    let selected_bank_account =
        data.master_accounts.bank_account_id == data.transaction.debited_account;
    let selected_cash_account =
        data.master_accounts.cash_account_id == data.transaction.debited_account;

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
                    attrs! {At::Value => data.tillgodolista_search_string},
                    {
                        let s = if selected_cash_account || selected_bank_account {
                            "Tillgodolista"
                        } else {
                            data.book_accounts
                                .get(&data.transaction.debited_account)
                                .map(|acc| acc.name.as_str())
                                .unwrap_or("[MISSING]")
                        };
                        attrs! {At::Placeholder => s}
                    },
                    input_ev(Ev::Input, Msg::StoreSearchDebit),
                    keyboard_ev(Ev::KeyDown, Msg::StoreDebitKeyDown),
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
                            Msg::StoreDebitSelect(data.master_accounts.bank_account_id)
                        ),
                        "Swish",
                    ],
                    if !data.tillgodolista_search_string.is_empty() {
                        div![
                            class![C.tillgodo_drop_down],
                            data.tillgodolista_search
                                .iter()
                                .map(|(_, _, acc, mem)| view_tillgodo(
                                    acc,
                                    mem,
                                    Msg::StoreDebitSelect(acc.id)
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
                            Msg::StoreDebitSelect(data.master_accounts.cash_account_id)
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
                attrs! {At::Value => data.store_search_string},
                attrs! {At::Placeholder => "sök varor"},
                input_ev(Ev::Input, Msg::StoreSearchInput),
                keyboard_ev(Ev::KeyDown, Msg::StoreSearchKeyDown),
            ],
        ],
        div![
            class![C.inventory_view],
            data.inventory_search
                .iter()
                .map(|(_, matches, element)| match element {
                    StoreItem::Item(item) => {
                        view_inventory_item(item, matches.iter().map(|&(_, i)| i))
                    }
                    StoreItem::Bundle(bundle) => {
                        view_inventory_bundle(bundle, matches.iter().map(|&(_, i)| i))
                    }
                })
                .collect::<Vec<_>>(),
        ],
        view_new_transaction(
            &data.transaction,
            data.override_transaction_total,
            &data.inventory
        ),
    ]
}
