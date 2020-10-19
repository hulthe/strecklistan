use futures::lock::Mutex;
use rocket::{post, State};
use rocket_contrib::json::Json;
use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;

use crate::routes::rest::izettle::IZettleErrorResponse;

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum BridgePayResult {
    PaymentOk,
    NoPendingTransaction(IZettleErrorResponse),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SuccessfulPaymentResponse {
    pub reference: Uuid,
    pub amount: i64,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PaymentResponse {
    TransactionPaid(SuccessfulPaymentResponse),
    TransactionFailed {
        reason: String,
    },
    TransactionCanceled,
}


#[post("/izettle/bridge/payment_response", data = "<payment_response>")]
pub async fn complete_izettle_transaction(
    payment_response: Json<PaymentResponse>,
) -> Json<BridgePayResult> {
    use BridgePayResult::*;

    match &*payment_response {
        PaymentResponse::TransactionPaid(transaction) => {
            todo!("Reimplement this, move the data from the izettle tables to the normal tables and perhaps add an entry to an izettle data table")
        }
        PaymentResponse::TransactionFailed { reason } => {
            // Do shit
        }
        PaymentResponse::TransactionCanceled => {
            // Do shit
        }
    }

    Json(NoPendingTransaction(IZettleErrorResponse {
        message: "No pending transaction".to_string(),
    }))
}
