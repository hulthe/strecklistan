--
-- PostgreSQL database dump
--

-- Data for Name: event_signups; Type: TABLE DATA; Schema: public; Owner: postgres
COPY public.event_signups (id, event, name, email) FROM stdin;
\.

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

-- Data for Name: inventory; Type: TABLE DATA; Schema: public; Owner: postgres
COPY public.inventory (name, price) FROM stdin;
Dr. Pepper	\N
\.

-- Data for Name: transactions; Type: TABLE DATA; Schema: public; Owner: postgres
COPY public.transactions (id, amount, description, "time") FROM stdin;
1	-120	AxFood-inköp	2019-04-18 16:35:42.56635
\.

-- Data for Name: transaction_items; Type: TABLE DATA; Schema: public; Owner: postgres
COPY public.transaction_items (id, transaction_id, item_name, item_price, change) FROM stdin;
1	1	Dr. Pepper	6	added
2	1	Dr. Pepper	6	added
3	1	Dr. Pepper	6	added
4	1	Dr. Pepper	6	added
5	1	Dr. Pepper	6	added
6	1	Dr. Pepper	6	added
7	1	Dr. Pepper	6	added
8	1	Dr. Pepper	6	added
9	1	Dr. Pepper	6	added
10	1	Dr. Pepper	6	added
11	1	Dr. Pepper	6	added
12	1	Dr. Pepper	6	added
13	1	Dr. Pepper	6	added
14	1	Dr. Pepper	6	added
15	1	Dr. Pepper	6	added
16	1	Dr. Pepper	6	added
17	1	Dr. Pepper	6	added
18	1	Dr. Pepper	6	added
19	1	Dr. Pepper	6	added
20	1	Dr. Pepper	6	added
21	1	Dr. Pepper	6	added
22	1	Dr. Pepper	6	added
23	1	Dr. Pepper	6	added
24	1	Dr. Pepper	6	added
\.

-- Data for Name: users; Type: TABLE DATA; Schema: public; Owner: postgres
COPY public.users (name, display_name, salted_pass, hash_iterations) FROM stdin;
laggit	LaggIT	8790b5087a6186e4bd9c8a664be012105881bbe124d5499700aad7bb2947b7563ba88bc932bbde2e7f971b9ad5fccebb17d4ace7716c420faf0ed4af3d424735e3f5c9d1d0e988666b74d7b378872460bf721cb5ef307de77e3d358479a04a6306bb88ef5569eac4c2dc86c82b19378f3cb617a4de1a2bda5894a4f763bad432	10000
\.

REFRESH MATERIALIZED VIEW public.inventory_stock;

