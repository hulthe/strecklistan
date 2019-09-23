pub mod accounting;
pub mod deposit;
pub mod store;
pub mod transactions;

#[derive(Debug, Clone, Copy)]
pub enum Page {
    NotFound,
    Root,
    Accounting,
    Store,
    Deposit,
    TransactionHistory,
}
