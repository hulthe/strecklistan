use crate::util::status_json::StatusJson as SJ;
use strecklistan_api::currency::Currency;
use rocket_contrib::json::Json;
use rocket::{get, State};
use serde_derive::{Serialize};
use futures::lock::Mutex;
use rocket::http::Status;

#[derive(Clone, Serialize)]
pub struct TransactionResult {
    pub amount: Currency,
}

pub struct IZettleState {
    pub pending_transaction: Option<TransactionResult>,
    pub last_transaction_accepted: bool,
}

#[get("/izettle/bridge/poll")]
pub async fn poll_for_transaction(
    izettle_state: State<'_, Mutex<IZettleState>>,
) -> Result<Json<TransactionResult>, SJ> {
    let guard = izettle_state.inner().lock().await;
    if let Some(pending_transaction) = guard.pending_transaction.clone() {
        if guard.last_transaction_accepted == false {
            return Ok(Json(pending_transaction));
        }
    }

    Err(Status::Accepted)?
}