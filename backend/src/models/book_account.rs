use serde::{Deserialize, Serialize};
use strecklistan_api::book_account::{BookAccount as BookAccountCommon, BookAccountType};

#[derive(Queryable, Serialize, Deserialize, Debug, PartialEq)]
pub struct BookAccount {
    pub id: i32,
    pub name: String,
    pub account_type: BookAccountType,
    pub creditor: Option<i32>,
}

impl From<BookAccount> for BookAccountCommon {
    fn from(val: BookAccount) -> Self {
        BookAccountCommon {
            id: val.id,
            name: val.name,
            account_type: val.account_type,
            creditor: val.creditor,
            balance: 0.into(),
        }
    }
}
