use crate::database::DatabasePool;
use crate::models::inventory::{InventoryBundle as InventoryBundleRel, InventoryBundleItem};
use crate::util::status_json::StatusJson as SJ;
use diesel::prelude::*;
use itertools::Itertools;
use rocket::{get, State};
use rocket_contrib::json::Json;
use strecklistan_api::inventory::InventoryBundle as InventoryBundleObj;
use strecklistan_api::inventory::{InventoryItemStock, InventoryItemTag};

#[get("/inventory/items")]
pub fn get_inventory(db_pool: State<DatabasePool>) -> Result<Json<Vec<InventoryItemStock>>, SJ> {
    let connection = db_pool.inner().get()?;

    use crate::schema::views::inventory_stock::dsl::inventory_stock;
    Ok(Json(inventory_stock.load(&connection)?))
}

#[get("/inventory/tags")]
pub fn get_tags(db_pool: State<DatabasePool>) -> Result<Json<Vec<InventoryItemTag>>, SJ> {
    let connection = db_pool.inner().get()?;

    use crate::schema::tables::inventory_tags::dsl::inventory_tags;
    Ok(Json(inventory_tags.load(&connection)?))
}

#[get("/inventory/bundles")]
pub fn get_inventory_bundles(
    db_pool: State<DatabasePool>,
) -> Result<Json<Vec<InventoryBundleObj>>, SJ> {
    let connection = db_pool.inner().get()?;

    use crate::schema::tables::inventory_bundle_items::dsl::{bundle_id, inventory_bundle_items};
    use crate::schema::tables::inventory_bundles::dsl::{id, inventory_bundles};

    let joined: Vec<(InventoryBundleRel, Option<InventoryBundleItem>)> = inventory_bundles
        .left_join(inventory_bundle_items.on(bundle_id.eq(id)))
        .load(&connection)?;

    let bundles = joined
        .into_iter()
        .group_by(|(bundle, _)| bundle.id)
        .into_iter()
        .map(|(_, mut elements)| {
            let (bundle, item) = elements.next().unwrap();
            InventoryBundleObj {
                id: bundle.id,
                name: bundle.name,
                price: bundle.price.into(),
                image_url: bundle.image_url,
                item_ids: std::iter::once(item)
                    .chain(elements.map(|(_, item)| item))
                    .flatten() // Remove None:s
                    .map(|item| item.item_id)
                    .collect(),
            }
        })
        .collect();

    Ok(Json(bundles))
}
