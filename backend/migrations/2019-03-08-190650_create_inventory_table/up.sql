CREATE TYPE INVENTORY_ITEM_CHANGE AS ENUM ('added', 'removed');

CREATE TABLE inventory (
    id SERIAL PRIMARY KEY,
    name TEXT UNIQUE,
    price INTEGER
);

COMMENT ON COLUMN inventory.name IS 'The unique name of the inventory item';
COMMENT ON COLUMN inventory.price IS 'The default sales price of the item';

CREATE TABLE inventory_tags (
    tag TEXT,
    item_id INTEGER NOT NULL REFERENCES inventory(id) ON DELETE CASCADE,
    PRIMARY KEY (tag, item_id)
);

CREATE TABLE transactions (
    id SERIAL PRIMARY KEY,
    amount INTEGER NOT NULL,
    description TEXT,
    time TIMESTAMP NOT NULL DEFAULT now()
);

COMMENT ON COLUMN transactions.amount IS
    'The amount of money that was transferred in the transaction';

CREATE TABLE transaction_bundles (
    id SERIAL PRIMARY KEY,
    transaction_id INTEGER NOT NULL REFERENCES transactions(id) ON DELETE CASCADE,
    description TEXT,
    price INTEGER CHECK (price >= 0),
    change INTEGER NOT NULL
);

COMMENT ON TABLE transaction_bundles IS
    'A bundle of items in a transaction. For single items or groups of items that are sold as a package.';
COMMENT ON COLUMN transaction_bundles.price IS
    'The actual price of the item bundle in this transaction. For human reference only.';

CREATE TABLE transaction_items (
    id SERIAL PRIMARY KEY,
    bundle_id INTEGER NOT NULL REFERENCES transaction_bundles(id) ON DELETE CASCADE,
    item_id INTEGER NOT NULL REFERENCES inventory(id)
);

COMMENT ON TABLE transaction_items IS
    'Individual items in a tansaction bundle.';

CREATE TABLE inventory_bundles (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    price INTEGER NOT NULL CHECK (price >= 0)
);

COMMENT ON TABLE inventory_bundles IS
'Premade bundles of inventory items.
Things you can buy which does not map to a single inventory item.
e.g. Food servings or 3 for 2 deals.';

CREATE TABLE inventory_bundle_items (
    id SERIAL PRIMARY KEY,
    bundle_id INTEGER NOT NULL REFERENCES inventory_bundles(id) ON DELETE CASCADE,
    item_id INTEGER NOT NULL REFERENCES inventory(id)
);

COMMENT ON TABLE transaction_items IS
    'Individual items in an inventory bundle.';

CREATE FUNCTION transaction_balance() RETURNS INTEGER AS $$
    SELECT COALESCE(SUM(amount)::INTEGER, 0) FROM transactions;
$$ language sql;

CREATE VIEW transactions_joined AS
SELECT tr.*, b.id as bundle_id, b.description as bundle_description, b.price as bundle_price, b.change, i.item_id FROM transactions AS tr
    LEFT JOIN transaction_bundles AS b ON tr.id = b.transaction_id
    LEFT JOIN transaction_items as i ON b.id = i.bundle_id;

-- Show the number of inventory items in stock by counting added
-- transaction_items and subtracting removed transaction_items.
CREATE MATERIALIZED VIEW inventory_stock AS
SELECT i.id, i.name, i.price, COALESCE(SUM(change), 0)::INTEGER AS stock FROM inventory as i
    LEFT JOIN transaction_items AS item ON item.item_id = i.id
    LEFT JOIN transaction_bundles as bundle ON bundle.id = item.bundle_id
GROUP BY i.id, i.name;

CREATE FUNCTION refresh_inventory_stock()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    REFRESH MATERIALIZED VIEW inventory_stock;
    RETURN NULL;
END
$$;

CREATE TRIGGER refresh_inventory_stock
AFTER INSERT OR UPDATE OR DELETE OR TRUNCATE
ON inventory
FOR EACH STATEMENT
EXECUTE PROCEDURE refresh_inventory_stock();

CREATE TRIGGER refresh_inventory_stock
AFTER INSERT OR UPDATE OR DELETE OR TRUNCATE
ON transaction_bundles
FOR EACH STATEMENT
EXECUTE PROCEDURE refresh_inventory_stock();

CREATE TRIGGER refresh_inventory_stock
AFTER INSERT OR UPDATE OR DELETE OR TRUNCATE
ON transaction_items
FOR EACH STATEMENT
EXECUTE PROCEDURE refresh_inventory_stock();
