use serde::{Deserialize, Serialize};
use strecklistan_api::book_account::{BookAccount as BookAccountCommon, BookAccountType};

#[derive(Queryable, Serialize, Deserialize, Debug, PartialEq)]
pub struct BookAccount {
    pub id: i32,
    pub name: String,
    pub account_type: BookAccountType,
    pub creditor: Option<i32>,
}

impl Into<BookAccountCommon> for BookAccount {
    fn into(self) -> BookAccountCommon {
        BookAccountCommon {
            id: self.id,
            name: self.name,
            account_type: self.account_type,
            creditor: self.creditor,
            balance: 0.into(),
        }
    }
}
