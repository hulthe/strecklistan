use crate::app::Msg;
use crate::generated::css_classes::C;
use itertools::Itertools;
use strecklistan_api::inventory::{
    InventoryBundle, InventoryBundleId, InventoryItemId, InventoryItemStock,
};
use seed::prelude::*;
use seed::*;

/// string: the string
/// highlight_chars: iterator over the indexes of highlighted characters in string
fn build_search_highlight_spans(
    string: &str,
    highlight_chars: impl IntoIterator<Item = usize>,
) -> Vec<Node<Msg>> {
    let mut highlights = highlight_chars.into_iter().peekable();
    string
        .chars()
        .enumerate()
        .group_by(|(i, _)| {
            // check if the next highlighted character is this one
            let highlighted = highlights.peek() == Some(i);
            if highlighted {
                highlights.next();
            }
            // group characters by whether they are highlighted or not
            highlighted
        })
        .into_iter()
        .map(|(highlighted, chars)| {
            let mut s = [0u8; 512];
            let mut len = 0;
            'outer: for (_, c) in chars {
                let mut buf = [0u8; 4];
                c.encode_utf8(&mut buf);
                for i in 0..c.len_utf8() {
                    if len >= 512 {
                        break 'outer; // FIXME: remove 512 byte cap
                    }
                    s[len] = buf[i];
                    len += 1;
                }
            }
            let s = std::str::from_utf8(&s[..]).expect("Invalid utf-8 string");

            if highlighted {
                span![class![C.text_green_200, C.underline], s]
            } else {
                span![s]
            }
        })
        .collect()
}

pub fn view_inventory_item(
    item: &InventoryItemStock,
    highlight_chars: impl IntoIterator<Item = usize>,
    add_item_ev: impl FnOnce(InventoryItemId, i32) -> Msg,
) -> Node<Msg> {
    div![
        class![C.inventory_item],
        simple_ev(Ev::Click, add_item_ev(item.id, 1)),
        p![
            class![C.inventory_item_header],
            build_search_highlight_spans(&item.name, highlight_chars),
        ],
        if let Some(image_url) = item.image_url.as_ref() {
            img![
                class![C.w_48, C.h_48, C.m_auto, C.my_1],
                attrs! { At::Src => image_url },
            ]
        } else {
            div![class![C.h_48, C.my_1]]
        },
        p![
            class![C.inventory_item_footer],
            class![match item.stock {
                n if n <= 0 => C.inventory_item_footer_red,
                n if n <= 10 => C.inventory_item_footer_yellow,
                _ => C.inventory_item_footer_green,
            }],
            format!("{} i lager.", item.stock),
        ],
    ]
}

pub fn view_inventory_bundle(
    bundle: &InventoryBundle,
    highlight_chars: impl IntoIterator<Item = usize>,
    add_bundle_ev: impl FnOnce(InventoryBundleId, i32) -> Msg,
) -> Node<Msg> {
    div![
        class![C.inventory_item],
        simple_ev(Ev::Click, add_bundle_ev(bundle.id, 1)),
        p![
            class![C.inventory_item_header],
            build_search_highlight_spans(&bundle.name, highlight_chars),
        ],
        if let Some(image_url) = bundle.image_url.as_ref() {
            img![
                class![C.w_48, C.h_48, C.m_auto, C.my_1],
                attrs! { At::Src => image_url },
            ]
        } else {
            div![class![C.h_48, C.my_1]]
        },
    ]
}
