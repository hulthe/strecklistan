--
-- PostgreSQL database dump
--

-- Data for Name: event_signups; Type: TABLE DATA; Schema: public; Owner: postgres
COPY public.event_signups (id, event, name, email) FROM stdin;
\.
SELECT setval('events_id_seq', 1, true);

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
-- ##################
SELECT setval('inventory_id_seq', 25, true);

-- Data for Name: users; Type: TABLE DATA; Schema: public; Owner: postgres
COPY public.users (name, display_name, salted_pass, hash_iterations) FROM stdin;
laggit	LaggIT	8790b5087a6186e4bd9c8a664be012105881bbe124d5499700aad7bb2947b7563ba88bc932bbde2e7f971b9ad5fccebb17d4ace7716c420faf0ed4af3d424735e3f5c9d1d0e988666b74d7b378872460bf721cb5ef307de77e3d358479a04a6306bb88ef5569eac4c2dc86c82b19378f3cb617a4de1a2bda5894a4f763bad432	10000
\.
-- ##################

COPY public.members (id, nickname, first_name, last_name) FROM stdin;
1	tux	Jokke	Boi
2	Fläkt	Steffe	Pojk
3	Santa	T-rex	Glassssmak
4	NaN	Karl	Wikström
\.
SELECT setval('members_id_seq', 4, true);

COPY public.book_accounts (id, name, account_type, creditor) FROM stdin;
1	Bankkonto	assets	\N
2	Kontantkassa	assets	\N
3	Försäljning	revenue	\N
4	Inköp	expenses	\N
5	Tillgodo/tux	liabilities	1
6	Tillgodo/Fläkt	liabilities	2
7	Tillgodo/Santa	liabilities	3
8	Tillgodo/NaN	liabilities	4
\.
SELECT setval('book_accounts_id_seq', 9, true);

-- Add inventory index
COPY public.inventory (id, price, name) FROM stdin;
01	\N	Algrens Bilar, Orginal
02	\N	Banana Skids
03	6	Chokladboll, Daim
04	6	Chokladboll, Kokos
05	6	Chokladboll, Pärlsocker
06	12	Chokladrullar
07	12	Chokladrullar, Daim
08	12	Chokladrullar, Mint
09	6	Djungelvrål
10	12	Gott & Blandat
11	6	Hallonlakritsskalle
12	6	Haribo Nallar
13	6	Haribo Persikor
14	6	Jättesalt
15	6	Kexchoklad
16	12	Kinasnacks
17	6	Lakrisal
18	6	Lollipop Fruit
19	6	Mars
20	6	Nappar, Fruit
21	6	Nappar, Kola
22	6	Nappar, Lakrits
23	6	Nappar, Zour
24	3	Pingvinstång, Jordgubb
25	3	Pingvinstång, Mint
26	6	Tutti Frutti
27	6	Wasa Sandwich
28	12	Tyrkisk Peber
29	6	ZOO Apor
30	6	Coca-Cola
31	6	Coca-Cola Vanilla
32	6	Coca-Cola Zero
33	6	Dr. Pepper
34	6	Fanta
35	6	Hallonsoda
36	6	Loka, Citron
37	6	Loka, Päron
38	6	Mountaindew
39	6	Pärondryck
40	6	Ramlösa, Granatäpple
41	6	Ramlösa, Hallon & björnbär
42	6	Smakis, Päron
43	6	Smakis, Äpple
44	6	Sockerdricka
45	6	Sprite, Citron
46	6	Trocadero
47	6	Zingo
48	6	Zingo, Tropical
49	6	Vimto
\.
SELECT setval('inventory_id_seq', 50, true);

COPY public.inventory_tags (tag, item_id) FROM stdin;
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
COPY public.inventory_bundles (id, name, price) FROM stdin;
1	Mat	25
2	Banana Skids	2
\.
SELECT setval('inventory_bundles_id_seq', 3, true);

COPY public.inventory_bundle_items (id, bundle_id, item_id) FROM stdin;
1	2	2
2	2	2
\.
SELECT setval('inventory_bundle_items_id_seq', 3, true);
-- ##################

-- Add some transactions
COPY public.transactions (id, description, debited_account, credited_account, amount) FROM stdin;
1	AxFood-inköp	4	1	500
2	Försäljning	5	3	8
3	Försäljning	1	3	4
4	Insättning	1	5	99
5	Insättning	1	8	99
\.
SELECT setval('transactions_id_seq', 5, true);

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

