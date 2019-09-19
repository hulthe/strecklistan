use crate::currency::Currency;

#[cfg(feature = "diesel_impl")]
use {diesel_derive_enum::DbEnum, diesel_derives::Queryable};

#[cfg(feature = "serde_impl")]
use serde_derive::{Deserialize, Serialize};

#[cfg_attr(feature = "serde_impl", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "diesel_impl", derive(DbEnum))]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BookAccountType {
    Expenses,
    Assets,
    Liabilities,
    Revenue,
}

#[cfg_attr(feature = "serde_impl", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "diesel_impl", derive(Queryable))]
#[derive(PartialEq, Clone)]
pub struct BookAccount {
    pub id: i32,
    pub name: String,
    pub account_type: BookAccountType,
    pub creditor: Option<i32>,
    pub balance: Currency,
}

#[cfg_attr(feature = "serde_impl", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(PartialEq, Clone)]
pub struct NewBookAccount {
    pub name: String,
    pub account_type: BookAccountType,
    pub creditor: Option<i32>,
}

#[cfg_attr(feature = "serde_impl", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct MasterAccounts {
    pub bank_account_id: i32,
    pub cash_account_id: i32,
    pub sales_account_id: i32,
    pub purchases_account_id: i32,
}

impl BookAccount {
    pub fn credit(&mut self, amount: Currency) {
        // TODO: Impl neg
        self.debit(-amount);
    }

    pub fn debit(&mut self, amount: Currency) {
        let amount = match self.account_type {
            BookAccountType::Expenses | BookAccountType::Assets => amount,
            BookAccountType::Liabilities | BookAccountType::Revenue => -amount,
        };

        self.balance += amount;
    }
}
