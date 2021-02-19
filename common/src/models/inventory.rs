use crate::currency::Currency;

#[cfg(feature = "diesel_impl")]
use diesel_derives::Queryable;

#[cfg(feature = "serde_impl")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "hash")]
use std::hash::{Hash, Hasher};

pub type InventoryItemId = i32;
pub type InventoryBundleId = i32;

#[cfg_attr(feature = "serde_impl", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "diesel_impl", derive(Queryable))]
#[derive(Clone)]
pub struct InventoryItem {
    pub id: InventoryItemId,
    pub name: String,
    pub price: Option<i32>,
    pub image_url: Option<String>,
}

impl PartialEq for InventoryItem {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for InventoryItem {}

#[cfg(feature = "hash")]
impl Hash for InventoryItem {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[cfg_attr(feature = "serde_impl", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "diesel_impl", derive(Queryable))]
#[derive(Clone)]
pub struct InventoryItemStock {
    pub id: InventoryItemId,
    pub name: String,
    pub price: Option<i32>,
    pub image_url: Option<String>,
    pub stock: i32,
}

impl PartialEq for InventoryItemStock {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for InventoryItemStock {}

#[cfg(feature = "hash")]
impl Hash for InventoryItemStock {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[cfg_attr(feature = "serde_impl", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "diesel_impl", derive(Queryable))]
#[derive(Clone)]
pub struct InventoryItemTag {
    pub tag: String,
    pub item_id: InventoryItemId,
}

#[cfg_attr(feature = "serde_impl", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct InventoryBundle {
    pub id: InventoryBundleId,
    pub name: String,
    pub price: Currency,
    pub image_url: Option<String>,
    pub item_ids: Vec<InventoryItemId>,
}

impl PartialEq for InventoryBundle {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for InventoryBundle {}
