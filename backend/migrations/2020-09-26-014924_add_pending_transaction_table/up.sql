CREATE TABLE izettle_transaction (
    id SERIAL PRIMARY KEY,
    description TEXT,
    time TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT now(),
    debited_account INTEGER NOT NULL REFERENCES book_accounts(id),
    credited_account INTEGER NOT NULL REFERENCES book_accounts(id),
    amount INTEGER NOT NULL CHECK (amount >= 0),
    paid BOOLEAN NOT NULL
);

CREATE TABLE izettle_transaction_bundle (
    id SERIAL PRIMARY KEY,
    transaction_id INTEGER NOT NULL REFERENCES izettle_transaction(id),
    description TEXT,
    price INTEGER NOT NULL,
    change INTEGER NOT NULL
);

CREATE TABLE izettle_transaction_item (
    id SERIAL PRIMARY KEY,
    bundle_id INTEGER NOT NULL REFERENCES izettle_transaction_bundle(id),
    item_id INTEGER NOT NULL REFERENCES inventory(id)
);