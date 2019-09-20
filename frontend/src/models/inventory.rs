use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::hash::{Hash, Hasher};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InventoryItem {
    id: i32,
    pub name: String,
    pub stock: i32,
}

impl InventoryItem {
    pub fn get_id(&self) -> i32 {
        self.id
    }
}

impl PartialEq for InventoryItem {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for InventoryItem {}

impl Hash for InventoryItem {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
