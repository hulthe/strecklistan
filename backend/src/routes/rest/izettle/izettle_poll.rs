use diesel::{ExpressionMethods, QueryDsl, QueryResult};
use diesel::result::Error;
use futures::lock::Mutex;
use rocket::{get, State};
use rocket_contrib::json::Json;
use serde_derive::Serialize;
use uuid::Uuid;

use BridgePollResult::*;
use strecklistan_api::currency::Currency;

use crate::database::DatabasePool;
use crate::diesel::RunQueryDsl;
use crate::models::izettle_transaction::IZettleTransaction;
use crate::routes::rest::izettle::IZettleErrorResponse;
use crate::schema::tables::izettle_transaction::dsl::izettle_transaction;
use crate::util::StatusJson;

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum BridgePollResult {
    PaymentOk(IZettleTransaction),
    NoPendingTransaction(IZettleErrorResponse),
}

#[get("/izettle/bridge/poll")]
pub async fn poll_for_transaction(
    db_pool: State<'_, DatabasePool>
) -> Result<Json<BridgePollResult>, StatusJson> {
    let connection = db_pool.inner().get()?;

    let transaction_res: QueryResult<IZettleTransaction> = {
        use crate::schema::tables::izettle_transaction::dsl::{
            id, amount, time,
        };

        izettle_transaction
            .order_by(time.asc())
            .select((id, amount))
            .first(&connection)
    };

    if let Err(Error::NotFound) = transaction_res {
        return Ok(Json(NoPendingTransaction(IZettleErrorResponse {
            message: "No pending transaction".to_string(),
        })));
    }

    Ok(Json(BridgePollResult::PaymentOk(transaction_res?)))
}
