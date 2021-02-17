use diesel::{ExpressionMethods, QueryDsl};
use rocket::http::Status;
use rocket::{get, State};
use log::error;
use rocket_contrib::json::Json;
use serde_derive::Serialize;
use strecklistan_api::izettle::IZettlePayment;
use crate::database::DatabasePool;
use crate::diesel::RunQueryDsl;
use crate::models::izettle_transaction::{
    IZettlePostTransaction, TRANSACTION_CANCELLED, TRANSACTION_FAILED, TRANSACTION_IN_PROGRESS,
    TRANSACTION_PAID,
};
use crate::util::status_json::StatusJson as SJ;
use crate::util::StatusJson;

#[derive(Clone, Serialize)]
pub struct IZettleResult {
    pub transaction_accepted: bool,
}

#[get("/izettle/client/poll/<izettle_transaction_id>")]
pub async fn poll_for_izettle(
    izettle_transaction_id: i32,
    db_pool: State<'_, DatabasePool>,
) -> Result<Json<IZettlePayment>, SJ> {
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
        Err(diesel::result::Error::NotFound) => Ok(Json(IZettlePayment::NoTransaction)),
        Ok(IZettlePostTransaction { status, .. }) if status == TRANSACTION_IN_PROGRESS => {
            Ok(Json(IZettlePayment::Pending))
        }
        Ok(IZettlePostTransaction { status, transaction_id, .. }) if status == TRANSACTION_PAID => {
            let transaction_id = transaction_id.ok_or_else(|| {
                error!("izettle_post_transaction {} marked as paid, not but transaction_id was None", izettle_transaction_id);
                SJ::new(Status::InternalServerError, "Internal Server Error")
            })?;
            Ok(Json(IZettlePayment::Paid { transaction_id }))
        }
        Ok(IZettlePostTransaction { status, .. }) if status == TRANSACTION_CANCELLED => {
            Ok(Json(IZettlePayment::Cancelled))
        }
        Ok(IZettlePostTransaction { status, error, .. }) if status == TRANSACTION_FAILED => {
            Ok(Json(IZettlePayment::Failed {
                reason: error.unwrap_or_else(|| "Unknown error".to_string()),
            }))
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
