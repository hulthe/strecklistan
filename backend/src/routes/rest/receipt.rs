use crate::database::DatabasePool;
use crate::models::izettle_transaction::IZettlePostTransaction;
use crate::models::transaction::relational;
use crate::util::status_json::StatusJson as SJ;
use diesel::prelude::*;
use rocket::response::content::Html;
use rocket::{get, State};
use rocket_dyn_templates::Template;
use serde::Serialize;
use strecklistan_api::currency::Currency;
use strecklistan_api::inventory::InventoryItem;
use strecklistan_api::transaction::TransactionId;

const RECEIPT_TEMPLATE_NAME: &str = "receipt";

#[derive(Debug, Serialize)]
struct ReceiptTemplateData {
    date: String,
    products: Vec<ReceiptItem>,
    total: f64,
    transaction_id: TransactionId,
    payment_meta: Vec<ReceiptMetaItem>,
}

#[derive(Debug, Serialize)]
struct ReceiptItem {
    pub product: String,
    pub count: i32,
    pub amount: f64,
}

#[derive(Debug, Serialize)]
struct ReceiptMetaItem {
    key: String,
    value: String,
}

impl ReceiptItem {
    fn new(product: String, count: i32, amount: Currency) -> ReceiptItem {
        ReceiptItem {
            product,
            count,
            amount: amount.as_f64(),
        }
    }
}

#[get("/receipt/<transaction_id>")]
pub async fn get_receipt(
    db_pool: &State<DatabasePool>,
    transaction_id: TransactionId,
) -> Result<Html<Template>, SJ> {
    let connection = db_pool.inner().get()?;

    let transaction: relational::Transaction = {
        use crate::schema::tables::transactions::dsl::{deleted_at, id as trans_id, transactions};

        transactions
            .filter(deleted_at.is_null())
            .filter(trans_id.eq(transaction_id))
            .first(&connection)?
    };

    let izettle: Option<IZettlePostTransaction> = {
        use crate::schema::tables::izettle_post_transaction::dsl::{
            izettle_post_transaction, transaction_id as iz_trans_id,
        };

        izettle_post_transaction
            .filter(iz_trans_id.eq(transaction_id))
            .first(&connection)
            .optional()?
    };

    let items: Vec<(
        relational::TransactionBundle,
        Option<relational::TransactionItem>,
    )> = {
        use crate::schema::tables::transaction_bundles::dsl::{
            id as bundle_id, transaction_bundles, transaction_id as bundle_trans_id,
        };
        use crate::schema::tables::transaction_items::dsl::{
            bundle_id as item_bundle_id, transaction_items,
        };
        transaction_bundles
            .filter(bundle_trans_id.eq(transaction.id))
            .left_join(transaction_items.on(bundle_id.eq(item_bundle_id)))
            .load(&connection)?
    };

    let bundle_only_products: Vec<ReceiptItem> = items
        .iter()
        .filter(|(_, item)| item.is_none())
        .map(|(bundle, _)| {
            ReceiptItem::new(
                bundle.description.clone().unwrap_or_default(),
                -bundle.change,
                bundle.price.unwrap_or(0).into(),
            )
        })
        .collect();

    let mut trans_item_products: Vec<ReceiptItem> = items
        .into_iter()
        .filter_map(|(bundle, item)| {
            use crate::schema::tables::inventory::dsl::{id as item_id, inventory};

            let item = item?;

            if let Ok(inventory_item) = inventory
                .filter(item_id.eq(item.item_id))
                .first::<InventoryItem>(&connection)
            {
                Some(ReceiptItem::new(
                    bundle.description.unwrap_or(inventory_item.name),
                    -bundle.change,
                    bundle.price.or(inventory_item.price).unwrap_or(0).into(),
                ))
            } else {
                None
            }
        })
        .collect::<Vec<ReceiptItem>>();

    let mut receipt_items = bundle_only_products;
    receipt_items.append(&mut trans_item_products);

    fn meta<K: ToString>(key: K, value: Option<String>) -> Option<ReceiptMetaItem> {
        value.map(|value| ReceiptMetaItem {
            key: key.to_string(),
            value,
        })
    }

    let mut payment_meta = vec![];

    if let Some(izettle) = izettle {
        payment_meta.append(&mut vec![
            izettle.card_type.and_then(|t| meta(t, izettle.masked_pan)),
            meta("Betalsätt", izettle.card_payment_entry_mode),
            meta("Bank", izettle.card_issuing_bank),
        ]);
    }

    let data = ReceiptTemplateData {
        date: transaction.time.format("%Y-%m-%d").to_string(),
        products: receipt_items,
        total: Currency::from(transaction.amount).as_f64(),
        transaction_id,
        payment_meta: payment_meta.into_iter().flatten().collect(),
    };

    Ok(Html(Template::render(RECEIPT_TEMPLATE_NAME, &data)))
}
