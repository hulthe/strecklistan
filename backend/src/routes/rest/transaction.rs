use crate::models::transaction::{object, relational};

use crate::database::DatabasePool;
use crate::util::status_json::StatusJson as SJ;
use diesel::prelude::*;
use itertools::Itertools;
use log::error;
use rocket::http::Status;
use rocket::{get, post, State};
use rocket_contrib::json::Json;
use std::collections::HashMap;

#[post("/transaction", data = "<transaction>")]
pub fn post_transaction(
    db_pool: State<DatabasePool>,
    transaction: Json<object::NewTransaction>,
) -> Result<Json<i32>, SJ> {
    let connection = db_pool.inner().get()?;

    let object::NewTransaction {
        amount,
        description,
        bundles,
    } = transaction.into_inner();

    let transaction = relational::NewTransaction {
        amount: amount,
        description: description,
        time: None,
    };

    connection.transaction::<_, SJ, _>(|| {
        println!("{:?}", transaction);
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
                bundle_price: bundle.bundle_price,
                change: bundle.change,
            };
            println!("{:?}", new_bundle);

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
            println!("{:?}", item_ids);

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

#[get("/transactions")]
pub fn get_transactions(
    db_pool: State<DatabasePool>,
) -> Result<Json<Vec<object::Transaction>>, SJ> {
    let connection = db_pool.inner().get()?;

    let joined: Vec<relational::TransactionJoined> = {
        use crate::schema::views::transactions_joined::dsl::*;
        transactions_joined
            .order_by(time.desc())
            .load(&connection)?
    };

    let joined: Vec<(relational::Transaction, Option<object::TransactionBundle>)> = joined
        .into_iter()
        .group_by(|x| x.bundle_id)
        .into_iter()
        .map(|(_, mut xs)| {
            let first = xs.next().unwrap();

            let transaction = relational::Transaction {
                id: first.id,
                amount: first.amount,
                description: first.description,
                time: first.time,
            };

            let bundle = match (first.bundle_id, first.change) {
                (Some(_bundle_id), Some(change)) => {
                    let mut item_ids = HashMap::new();
                    std::iter::once(first.item_id)
                        .chain(xs.map(|x| x.item_id))
                        .flatten()
                        .for_each(|item_id| *item_ids.entry(item_id).or_default() += 1);

                    Some(object::TransactionBundle {
                        bundle_price: first.bundle_price,
                        change: change,
                        item_ids,
                    })
                }
                (None, None) => None,
                (bundle_id, change) => {
                    error!(
                        "Invalid output from transactions_joined view.\n\
                         `bundle_id` or `change` was null, but not both.\n\
                         `bundle_id`: {:?}, `change`: {:?}\n\
                         `transaction_id`: {}",
                        bundle_id, change, transaction.id,
                    );

                    return Err(SJ::new(
                        Status::InternalServerError,
                        "Invalid output from transactions_joined view. See logs.",
                    ));
                }
            };

            Ok((transaction, bundle))
        })
        .collect::<Result<_, SJ>>()?;

    let transactions: Vec<object::Transaction> = joined
        .into_iter()
        .group_by(|(t, _)| t.id)
        .into_iter()
        .map(|(_, mut xs)| {
            let (tf, bf) = xs.next().unwrap();

            object::Transaction {
                id: tf.id,
                amount: tf.amount,
                description: tf.description,
                time: tf.time,
                bundles: std::iter::once(bf)
                    .chain(xs.map(|(_, bf)| bf))
                    .flatten()
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
