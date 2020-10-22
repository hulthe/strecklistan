use crate::database::DatabasePool;
use crate::diesel::RunQueryDsl;
use crate::models::izettle_transaction::IZettleTransactionPartial;
use crate::routes::rest::izettle::IZettleErrorResponse;
use crate::schema::tables::izettle_transaction::dsl::izettle_transaction;
use crate::util::StatusJson;
use diesel::result::Error;
use diesel::{ExpressionMethods, QueryDsl, QueryResult};
use rocket::{get, State};
use rocket_contrib::json::Json;
use serde_derive::Serialize;
use BridgePollResult::*;

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum BridgePollResult {
    PaymentOk(IZettleTransactionPartial),
    NoPendingTransaction(IZettleErrorResponse),
}

#[get("/izettle/bridge/poll")]
pub async fn poll_for_transaction(
    db_pool: State<'_, DatabasePool>,
) -> Result<Json<BridgePollResult>, StatusJson> {
    let connection = db_pool.inner().get()?;

    let transaction_res: QueryResult<IZettleTransactionPartial> = {
        use crate::schema::tables::izettle_transaction::dsl::{amount, id, time};

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

    // TODO: Should sleep for up to 5s before responding

    Ok(Json(PaymentOk(transaction_res?)))
}
