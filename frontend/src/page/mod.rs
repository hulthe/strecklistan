pub mod analytics;
pub mod deposit;
pub mod inventory;
pub mod loading;
pub mod store;
pub mod transactions;

#[derive(Debug, Clone, Copy)]
pub enum Page {
    Analytics,
    Deposit,
    Inventory,
    NotFound,
    Store,
    TransactionHistory,
}
