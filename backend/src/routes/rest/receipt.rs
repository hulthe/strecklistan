use crate::database::DatabasePool;
use crate::models::izettle_transaction::IZettlePostTransaction;
use crate::models::transaction::relational;
use crate::util::status_json::StatusJson as SJ;
use diesel::prelude::*;
use rocket::response::content::Html;
use rocket::{get, State};
use rocket_dyn_templates::Template;
use serde::Serialize;
use strecklistan_api::inventory::InventoryItem;

const RECEIPT_TEMPLATE_NAME: &str = "receipt";

#[derive(Debug, Serialize)]
struct ReceiptItem {
    pub product: String,
    pub count: u32,
    pub amount: f32,
}

impl ReceiptItem {
    fn new(product: String, count: u32, amount: i32) -> ReceiptItem {
        ReceiptItem {
            product,
            count,
            amount: amount as f32 / 100f32,
        }
    }
}

#[derive(Debug, Serialize)]
struct ReceiptTemplateData {
    date: String,
    products: Vec<ReceiptItem>,
    total: f32,
    card_type: String,
    card_number_last_four: String,
    payment_method: String,
    bank: String,
}

#[get("/receipt/<transaction_id>")]
pub async fn get_receipt(
    db_pool: &State<DatabasePool>,
    transaction_id: i32,
) -> Result<Html<Template>, SJ> {
    let connection = db_pool.inner().get()?;

    let transaction: relational::Transaction = {
        use crate::schema::tables::transactions::dsl::{deleted_at, id as trans_id, transactions};

        transactions
            .filter(deleted_at.is_null())
            .filter(trans_id.eq(transaction_id))
            .first(&connection)?
    };

    let izettle_transaction: IZettlePostTransaction = {
        use crate::schema::tables::izettle_post_transaction::dsl::{
            izettle_post_transaction, transaction_id as iz_trans_id,
        };

        izettle_post_transaction
            .filter(iz_trans_id.eq(transaction_id))
            .first(&connection)?
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
                bundle.change.abs() as u32,
                bundle.price.unwrap_or_default(),
            )
        })
        .collect();

    let mut trans_item_products: Vec<ReceiptItem> = items
        .into_iter()
        .filter(|(_, item)| item.is_some())
        .filter_map(|(bundle, item)| {
            use crate::schema::tables::inventory::dsl::{deleted_at, id as item_id, inventory};

            // Safe because of previous is_some() check.
            let item = item.unwrap();

            if let Ok(inventory_item) = inventory
                .filter(deleted_at.is_null())
                .filter(item_id.eq(item.item_id))
                .first::<InventoryItem>(&connection)
            {
                Some(ReceiptItem::new(
                    inventory_item
                        .name
                        .unwrap_or_else(|| bundle.description.unwrap_or_default()),
                    bundle.change.abs() as u32,
                    bundle
                        .price
                        .unwrap_or_else(|| inventory_item.price.unwrap_or_default()),
                ))
            } else {
                None
            }
        })
        .collect::<Vec<ReceiptItem>>();

    let mut receipt_items = bundle_only_products;
    receipt_items.append(&mut trans_item_products);

    let data = ReceiptTemplateData {
        date: transaction.time.format("%Y-%m-%d").to_string(),
        products: receipt_items,
        total: transaction.amount as f32 / 100f32,
        card_type: izettle_transaction.card_type.unwrap_or_default(),
        card_number_last_four: izettle_transaction.masked_pan.unwrap_or_default(),
        payment_method: izettle_transaction
            .card_payment_entry_mode
            .unwrap_or_default(),
        bank: izettle_transaction.card_issuing_bank.unwrap_or_default(),
    };

    Ok(Html(Template::render(RECEIPT_TEMPLATE_NAME, &data)))
}
