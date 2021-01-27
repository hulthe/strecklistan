use diesel::{ExpressionMethods, QueryDsl};
use rocket::{get, State};
use rocket_contrib::json::Json;
use serde_derive::Serialize;

use ClientPollResult::*;

use crate::database::DatabasePool;
use crate::diesel::RunQueryDsl;
use crate::models::izettle_transaction::{
    IZettlePostTransaction, TRANSACTION_CANCELED, TRANSACTION_FAILED, TRANSACTION_IN_PROGRESS,
    TRANSACTION_PAID,
};
use crate::routes::rest::izettle::IZettleErrorResponse;
use crate::util::status_json::StatusJson as SJ;
use crate::util::StatusJson;
use rocket::http::Status;

#[derive(Clone, Serialize)]
pub struct IZettleResult {
    pub transaction_accepted: bool,
}

#[derive(Serialize)]
pub enum ClientPollResult {
    Paid,
    NotPaid,
    Canceled,
    Failed,
    NoTransaction(IZettleErrorResponse),
}

#[get("/izettle/client/poll/<izettle_transaction_id>")]
pub async fn poll_for_izettle(
    izettle_transaction_id: i32,
    db_pool: State<'_, DatabasePool>,
) -> Result<Json<ClientPollResult>, SJ> {
    let connection = db_pool.inner().get()?;

    let post_izettle_transaction: Result<IZettlePostTransaction, diesel::result::Error> = {
        use crate::schema::tables::izettle_post_transaction::dsl::{
            izettle_post_transaction, izettle_transaction_id as iz_id,
        };

        izettle_post_transaction
            .filter(iz_id.eq(izettle_transaction_id))
            .first(&connection)
    };

    match post_izettle_transaction {
        Err(diesel::result::Error::NotFound) => Ok(Json(NoTransaction(IZettleErrorResponse {
            message: format!("No transaction with id {}", izettle_transaction_id),
        }))),
        Ok(IZettlePostTransaction { status, .. }) if status == TRANSACTION_IN_PROGRESS => {
            Ok(Json(NotPaid))
        }
        Ok(IZettlePostTransaction { status, .. }) if status == TRANSACTION_PAID => Ok(Json(Paid)),
        Ok(IZettlePostTransaction { status, .. }) if status == TRANSACTION_CANCELED => {
            Ok(Json(Canceled))
        }
        Ok(IZettlePostTransaction { status, .. }) if status == TRANSACTION_FAILED => {
            Ok(Json(Failed))
        }
        Err(err) => Err(err.into()),
        Ok(transaction) => Err(StatusJson {
            status: Status {
                code: 500,
                reason: "invalid status",
            },
            description: format!(
                "Invalid status {}, perhaps add it to the match.",
                transaction.status
            ),
        }),
    }
}
