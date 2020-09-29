pub mod book_account;
pub mod event;
pub mod inventory;
pub mod member;
pub mod transaction;
pub mod izettle_poll;
pub mod izettle_transaction;
pub mod izettle_transaction_poll;

use rocket::get;

#[get("/version")]
pub fn get_api_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
