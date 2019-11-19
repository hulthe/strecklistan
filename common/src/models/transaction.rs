use crate::currency::Currency;
use crate::models::book_account::BookAccountId;
use crate::models::inventory::InventoryItemId;
use chrono::NaiveDateTime;
use std::collections::HashMap;

#[cfg(feature = "serde_impl")]
use serde_derive::{Deserialize, Serialize};

pub type TransactionId = i32;

#[cfg_attr(feature = "serde_impl", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone, PartialEq, Eq)]
pub struct NewTransaction {
    pub description: Option<String>,
    pub bundles: Vec<TransactionBundle>,
    pub debited_account: BookAccountId,
    pub credited_account: BookAccountId,
    pub amount: Currency,
}

#[cfg_attr(feature = "serde_impl", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct Transaction {
    pub id: TransactionId,
    pub description: Option<String>,
    pub time: NaiveDateTime,
    pub bundles: Vec<TransactionBundle>,
    pub debited_account: BookAccountId,
    pub credited_account: BookAccountId,
    pub amount: Currency,
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
    pub price: Option<Currency>,
    pub change: i32,
    pub item_ids: HashMap<InventoryItemId, u32>,
}
