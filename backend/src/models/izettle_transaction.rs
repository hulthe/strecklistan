use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};
pub use strecklistan_api::transaction as object;

use crate::schema::tables::{
    izettle_transaction, izettle_transaction_bundle, izettle_transaction_item,
};

#[derive(Insertable, Serialize, Deserialize, Debug, PartialEq)]
#[table_name = "izettle_transaction"]
pub struct NewIZettleTransaction {
    pub description: Option<String>,
    pub time: Option<DateTime<Utc>>,
    pub debited_account: i32,
    pub credited_account: i32,
    pub amount: i32,
}

#[derive(Queryable, Serialize, Deserialize, Debug, PartialEq)]
pub struct IZettleTransactionPartial {
    pub id: i32,
    pub amount: i32,
}

#[derive(Queryable, Serialize, Deserialize, Debug, PartialEq)]
pub struct IZettleTransaction {
    pub id: i32,
    pub description: Option<String>,
    pub time: DateTime<Utc>,
    pub debited_account: i32,
    pub credited_account: i32,
    pub amount: i32,
}

#[derive(Queryable, Serialize, Deserialize, Debug, PartialEq)]
pub struct IZettlePostTransaction {
    pub id: i32,
    pub izettle_transaction_id: i32,
    pub transaction_id: Option<i32>,
}

#[derive(Insertable, Serialize, Deserialize, Debug, PartialEq)]
#[table_name = "izettle_transaction_bundle"]
pub struct NewIZettleTransactionBundle {
    pub transaction_id: i32,
    pub description: Option<String>,
    pub price: Option<i32>,
    pub change: i32,
}

#[derive(Insertable, Serialize, Deserialize, Debug, PartialEq)]
#[table_name = "izettle_transaction_item"]
pub struct NewIZettleTransactionItem {
    pub bundle_id: i32,
    pub item_id: i32,
}
