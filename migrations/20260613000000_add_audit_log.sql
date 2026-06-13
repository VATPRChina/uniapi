CREATE TABLE "audit_log" (
    entity_kind text NOT NULL,
    entity_id uuid NOT NULL,
    before json NOT NULL,
    after json NOT NULL,
    operated_by uuid NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE INDEX ix_audit_log_entity ON public.audit_log USING btree (entity_kind, entity_id);

CREATE INDEX ix_audit_log_operated_by ON public.audit_log USING btree (operated_by);

CREATE INDEX ix_audit_log_created_at ON public.audit_log USING btree (created_at);

ALTER TABLE ONLY "audit_log"
    ADD CONSTRAINT fk_audit_log_user_operated_by FOREIGN KEY (operated_by) REFERENCES public."user"(id);
