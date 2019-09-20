use crate::app::Msg;
use crate::generated::css_classes::C;
use laggit_api::book_account::BookAccount;
use laggit_api::inventory::InventoryItemStock as InventoryItem;
use laggit_api::transaction::{NewTransaction, Transaction};
use seed::prelude::*;
use seed::*;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

pub fn view_transaction(
    transaction: &Transaction,
    inventory: &HashMap<i32, Rc<InventoryItem>>,
    book_accounts: &HashMap<i32, Rc<BookAccount>>,
) -> Node<Msg> {
    div![
        class![C.transaction_view],
        p![
            span![format!("#{} ", transaction.id)],
            span![transaction
                .description
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or("Transaktion")],
            button![
                class![C.transaction_view_delete_button],
                simple_ev(Ev::Click, Msg::DeleteTransaction(transaction.id)),
                "X"
            ],
        ],
        p![
            class![C.mt_2],
            span!["Debet: "],
            span![
                class![C.font_bold],
                book_accounts
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
                book_accounts
                    .get(&transaction.credited_account)
                    .map(|acc| acc.name.as_str())
                    .unwrap_or("[MISSING]")
            ],
        ],
        transaction
            .bundles
            .iter()
            .map(|bundle| {
                let mut items = bundle.item_ids.keys().map(|id| &inventory[id]);

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
        p![format!("Totalt: {}:-", transaction.amount)],
    ]
}

pub fn view_new_transaction(
    transaction: &NewTransaction,
    override_total: bool,
    inventory: &HashMap<i32, Rc<InventoryItem>>,
    transaction_set_bundle_change_ev: impl FnOnce(usize, i32) -> Msg + Clone + 'static,
    transaction_total_input_ev: impl FnOnce(String) -> Msg + Clone + 'static,
    confirm_purchase_ev: Msg,
) -> Node<Msg> {
    div![
        class![C.new_transaction_view],
        transaction
            .bundles
            .iter()
            .enumerate()
            .rev() // display newest bundle first
            .map(|(bundle_index, bundle)| {
                let mut items = bundle.item_ids.keys().map(|id| &inventory[id]);

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
                    if bundle.change == 0 {
                        class![C.line_through, C.transaction_entry]
                    } else {
                        class![C.transaction_entry]
                    },
                    input![
                        class![C.new_transaction_bundle_amount_field],
                        attrs! { At::Value => -bundle.change },
                        attrs! { At::Type => "number" },
                        {
                            let ev = transaction_set_bundle_change_ev.clone();
                            input_ev(Ev::Input, move |input| {
                                ev(bundle_index, -input.parse().unwrap_or(0))
                            })
                        },
                    ],
                    span![class![C.transaction_entry_item_name], format!("x {}", name),],
                    span![
                        class![C.transaction_entry_item_price],
                        format!("{}:-", price),
                    ],
                ]
            })
            .collect::<Vec<_>>(),
        p![span!["Totalt: "], {
            let amount = transaction.amount.to_string();
            let _len = (amount.len() as f32) / 2.0 + 0.5;
            input![
                class![C.new_transaction_total_field, C.border_on_focus],
                attrs! { At::Value => &amount },
                attrs! { At::Type => "number" },
                if override_total {
                    attrs! { At::Style => "color: #762;" }
                } else {
                    attrs! { At::Style => "color: black;" }
                },
                input_ev(Ev::Input, transaction_total_input_ev),
            ]
        },],
        button![
            class![C.wide_button, C.border_on_focus],
            simple_ev(Ev::Click, confirm_purchase_ev),
            "Confirm Purchase",
        ],
    ]
}
