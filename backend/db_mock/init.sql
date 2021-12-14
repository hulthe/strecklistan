--
-- PostgreSQL database dump
--

-- Data for Name: event_signups; Type: TABLE DATA; Schema: public; Owner: postgres
COPY public.event_signups (id, event, name, email) FROM stdin;
\.
SELECT setval('event_signups_id_seq', 1, true);

-- Data for Name: events; Type: TABLE DATA; Schema: public; Owner: postgres
COPY public.events (id, title, background, location, start_time, end_time, price, published) FROM stdin;
1	My Event 1	http://imgur.ru	Hubben 2.1	2019-01-02 17:01:00	2019-01-02 23:59:59	0	t
2	My Event 2	http://imgur.ru	Hubben 2.1	2019-01-16 17:01:00	2019-01-16 23:59:59	0	f
3	My Event 3	http://imgur.ru	Hubben 2.1	2019-02-02 17:01:00	2019-02-02 23:59:59	0	t
4	My Event 4	http://imgur.ru	Hubben 2.1	2019-02-16 17:01:00	2019-02-16 23:59:59	0	t
5	My Event 5	http://imgur.ru	Hubben 2.1	2019-03-02 17:01:00	2019-03-02 23:59:59	0	t
6	My Event 6	http://imgur.ru	Hubben 2.1	2019-03-16 17:01:00	2019-03-16 23:59:59	0	t
7	My Event 7	http://imgur.ru	Hubben 2.1	2019-04-02 17:01:00	2019-04-02 23:59:59	0	t
8	My Event 8	http://imgur.ru	Hubben 2.1	2019-04-16 17:01:00	2019-04-16 23:59:59	0	f
9	My Event 9	http://imgur.ru	Hubben 2.1	2019-05-02 17:01:00	2019-05-02 23:59:59	0	t
10	My Event 10	http://imgur.ru	Hubben 2.1	2019-05-16 17:01:00	2019-05-16 23:59:59	0	t
11	My Event 11	http://imgur.ru	Hubben 2.1	2019-06-02 17:01:00	2019-06-02 23:59:59	0	t
12	My Event 12	http://imgur.ru	Hubben 2.1	2019-06-16 17:01:00	2019-06-16 23:59:59	0	t
13	My Event 13	http://imgur.ru	Hubben 2.1	2019-07-02 17:01:00	2019-07-02 23:59:59	0	f
14	My Event 14	http://imgur.ru	Hubben 2.1	2019-07-16 17:01:00	2019-07-16 23:59:59	0	f
15	My Event 15	http://imgur.ru	Hubben 2.1	2019-08-02 17:01:00	2019-08-02 23:59:59	0	t
16	My Event 16	http://imgur.ru	Hubben 2.1	2019-08-16 17:01:00	2019-08-16 23:59:59	0	f
17	My Event 17	http://imgur.ru	Hubben 2.1	2019-09-02 17:01:00	2019-09-02 23:59:59	0	t
18	My Event 18	http://imgur.ru	Hubben 2.1	2019-09-16 17:01:00	2019-09-16 23:59:59	0	t
19	My Event 19	http://imgur.ru	Hubben 2.1	2019-10-02 17:01:00	2019-10-02 23:59:59	0	f
20	My Event 20	http://imgur.ru	Hubben 2.1	2019-10-16 17:01:00	2019-10-16 23:59:59	0	f
21	My Event 21	http://imgur.ru	Hubben 2.1	2019-11-02 17:01:00	2019-11-02 23:59:59	0	f
22	My Event 22	http://imgur.ru	Hubben 2.1	2019-11-16 17:01:00	2019-11-16 23:59:59	0	t
23	My Event 23	http://imgur.ru	Hubben 2.1	2019-12-02 17:01:00	2019-12-02 23:59:59	0	t
24	My Event 24	http://imgur.ru	Hubben 2.1	2019-12-16 17:01:00	2019-12-16 23:59:59	0	f
\.
SELECT setval('events_id_seq', 25, true);
-- ##################

-- Data for Name: users; Type: TABLE DATA; Schema: public; Owner: postgres
COPY public.users (name, display_name, salted_pass, hash_iterations) FROM stdin;
laggit	LaggIT	8790b5087a6186e4bd9c8a664be012105881bbe124d5499700aad7bb2947b7563ba88bc932bbde2e7f971b9ad5fccebb17d4ace7716c420faf0ed4af3d424735e3f5c9d1d0e988666b74d7b378872460bf721cb5ef307de77e3d358479a04a6306bb88ef5569eac4c2dc86c82b19378f3cb617a4de1a2bda5894a4f763bad432	10000
\.
-- ##################

COPY public.members (id, nickname, first_name, last_name) FROM stdin;
1	tux	Tux	Tuxxersson
2	Fläkt	Stek	Fan
3	Tomten	Fader	Jul
4	NaN	Ett	Genom Noll
5	Pixie	Gillar	Cola
6	\N	Anti	Loop
7	\N	Efter	Lyst
\.
SELECT setval('members_id_seq', 8, true);

COPY public.book_accounts (id, name, account_type, creditor) FROM stdin;
1	Bankkonto	assets	\N
2	Kontantkassa	assets	\N
3	Försäljning	revenue	\N
4	Inköp	expenses	\N
5	Tillgodo/tux	liabilities	1
6	Tillgodo/Fläkt	liabilities	2
7	Tillgodo/Tomten	liabilities	3
8	Tillgodo/NaN	liabilities	4
9	Tillgodo/Pixie	liabilities	5
10	Tillgodo/LP	liabilities	6
11	Tillgodo/GP	liabilities	7
\.
SELECT setval('book_accounts_id_seq', 12, true);

-- Add inventory index
COPY public.inventory (id, price, name, image_url) FROM stdin;
01	\N	Filidutter	https://drawit-shop.chalmers.it/images/products/filidutter.webp
02	\N	Fizzy Pop	https://drawit-shop.chalmers.it/images/products/fizzypop.webp
03	600	Mentos Rainbow	https://drawit-shop.chalmers.it/images/products/mentos_rainbow.webp
04	600	Delicatoboll	https://drawit-shop.chalmers.it/images/products/delicatoboll.webp
05	600	Pärlboll	https://drawit-shop.chalmers.it/images/products/pärlboll.webp
06	1200	Chokladrullar	https://drawit-shop.chalmers.it/images/products/marabou_chokladrulle.webp
07	1200	Chokladrullar, Daim	https://drawit-shop.chalmers.it/images/products/marabou_chokladrulle_daim.webp
08	1200	Chokladrullar, Mint	https://drawit-shop.chalmers.it/images/products/marabou_chokladrulle_mint.webp
09	600	Djungelvrål	https://drawit-shop.chalmers.it/images/products/djungelvrål.webp
10	1200	Gott & Blandat	https://drawit-shop.chalmers.it/images/products/gott_och_blandat.webp
11	600	Ginger Beer	https://drawit-shop.chalmers.it/images/products/ginger_beer.webp
12	600	Haribo Nallar	https://drawit-shop.chalmers.it/images/products/haribo_goldbears.webp
13	600	Kryptoniter	https://drawit-shop.chalmers.it/images/products/kryptoniter.webp
14	600	Jättesalt	https://drawit-shop.chalmers.it/images/products/jättesalt.webp
15	600	Kexchoklad	https://drawit-shop.chalmers.it/images/products/kexchoklad.webp
16	1200	Kinasnacks	https://drawit-shop.chalmers.it/images/products/kinasnacks.webp
17	600	Lakrisal	https://drawit-shop.chalmers.it/images/products/lakrisal.webp
18	600	Lollipop Fruit	\N
19	600	Mars	https://drawit-shop.chalmers.it/images/products/mars.webp
20	600	Nappar, Fruit	https://drawit-shop.chalmers.it/images/products/haribo_nappar.webp
21	600	Nappar, Kola	https://drawit-shop.chalmers.it/images/products/haribo_nappar_kola.webp
22	600	Nappar, Lakrits	https://drawit-shop.chalmers.it/images/products/haribo_nappar_lakrits.webp
23	600	Nappar, Sura	https://drawit-shop.chalmers.it/images/products/haribo_nappar_sura.webp
24	300	Pingvinstång, Sur Smultron	https://drawit-shop.chalmers.it/images/products/pingvinstång_sur_smultron.webp
25	300	Pingvinstång, Mint	https://drawit-shop.chalmers.it/images/products/pingvinstång_mint.webp
26	600	Tutti Frutti Rings	https://drawit-shop.chalmers.it/images/products/tutti_frutti_rings.webp
27	600	Wasa Sandwich	https://drawit-shop.chalmers.it/images/products/wasa_sandwich_tomato_basil.webp
28	1200	Tyrkisk Peber	https://drawit-shop.chalmers.it/images/products/tyrkisk_peber.webp
29	600	ZOO Apor	\N
30	600	Pepsi	https://drawit-shop.chalmers.it/images/products/pepsi.webp
31	600	Pepsi Max	https://drawit-shop.chalmers.it/images/products/pepsi_max.webp
32	600	Pepsi Max Lime	https://drawit-shop.chalmers.it/images/products/pepsi_max_lime.webp
33	600	Dr. Pepper	https://drawit-shop.chalmers.it/images/products/dr_pepper.webp
34	600	Zingo Tropical	\N
35	600	Hallonsoda	https://drawit-shop.chalmers.it/images/products/hallonsoda.webp
36	600	Loka, Citron	https://drawit-shop.chalmers.it/images/products/loka_citron.webp
37	600	Loka, Hallon	https://drawit-shop.chalmers.it/images/products/loka_hallon.webp
38	600	Mountaindew	https://drawit-shop.chalmers.it/images/products/mountain_dew.webp
39	600	Päronsoda	https://drawit-shop.chalmers.it/images/products/päronsoda.webp
40	600	Ramlösa, Granatäpple	\N
41	600	Ramlösa, Hallon & björnbär	\N
42	600	Smakis, Päron	https://drawit-shop.chalmers.it/images/products/smakis_päron.webp
43	600	Smakis, Äpple	https://drawit-shop.chalmers.it/images/products/smakis_päron.webp
44	600	7up	https://drawit-shop.chalmers.it/images/products/7up.webp
45	600	7up Free	https://drawit-shop.chalmers.it/images/products/7up_free.webp
46	600	Trocadero	https://drawit-shop.chalmers.it/images/products/trocadero.webp
47	600	Zingo	https://drawit-shop.chalmers.it/images/products/zingo_orange.webp
48	600	Zingo Sorbet	https://drawit-shop.chalmers.it/images/products/zingo_sorbet_light.webp
49	600	Vimto	https://drawit-shop.chalmers.it/images/products/vimto.webp
\.
SELECT setval('inventory_id_seq', 50, true);

COPY public.inventory_tags (tag, item_id) FROM stdin;
burkläsk	11
burkläsk	30
burkläsk	31
burkläsk	32
burkläsk	33
burkläsk	34
burkläsk	35
burkläsk	36
burkläsk	37
burkläsk	38
burkläsk	39
burkläsk	40
burkläsk	41
burkläsk	42
burkläsk	43
burkläsk	44
burkläsk	45
burkläsk	46
burkläsk	47
burkläsk	48
burkläsk	49
\.
-- ##################

-- Add inventory bundles
COPY public.inventory_bundles (id, name, price, image_url) FROM stdin;
1	Mat	2500	https://drawit-shop.chalmers.it/images/products/mat.webp
2	Mintstång: 3 för 2	600	https://drawit-shop.chalmers.it/images/products/pingvinstång_mint.webp
\.
SELECT setval('inventory_bundles_id_seq', 3, true);

COPY public.inventory_bundle_items (id, bundle_id, item_id) FROM stdin;
1	2	25
2	2	25
3	2	25
\.
SELECT setval('inventory_bundle_items_id_seq', 4, true);
-- ##################

-- Add some transactions
COPY public.transactions (id, description, debited_account, credited_account, amount) FROM stdin;
1	AxFood-inköp	4	1	50000
2	Försäljning	5	3	800
3	Försäljning	1	3	400
4	Insättning	1	5	9900
5	Insättning	1	8	9900
\.
SELECT setval('transactions_id_seq', 6, true);

COPY public.transaction_bundles (transaction_id, id, change) FROM stdin;
1	1	24
1	2	30
1	3	24
1	4	12
1	5	50
1	6	8
2	7	-2
2	8	-1
3	9	-5
\.
SELECT setval('transaction_bundles_id_seq', 10, true);

COPY public.transaction_items (bundle_id, item_id) FROM stdin;
1	30
2	33
3	34
4	49
5	25
6	17
7	25
7	25
8	34
9	49
\.
SELECT setval('transaction_items_id_seq', 10, true);

REFRESH MATERIALIZED VIEW public.inventory_stock;
-- ##################

