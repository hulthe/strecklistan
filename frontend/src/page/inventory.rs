use crate::app::Msg;
use crate::components::parsed_input::{ParsedInput, ParsedInputMsg};
use crate::generated::css_classes::C;
use crate::page::loading::Loading;
use crate::strings;
use seed::prelude::*;
use seed::*;
use seed_fetcher::{event, NotAvailable, ResourceStore, Resources};
use std::collections::HashMap;
use strecklistan_api::{
    currency::Currency,
    inventory::{
        InventoryBundle, InventoryBundleId, InventoryItemId, InventoryItemStock as InventoryItem,
    },
};

#[derive(Clone, Debug)]
pub enum InventoryMsg {
    ResFetched(event::Fetched),
    ResMarkDirty(event::MarkDirty),

    BundleNameInput(InventoryBundleId, ParsedInputMsg),
    BundlePriceInput(InventoryBundleId, ParsedInputMsg),
    BundleImgInput(InventoryBundleId, ParsedInputMsg),

    ItemNameInput(InventoryItemId, ParsedInputMsg),
    ItemPriceInput(InventoryItemId, ParsedInputMsg),
    ItemImgInput(InventoryItemId, ParsedInputMsg),
}

pub struct InventoryPage {
    bundle_rows: HashMap<InventoryBundleId, Row>,
    item_rows: HashMap<InventoryItemId, Row>,
}

#[derive(Resources)]
struct Res<'a> {
    #[url = "/api/inventory/bundles"]
    bundles: &'a HashMap<InventoryBundleId, InventoryBundle>,

    #[url = "/api/inventory/items"]
    items: &'a HashMap<InventoryItemId, InventoryItem>,
}

struct Row {
    name: ParsedInput<String>,
    price: ParsedInput<Currency>,
    image: ParsedInput<String>,
}

impl InventoryPage {
    pub fn new(rs: &ResourceStore, orders: &mut impl Orders<InventoryMsg>) -> Self {
        orders.subscribe(InventoryMsg::ResFetched);
        orders.subscribe(InventoryMsg::ResMarkDirty);
        let mut p = InventoryPage {
            item_rows: HashMap::new(),
            bundle_rows: HashMap::new(),
        };
        if let Ok(state) = Res::acquire(rs, orders) {
            p.rebuild_data(&state);
        }
        p
    }

    pub fn update(
        &mut self,
        msg: InventoryMsg,
        rs: &ResourceStore,
        orders: &mut impl Orders<Msg>,
    ) -> Result<(), NotAvailable> {
        let res = Res::acquire(rs, orders)?;

        match msg {
            InventoryMsg::ResFetched(_) => self.rebuild_data(&res),
            InventoryMsg::ResMarkDirty(_) => {}
            InventoryMsg::BundleNameInput(id, msg) => {
                self.bundle_rows
                    .get_mut(&id)
                    .map(|row| row.name.update(msg));
            }
            InventoryMsg::BundlePriceInput(id, msg) => {
                self.bundle_rows
                    .get_mut(&id)
                    .map(|row| row.price.update(msg));
            }
            InventoryMsg::BundleImgInput(id, msg) => {
                self.bundle_rows
                    .get_mut(&id)
                    .map(|row| row.image.update(msg));
            }
            InventoryMsg::ItemNameInput(id, msg) => {
                self.item_rows.get_mut(&id).map(|row| row.name.update(msg));
            }
            InventoryMsg::ItemPriceInput(id, msg) => {
                self.item_rows.get_mut(&id).map(|row| row.price.update(msg));
            }
            InventoryMsg::ItemImgInput(id, msg) => {
                self.item_rows.get_mut(&id).map(|row| row.image.update(msg));
            }
        }

        Ok(())
    }

    fn rebuild_data(&mut self, res: &Res) {
        fn err<T>(input: ParsedInput<T>) -> ParsedInput<T> {
            input.with_error_message(strings::INVALID_MONEY_MESSAGE_SHORT)
        }

        self.bundle_rows = res
            .bundles
            .iter()
            .map(|(&id, bundle)| {
                let row = Row {
                    name: ParsedInput::new_with_text(&bundle.name),
                    price: err(ParsedInput::new_with_value(Currency::from(bundle.price))),
                    image: ParsedInput::new_with_text(bundle.image_url.as_deref().unwrap_or("")),
                };
                (id, row)
            })
            .collect();

        self.item_rows = res
            .items
            .iter()
            .map(|(&id, item)| {
                let row = Row {
                    name: ParsedInput::new_with_text(&item.name),
                    price: err(match item.price {
                        Some(price) => ParsedInput::new_with_value(Currency::from(price)),
                        None => ParsedInput::new(),
                    }),
                    image: ParsedInput::new_with_text(item.image_url.as_deref().unwrap_or("")),
                };
                (id, row)
            })
            .collect();
    }

    pub fn view(&self, rs: &ResourceStore) -> Node<Msg> {
        let res = match Res::acquire_now(rs) {
            Ok(res) => res,
            Err(_) => return Loading::view(),
        };

        fn view_input<T>(input: &ParsedInput<T>) -> Node<ParsedInputMsg> {
            td![input.view(C![C.inventory_page_input])]
        }

        let bundle_row = |(&id, _bundle): (&InventoryBundleId, &InventoryBundle)| {
            let row = &self.bundle_rows[&id];
            tr![
                view_input(&row.name).map_msg(move |msg| InventoryMsg::BundleNameInput(id, msg)),
                view_input(&row.price).map_msg(move |msg| InventoryMsg::BundlePriceInput(id, msg)),
                view_input(&row.image).map_msg(move |msg| InventoryMsg::BundleImgInput(id, msg)),
            ]
        };

        let item_row = |(&id, _item): (&InventoryItemId, &InventoryItem)| {
            let row = &self.item_rows[&id];
            tr![
                view_input(&row.name).map_msg(move |msg| InventoryMsg::ItemNameInput(id, msg)),
                view_input(&row.price).map_msg(move |msg| InventoryMsg::ItemPriceInput(id, msg)),
                view_input(&row.image).map_msg(move |msg| InventoryMsg::ItemImgInput(id, msg)),
            ]
        };

        div![
            C![C.inventory_page],
            table![
                tr![td![
                    attrs! { At::ColSpan => 3 },
                    button![
                        C![C.wide_button],
                        attrs! { At::Disabled => true }, // TODO
                        "Spara ändringar",
                    ],
                ]],
                h1![strings::INVENTORY_BUNDLES],
                tr![th!["Namn"], th!["Pris"], th!["Bild"]],
                res.bundles.iter().map(bundle_row),
                h1![strings::INVENTORY_ITEMS],
                tr![th!["Namn"], th!["Pris"], th!["Bild"]],
                res.items.iter().map(item_row),
            ],
        ]
        .map_msg(Msg::Inventory)
    }
}
