use strecklistan_api::currency::Currency;
use rocket_contrib::json::Json;
use rocket::{get, State};
use serde_derive::Serialize;
use futures::lock::Mutex;
use crate::routes::rest::izettle::IZettleErrorResponse;
use uuid::Uuid;

#[derive(Clone, Serialize)]
pub struct TransactionResult {
    pub reference: Uuid,
    pub amount: Currency,
    pub paid: bool
}

pub struct IZettleState {
    pub pending_transaction: Option<TransactionResult>,
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum BridgePollResult {
    PaymentOk(TransactionResult),
    NoPendingTransaction(IZettleErrorResponse)
}

use BridgePollResult::*;

#[get("/izettle/bridge/poll")]
pub async fn poll_for_transaction(
    izettle_state: State<'_, Mutex<IZettleState>>,
) -> Json<BridgePollResult> {
    let guard = izettle_state.inner().lock().await;
    if let Some(pending_transaction) = guard.pending_transaction.clone() {
        return Json(PaymentOk(pending_transaction))
    }

    Json(NoPendingTransaction(IZettleErrorResponse {
        message: "No pending transaction".to_string(),
    }))
}
