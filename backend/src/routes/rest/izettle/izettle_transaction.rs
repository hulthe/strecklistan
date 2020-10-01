use crate::util::status_json::StatusJson as SJ;
use rocket_contrib::json::Json;
use crate::models::transaction::object;
use rocket::{post, State};
use serde_derive::{Serialize};
use futures::lock::Mutex;
use rocket::http::Status;
use crate::routes::rest::izettle::izettle_poll::{IZettleState, TransactionResult};

#[post("/izettle/client/transaction", data = "<transaction>")]
pub async fn begin_izettle_transaction(
    transaction: Json<object::NewTransaction>,
    izettle_state: State<'_, Mutex<IZettleState>>,
) -> Json<i32> {

    let mut guard = izettle_state.inner().lock().await;
    guard.pending_transaction = Some(TransactionResult {
        amount: transaction.amount,
        paid: false,
    });

    Json(0)
}