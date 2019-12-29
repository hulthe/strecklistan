DROP MATERIALIZED VIEW inventory_stock;

ALTER TABLE inventory
    DROP COLUMN image_url;

ALTER TABLE inventory_bundles
    DROP COLUMN image_url;

-- -- snipped from 2019-03-08-190650_create_inventory_table/up.sql -- --
CREATE MATERIALIZED VIEW inventory_stock AS
SELECT i.id, i.name, i.price, COALESCE(SUM(change), 0)::INTEGER AS stock FROM inventory as i
    LEFT JOIN transaction_items AS item ON item.item_id = i.id
    LEFT JOIN transaction_bundles as bundle ON bundle.id = item.bundle_id
GROUP BY i.id, i.name;
