ALTER TABLE public.atc_booking
    ADD COLUMN remarks text,
    ADD COLUMN created_at timestamp with time zone,
    ADD COLUMN updated_at timestamp with time zone,
    ADD COLUMN deleted_at timestamp with time zone;

UPDATE public.atc_booking
SET created_at = booked_at,
    updated_at = booked_at;

ALTER TABLE public.atc_booking
    ALTER COLUMN created_at SET DEFAULT CURRENT_TIMESTAMP,
    ALTER COLUMN created_at SET NOT NULL,
    ALTER COLUMN updated_at SET DEFAULT CURRENT_TIMESTAMP,
    ALTER COLUMN updated_at SET NOT NULL;

CREATE INDEX ix_atc_booking_upcoming
    ON public.atc_booking (end_at, start_at)
    WHERE deleted_at IS NULL;
