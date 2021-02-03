use crate::app::Msg;
use crate::generated::css_classes::C;
use crate::views::{ParsedInput, ParsedInputMsg};
use seed::prelude::*;
use seed::*;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;
use strecklistan_api::{
    currency::Currency,
    inventory::{InventoryItemId, InventoryItemStock as InventoryItem},
    transaction::NewTransaction,
};

pub fn view_new_transaction(
    transaction: &NewTransaction,
    override_total: bool,
    enable_confirm_button: bool,
    confirm_button_message: Option<&str>,
    inventory: &HashMap<InventoryItemId, Rc<InventoryItem>>,
    transaction_set_bundle_change_ev: impl FnOnce(usize, i32) -> Msg + Clone + 'static,
    transaction_total: &ParsedInput<Currency>,
    transaction_total_input_ev: impl FnOnce(ParsedInputMsg) -> Msg + Clone + 'static,
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
                        class![C.new_transaction_bundle_amount_field, C.border_on_focus],
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
            let color = if override_total {
                "color: #762;"
            } else {
                "color: black;"
            };
            let attrs = attrs! {
                At::Style => color,
                At::Class => C.new_transaction_total_field,
                At::Class => C.border_on_focus,
            };
            transaction_total
                .view(attrs)
                .map_msg(transaction_total_input_ev)
        }],
        if enable_confirm_button {
            button![
                class![C.wide_button, C.border_on_focus],
                simple_ev(Ev::Click, confirm_purchase_ev),
                "Slutför Köp",
            ]
        } else {
            button![
                class![C.wide_button, C.border_on_focus],
                div![
                    class![C.lds_ripple],
                    style! {
                        St::Position => "absolute",
                        St::MarginTop => "-20px",
                    },
                    div![],
                    div![],
                ],
                attrs! { At::Disabled => true },
                "Slutför Köp",
            ]
        },
        if let Some(message) = confirm_button_message {
            div![class![C.wide_button_message], message,]
        } else {
            empty![]
        },
    ]
}
