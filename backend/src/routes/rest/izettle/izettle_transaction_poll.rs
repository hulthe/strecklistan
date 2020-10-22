use futures::lock::Mutex;
use rocket::{get, State};
use rocket_contrib::json::Json;
use serde_derive::Serialize;

use ClientPollResult::*;

use diesel::{QueryDsl, JoinOnDsl, ExpressionMethods, Connection};
use crate::diesel::RunQueryDsl;
use crate::util::status_json::StatusJson as SJ;
use crate::routes::rest::izettle::IZettleErrorResponse;
use crate::models::izettle_transaction::IZettlePostTransaction;
use crate::database::DatabasePool;

#[derive(Clone, Serialize)]
pub struct IZettleResult {
    pub transaction_accepted: bool,
}

#[derive(Serialize)]
pub enum ClientPollResult {
    Paid,
    NotPaid,
    Canceled,
    NoTransaction(IZettleErrorResponse),
}

#[get("/izettle/client/poll/<izettle_transaction_id>")]
pub async fn poll_for_izettle(
    izettle_transaction_id: i32,
    db_pool: State<'_, DatabasePool>,
) -> Result<Json<ClientPollResult>, SJ> {
    let connection = db_pool.inner().get()?;

    let post_izettle_transaction = {
        use crate::schema::tables::izettle_post_transaction::dsl::{
            izettle_transaction_id as iz_id, izettle_post_transaction
        };

        izettle_post_transaction
            .filter(iz_id.eq(izettle_transaction_id))
            .first(&connection)
    };

    match post_izettle_transaction {
        Err(diesel::result::Error::NotFound) => {
            return Ok(Json(NotPaid))
        }
        Ok(IZettlePostTransaction{
            transaction_id: None,
            ..
           }) => {
            return Ok(Json(Canceled))
        }
        Ok(_) => {
            return Ok(Json(Paid))
        }
        Err(err) => {
            return Err(err.into())
        }
    }
}
