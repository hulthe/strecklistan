use serde_derive::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize, Debug, PartialEq)]
pub struct InventoryBundle {
    pub id: i32,
    pub name: String,
    pub price: i32,
    pub image_url: Option<String>,
}

#[derive(Queryable, Serialize, Deserialize, Debug, PartialEq)]
pub struct InventoryBundleItem {
    pub id: i32,
    pub bundle_id: i32,
    pub item_id: i32,
}
