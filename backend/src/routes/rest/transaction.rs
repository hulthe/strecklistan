use crate::models::transaction::{object, relational};

use crate::database::DatabasePool;
use crate::util::status_json::StatusJson as SJ;
use diesel::prelude::*;
use itertools::Itertools;
use rocket::{delete, get, post, State};
use rocket_contrib::json::Json;
use std::collections::HashMap;

#[post("/transaction", data = "<transaction>")]
pub fn post_transaction(
    db_pool: State<DatabasePool>,
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

    let transaction = relational::NewTransaction {
        description: description,
        time: None,
        debited_account,
        credited_account,
        amount: amount.into(),
    };

    connection.transaction::<_, SJ, _>(|| {
        let transaction_id = {
            use crate::schema::tables::transactions::dsl::*;
            diesel::insert_into(transactions)
                .values(transaction)
                .returning(id)
                .get_result(&connection)?
        };

        for bundle in bundles.into_iter() {
            let new_bundle = relational::NewTransactionBundle {
                transaction_id,
                description: bundle.description,
                price: bundle.price.map(|p| p.into()),
                change: bundle.change,
            };

            let bundle_id = {
                use crate::schema::tables::transaction_bundles::dsl::*;
                diesel::insert_into(transaction_bundles)
                    .values(&new_bundle)
                    .returning(id)
                    .get_result(&connection)?
            };

            let item_ids: Vec<_> = bundle
                .item_ids
                .into_iter()
                .flat_map(|(item_id, count)| std::iter::repeat(item_id).take(count as usize))
                .map(|item_id| relational::NewTransactionItem {
                    bundle_id: bundle_id,
                    item_id,
                })
                .collect();

            {
                use crate::schema::tables::transaction_items::dsl::*;
                diesel::insert_into(transaction_items)
                    .values(&item_ids)
                    .execute(&connection)?;
            }
        }

        Ok(Json(transaction_id))
    })
}

#[delete("/transaction/<transaction_id>")]
pub fn delete_transaction(
    db_pool: State<DatabasePool>,
    transaction_id: i32,
) -> Result<Json<i32>, SJ> {
    let connection = db_pool.inner().get()?;

    use crate::schema::tables::transactions::dsl::*;
    let deleted_id = diesel::delete(transactions)
        .filter(id.eq(transaction_id))
        .returning(id)
        .get_result(&connection)?;

    Ok(Json(deleted_id))
}

#[get("/transactions")]
pub fn get_transactions(
    db_pool: State<DatabasePool>,
) -> Result<Json<Vec<object::Transaction>>, SJ> {
    let connection = db_pool.inner().get()?;

    let joined: Vec<(
        relational::Transaction,
        Option<relational::TransactionBundle>,
        Option<relational::TransactionItem>,
    )> = {
        use crate::schema::tables::transaction_bundles::dsl::{
            id as bundle_id, transaction_bundles, transaction_id as bundle_transaction_id,
        };
        use crate::schema::tables::transaction_items::dsl::{
            bundle_id as item_bundle_id, transaction_items,
        };
        use crate::schema::tables::transactions::dsl::{id as transaction_id, time, transactions};
        transactions
            .left_join(transaction_bundles.on(transaction_id.eq(bundle_transaction_id)))
            .left_join(transaction_items.on(bundle_id.eq(item_bundle_id)))
            .order_by(time.desc())
            .order_by(transaction_id.desc())
            .load(&connection)?
    };

    let transactions: Vec<object::Transaction> = joined
        .into_iter()
        .group_by(|(tr, _, _)| tr.id)
        .into_iter()
        .map(|(_, mut xs)| {
            let (t0, b0, i0) = xs.next().unwrap();

            object::Transaction {
                id: t0.id,
                description: t0.description,
                time: t0.time,
                debited_account: t0.debited_account,
                credited_account: t0.credited_account,
                amount: t0.amount.into(),
                bundles: std::iter::once(b0.map(|b0| (b0, i0)))
                    .chain(xs.map(|(_, bx, ix)| bx.map(|bx| (bx, ix))))
                    .flatten()
                    .group_by(|(bx, _)| bx.id)
                    .into_iter()
                    .map(|(_, mut xs)| {
                        let (bundle, i0) = xs.next().unwrap();
                        let mut item_ids = HashMap::new();
                        std::iter::once(i0)
                            .chain(xs.map(|(_, ix)| ix))
                            .flatten()
                            .for_each(|i| *item_ids.entry(i.item_id).or_default() += 1);

                        object::TransactionBundle {
                            description: bundle.description,
                            price: bundle.price.map(|p| p.into()),
                            change: bundle.change,
                            item_ids,
                        }
                    })
                    .collect(),
            }
        })
        .collect();

    Ok(Json(transactions))
}

// Alternate implementation without joins
// Probably slower...
//#[get("/transactions")]
//pub fn get_transactions(
//    db_pool: State<DatabasePool>,
//) -> Result<Json<Vec<object::Transaction>>, SJ> {
//    let connection = db_pool.inner().get()?;
//
//    let transactions: Vec<relational::Transaction> = {
//        use crate::schema::tables::transactions::dsl::*;
//        transactions.order_by(time.desc()).load(&connection)?
//    };
//
//    let transactions: Vec<object::Transaction> = transactions
//        .into_iter()
//        .map(|transaction| {
//            let bundles: Vec<relational::TransactionBundle> = {
//                use crate::schema::tables::transaction_bundles::dsl::*;
//                transaction_bundles
//                    .filter(transaction_id.eq(transaction.id))
//                    .load(&connection)?
//            };
//
//            let bundles = bundles
//                .into_iter()
//                .map(|bundle| {
//                    let item_ids: Vec<i32> = {
//                        use crate::schema::tables::transaction_items::dsl::*;
//
//                        transaction_items
//                            .filter(bundle_id.eq(bundle.id))
//                            .select(item_id)
//                            .load(&connection)?
//                    };
//
//                    Ok(object::TransactionBundle {
//                        bundle_price: bundle.bundle_price,
//                        change: bundle.change,
//                        item_ids,
//                    })
//                })
//                .collect::<Result<Vec<object::TransactionBundle>, SJ>>()?;
//
//            Ok(object::Transaction {
//                id: transaction.id,
//                amount: transaction.amount,
//                description: transaction.description,
//                time: transaction.time,
//                bundles,
//            })
//        })
//        .collect::<Result<Vec<object::Transaction>, SJ>>()?;
//
//    Ok(Json(transactions))
//}
