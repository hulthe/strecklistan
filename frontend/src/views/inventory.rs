use crate::app::Msg;
use crate::generated::css_classes::C;
//use crate::models::inventory::InventoryItem;
use itertools::Itertools;
use laggit_api::inventory::{
    InventoryBundle, InventoryBundleId, InventoryItemId, InventoryItemStock,
};
use seed::prelude::*;
use seed::*;
use staticvec::*;

pub fn view_inventory_item(
    item: &InventoryItemStock,
    highlight_chars: impl IntoIterator<Item = usize>,
    add_item_ev: impl FnOnce(InventoryItemId, i32) -> Msg,
) -> Node<Msg> {
    let mut highlights = highlight_chars.into_iter().peekable();
    div![
        class![C.inventory_item],
        simple_ev(Ev::Click, add_item_ev(item.id, 1)),
        //simple_ev(Ev::Click, Msg::AddItemToNewTransaction(item.id, 1)),
        item.name
            .chars()
            .enumerate()
            .group_by(|(i, _)| if Some(i) == highlights.peek() {
                highlights.next();
                true
            } else {
                false
            })
            .into_iter()
            .map(|(highlighted, chars)| {
                let mut s = StaticVec::<u8, 512>::new();
                for (_, c) in chars {
                    let mut buf = [0u8; 4];
                    c.encode_utf8(&mut buf);
                    for i in 0..c.len_utf8() {
                        s.push(buf[i]);
                    }
                }
                let s = std::str::from_utf8(&s[..]).expect("Invalid utf-8 string");

                if highlighted {
                    span![class![C.text_green_900, C.underline], s]
                } else {
                    span![s]
                }
            })
            .collect::<Vec<_>>(),
        div![
            class![C.w_48, C.h_48,],
            // TODO: picture
        ],
        div![
            class![C.flex, C.text_center, C.text_2xl,],
            div![class![C.flex_1]],
            div![format!("{} i lager.", item.stock),],
        ],
    ]
}

pub fn view_inventory_bundle(
    bundle: &InventoryBundle,
    highlight_chars: impl IntoIterator<Item = usize>,
    add_bundle_ev: impl FnOnce(InventoryBundleId, i32) -> Msg,
) -> Node<Msg> {
    let mut highlights = highlight_chars.into_iter().peekable();
    div![
        class![C.inventory_bundle],
        simple_ev(Ev::Click, add_bundle_ev(bundle.id, 1)),
        bundle
            .name
            .chars()
            .enumerate()
            .group_by(|(i, _)| if Some(i) == highlights.peek() {
                highlights.next();
                true
            } else {
                false
            })
            .into_iter()
            .map(|(highlighted, chars)| {
                let mut s = StaticVec::<u8, 512>::new();
                for (_, c) in chars {
                    let mut buf = [0u8; 4];
                    c.encode_utf8(&mut buf);
                    for i in 0..c.len_utf8() {
                        s.push(buf[i]);
                    }
                }
                let s = std::str::from_utf8(&s[..]).expect("Invalid utf-8 string");

                if highlighted {
                    span![class![C.text_green_900, C.underline], s]
                } else {
                    span![s]
                }
            })
            .collect::<Vec<_>>(),
        div![
            class![C.w_48, C.h_48,],
            // TODO: picture
        ],
    ]
}
