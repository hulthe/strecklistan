use rocket::{post, State};
use rocket_contrib::json::Json;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::database::DatabasePool;
use crate::diesel::RunQueryDsl;
use crate::models::izettle_transaction::{TRANSACTION_PAID, TRANSACTION_FAILED, TRANSACTION_CANCELED};
use crate::models::transaction::relational::{NewTransaction, NewTransactionBundle, NewTransactionItem};
use crate::routes::rest::izettle::IZettleErrorResponse;
use crate::util::status_json::StatusJson as SJ;
use diesel::{Connection, ExpressionMethods, JoinOnDsl, QueryDsl, PgConnection};
use crate::models::transaction::relational;
use diesel::r2d2::{PooledConnection, ConnectionManager};

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum BridgePayResult {
    PaymentOk,
    Acknowledge,
    NoPendingTransaction(IZettleErrorResponse)
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PaymentResponse {
    TransactionPaid,
    TransactionFailed { reason: String },
    TransactionCanceled,
}

#[post(
    "/izettle/bridge/payment_response/<reference>",
    data = "<payment_response>"
)]
pub async fn complete_izettle_transaction(
    reference: i32,
    payment_response: Json<PaymentResponse>,
    db_pool: State<'_, DatabasePool>,
) -> Result<Json<BridgePayResult>, SJ> {
    use BridgePayResult::*;
    let connection = db_pool.inner().get()?;

    connection.transaction::<_, SJ, _>(|| {
        let joined: Vec<(
            relational::Transaction,
            Option<relational::TransactionBundle>,
            Option<relational::TransactionItem>,
        )> = {
            use crate::schema::tables::izettle_transaction_bundle::dsl::{
                id as bundle_id, izettle_transaction_bundle, transaction_id as bundle_trans_id,
            };
            use crate::schema::tables::izettle_transaction_item::dsl::{
                bundle_id as item_bundle_id, izettle_transaction_item,
            };
            use crate::schema::tables::izettle_transaction::dsl::{
                id as transaction_id, izettle_transaction
            };
            izettle_transaction
                .left_join(izettle_transaction_bundle.on(bundle_trans_id.eq(transaction_id)))
                .left_join(izettle_transaction_item.on(item_bundle_id.eq(bundle_id)))
                .filter(transaction_id.eq(reference))
                .load(&connection)?
        };

        let izettle_transaction_id: i32;

        match joined.first() {
            Some((izettle_tran, _, _)) => {
                izettle_transaction_id = izettle_tran.id;

                use crate::schema::tables::izettle_transaction::dsl::{
                    id as iz_id,
                    izettle_transaction,
                };
                diesel::delete(izettle_transaction)
                    .filter(iz_id.eq(izettle_transaction_id))
                    .execute(&connection)?;
            },
            None => {
                return Ok(Json(NoPendingTransaction(IZettleErrorResponse {
                    message: format
                    !("No transaction with id {}", reference)
                })));
            }
        }

        match &*payment_response {
            PaymentResponse::TransactionPaid => {
                // Map the old izettle bundle ids to the new bundle ids.
                let mut bundle_ids: HashMap<i32, i32> = HashMap::new();
                let mut transactions_id: i32 = -1;

                for (izettle_tran, bundle_opt, item_opt) in joined.iter() {
                    // Make sure to only insert the transaction once.
                    if transactions_id < 0 {
                        let new_trans: NewTransaction = NewTransaction {
                            description: izettle_tran.description.clone(),
                            time: Some(izettle_tran.time),
                            debited_account: izettle_tran.debited_account,
                            credited_account: izettle_tran.credited_account,
                            amount: izettle_tran.amount
                        };

                        use crate::schema::tables::transactions::dsl::*;
                        transactions_id = diesel::insert_into(transactions)
                            .values(new_trans)
                            .returning(id)
                            .get_result(&connection)?;
                    }

                    match bundle_opt {
                        Some(bundle) => {
                            // Only insert once per bundle.
                            if bundle_ids.contains_key(&bundle.id) == false {
                                let new_bundle: NewTransactionBundle = NewTransactionBundle {
                                    transaction_id: transactions_id,
                                    description: bundle.description.clone(),
                                    price: bundle.price,
                                    change: bundle.change,
                                };

                                use crate::schema::tables::transaction_bundles::dsl::*;
                                bundle_ids.insert(bundle.id, diesel::insert_into(transaction_bundles)
                                    .values(new_bundle)
                                    .returning(id)
                                    .get_result(&connection)?
                                );
                            }

                            match item_opt {
                                Some(item) => {
                                    let new_item: NewTransactionItem = NewTransactionItem {
                                        bundle_id: bundle_ids[&item.bundle_id],
                                        item_id: item.item_id
                                    };

                                    use crate::schema::tables::transaction_items::dsl::*;
                                    diesel::insert_into(transaction_items)
                                        .values(new_item)
                                        .execute(&connection)?;
                                }
                                None => {}
                            }
                        },
                        None => {}
                    }
                }

                update_izettle_post_transaction(izettle_transaction_id, TRANSACTION_PAID.to_string(), Some(transactions_id), &connection);
                Ok(Json(PaymentOk))
            }
            PaymentResponse::TransactionFailed {reason} => {
                // TODO: Use the reason for something.
                update_izettle_post_transaction(izettle_transaction_id, TRANSACTION_FAILED.to_string(), None, &connection);
                Ok(Json(Acknowledge))
            }
            PaymentResponse::TransactionCanceled => {
                update_izettle_post_transaction(izettle_transaction_id, TRANSACTION_CANCELED.to_string(), None, &connection);
                Ok(Json(Acknowledge))
            }
        }
    })
}

fn update_izettle_post_transaction(
    izettle_transaction_id: i32,
    status: String,
    transaction_id: Option<i32>,
    connection: &PooledConnection<ConnectionManager<PgConnection>>) {

    use crate::schema::tables::izettle_post_transaction::dsl::{
        izettle_post_transaction,
        transaction_id as tran_id,
        status as stat,
        izettle_transaction_id as iz_tran_id
    };

    diesel::update(izettle_post_transaction)
        .filter(iz_tran_id.eq(izettle_transaction_id))
        .set((
            tran_id.eq(transaction_id),
            stat.eq(status),
        ))
        .execute(connection);
}
