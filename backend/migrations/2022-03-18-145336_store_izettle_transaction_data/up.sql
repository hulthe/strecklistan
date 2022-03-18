ALTER TABLE izettle_post_transaction
    ADD COLUMN card_type TEXT,
    ADD COLUMN card_payment_entry_mode TEXT,
    ADD COLUMN card_issuing_bank TEXT,
    ADD COLUMN masked_pan TEXT;

