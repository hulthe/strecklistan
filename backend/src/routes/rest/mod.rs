pub mod book_account;
pub mod event;
pub mod inventory;
pub mod member;
pub mod transaction;

use rocket::get;

#[get("/version")]
pub fn get_api_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
