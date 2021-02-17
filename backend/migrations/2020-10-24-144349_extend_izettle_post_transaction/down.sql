ALTER TABLE izettle_post_transaction
    DROP COLUMN error;

ALTER TABLE izettle_post_transaction
DROP COLUMN status;

-- bring back `id` primary key
ALTER TABLE izettle_post_transaction DROP CONSTRAINT izettle_post_transaction_pkey;
ALTER TABLE izettle_post_transaction ADD COLUMN id SERIAL;
UPDATE izettle_post_transaction SET id = izettle_transaction_id;
ALTER TABLE izettle_post_transaction ADD PRIMARY KEY (id);
