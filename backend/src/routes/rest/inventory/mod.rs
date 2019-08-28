pub mod transaction;

use crate::database::DatabasePool;
use crate::models::inventory::InventoryItemStock;
use crate::util::status_json::StatusJson as SJ;
use diesel::prelude::*;
use rocket::{get, State};
use rocket_contrib::json::Json;

#[get("/inventory")]
pub fn get_inventory(db_pool: State<DatabasePool>) -> Result<Json<Vec<InventoryItemStock>>, SJ> {
    let connection = db_pool.inner().get()?;

    use crate::schema::views::inventory_stock::dsl::inventory_stock;
    Ok(Json(inventory_stock.load(&connection)?))
}
