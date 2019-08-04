CREATE TYPE INVENTORY_ITEM_CHANGE AS ENUM ('added', 'removed');

CREATE TABLE inventory (
    name VARCHAR(128) PRIMARY KEY,
    price INTEGER
);

COMMENT ON COLUMN inventory.name IS 'The unique name of the inventory item';
COMMENT ON COLUMN inventory.price IS 'The default sales price of the item';

CREATE TABLE transactions (
    id SERIAL PRIMARY KEY,
    amount INTEGER NOT NULL,
    description TEXT,
    time TIMESTAMP NOT NULL DEFAULT now()
);

COMMENT ON COLUMN inventory.name IS 'The unique name of the inventory item';
COMMENT ON COLUMN transactions.amount IS
    'The amount of money that was transferred in the transaction';

CREATE TABLE transaction_items (
    id SERIAL PRIMARY KEY,
    transaction_id SERIAL NOT NULL REFERENCES transactions(id),
    item_name VARCHAR(128) NOT NULL REFERENCES inventory(name),
    item_price INTEGER CHECK (item_price >= 0),
    change INVENTORY_ITEM_CHANGE NOT NULL
);

COMMENT ON COLUMN transaction_items.item_price IS
    'The actual price of the item. For human reference only.';

CREATE FUNCTION transaction_balance() RETURNS INTEGER AS $$
    SELECT COALESCE(SUM(amount)::INTEGER, 0) FROM transactions;
$$ language sql;

-- Show the number of inventory items in stock by counting added
-- transaction_items and subtracting removed transaction_items.
CREATE MATERIALIZED VIEW inventory_stock AS
SELECT added.name, added.price, (added.count - removed.count)::INTEGER AS stock FROM (
    SELECT COUNT(id), name, price
    FROM inventory as i
    LEFT JOIN transaction_items as ts ON ts.item_name = i.name AND ts.change = 'added'
    GROUP BY name
) added
RIGHT JOIN (
    SELECT COUNT(id), name
    FROM inventory as i
    LEFT JOIN transaction_items as ts ON ts.item_name = i.name AND ts.change = 'removed'
    GROUP BY name
) AS removed ON added.name = removed.name;

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
ON transaction_items
FOR EACH STATEMENT
EXECUTE PROCEDURE refresh_inventory_stock();
