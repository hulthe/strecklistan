use diesel::{Connection, RunQueryDsl};
use futures::lock::Mutex;
use rocket::{post, State};
use rocket_contrib::json::Json;
use uuid::Uuid;

use crate::database::DatabasePool;
use crate::models::transaction::{object, relational};
use crate::models::transaction::relational::NewTransaction;
use crate::schema::tables::izettle_transaction_bundle::dsl::izettle_transaction_bundle;
use crate::schema::tables::izettle_transaction_item::dsl::izettle_transaction_item;
use crate::util::status_json::StatusJson as SJ;
use crate::models::izettle_transaction::{NewIZettleTransaction, NewIZettleTransactionBundle, NewIZettleTransactionItem};

#[post("/izettle/client/transaction", data = "<transaction>")]
pub async fn begin_izettle_transaction(
    db_pool: State<'_, DatabasePool>,
    transaction: Json<object::NewTransaction>,
) -> Result<Json<i32>, SJ> {
    let connection = db_pool.inner().get()?;

    let object::NewTransaction {
        description,
        bundles,
        debited_account,
        credited_account,
        amount,
    } = transaction.into_inner();

    let transaction = NewIZettleTransaction {
        description,
        time: None,
        debited_account,
        credited_account,
        amount: amount.into(),
        paid: false,
    };

    connection.transaction::<_, SJ, _>(|| {
        let transaction_id = {
            use crate::schema::tables::izettle_transaction::dsl::*;
            diesel::insert_into(izettle_transaction)
                .values(transaction)
                .returning(id)
                .get_result(&connection)?
        };

        for bundle in bundles.into_iter() {
            let new_bundle = NewIZettleTransactionBundle {
                transaction_id,
                description: bundle.description,
                price: bundle.price.map(|p| p.into()),
                change: bundle.change,
            };

            let bundle_id = {
                use crate::schema::tables::izettle_transaction_bundle::dsl::*;
                diesel::insert_into(izettle_transaction_bundle)
                    .values(&new_bundle)
                    .returning(id)
                    .get_result(&connection)?
            };

            let item_ids: Vec<_> = bundle
                .item_ids
                .into_iter()
                .flat_map(|(item_id, count)| std::iter::repeat(item_id).take(count as usize))
                .map(|item_id| NewIZettleTransactionItem {
                    bundle_id,
                    item_id,
                })
                .collect();

            {
                use crate::schema::tables::izettle_transaction_item::dsl::*;
                diesel::insert_into(izettle_transaction_item)
                    .values(&item_ids)
                    .execute(&connection)?;
            }
        }

        Ok(Json(transaction_id))
    })
}
