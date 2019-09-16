use chrono::NaiveDateTime;
use std::collections::HashMap;

#[cfg(feature = "serde_impl")]
use serde_derive::{Deserialize, Serialize};

#[cfg_attr(feature = "serde_impl", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, PartialEq, Eq)]
pub struct NewTransaction {
    pub description: Option<String>,
    pub bundles: Vec<TransactionBundle>,
    pub debited_account: i32,
    pub credited_account: i32,
    pub amount: i32,
}

#[cfg_attr(feature = "serde_impl", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct Transaction {
    pub id: i32,
    pub description: Option<String>,
    pub time: NaiveDateTime,
    pub bundles: Vec<TransactionBundle>,
    pub debited_account: i32,
    pub credited_account: i32,
    pub amount: i32,
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
    pub description: Option<String>,
    pub price: Option<i32>,
    pub change: i32,
    pub item_ids: HashMap<i32, u32>,
}
