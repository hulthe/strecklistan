#[cfg(feature = "diesel_impl")]
use diesel_derives::Queryable;

#[cfg(feature = "serde_impl")]
use serde_derive::{Deserialize, Serialize};

#[cfg(feature = "hash")]
use std::hash::{Hash, Hasher};

#[cfg_attr(feature = "serde_impl", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "diesel_impl", derive(Queryable))]
#[derive(Getters, Clone)]
pub struct InventoryItem {
    #[get = "pub"]
    id: i32,
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
#[derive(Getters, Clone)]
pub struct InventoryItemStock {
    #[get = "pub"]
    id: i32,
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
