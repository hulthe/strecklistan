pub mod analytics;
pub mod deposit;
pub mod store;
pub mod transactions;

#[derive(Debug, Clone, Copy)]
pub enum Page {
    NotFound,
    Root,
    Store,
    Deposit,
    TransactionHistory,
    Analytics,
}
