ALTER TABLE public.event_signups DROP CONSTRAINT event_is_published;

DROP FUNCTION public.event_is_published(INTEGER);

