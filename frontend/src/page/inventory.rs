use crate::app::Msg;
use crate::components::parsed_input::{ParsedInput, ParsedInputMsg};
use crate::generated::css_classes::C;
use crate::notification_manager::{Notification, NotificationMessage};
use crate::page::loading::Loading;
use crate::strings;
use crate::util::simple_ev;
use seed::fetch;
use seed::prelude::*;
use seed::*;
use seed_fetcher::{event, NotAvailable, ResourceStore, Resources};
use std::collections::HashMap;
use strecklistan_api::{
    currency::Currency,
    inventory::{
        InventoryBundle, InventoryBundleId, InventoryItemId, InventoryItemStock as InventoryItem,
        NewInventoryBundle,
    },
};

#[derive(Clone, Debug)]
pub enum InventoryMsg {
    ResFetched(event::Fetched),
    ResMarkDirty(event::MarkDirty),

    DeleteBundle(InventoryBundleId),
    SaveBundles,
    ChangesSaved,
    ServerError(String),

    BundleInput(Field, InventoryBundleId, ParsedInputMsg),
    ItemInput(Field, InventoryItemId, ParsedInputMsg),
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

#[derive(Clone)]
struct Row {
    name: ParsedInput<String>,
    price: ParsedInput<Currency>,
    image: ParsedInput<String>,
}

#[derive(Clone, Debug)]
pub enum Field {
    Name,
    Price,
    Image,
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

        let mut orders_local = orders.proxy(Msg::Inventory);

        match msg {
            InventoryMsg::ResFetched(_) => self.rebuild_data(&res),
            InventoryMsg::ResMarkDirty(_) => {}
            InventoryMsg::DeleteBundle(id) => {
                orders_local.perform_cmd(async move {
                    let result: fetch::Result<()> = async {
                        Request::new(format!("/api/inventory/bundle/{}", id))
                            .method(Method::Delete)
                            .fetch()
                            .await?
                            .check_status()?;

                        Ok(())
                    }
                    .await;

                    match result {
                        Ok(_) => InventoryMsg::ChangesSaved,
                        Err(e) => {
                            error!("Failed to save inventory changes", e);
                            InventoryMsg::ServerError(format!("{:?}", e)) // TODO
                        }
                    }
                });
            }
            InventoryMsg::SaveBundles => {
                let bundle_rows = self.bundle_rows.clone();
                let bundles = res.bundles.clone();

                // TODO: save items
                //let item_rows = self.item_rows.clone();
                //let items = res.items.clone();

                orders_local.perform_cmd(async move {
                    let result: fetch::Result<()> = async {
                        for (id, bundle) in bundles {
                            let new_bundle = NewInventoryBundle {
                                name: bundle_rows[&id]
                                    .name
                                    .get_value()
                                    .unwrap_or(&bundle.name)
                                    .to_string(),
                                price: *bundle_rows[&id].price.get_value().unwrap_or(&bundle.price),
                                image_url: bundle_rows[&id]
                                    .image
                                    .get_value()
                                    .map(|s| s.to_string())
                                    .filter(|s| !s.is_empty()),
                                item_ids: bundle.item_ids, // TODO: allow changing items
                            };

                            Request::new(format!("/api/inventory/bundle/{}", id))
                                .method(Method::Put)
                                .json(&new_bundle)?
                                .fetch()
                                .await?
                                .check_status()?;
                        }

                        // TODO: save items
                        Ok(())
                    }
                    .await;

                    match result {
                        Ok(_) => InventoryMsg::ChangesSaved,
                        Err(e) => {
                            error!("Failed to save inventory changes", e);
                            InventoryMsg::ServerError(format!("{:?}", e)) // TODO
                        }
                    }
                });
            }
            InventoryMsg::ChangesSaved => {
                rs.mark_as_dirty(Res::bundles_url(), orders);
            }
            InventoryMsg::ServerError(message) => {
                orders.send_msg(Msg::Notification(NotificationMessage::ShowNotification {
                    duration_ms: 10000,
                    notification: Notification {
                        title: strings::SERVER_ERROR.to_string(),
                        body: Some(message),
                    },
                }));
            }
            InventoryMsg::BundleInput(field, id, msg) => {
                let row = self.bundle_rows.get_mut(&id);
                match field {
                    Field::Name => row.map(|row| row.name.update(msg)),
                    Field::Price => row.map(|row| row.price.update(msg)),
                    Field::Image => row.map(|row| row.image.update(msg)),
                };
            }
            InventoryMsg::ItemInput(field, id, msg) => {
                let row = self.item_rows.get_mut(&id);
                match field {
                    Field::Name => row.map(|row| row.name.update(msg)),
                    Field::Price => row.map(|row| row.price.update(msg)),
                    Field::Image => row.map(|row| row.image.update(msg)),
                };
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
        use Field::*;
        use InventoryMsg::{BundleInput, ItemInput};

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
                view_input(&row.name).map_msg(move |msg| BundleInput(Name, id, msg)),
                view_input(&row.price).map_msg(move |msg| BundleInput(Price, id, msg)),
                view_input(&row.image).map_msg(move |msg| BundleInput(Image, id, msg)),
                td![button![
                    "X",
                    simple_ev(Ev::Click, InventoryMsg::DeleteBundle(id)),
                ]],
            ]
        };

        let item_row = |(&id, _item): (&InventoryItemId, &InventoryItem)| {
            let row = &self.item_rows[&id];
            tr![
                view_input(&row.name).map_msg(move |msg| ItemInput(Name, id, msg)),
                view_input(&row.price).map_msg(move |msg| ItemInput(Price, id, msg)),
                view_input(&row.image).map_msg(move |msg| ItemInput(Image, id, msg)),
            ]
        };

        div![
            C![C.inventory_page],
            table![
                h1![strings::INVENTORY_BUNDLES],
                tr![th!["Namn"], th!["Pris"], th!["Bild"], th!["Radera"]],
                res.bundles.iter().map(bundle_row),
                tr![td![
                    attrs! { At::ColSpan => 4 },
                    button![
                        C![C.wide_button],
                        simple_ev(Ev::Click, InventoryMsg::SaveBundles),
                        "Spara ändringar",
                    ],
                ]],
                h1![strings::INVENTORY_ITEMS],
                tr![th!["Namn"], th!["Pris"], th!["Bild"]],
                res.items.iter().map(item_row),
            ],
        ]
        .map_msg(Msg::Inventory)
    }
}
