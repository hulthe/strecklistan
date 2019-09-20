use crate::app::{Msg, StateReady};
use crate::generated::css_classes::C;
use crate::views::view_transaction;
use seed::prelude::*;
use seed::*;

pub fn transactions_page(data: &StateReady) -> Node<Msg> {
    div![
        class![C.transactions_page],
        data.transaction_history
            .iter()
            .map(|t| view_transaction(t, &data.inventory, &data.book_accounts))
            .collect::<Vec<_>>(),
    ]
}
