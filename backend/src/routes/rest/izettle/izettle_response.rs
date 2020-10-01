use crate::util::status_json::StatusJson as SJ;
use strecklistan_api::currency::Currency;
use rocket_contrib::json::Json;
use crate::models::transaction::object;
use rocket::{post, State};
use serde_derive::{Serialize};
use futures::lock::Mutex;
use rocket::http::Status;
use crate::routes::rest::izettle::izettle_poll::IZettleState;
use crate::routes::rest::izettle::IZettleErrorResponse;

#[derive(Serialize)]
pub enum BridgePayResult {
    PaymentOk(i32),
    NoPendingTransaction(IZettleErrorResponse)
}

use BridgePayResult::*;


#[post("/izettle/bridge/pay", data = "<transaction>")]
pub async fn complete_izettle_transaction(
    transaction: Json<Currency>,
    izettle_state: State<'_, Mutex<IZettleState>>,
) -> Json<BridgePayResult> {
    let mut guard = izettle_state.inner().lock().await;

    if let Some(pending_transaction) = guard.pending_transaction.as_mut() {
        if pending_transaction.amount == *transaction {
            pending_transaction.paid = true;
            return Json(PaymentOk(42));
        }
    }

    Json(NoPendingTransaction(IZettleErrorResponse {
        message: "No pending transaction".to_string(),
    }))
}