CREATE TABLE events (
    id SERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    background TEXT NOT NULL,
    location TEXT NOT NULL,
    start_time TIMESTAMP NOT NULL,
-- TODO: Add contraint end_time > start_time
    end_time TIMESTAMP NOT NULL,
    price INTEGER NOT NULL DEFAULT 0,
    published BOOLEAN NOT NULL DEFAULT FALSE
)

