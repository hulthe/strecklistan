CREATE TABLE members (
    id SERIAL PRIMARY KEY,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    nickname TEXT
);

COMMENT ON TABLE members IS
'This is used for keepin track of the tillgodolista.
See `book_accounts`.';

CREATE TYPE BOOK_ACCOUNT_TYPE AS ENUM ('expenses', 'assets', 'liabilities', 'revenue');

CREATE TABLE book_accounts (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    account_type BOOK_ACCOUNT_TYPE NOT NULL,
    creditor INTEGER UNIQUE REFERENCES members(id)
        CHECK(creditor IS NULL OR account_type = 'liabilities'::BOOK_ACCOUNT_TYPE)
);

COMMENT ON TABLE book_accounts IS
'Accounts to be credited/debited by transactions.
Used for book keeping and for keeping tillgodolistor.';

COMMENT ON COLUMN book_accounts.account_type IS
'Used for book keeping and for determining the effect of debiting/crediting the account.

When looking at all accounts, the following will always hold true:
  expenses + assets = liabilities + revenue

Crediting an expenses/assets account will increase it,
debiting a liabilities/revenue account will decrease it,
and vice-versa.';

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
    description TEXT,
    time TIMESTAMP NOT NULL DEFAULT now(),
    debited_account INTEGER NOT NULL REFERENCES book_accounts(id),
    credited_account INTEGER NOT NULL REFERENCES book_accounts(id)
        CHECK (debited_account != credited_account),
    amount INTEGER NOT NULL CHECK (amount >= 0)
);

COMMENT ON COLUMN transactions.amount IS
'The amount of money that was transferred in the transaction.
This is stored as the smallest possible fraction for the desired currency. e.g. Öre.';

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

COMMENT ON TABLE transaction_items IS 'Individual items in a tansaction bundle.';

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

COMMENT ON TABLE transaction_items IS 'Individual items in an inventory bundle.';

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
EXECUTE PROCEDURE refresh_inventory_stock();

CREATE TRIGGER refresh_inventory_stock
AFTER INSERT OR UPDATE OR DELETE OR TRUNCATE
ON transaction_bundles
EXECUTE PROCEDURE refresh_inventory_stock();

CREATE TRIGGER refresh_inventory_stock
AFTER INSERT OR UPDATE OR DELETE OR TRUNCATE
ON transaction_items
EXECUTE PROCEDURE refresh_inventory_stock();
