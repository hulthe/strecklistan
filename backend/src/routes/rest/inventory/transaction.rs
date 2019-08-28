use crate::models::inventory::{object, relational};

use crate::database::DatabasePool;
use crate::util::status_json::StatusJson as SJ;
use diesel::prelude::*;
use itertools::Itertools;
use rocket::{get, State};
use rocket_contrib::json::Json;

#[get("/transactions")]
pub fn get_transactions(
    db_pool: State<DatabasePool>,
) -> Result<Json<Vec<object::Transaction>>, SJ> {
    let connection = db_pool.inner().get()?;

    use crate::schema::views::transactions_joined::dsl::*;

    let joined: Vec<relational::TransactionJoined> = transactions_joined
        .order_by(time.desc())
        .load(&connection)?;

    let transactions: Vec<object::Transaction> = joined
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

            let bundle = object::TransactionBundle {
                bundle_price: first.bundle_price,
                change: first.change,
                item_ids: std::iter::once(first.item_id)
                    .chain(xs.map(|x| x.item_id))
                    .collect(),
            };

            (transaction, bundle)
        })
        .group_by(|(t, _)| t.id)
        .into_iter()
        .map(|(_, mut xs)| {
            let (tf, bf) = xs.next().unwrap();

            object::Transaction {
                id: tf.id,
                amount: tf.amount,
                description: tf.description,
                time: tf.time,
                bundles: std::iter::once(bf).chain(xs.map(|(_, bf)| bf)).collect(),
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
