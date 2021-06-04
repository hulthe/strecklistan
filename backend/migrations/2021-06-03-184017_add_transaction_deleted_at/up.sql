-- Your SQL goes here
ALTER TABLE transactions
ADD COLUMN deleted_at TIMESTAMP WITH TIME ZONE;


DROP MATERIALIZED VIEW inventory_stock;

-- -- snipped from 2019-12-28-230948_add_inventory_item_image_link/up.sql -- --
CREATE MATERIALIZED VIEW inventory_stock AS
-- Take deleted_at into consideration, ignore deleted transactions
SELECT i.id, i.name, i.price, i.image_url, COALESCE(SUM(change), 0)::INTEGER AS stock
FROM inventory AS i
    LEFT JOIN transaction_items AS item ON item.item_id = i.id
    LEFT JOIN transaction_bundles AS bundle ON bundle.id = item.bundle_id
    LEFT JOIN transactions ON transactions.id = bundle.transaction_id
WHERE deleted_at IS NULL
GROUP BY i.id, i.name;

CREATE TRIGGER refresh_inventory_stock
    AFTER INSERT OR UPDATE OR DELETE OR TRUNCATE
    ON transactions
EXECUTE PROCEDURE refresh_inventory_stock();
