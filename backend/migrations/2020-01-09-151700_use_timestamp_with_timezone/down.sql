DROP VIEW events_with_signups;

ALTER TABLE transactions
    ALTER COLUMN time TYPE timestamp without time zone;

ALTER TABLE events
    ALTER COLUMN end_time TYPE timestamp without time zone,
    ALTER COLUMN start_time TYPE timestamp without time zone;

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
                event
            FROM
                event_signups
            GROUP BY
                event
        ) t_signup_count
    ON events.id = t_signup_count.event;
