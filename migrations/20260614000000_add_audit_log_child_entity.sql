ALTER TABLE ONLY "audit_log"
    ADD COLUMN child_entity_kind text,
    ADD COLUMN child_entity_id uuid;

CREATE INDEX ix_audit_log_child_entity ON public.audit_log USING btree (child_entity_kind, child_entity_id);

ALTER TABLE ONLY "audit_log"
    ADD CONSTRAINT ck_audit_log_child_entity_complete CHECK (
        (child_entity_kind IS NULL) = (child_entity_id IS NULL)
    );
