use crate::currency::Currency;
use crate::models::member::MemberId;

#[cfg(feature = "diesel_impl")]
use {diesel_derive_enum::DbEnum, diesel_derives::Queryable};

#[cfg(feature = "serde_impl")]
use serde_derive::{Deserialize, Serialize};

#[cfg(feature = "hash")]
use std::hash::{Hash, Hasher};

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

pub type BookAccountId = i32;

#[cfg_attr(feature = "serde_impl", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[cfg_attr(feature = "diesel_impl", derive(Queryable))]
#[derive(Clone)]
pub struct BookAccount {
    pub id: BookAccountId,
    pub name: String,
    pub account_type: BookAccountType,
    pub creditor: Option<MemberId>,
    pub balance: Currency,
}

impl PartialEq for BookAccount {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for BookAccount {}

#[cfg(feature = "hash")]
impl Hash for BookAccount {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[cfg_attr(feature = "serde_impl", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(PartialEq, Clone)]
pub struct NewBookAccount {
    pub name: String,
    pub account_type: BookAccountType,
    pub creditor: Option<MemberId>,
}

#[cfg_attr(feature = "serde_impl", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Clone)]
pub struct MasterAccounts {
    pub bank_account_id: BookAccountId,
    pub cash_account_id: BookAccountId,
    pub sales_account_id: BookAccountId,
    pub purchases_account_id: BookAccountId,
}

impl BookAccount {
    pub fn credit_diff(&self, amount: Currency) -> Currency {
        self.debit_diff(-amount)
    }

    pub fn debit_diff(&self, amount: Currency) -> Currency {
        match self.account_type {
            BookAccountType::Expenses | BookAccountType::Assets => amount,
            BookAccountType::Liabilities | BookAccountType::Revenue => -amount,
        }
    }

    pub fn credit(&mut self, amount: Currency) {
        self.debit(-amount);
    }

    pub fn debit(&mut self, amount: Currency) {
        self.balance += self.debit_diff(amount);
    }
}
