use futures::lock::Mutex;
use rocket::{post, State};
use rocket_contrib::json::Json;
use serde_derive::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

use crate::routes::rest::izettle::IZettleErrorResponse;
use crate::util::status_json::StatusJson as SJ;
use crate::models::transaction::relational;
use crate::models::izettle_transaction::{IZettleTransaction, NewIZettleTransactionBundle};
use diesel::{QueryDsl, JoinOnDsl, ExpressionMethods, Connection};
use crate::schema::tables::izettle_transaction_bundle::dsl::izettle_transaction_bundle;
use crate::schema::tables::izettle_transaction_item::dsl::izettle_transaction_item;
use crate::database::DatabasePool;
use crate::diesel::RunQueryDsl;
use crate::schema::tables::izettle_transaction::dsl::izettle_transaction;
use crate::models::transaction::relational::{TransactionBundle, TransactionItem};
use crate::schema::tables::transaction_bundles::dsl::transaction_bundles;

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum BridgePayResult {
    PaymentOk,
    NoPendingTransaction(IZettleErrorResponse),
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PaymentResponse {
    TransactionPaid{
        reference: i32
    },
    TransactionFailed {
        reason: String,
    },
    TransactionCanceled,
}


#[post("/izettle/bridge/payment_response", data = "<payment_response>")]
pub async fn complete_izettle_transaction(
    payment_response: Json<PaymentResponse>,
    db_pool: State<'_, DatabasePool>,
) -> Result<Json<BridgePayResult>, SJ> {
    use BridgePayResult::*;

    match &*payment_response {
        PaymentResponse::TransactionPaid { reference } => {
            let connection = db_pool.inner().get()?;

            connection.transaction::<_, SJ, _>(|| {

                // Select the transaction with the given reference
                // Move the transaction, item/bundle to the standard tables.

                // let joined: Vec<(
                //     IZettleTransaction,
                //     Option<relational::TransactionBundle>,
                //     Option<relational::TransactionItem>,
                // )> = {
                //     use crate::schema::tables::izettle_transaction_bundle::dsl::{
                //         id as bundle_id, izettle_transaction_bundle, transaction_id as bundle_transaction_id,
                //     };
                //     use crate::schema::tables::izettle_transaction_item::dsl::{
                //         bundle_id as item_bundle_id, izettle_transaction_item,
                //     };
                //     use crate::schema::tables::izettle_transaction::dsl::{id as transaction_id, time, izettle_transaction};
                //     izettle_transaction
                //         .left_join(izettle_transaction_bundle.on(transaction_id.eq(bundle_transaction_id)))
                //         .left_join(izettle_transaction_item.on(bundle_id.eq(item_bundle_id)))
                //         .filter(transaction_id.eq(*reference))
                //         .load(&connection)?
                // };

                let cached_transaction: IZettleTransaction = {
                    use crate::schema::tables::izettle_transaction::dsl::{id as transaction_id, izettle_transaction};
                    let result = izettle_transaction
                        .filter(transaction_id.eq(*reference))
                        .first(&connection);

                    match result {
                        Err(diesel::result::Error::NotFound) => {
                            return Ok(Json(NoPendingTransaction(IZettleErrorResponse {
                                message: "No pending transaction".to_string(),
                            })));
                        }
                        _ => result?
                    }
                };

                let cached_bundles: Vec<TransactionBundle> = {
                    use crate::schema::tables::izettle_transaction_bundle::dsl::{
                        izettle_transaction_bundle,
                        transaction_id as bundle_transaction_id
                    };

                    izettle_transaction_bundle
                        .filter(bundle_transaction_id.eq(*reference))
                        .load(&connection)?
                };

                let cached_items: Vec<TransactionItem> = {
                    use crate::schema::tables::izettle_transaction_item::dsl::{
                        id as item_db_id,
                        bundle_id as item_bundle_id,
                        item_id,
                        izettle_transaction_item,
                    };

                    use crate::schema::tables::izettle_transaction_bundle::dsl::{
                        izettle_transaction_bundle,
                        transaction_id as bundle_transaction_id,
                    };

                    izettle_transaction_item
                        .left_join(izettle_transaction_bundle.on(bundle_transaction_id.eq(item_bundle_id)))
                        .select((item_db_id, item_bundle_id, item_id))
                        .load(&connection)?
                };

                let trans_id: i32 = {
                    use crate::schema::tables::transactions::dsl::*;
                    diesel::insert_into(transactions)
                        .values((
                            description.eq(cached_transaction.description),
                            debited_account.eq(cached_transaction.debited_account),
                            credited_account.eq(cached_transaction.credited_account),
                            amount.eq(cached_transaction.amount),
                        ))
                        .returning(id)
                        .get_result(&connection)?
                };

                // Map izettle_transaction_bundle ids to transaction_bundle ids.
                let mut bundle_ids: HashMap<i32, i32> = HashMap::new();

                for bundle in cached_bundles.iter() {
                    use crate::schema::tables::transaction_bundles::dsl::*;
                    let bundle_id = diesel::insert_into(transaction_bundles)
                        .values((
                            transaction_id.eq(trans_id),
                            description.eq(&bundle.description),
                            price.eq(bundle.price),
                            change.eq(bundle.change),
                        ))
                        .returning(id)
                        .get_result(&connection)?;
                    bundle_ids.insert(bundle.id, bundle_id);
                }

                let mut item_ids: Vec<i32> = Vec::new();

                for item in cached_items.iter() {
                    use crate::schema::tables::transaction_items::dsl::*;
                    let izettle_item_id = diesel::insert_into(transaction_items)
                        .values((
                            bundle_id.eq(bundle_ids[&item.id]),
                            item_id.eq(&item.item_id)
                        ))
                        .returning(id)
                        .get_result(&connection)?;
                    item_ids.push(izettle_item_id);
                }

                // Add data to the izettle post transactions table
                {
                    use crate::schema::tables::izettle_post_transaction::dsl::{izettle_transaction_id, transaction_id as post_id, izettle_post_transaction};
                    diesel::insert_into(izettle_post_transaction)
                        .values((
                            izettle_transaction_id.eq(cached_transaction.id),
                            post_id.eq(trans_id)
                        ))
                        .execute(&connection)?;
                }

                // Remove the entries from the izettle tables.
                for item in cached_items.into_iter() {
                    use crate::schema::tables::izettle_transaction_item::dsl::{id as item_id, izettle_transaction_item};
                    diesel::delete(izettle_transaction_item
                        .filter(item_id.eq(item.id)))
                        .execute(&connection)?;
                }

                for bundle in cached_bundles.into_iter() {
                    use crate::schema::tables::izettle_transaction_bundle::dsl::{id as bundle_id, izettle_transaction_bundle};
                    diesel::delete(izettle_transaction_bundle
                        .filter(bundle_id.eq(bundle.id)))
                        .execute(&connection)?;
                }

                {
                    use crate::schema::tables::izettle_transaction::dsl::{id as transaction_id, izettle_transaction};
                    diesel::delete(izettle_transaction
                        .filter(transaction_id.eq(cached_transaction.id)))
                        .execute(&connection)?;
                }

                Ok(Json(PaymentOk))
            })
        }
        PaymentResponse::TransactionFailed { reason } => {
            // Do shit
            todo!("Implement")
        }
        PaymentResponse::TransactionCanceled => {
            // Do shit
            todo!("Implement")
        }
    }
}
