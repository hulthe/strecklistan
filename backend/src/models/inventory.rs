//use crate::schema::tables::{transaction_bundles, transaction_items, transactions};
use serde_derive::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize, Debug, PartialEq)]
pub struct InventoryBundle {
    pub id: i32,
    pub name: String,
    pub price: i32,
}

#[derive(Queryable, Serialize, Deserialize, Debug, PartialEq)]
pub struct InventoryBundleItem {
    pub id: i32,
    pub bundle_id: i32,
    pub item_id: i32,
}
