use crate::database::DatabasePool;
use crate::diesel::RunQueryDsl;
use crate::models::izettle_transaction::{
    IZettleTransaction, TRANSACTION_CANCELLED, TRANSACTION_FAILED, TRANSACTION_PAID,
};
use crate::models::transaction::relational;
use crate::models::transaction::relational::{
    NewTransaction, NewTransactionBundle, NewTransactionItem,
};
use crate::util::status_json::StatusJson as SJ;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::{Connection, ExpressionMethods, JoinOnDsl, PgConnection, QueryDsl};
use itertools::Itertools;
use log::info;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{post, State};
use serde::{Deserialize, Serialize};
use std::iter;

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PaymentResponse {
    TransactionPaid,
    TransactionFailed { reason: String },
    TransactionCancelled,
}

#[post(
    "/izettle/bridge/payment_response/<reference>",
    data = "<payment_response>"
)]
pub async fn complete_izettle_transaction(
    reference: i32,
    payment_response: Json<PaymentResponse>,
    db_pool: &State<DatabasePool>,
) -> Result<SJ, SJ> {
    let connection = db_pool.inner().get()?;

    connection.transaction::<_, SJ, _>(|| {
        let joined: Vec<(
            IZettleTransaction,
            Option<relational::TransactionBundle>,
            Option<relational::TransactionItem>,
        )> = {
            use crate::schema::tables::izettle_transaction::dsl::{
                id as transaction_id, izettle_transaction,
            };
            use crate::schema::tables::izettle_transaction_bundle::dsl::{
                id as bundle_id, izettle_transaction_bundle, transaction_id as bundle_trans_id,
            };
            use crate::schema::tables::izettle_transaction_item::dsl::{
                bundle_id as item_bundle_id, izettle_transaction_item,
            };
            izettle_transaction
                .left_join(izettle_transaction_bundle.on(bundle_trans_id.eq(transaction_id)))
                .left_join(izettle_transaction_item.on(item_bundle_id.eq(bundle_id)))
                .filter(transaction_id.eq(reference))
                .load(&connection)?
        };

        let grouped = joined
            .into_iter()
            .group_by(|(transaction, _, _)| transaction.id);

        let (izettle_transaction_id, mut transaction_rows) = match grouped.into_iter().next() {
            Some(group) => group,
            None => {
                return Err(SJ::new(
                    Status::BadRequest,
                    format!("No pending transaction with reference {}", reference),
                ));
            }
        };

        {
            // Delete the transaction from izettle_transaction
            use crate::schema::tables::izettle_transaction::dsl::{
                id as iz_id, izettle_transaction,
            };
            diesel::delete(izettle_transaction)
                .filter(iz_id.eq(izettle_transaction_id))
                .execute(&connection)?;
        }

        match &*payment_response {
            PaymentResponse::TransactionPaid => {
                // Get all the joined rows for the selected izettle transaction
                let (izettle_transaction, bundle0, item0) = transaction_rows.next().unwrap();

                // Insert transaction row from izettle_transaction to regular transaction table
                let new_transaction_id = {
                    let new_transaction: NewTransaction = NewTransaction {
                        description: izettle_transaction.description.clone(),
                        time: Some(izettle_transaction.time),
                        debited_account: izettle_transaction.debited_account,
                        credited_account: izettle_transaction.credited_account,
                        amount: izettle_transaction.amount,
                    };

                    use crate::schema::tables::transactions::dsl::*;
                    diesel::insert_into(transactions)
                        .values(new_transaction)
                        .returning(id)
                        .get_result(&connection)?
                };

                // Iterate over all the joined rows for each *bundle* in the transaction
                let bundles = iter::once((bundle0, item0))
                    .chain(transaction_rows.map(|(_, bundle, item)| (bundle, item)))
                    .filter_map(|(bundle, item)| bundle.map(|bundle| (bundle, item)))
                    .group_by(|(bundle, _)| bundle.id);
                for (_bundle_id, mut bundle_rows) in bundles.into_iter() {
                    let (bundle, item0) = bundle_rows.next().unwrap();

                    // Insert bundle row from izettle_transaction_bundle to regular bundle table
                    let new_bundle_id: i32 = {
                        let new_bundle: NewTransactionBundle = NewTransactionBundle {
                            transaction_id: new_transaction_id,
                            description: bundle.description.clone(),
                            price: bundle.price,
                            change: bundle.change,
                        };

                        use crate::schema::tables::transaction_bundles::dsl::*;
                        diesel::insert_into(transaction_bundles)
                            .values(new_bundle)
                            .returning(id)
                            .get_result(&connection)?
                    };

                    // Iterate over all the joined rows for each *item* in the bundle
                    let items = iter::once(item0)
                        .chain(bundle_rows.map(|(_, item)| item))
                        .flatten();
                    for item in items {
                        // Insert item row ...
                        let new_item: NewTransactionItem = NewTransactionItem {
                            bundle_id: new_bundle_id,
                            item_id: item.item_id,
                        };

                        use crate::schema::tables::transaction_items::dsl::*;
                        diesel::insert_into(transaction_items)
                            .values(new_item)
                            .execute(&connection)?;
                    }
                }

                // Mark the transaction in izettle_transaction as paid
                update_izettle_post_transaction(
                    izettle_transaction_id,
                    TRANSACTION_PAID.to_string(),
                    Some(new_transaction_id),
                    None,
                    &connection,
                )?;

                Ok(SJ::new(Status::Ok, "Transcation completed"))
            }
            PaymentResponse::TransactionFailed { reason } => {
                info!("IZettle failed due to: {}", reason);

                // Mark the transaction as failed
                update_izettle_post_transaction(
                    izettle_transaction_id,
                    TRANSACTION_FAILED.to_string(),
                    None,
                    Some(reason.clone()),
                    &connection,
                )?;

                Ok(SJ::new(Status::Ok, "Transcation cancelled with failure"))
            }
            PaymentResponse::TransactionCancelled => {
                // Mark the transaction as cancelled
                update_izettle_post_transaction(
                    izettle_transaction_id,
                    TRANSACTION_CANCELLED.to_string(),
                    None,
                    None,
                    &connection,
                )?;

                Ok(SJ::new(Status::Ok, "Transaction cancelled"))
            }
        }
    })
}

fn update_izettle_post_transaction(
    izettle_transaction_id: i32,
    status: String,
    transaction_id: Option<i32>,
    error: Option<String>,
    connection: &PooledConnection<ConnectionManager<PgConnection>>,
) -> Result<(), diesel::result::Error> {
    use crate::schema::tables::izettle_post_transaction::dsl::{
        error as err, izettle_post_transaction, izettle_transaction_id as iz_tran_id,
        status as stat, transaction_id as tran_id,
    };

    diesel::update(izettle_post_transaction)
        .filter(iz_tran_id.eq(izettle_transaction_id))
        .set((tran_id.eq(transaction_id), stat.eq(status), err.eq(error)))
        .execute(connection)?;

    Ok(())
}
