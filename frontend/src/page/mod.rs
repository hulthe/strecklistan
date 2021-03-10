pub mod analytics;
pub mod deposit;
pub mod loading;
pub mod store;
pub mod transactions;

#[derive(Debug, Clone, Copy)]
pub enum Page {
    NotFound,
    Store,
    Deposit,
    TransactionHistory,
    Analytics,
}
