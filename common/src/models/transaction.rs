use chrono::NaiveDateTime;
use std::collections::HashMap;

#[cfg(feature = "serde_impl")]
use serde_derive::{Deserialize, Serialize};

#[cfg_attr(feature = "serde_impl", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, PartialEq, Eq)]
pub struct NewTransaction {
    pub amount: i32,
    pub description: Option<String>,
    pub bundles: Vec<TransactionBundle>,
}

impl Default for NewTransaction {
    fn default() -> Self {
        NewTransaction {
            amount: 0,
            description: None,
            bundles: vec![],
        }
    }
}

#[cfg_attr(feature = "serde_impl", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct Transaction {
    pub id: i32,
    pub amount: i32,
    pub description: Option<String>,
    pub time: NaiveDateTime,
    pub bundles: Vec<TransactionBundle>,
}

impl PartialEq for Transaction {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Transaction {}

#[cfg_attr(feature = "serde_impl", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, PartialEq, Eq)]
pub struct TransactionBundle {
    pub bundle_price: Option<i32>,
    pub change: i32,
    pub item_ids: HashMap<i32, u32>,
}

impl TransactionBundle {
    pub fn items_eq(&self, other: &Self) -> bool {
        self.item_ids == other.item_ids
    }
}
