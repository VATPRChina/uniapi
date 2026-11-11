CREATE TABLE public.event_discord_message (
    event_id uuid PRIMARY KEY REFERENCES public.event(id) ON DELETE CASCADE,
    guild_id text NOT NULL,
    message_id text NOT NULL,
    status text NOT NULL CHECK (status IN ('Sync', 'OutOfSync')),
    synced_at timestamptz NOT NULL
);
