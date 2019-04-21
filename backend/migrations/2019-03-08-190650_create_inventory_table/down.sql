DROP TRIGGER refresh_inventory_stock ON transaction_items;
DROP TRIGGER refresh_inventory_stock ON inventory;
DROP FUNCTION refresh_inventory_stock;
DROP MATERIALIZED VIEW inventory_stock;
DROP TABLE transaction_items;
DROP TABLE transactions;
DROP TABLE inventory;
DROP FUNCTION transaction_balance;
DROP TYPE INVENTORY_ITEM_CHANGE;
