use rocket_contrib::json::Json;
use rocket::{post, State};
use serde_derive::{Serialize, Deserialize};
use futures::lock::Mutex;
use crate::routes::rest::izettle::izettle_poll::IZettleState;
use crate::routes::rest::izettle::IZettleErrorResponse;
use uuid::Uuid;

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum BridgePayResult {
    PaymentOk(i32),
    NoPendingTransaction(IZettleErrorResponse)
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
    izettle_state: State<'_, Mutex<IZettleState>>,
) -> Json<BridgePayResult> {
    use BridgePayResult::*;

    match &*payment_response {
        PaymentResponse::TransactionPaid(transaction) => {
            let mut guard = izettle_state.inner().lock().await;

            if let Some(pending_transaction) = guard.pending_transaction.as_mut() {
                if transaction.reference == pending_transaction.reference {
                    pending_transaction.paid = true;
                    return Json(PaymentOk(42));
                }
            }

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
