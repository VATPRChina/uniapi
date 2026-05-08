CREATE TABLE "atc_application" (
    id uuid NOT NULL,
    user_id uuid NOT NULL,
    application_filing_id uuid NOT NULL,
    review_filing_id uuid,
    applied_at timestamp with time zone NOT NULL,
    status text DEFAULT ''::text NOT NULL
);

CREATE TABLE "atc_booking" (
    id uuid NOT NULL,
    user_id uuid NOT NULL,
    callsign text NOT NULL,
    booked_at timestamp with time zone NOT NULL,
    start_at timestamp with time zone NOT NULL,
    end_at timestamp with time zone NOT NULL
);

CREATE TABLE "device_authorization" (
    device_code uuid NOT NULL,
    user_code text NOT NULL,
    expires_at timestamp with time zone NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    client_id text NOT NULL,
    user_id uuid
);

CREATE TABLE "event" (
    id uuid NOT NULL,
    title text NOT NULL,
    start_at timestamp with time zone NOT NULL,
    end_at timestamp with time zone NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    end_booking_at timestamp with time zone DEFAULT '-infinity'::timestamp with time zone,
    start_booking_at timestamp with time zone DEFAULT '-infinity'::timestamp with time zone,
    image_url text,
    description text DEFAULT ''::text NOT NULL,
    start_atc_booking_at timestamp with time zone,
    community_link text,
    vatsim_link text,
    title_en text,
    is_approved boolean
);

CREATE TABLE "event_airspace" (
    id uuid NOT NULL,
    event_id uuid NOT NULL,
    name text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    icao_codes text[] DEFAULT ARRAY[]::text[] NOT NULL,
    description text DEFAULT ''::text NOT NULL
);

CREATE TABLE "event_atc_position" (
    id uuid NOT NULL,
    event_id uuid NOT NULL,
    callsign text NOT NULL,
    start_at timestamp with time zone NOT NULL,
    end_at timestamp with time zone NOT NULL,
    remarks text,
    position_kind_id text NOT NULL,
    minimum_controller_state integer NOT NULL
);

CREATE TABLE "event_atc_position_booking" (
    event_atc_position_id uuid NOT NULL,
    user_id uuid NOT NULL,
    created_at timestamp with time zone NOT NULL,
    atc_booking_id uuid
);

CREATE TABLE "event_booking" (
    id uuid NOT NULL,
    user_id uuid NOT NULL,
    event_slot_id uuid NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE "event_slot" (
    id uuid NOT NULL,
    event_airspace_id uuid NOT NULL,
    enter_at timestamp with time zone NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    leave_at timestamp with time zone,
    aircraft_type_icao text,
    callsign text
);

CREATE TABLE "session" (
    token uuid NOT NULL,
    user_id uuid NOT NULL,
    user_updated_at timestamp with time zone NOT NULL,
    expires_in timestamp with time zone NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    code uuid,
    client_id text DEFAULT ''::text NOT NULL
);

CREATE TABLE "sheet" (
    id text NOT NULL,
    name text NOT NULL
);

CREATE TABLE "sheet_field" (
    sheet_id text NOT NULL,
    sequence bigint NOT NULL,
    name_zh text NOT NULL,
    name_en text,
    kind text NOT NULL,
    single_choice_options text[] NOT NULL,
    id text DEFAULT ''::text NOT NULL,
    is_deleted boolean DEFAULT false NOT NULL,
    description_en text,
    description_zh text
);

CREATE TABLE "sheet_filing" (
    id uuid NOT NULL,
    sheet_id text NOT NULL,
    user_id uuid NOT NULL,
    filed_at timestamp with time zone NOT NULL
);

CREATE TABLE "sheet_filing_answer" (
    sheet_id text NOT NULL,
    filing_id uuid NOT NULL,
    answer text NOT NULL,
    field_id text DEFAULT ''::text NOT NULL
);

CREATE TABLE "training" (
    id uuid NOT NULL,
    trainer_id uuid NOT NULL,
    trainee_id uuid NOT NULL,
    record_sheet_filing_id uuid,
    created_at timestamp with time zone DEFAULT '-infinity'::timestamp with time zone NOT NULL,
    end_at timestamp with time zone DEFAULT '-infinity'::timestamp with time zone NOT NULL,
    name text DEFAULT ''::text NOT NULL,
    start_at timestamp with time zone DEFAULT '-infinity'::timestamp with time zone NOT NULL,
    updated_at timestamp with time zone DEFAULT '-infinity'::timestamp with time zone NOT NULL,
    deleted_at timestamp with time zone
);

CREATE TABLE "training_application" (
    id uuid NOT NULL,
    trainee_id uuid NOT NULL,
    name text NOT NULL,
    train_id uuid,
    created_at timestamp with time zone DEFAULT '-infinity'::timestamp with time zone NOT NULL,
    updated_at timestamp with time zone DEFAULT '-infinity'::timestamp with time zone NOT NULL,
    deleted_at timestamp with time zone
);

CREATE TABLE "training_application_response" (
    id uuid NOT NULL,
    application_id uuid NOT NULL,
    trainer_id uuid NOT NULL,
    comment text NOT NULL,
    created_at timestamp with time zone NOT NULL,
    updated_at timestamp with time zone NOT NULL,
    slot_id uuid
);

CREATE TABLE "training_application_slot" (
    id uuid NOT NULL,
    application_id uuid NOT NULL,
    start_at timestamp with time zone NOT NULL,
    end_at timestamp with time zone NOT NULL
);

CREATE TABLE "user" (
    id uuid NOT NULL,
    cid text NOT NULL,
    full_name text NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    roles text[] NOT NULL,
    email text
);

CREATE TABLE "user_atc_permission" (
    user_id uuid NOT NULL,
    position_kind_id text NOT NULL,
    state text NOT NULL,
    solo_expires_at timestamp with time zone
);

CREATE TABLE "user_atc_status" (
    user_id uuid NOT NULL,
    is_visiting boolean NOT NULL,
    is_absent boolean NOT NULL,
    rating text DEFAULT ''::text NOT NULL
);

ALTER TABLE ONLY "atc_application"
    ADD CONSTRAINT pk_atc_application PRIMARY KEY (id);

ALTER TABLE ONLY "atc_booking"
    ADD CONSTRAINT pk_atc_booking PRIMARY KEY (id);

ALTER TABLE ONLY "device_authorization"
    ADD CONSTRAINT pk_device_authorization PRIMARY KEY (device_code);

ALTER TABLE ONLY "event"
    ADD CONSTRAINT pk_event PRIMARY KEY (id);

ALTER TABLE ONLY "event_airspace"
    ADD CONSTRAINT pk_event_airspace PRIMARY KEY (id);

ALTER TABLE ONLY "event_atc_position"
    ADD CONSTRAINT pk_event_atc_position PRIMARY KEY (id);

ALTER TABLE ONLY "event_atc_position_booking"
    ADD CONSTRAINT pk_event_atc_position_booking PRIMARY KEY (event_atc_position_id);

ALTER TABLE ONLY "event_booking"
    ADD CONSTRAINT pk_event_booking PRIMARY KEY (id);

ALTER TABLE ONLY "event_slot"
    ADD CONSTRAINT pk_event_slot PRIMARY KEY (id);

ALTER TABLE ONLY "session"
    ADD CONSTRAINT pk_session PRIMARY KEY (token);

ALTER TABLE ONLY "sheet"
    ADD CONSTRAINT pk_sheet PRIMARY KEY (id);

ALTER TABLE ONLY "sheet_field"
    ADD CONSTRAINT pk_sheet_field PRIMARY KEY (sheet_id, id);

ALTER TABLE ONLY "sheet_filing"
    ADD CONSTRAINT pk_sheet_filing PRIMARY KEY (id);

ALTER TABLE ONLY "sheet_filing_answer"
    ADD CONSTRAINT pk_sheet_filing_answer PRIMARY KEY (sheet_id, field_id, filing_id);

ALTER TABLE ONLY "training"
    ADD CONSTRAINT pk_training PRIMARY KEY (id);

ALTER TABLE ONLY "training_application"
    ADD CONSTRAINT pk_training_application PRIMARY KEY (id);

ALTER TABLE ONLY "training_application_response"
    ADD CONSTRAINT pk_training_application_response PRIMARY KEY (id);

ALTER TABLE ONLY "training_application_slot"
    ADD CONSTRAINT pk_training_application_slot PRIMARY KEY (id);

ALTER TABLE ONLY "user"
    ADD CONSTRAINT pk_user PRIMARY KEY (id);

ALTER TABLE ONLY "user_atc_permission"
    ADD CONSTRAINT pk_user_atc_permission PRIMARY KEY (user_id, position_kind_id);

ALTER TABLE ONLY "user_atc_status"
    ADD CONSTRAINT pk_user_atc_status PRIMARY KEY (user_id);

CREATE INDEX ix_atc_application_application_filing_id ON public.atc_application USING btree (application_filing_id);

CREATE INDEX ix_atc_application_review_filing_id ON public.atc_application USING btree (review_filing_id);

CREATE INDEX ix_atc_application_user_id ON public.atc_application USING btree (user_id);

CREATE INDEX ix_atc_booking_user_id ON public.atc_booking USING btree (user_id);

CREATE UNIQUE INDEX ix_device_authorization_user_code ON public.device_authorization USING btree (user_code);

CREATE INDEX ix_device_authorization_user_id ON public.device_authorization USING btree (user_id);

CREATE INDEX ix_event_airspace_event_id ON public.event_airspace USING btree (event_id);

CREATE UNIQUE INDEX ix_event_atc_position_booking_atc_booking_id ON public.event_atc_position_booking USING btree (atc_booking_id);

CREATE INDEX ix_event_atc_position_booking_user_id ON public.event_atc_position_booking USING btree (user_id);

CREATE INDEX ix_event_atc_position_event_id ON public.event_atc_position USING btree (event_id);

CREATE UNIQUE INDEX ix_event_booking_event_slot_id ON public.event_booking USING btree (event_slot_id);

CREATE INDEX ix_event_booking_user_id ON public.event_booking USING btree (user_id);

CREATE INDEX ix_event_slot_event_airspace_id ON public.event_slot USING btree (event_airspace_id);

CREATE INDEX ix_session_user_id ON public.session USING btree (user_id);

CREATE INDEX ix_sheet_filing_answer_filing_id ON public.sheet_filing_answer USING btree (filing_id);

CREATE INDEX ix_sheet_filing_sheet_id ON public.sheet_filing USING btree (sheet_id);

CREATE INDEX ix_sheet_filing_user_id ON public.sheet_filing USING btree (user_id);

CREATE INDEX ix_training_application_response_application_id ON public.training_application_response USING btree (application_id);

CREATE UNIQUE INDEX ix_training_application_response_slot_id ON public.training_application_response USING btree (slot_id);

CREATE INDEX ix_training_application_response_trainer_id ON public.training_application_response USING btree (trainer_id);

CREATE INDEX ix_training_application_slot_application_id ON public.training_application_slot USING btree (application_id);

CREATE UNIQUE INDEX ix_training_application_train_id ON public.training_application USING btree (train_id);

CREATE INDEX ix_training_application_trainee_id ON public.training_application USING btree (trainee_id);

CREATE INDEX ix_training_record_sheet_filing_id ON public.training USING btree (record_sheet_filing_id);

CREATE INDEX ix_training_trainee_id ON public.training USING btree (trainee_id);

CREATE INDEX ix_training_trainer_id ON public.training USING btree (trainer_id);

CREATE UNIQUE INDEX ix_user_cid ON public."user" USING btree (cid);

CREATE UNIQUE INDEX ix_user_email ON public."user" USING btree (email);

ALTER TABLE ONLY "atc_application"
    ADD CONSTRAINT fk_atc_application_sheet_filing_application_filing_id FOREIGN KEY (application_filing_id) REFERENCES public.sheet_filing(id) ON DELETE CASCADE;

ALTER TABLE ONLY "atc_application"
    ADD CONSTRAINT fk_atc_application_sheet_filing_review_filing_id FOREIGN KEY (review_filing_id) REFERENCES public.sheet_filing(id);

ALTER TABLE ONLY "atc_application"
    ADD CONSTRAINT fk_atc_application_user_user_id FOREIGN KEY (user_id) REFERENCES public."user"(id) ON DELETE CASCADE;

ALTER TABLE ONLY "atc_booking"
    ADD CONSTRAINT fk_atc_booking_user_user_id FOREIGN KEY (user_id) REFERENCES public."user"(id) ON DELETE CASCADE;

ALTER TABLE ONLY "device_authorization"
    ADD CONSTRAINT fk_device_authorization_user_user_id FOREIGN KEY (user_id) REFERENCES public."user"(id);

ALTER TABLE ONLY "event_airspace"
    ADD CONSTRAINT fk_event_airspace_event_event_id FOREIGN KEY (event_id) REFERENCES public.event(id) ON DELETE CASCADE;

ALTER TABLE ONLY "event_atc_position_booking"
    ADD CONSTRAINT fk_event_atc_position_booking_atc_booking_atc_booking_id FOREIGN KEY (atc_booking_id) REFERENCES public.atc_booking(id);

ALTER TABLE ONLY "event_atc_position_booking"
    ADD CONSTRAINT fk_event_atc_position_booking_event_atc_position_event_atc_pos FOREIGN KEY (event_atc_position_id) REFERENCES public.event_atc_position(id) ON DELETE CASCADE;

ALTER TABLE ONLY "event_atc_position_booking"
    ADD CONSTRAINT fk_event_atc_position_booking_user_user_id FOREIGN KEY (user_id) REFERENCES public."user"(id) ON DELETE CASCADE;

ALTER TABLE ONLY "event_atc_position"
    ADD CONSTRAINT fk_event_atc_position_event_event_id FOREIGN KEY (event_id) REFERENCES public.event(id) ON DELETE CASCADE;

ALTER TABLE ONLY "event_booking"
    ADD CONSTRAINT fk_event_booking_event_slot_event_slot_id FOREIGN KEY (event_slot_id) REFERENCES public.event_slot(id) ON DELETE CASCADE;

ALTER TABLE ONLY "event_booking"
    ADD CONSTRAINT fk_event_booking_user_user_id FOREIGN KEY (user_id) REFERENCES public."user"(id) ON DELETE CASCADE;

ALTER TABLE ONLY "event_slot"
    ADD CONSTRAINT fk_event_slot_event_airspace_event_airspace_id FOREIGN KEY (event_airspace_id) REFERENCES public.event_airspace(id) ON DELETE CASCADE;

ALTER TABLE ONLY "session"
    ADD CONSTRAINT fk_session_user_user_id FOREIGN KEY (user_id) REFERENCES public."user"(id) ON DELETE CASCADE;

ALTER TABLE ONLY "sheet_field"
    ADD CONSTRAINT fk_sheet_field_sheet_sheet_id FOREIGN KEY (sheet_id) REFERENCES public.sheet(id) ON DELETE CASCADE;

ALTER TABLE ONLY "sheet_filing_answer"
    ADD CONSTRAINT fk_sheet_filing_answer_sheet_field_sheet_id_field_id FOREIGN KEY (sheet_id, field_id) REFERENCES public.sheet_field(sheet_id, id) ON DELETE CASCADE;

ALTER TABLE ONLY "sheet_filing_answer"
    ADD CONSTRAINT fk_sheet_filing_answer_sheet_filing_filing_id FOREIGN KEY (filing_id) REFERENCES public.sheet_filing(id) ON DELETE CASCADE;

ALTER TABLE ONLY "sheet_filing"
    ADD CONSTRAINT fk_sheet_filing_sheet_sheet_id FOREIGN KEY (sheet_id) REFERENCES public.sheet(id) ON DELETE CASCADE;

ALTER TABLE ONLY "sheet_filing"
    ADD CONSTRAINT fk_sheet_filing_user_user_id FOREIGN KEY (user_id) REFERENCES public."user"(id) ON DELETE CASCADE;

ALTER TABLE ONLY "training_application_response"
    ADD CONSTRAINT fk_training_application_response_training_application_applicat FOREIGN KEY (application_id) REFERENCES public.training_application(id) ON DELETE CASCADE;

ALTER TABLE ONLY "training_application_response"
    ADD CONSTRAINT fk_training_application_response_training_application_slot_slo FOREIGN KEY (slot_id) REFERENCES public.training_application_slot(id);

ALTER TABLE ONLY "training_application_response"
    ADD CONSTRAINT fk_training_application_response_user_trainer_id FOREIGN KEY (trainer_id) REFERENCES public."user"(id) ON DELETE CASCADE;

ALTER TABLE ONLY "training_application_slot"
    ADD CONSTRAINT fk_training_application_slot_training_application_application_ FOREIGN KEY (application_id) REFERENCES public.training_application(id) ON DELETE CASCADE;

ALTER TABLE ONLY "training_application"
    ADD CONSTRAINT fk_training_application_training_train_id FOREIGN KEY (train_id) REFERENCES public.training(id);

ALTER TABLE ONLY "training_application"
    ADD CONSTRAINT fk_training_application_user_trainee_id FOREIGN KEY (trainee_id) REFERENCES public."user"(id) ON DELETE CASCADE;

ALTER TABLE ONLY "training"
    ADD CONSTRAINT fk_training_sheet_filing_record_sheet_filing_id FOREIGN KEY (record_sheet_filing_id) REFERENCES public.sheet_filing(id);

ALTER TABLE ONLY "training"
    ADD CONSTRAINT fk_training_user_trainee_id FOREIGN KEY (trainee_id) REFERENCES public."user"(id) ON DELETE CASCADE;

ALTER TABLE ONLY "training"
    ADD CONSTRAINT fk_training_user_trainer_id FOREIGN KEY (trainer_id) REFERENCES public."user"(id) ON DELETE CASCADE;

ALTER TABLE ONLY "user_atc_permission"
    ADD CONSTRAINT fk_user_atc_permission_user_user_id FOREIGN KEY (user_id) REFERENCES public."user"(id) ON DELETE CASCADE;

ALTER TABLE ONLY "user_atc_status"
    ADD CONSTRAINT fk_user_atc_status_user_user_id FOREIGN KEY (user_id) REFERENCES public."user"(id) ON DELETE CASCADE;
