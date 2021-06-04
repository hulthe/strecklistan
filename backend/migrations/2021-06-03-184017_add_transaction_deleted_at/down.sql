-- This file should undo anything in `up.sql`
DROP TRIGGER refresh_inventory_stock ON transactions;

DROP MATERIALIZED VIEW inventory_stock;

-- -- snipped from 2019-12-28-230948_add_inventory_item_image_link/up.sql -- --
CREATE MATERIALIZED VIEW inventory_stock AS
SELECT i.id, i.name, i.price, i.image_url, COALESCE(SUM(change), 0)::INTEGER AS stock
FROM inventory as i
         LEFT JOIN transaction_items AS item ON item.item_id = i.id
         LEFT JOIN transaction_bundles as bundle ON bundle.id = item.bundle_id
GROUP BY i.id, i.name;

DELETE FROM transactions
    WHERE deleted_at IS NOT NULL;

ALTER TABLE transactions
    DROP COLUMN deleted_at;

