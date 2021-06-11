use crate::database::DatabasePool;
use crate::diesel::RunQueryDsl;
use crate::models::izettle_transaction::IZettleTransactionPartial;
use crate::schema::tables::izettle_transaction::dsl::izettle_transaction;
use crate::util::ser::{Ser, SerAccept};
use crate::util::StatusJson;
use diesel::result::Error;
use diesel::{ExpressionMethods, QueryDsl, QueryResult};
use rocket::{get, State};
use serde::Serialize;

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum BridgePollResult {
    PendingPayment(IZettleTransactionPartial),
    NoPendingTransaction,
}

#[get("/izettle/bridge/poll")]
pub async fn poll_for_transaction(
    db_pool: &State<DatabasePool>,
    accept: SerAccept,
) -> Result<Ser<BridgePollResult>, StatusJson> {
    let connection = db_pool.inner().get()?;

    let transaction_res: QueryResult<IZettleTransactionPartial> = {
        use crate::schema::tables::izettle_transaction::dsl::{amount, id, time};

        izettle_transaction
            .order_by(time.asc())
            .select((id, amount))
            .first(&connection)
    };

    if let Err(Error::NotFound) = transaction_res {
        return Ok(accept.ser(BridgePollResult::NoPendingTransaction));
    }

    // Potential optimization: This function could sleep for up
    // to a few seconds if there is no pending transaction.
    // This way the latency between the server and the bridge would be lower.
    Ok(accept.ser(BridgePollResult::PendingPayment(transaction_res?)))
}
