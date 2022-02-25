DROP MATERIALIZED VIEW inventory_stock;

ALTER TABLE inventory
    ADD COLUMN deleted_at TIMESTAMP WITH TIME ZONE,
    DROP CONSTRAINT inventory_name_key;

----- snipped from 2021-06-03-184017_add_transaction_deleted_at/up.sql -----
CREATE MATERIALIZED VIEW inventory_stock AS
-- Take deleted_at of inventory into consideration
SELECT i.id, i.name, i.price, i.image_url, i.deleted_at, COALESCE(SUM(change), 0)::INTEGER AS stock
FROM inventory AS i
    LEFT JOIN transaction_items AS item ON item.item_id = i.id
    LEFT JOIN transaction_bundles AS bundle ON bundle.id = item.bundle_id
    LEFT JOIN transactions ON transactions.id = bundle.transaction_id
WHERE transactions.deleted_at IS NULL
GROUP BY i.id, i.name;
