DROP TRIGGER refresh_inventory_stock ON transaction_bundles;
DROP TRIGGER refresh_inventory_stock ON transaction_items;
DROP TRIGGER refresh_inventory_stock ON inventory;
DROP FUNCTION refresh_inventory_stock;
DROP MATERIALIZED VIEW inventory_stock;
DROP TABLE inventory_bundle_items;
DROP TABLE inventory_bundles;
DROP TABLE transaction_items;
DROP TABLE transaction_bundles;
DROP TABLE transactions;
DROP TABLE inventory_tags;
DROP TABLE inventory;
DROP TABLE book_accounts;
DROP TYPE BOOK_ACCOUNT_TYPE;
DROP TABLE members;
