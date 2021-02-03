ALTER TABLE izettle_post_transaction
ADD COLUMN status TEXT NOT NULL DEFAULT 'in_progress'
    CHECK (status IN ('paid', 'in_progress', 'canceled', 'failed'));

ALTER TABLE izettle_post_transaction
ADD COLUMN error TEXT DEFAULT NULL;