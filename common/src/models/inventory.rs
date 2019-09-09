#[cfg(feature = "diesel_impl")]
use diesel_derives::Queryable;

#[cfg(feature = "serde_impl")]
use serde_derive::{Deserialize, Serialize};

#[cfg(feature = "hash")]
use std::hash::{Hash, Hasher};

#[cfg_attr(feature = "serde_impl", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "diesel_impl", derive(Queryable))]
#[derive(Clone)]
pub struct InventoryItem {
    pub id: i32,
    pub name: String,
    pub price: Option<i32>,
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
    pub id: i32,
    pub name: String,
    pub price: Option<i32>,
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
    pub item_id: i32,
}

#[cfg_attr(feature = "serde_impl", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct InventoryBundle {
    pub id: i32,
    pub name: String,
    pub price: i32,
    pub item_ids: Vec<i32>,
}

impl PartialEq for InventoryBundle {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for InventoryBundle {}
