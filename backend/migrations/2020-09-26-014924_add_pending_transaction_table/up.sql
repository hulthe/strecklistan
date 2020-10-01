CREATE TABLE izettle_transactions (
    id SERIAL PRIMARY KEY,
    description TEXT,
    debited_account INTEGER NOT NULL REFERENCES book_accounts(id),
    credited_account INTEGER NOT NULL REFERENCES book_accounts(id),
    amount INTEGER NOT NULL CHECK (amount >= 0),
    paid BOOLEAN NOT NULL
);

CREATE TABLE izettle_transaction_bundles (
    id SERIAL PRIMARY KEY,
    transaction_id INTEGER NOT NULL REFERENCES izettle_transaction(id),
    description TEXT,
    prince INTEGER NOT NULL,
    change INTEGER NOT NULL
);

CREATE TABLE izettle_transaction_items (
    id SERIAL PRIMARY KEY,
    bundle_id INTEGER NOT NULL REFERENCES izettle_transaction_bundle(id),
    item_id INTEGER NOT NULL REFERENCES inventory(id)
);