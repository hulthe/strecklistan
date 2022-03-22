use crate::currency::Currency;
use crate::models::book_account::BookAccountId;
use crate::models::inventory::InventoryItemId;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[cfg(feature = "serde_impl")]
use serde::{Deserialize, Serialize};

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
    pub time: DateTime<Utc>,
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

impl TransactionBundle {
    pub fn render<'a, F>(
        &'a self,
        get_item: &'a F,
    ) -> RenderedBundle<'a, impl Iterator<Item = RenderedItem<'a>>>
    where
        F: Fn(InventoryItemId) -> &'a str + 'a,
    {
        let mut items = self
            .item_ids
            .iter()
            .map(|(&id, &count)| (count, get_item(id)));

        RenderedBundle {
            name: self
                .description
                .as_deref()
                .or_else(|| items.next().map(|(_, name)| name))
                .unwrap_or("---"),

            price: self.price,
            change: self.change,
            items,
        }
    }
}

pub type RenderedItem<'a> = (u32, &'a str);

pub struct RenderedBundle<'a, I>
where
    I: Iterator<Item = RenderedItem<'a>> + 'a,
{
    pub name: &'a str,
    pub change: i32,
    pub price: Option<Currency>,
    pub items: I,
}
