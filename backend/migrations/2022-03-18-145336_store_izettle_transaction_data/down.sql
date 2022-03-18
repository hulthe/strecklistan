ALTER TABLE izettle_post_transaction
    DROP COLUMN card_type,
    DROP COLUMN card_payment_entry_mode,
    DROP COLUMN card_issuing_bank,
    DROP COLUMN masked_pan;