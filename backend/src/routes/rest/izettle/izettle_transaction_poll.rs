use futures::lock::Mutex;
use rocket::{get, State};
use rocket_contrib::json::Json;
use serde_derive::Serialize;

use ClientPollResult::*;

use crate::routes::rest::izettle::IZettleErrorResponse;

#[derive(Clone, Serialize)]
pub struct IZettleResult {
    pub transaction_accepted: bool,
}

#[derive(Serialize)]
pub enum ClientPollResult {
    NoTransaction(IZettleErrorResponse),
    NotPaid(IZettleErrorResponse),
    Paid(IZettleResult),
}

#[get("/izettle/client/poll")]
pub async fn poll_for_izettle(
) -> Json<ClientPollResult> {
    todo!("Reimplement this, check if the transaction id is added to the paid izettle transactions");

    return Json(NoTransaction(IZettleErrorResponse {
        message: "No transaction waiting for izettle result currently.".to_string()
    }));
}
