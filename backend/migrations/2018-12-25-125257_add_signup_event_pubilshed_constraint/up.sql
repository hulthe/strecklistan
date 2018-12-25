-- Add constraint where event_signups have to
-- reference events which have been published.

CREATE OR REPLACE FUNCTION event_is_published(INTEGER) RETURNS BOOLEAN AS $$
SELECT EXISTS (
    SELECT 1
    FROM "events"
    WHERE id = $1
      AND published = TRUE
);
$$ language sql;

ALTER TABLE public.event_signups
    ADD CONSTRAINT event_is_published CHECK (event_is_published(event));

COMMENT ON CONSTRAINT event_is_published ON public.event_signups
    IS 'The signup have to reference a published event';

