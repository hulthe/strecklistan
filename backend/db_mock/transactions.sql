-- Add inventory index
INSERT INTO inventory (id, name, price) VALUES (01, 'Algrens Bilar, Orginal', NULL);
INSERT INTO inventory (id, name, price) VALUES (02, 'Banana Skids', NULL);
INSERT INTO inventory (id, name, price) VALUES (03, 'Chokladboll, Daim', 6);
INSERT INTO inventory (id, name, price) VALUES (04, 'Chokladboll, Kokos', 6);
INSERT INTO inventory (id, name, price) VALUES (05, 'Chokladboll, Pärlsocker', 6);
INSERT INTO inventory (id, name, price) VALUES (06, 'Chokladrullar', 12);
INSERT INTO inventory (id, name, price) VALUES (07, 'Chokladrullar, Daim', 12);
INSERT INTO inventory (id, name, price) VALUES (08, 'Chokladrullar, Mint', 12);
INSERT INTO inventory (id, name, price) VALUES (09, 'Djungelvrål', 6);
INSERT INTO inventory (id, name, price) VALUES (10, 'Gott & Blandat', 12);
INSERT INTO inventory (id, name, price) VALUES (11, 'Hallonlakritsskalle', 6);
INSERT INTO inventory (id, name, price) VALUES (12, 'Haribo Nallar', 6);
INSERT INTO inventory (id, name, price) VALUES (13, 'Haribo Persikor', 6);
INSERT INTO inventory (id, name, price) VALUES (14, 'Jättesalt', 6);
INSERT INTO inventory (id, name, price) VALUES (15, 'Kexchoklad', 6);
INSERT INTO inventory (id, name, price) VALUES (16, 'Kinasnacks', 12);
INSERT INTO inventory (id, name, price) VALUES (17, 'Lakrisal', 6);
INSERT INTO inventory (id, name, price) VALUES (18, 'Lollipop Fruit', 6);
INSERT INTO inventory (id, name, price) VALUES (19, 'Mars', 6);
INSERT INTO inventory (id, name, price) VALUES (20, 'Nappar, Fruit', 6);
INSERT INTO inventory (id, name, price) VALUES (21, 'Nappar, Kola', 6);
INSERT INTO inventory (id, name, price) VALUES (22, 'Nappar, Lakrits', 6);
INSERT INTO inventory (id, name, price) VALUES (23, 'Nappar, Zour', 6);
INSERT INTO inventory (id, name, price) VALUES (24, 'Pingvinstång, Jordgubb', 6);
INSERT INTO inventory (id, name, price) VALUES (25, 'Pingvinstång, Mint', 6);
INSERT INTO inventory (id, name, price) VALUES (26, 'Tutti Frutti', 6);
INSERT INTO inventory (id, name, price) VALUES (27, 'Wasa Sandwich', 6);
INSERT INTO inventory (id, name, price) VALUES (28, 'Tyrkisk Peber', 12);
INSERT INTO inventory (id, name, price) VALUES (29, 'ZOO Apor', 6);
INSERT INTO inventory (id, name, price) VALUES (30, 'Coca-Cola', 6);
INSERT INTO inventory (id, name, price) VALUES (31, 'Coca-Cola Vanilla', 6);
INSERT INTO inventory (id, name, price) VALUES (32, 'Coca-Cola Zero', 6);
INSERT INTO inventory (id, name, price) VALUES (33, 'Dr. Pepper', 6);
INSERT INTO inventory (id, name, price) VALUES (34, 'Fanta', 6);
INSERT INTO inventory (id, name, price) VALUES (35, 'Hallonsoda', 6);
INSERT INTO inventory (id, name, price) VALUES (36, 'Loka, Citron', 6);
INSERT INTO inventory (id, name, price) VALUES (37, 'Loka, Päron', 6);
INSERT INTO inventory (id, name, price) VALUES (38, 'Mountaindew', 6);
INSERT INTO inventory (id, name, price) VALUES (39, 'Pärondryck', 6);
INSERT INTO inventory (id, name, price) VALUES (40, 'Ramlösa, Granatäpple', 6);
INSERT INTO inventory (id, name, price) VALUES (41, 'Ramlösa, Hallon & björnbär', 6);
INSERT INTO inventory (id, name, price) VALUES (42, 'Smakis, Päron', 6);
INSERT INTO inventory (id, name, price) VALUES (43, 'Smakis, Äpple', 6);
INSERT INTO inventory (id, name, price) VALUES (44, 'Sockerdricka', 6);
INSERT INTO inventory (id, name, price) VALUES (45, 'Sprite, Citron', 6);
INSERT INTO inventory (id, name, price) VALUES (46, 'Trocadero', 6);
INSERT INTO inventory (id, name, price) VALUES (47, 'Zingo', 6);
INSERT INTO inventory (id, name, price) VALUES (48, 'Zingo, Tropical', 6);
INSERT INTO inventory (id, name, price) VALUES (49, 'Vimto', 6);


-- Purchase from AxFood
INSERT INTO transactions (id, amount) VALUES (1, -500);

INSERT INTO transaction_bundles (transaction_id, id, change) VALUES (1, 1, 24);
INSERT INTO transaction_items (bundle_id, item_id) VALUES (1, 30);

INSERT INTO transaction_bundles (transaction_id, id, change) VALUES (1, 2, 30);
INSERT INTO transaction_items (bundle_id, item_id) VALUES (2, 33);

INSERT INTO transaction_bundles (transaction_id, id, change) VALUES (1, 3, 24);
INSERT INTO transaction_items (bundle_id, item_id) VALUES (3, 34);

INSERT INTO transaction_bundles (transaction_id, id, change) VALUES (1, 4, 12);
INSERT INTO transaction_items (bundle_id, item_id) VALUES (4, 49);

INSERT INTO transaction_bundles (transaction_id, id, change) VALUES (1, 5, 50);
INSERT INTO transaction_items (bundle_id, item_id) VALUES (5, 25);

INSERT INTO transaction_bundles (transaction_id, id, change) VALUES (1, 6, 8);
INSERT INTO transaction_items (bundle_id, item_id) VALUES (6, 17);


-- Random Sale
INSERT INTO transactions (id, amount) VALUES (2, 18);

INSERT INTO transaction_bundles (transaction_id, id, change) VALUES (2, 7, -2);
INSERT INTO transaction_items (bundle_id, item_id) VALUES (7, 25);
INSERT INTO transaction_items (bundle_id, item_id) VALUES (7, 25);

INSERT INTO transaction_bundles (transaction_id, id, change) VALUES (2, 8, -1);
INSERT INTO transaction_items (bundle_id, item_id) VALUES (8, 34);


-- Random Sale
INSERT INTO transactions (id, amount) VALUES (3, 24);

INSERT INTO transaction_bundles (transaction_id, id, change) VALUES (3, 9, -5);
INSERT INTO transaction_items (bundle_id, item_id) VALUES (9, 49);
