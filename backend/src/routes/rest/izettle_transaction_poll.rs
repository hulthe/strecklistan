use crate::util::status_json::StatusJson as SJ;
use rocket_contrib::json::Json;
use serde_derive::{Serialize};
use rocket::{get, State};
use rocket::http::Status;
use futures::lock::Mutex;


#[derive(Clone, Serialize)]
pub struct IZettleResult {
    pub transaction_accepted: bool,
}

#[get("/izettle/client/poll")]
pub async fn poll_for_izettle(izettle_state: State<'_, Mutex<IZettleResult>>) -> Result<Json<IZettleResult>, SJ> {
    let guard = izettle_state.inner().lock().await;
    if guard.transaction_accepted {
        return Ok(Json(IZettleResult {
            transaction_accepted: true,
        }));
    }
    Err(Status::Accepted)?
}