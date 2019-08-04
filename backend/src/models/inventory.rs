use diesel_derive_enum::DbEnum;
use chrono::NaiveDateTime;
use juniper_codegen::{GraphQLObject, GraphQLEnum};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize, GraphQLEnum, DbEnum)]
pub enum InventoryItemChange {
    Added,
    Removed,
}

#[derive(Queryable, GraphQLObject, Serialize, Deserialize, Debug, PartialEq)]
pub struct InventoryItem {
    pub name: String,
    pub price: Option<i32>,
}

#[derive(Queryable, GraphQLObject, Serialize, Deserialize, Debug, PartialEq)]
pub struct InventoryItemStock {
    pub name: String,
    pub stock: i32,
    pub price: Option<i32>,
}

#[derive(Queryable, GraphQLObject, Serialize, Deserialize, Debug, PartialEq)]
pub struct Transaction {
    pub id: i32,
    pub amount: i32,
    pub description: Option<String>,
    pub time: NaiveDateTime,
}

#[derive(Queryable, GraphQLObject, Serialize, Deserialize, Debug, PartialEq)]
pub struct TransactionItem {
    pub id: i32,
    pub transaction_id: i32,
    pub item_name: String,
    pub item_price: Option<i32>,
    pub change: InventoryItemChange,
}
