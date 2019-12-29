ALTER TABLE inventory
    ADD COLUMN image_url TEXT;

ALTER TABLE inventory_bundles
    ADD COLUMN image_url TEXT;

DROP MATERIALIZED VIEW inventory_stock;

-- -- snipped from 2019-03-08-190650_create_inventory_table/up.sql -- --
CREATE MATERIALIZED VIEW inventory_stock AS
-- add image_url to SELECT
SELECT i.id, i.name, i.price, i.image_url, COALESCE(SUM(change), 0)::INTEGER AS stock FROM inventory as i
    LEFT JOIN transaction_items AS item ON item.item_id = i.id
    LEFT JOIN transaction_bundles as bundle ON bundle.id = item.bundle_id
GROUP BY i.id, i.name;
