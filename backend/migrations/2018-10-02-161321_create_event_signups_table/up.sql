CREATE TABLE event_signups (
    id SERIAL PRIMARY KEY,
    event_id INTEGER REFERENCES events(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL
);

CREATE OR REPLACE VIEW public.events_with_signups AS
SELECT
    events.*,
    COALESCE(t_signup_count.count, 0) AS signups
FROM
    events
    LEFT JOIN
        (
            SELECT
                count(id),
                event_id
            FROM
                event_signups
            GROUP BY
                event_id
        ) t_signup_count
    ON events.id = t_signup_count.event_id;
