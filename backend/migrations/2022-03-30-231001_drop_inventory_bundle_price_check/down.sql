ALTER TABLE inventory_bundles
	ADD CONSTRAINT inventory_bundles_price_check CHECK (price >= 0);

ALTER TABLE transaction_bundles
	ADD CONSTRAINT transaction_bundles_price_check CHECK (price >= 0);

