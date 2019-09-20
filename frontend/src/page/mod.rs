mod deposit;
mod store;
mod transactions;

use self::deposit::deposition_page;
use self::store::store_page;
use self::transactions::transactions_page;
use crate::app::{Msg, StateReady};
use crate::generated::css_classes::C;
use seed::prelude::*;
use seed::*;

#[derive(Debug, Clone, Copy)]
pub enum Page {
    NotFound,
    Root,
    Store,
    Deposit,
    TransactionHistory,
}

impl Page {
    pub fn view(&self, data: &StateReady) -> Node<Msg> {
        match self {
            Page::Store => store_page(data),
            Page::Deposit => deposition_page(data),
            Page::TransactionHistory => transactions_page(data),
            Page::Root | Page::NotFound => div![class![C.not_found_message, C.unselectable], "404"],
        }
    }
}
