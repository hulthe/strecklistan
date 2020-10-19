use rocket_contrib::json::Json;
use serde_derive::Serialize;
use rocket::{get, State};
use futures::lock::Mutex;
use crate::routes::rest::izettle::izettle_poll::IZettleState;
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

use ClientPollResult::*;

#[get("/izettle/client/poll")]
pub async fn poll_for_izettle(
    izettle_state: State<'_, Mutex<IZettleState>>
) -> Json<ClientPollResult> {
    let guard = izettle_state.inner().lock().await;
    if let Some(pending_transaction) = guard.pending_transaction.as_ref() {
        if pending_transaction.paid {
            return Json(Paid(IZettleResult {
                transaction_accepted: true,
            }));
        } else {
            return Json(NotPaid(IZettleErrorResponse {
                message: "transaction_not_paid".to_string(),
            }));
        }
    }

    return Json(NoTransaction(IZettleErrorResponse {
        message: "No transaction waiting for izettle result currently.".to_string()
    }))
}
