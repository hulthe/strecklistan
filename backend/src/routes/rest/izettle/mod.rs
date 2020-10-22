pub mod izettle_poll;
pub mod izettle_response;
pub mod izettle_transaction;
pub mod izettle_transaction_poll;
use serde_derive::Serialize;

#[derive(Clone, Serialize)]
pub struct IZettleErrorResponse {
    pub message: String,
}
