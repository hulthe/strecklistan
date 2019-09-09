/// Object oriented data models - for consumers of the API.
pub use laggit_api::transaction as object;

/// Relational data models - as represented by the database.
pub mod relational {
    use crate::schema::tables::{transaction_bundles, transaction_items, transactions};
    use chrono::NaiveDateTime;
    use serde_derive::{Deserialize, Serialize};

    #[derive(Insertable, Serialize, Deserialize, Debug, PartialEq)]
    #[table_name = "transactions"]
    pub struct NewTransaction {
        pub amount: i32,
        pub description: Option<String>,
        pub time: Option<NaiveDateTime>,
    }

    #[derive(Queryable, Serialize, Deserialize, Debug, PartialEq)]
    pub struct Transaction {
        pub id: i32,
        pub amount: i32,
        pub description: Option<String>,
        pub time: NaiveDateTime,
    }

    #[derive(Insertable, Serialize, Deserialize, Debug, PartialEq)]
    #[table_name = "transaction_bundles"]
    pub struct NewTransactionBundle {
        pub transaction_id: i32,
        pub description: Option<String>,
        pub price: Option<i32>,
        pub change: i32,
    }

    #[derive(Queryable, Serialize, Deserialize, Debug, PartialEq)]
    pub struct TransactionBundle {
        pub id: i32,
        pub transaction_id: i32,
        pub description: Option<String>,
        pub price: Option<i32>,
        pub change: i32,
    }

    #[derive(Insertable, Serialize, Deserialize, Debug, PartialEq)]
    #[table_name = "transaction_items"]
    pub struct NewTransactionItem {
        pub bundle_id: i32,
        pub item_id: i32,
    }

    #[derive(Queryable, Serialize, Deserialize, Debug, PartialEq)]
    pub struct TransactionItem {
        pub id: i32,
        pub bundle_id: i32,
        pub item_id: i32,
    }
}
