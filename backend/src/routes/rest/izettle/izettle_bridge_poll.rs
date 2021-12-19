use crate::database::DatabasePool;
use crate::diesel::RunQueryDsl;
use crate::models::izettle_transaction::IZettleTransactionPartial;
use crate::routes::rest::izettle::IZettleNotifier;
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
    notifier: &State<IZettleNotifier>,
    accept: SerAccept,
) -> Result<Ser<BridgePollResult>, StatusJson> {
    let connection = db_pool.inner().get()?;

    let notification = notifier.wait();

    let query_transaction = move || -> QueryResult<IZettleTransactionPartial> {
        use crate::schema::tables::izettle_transaction::dsl::{amount, id, time};

        izettle_transaction
            .order_by(time.asc())
            .select((id, amount))
            .first(&connection)
    };

    let transaction_res = query_transaction();
    if let Err(Error::NotFound) = transaction_res {
        if notification.await {
            let transaction_res = query_transaction();
            if let Err(Error::NotFound) = transaction_res {
                Ok(accept.ser(BridgePollResult::NoPendingTransaction))
            } else {
                Ok(accept.ser(BridgePollResult::PendingPayment(transaction_res?)))
            }
        } else {
            Ok(accept.ser(BridgePollResult::NoPendingTransaction))
        }
    } else {
        Ok(accept.ser(BridgePollResult::PendingPayment(transaction_res?)))
    }
}
