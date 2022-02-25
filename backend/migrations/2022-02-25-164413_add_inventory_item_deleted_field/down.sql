DROP TRIGGER refresh_inventory_stock ON inventory;

DROP MATERIALIZED VIEW inventory_stock;

ALTER TABLE public.inventory
    DROP COLUMN deleted_at,
    ADD CONSTRAINT inventory_name_key UNIQUE(name);

----- snipped from 2021-06-03-184017_add_transaction_deleted_at/up.sql -----
CREATE MATERIALIZED VIEW inventory_stock AS
SELECT i.id, i.name, i.price, i.image_url, COALESCE(SUM(change), 0)::INTEGER AS stock
FROM inventory AS i
    LEFT JOIN transaction_items AS item ON item.item_id = i.id
    LEFT JOIN transaction_bundles AS bundle ON bundle.id = item.bundle_id
    LEFT JOIN transactions ON transactions.id = bundle.transaction_id
WHERE deleted_at IS NULL
GROUP BY i.id, i.name;

CREATE TRIGGER refresh_inventory_stock
    AFTER INSERT OR UPDATE OR DELETE OR TRUNCATE
    ON inventory
EXECUTE PROCEDURE refresh_inventory_stock();

