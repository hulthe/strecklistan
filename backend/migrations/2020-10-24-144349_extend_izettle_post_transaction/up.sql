-- Remove column `id` and set izettle_transaction_id as primary key
ALTER TABLE izettle_post_transaction DROP CONSTRAINT izettle_post_transaction_pkey;
ALTER TABLE izettle_post_transaction DROP COLUMN id;
ALTER TABLE izettle_post_transaction ADD PRIMARY KEY (izettle_transaction_id);

-- Add status enum column
ALTER TABLE izettle_post_transaction
ADD COLUMN status TEXT NOT NULL DEFAULT 'in_progress'
    CHECK (status IN ('paid', 'in_progress', 'cancelled', 'failed'));

-- Try to guess what the status should be
UPDATE izettle_post_transaction SET status = 'cancelled' WHERE transaction_id IS NULL;
UPDATE izettle_post_transaction SET status = 'paid' WHERE transaction_id IS NOT NULL;

-- Add error message column
ALTER TABLE izettle_post_transaction ADD COLUMN error TEXT DEFAULT NULL;
