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

#[derive(Clone, Serialize)]
pub struct SuccessfulPaymentResponse {
    pub date: Date<Utc>,
    pub time: NaiveTime,
    pub recipe_nr: i32,
    pub amount: i64,
    pub fee: i64,
    pub payment_method: String,
    pub card_issues: String,
    pub staff_name: String,
    pub description: String,
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum PaymentResponse {
    Successful(SuccessfulPaymentResponse),
    Failed{
        reason: String,
    },
    Canceled,
}

use BridgePayResult::*;
use chrono::{Utc, Date, NaiveTime};


#[post("/izettle/bridge/payment_response", data = "<payment_response>")]
pub async fn complete_izettle_transaction(
    payment_response: Json<PaymentResponse>,
    izettle_state: State<'_, Mutex<IZettleState>>,
) -> Json<BridgePayResult> {
    match *payment_response {
        PaymentResponse::Successful(transaction) => {
            let mut guard = izettle_state.inner().lock().await;

            if let Some(pending_transaction) = guard.pending_transaction.as_mut() {
                if pending_transaction.amount == *pending_transaction.amount {
                    pending_transaction.paid = true;
                    return Json(PaymentOk(42));
                }
            }

        }
        PaymentResponse::Failed { reason } => {
            // Do shit
        }
        PaymentResponse::Canceled => {
            // Do shit
        }
    }

    Json(NoPendingTransaction(IZettleErrorResponse {
        message: "No pending transaction".to_string(),
    }))
}