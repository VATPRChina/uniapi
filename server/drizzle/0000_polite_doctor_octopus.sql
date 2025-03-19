-- Current sql file was generated after introspecting the database
-- If you want to run this migration please uncomment this code before executing migrations
CREATE TABLE "__EFMigrationsHistory" (
	"migration_id" varchar(150) PRIMARY KEY NOT NULL,
	"product_version" varchar(32) NOT NULL
);
--> statement-breakpoint
CREATE TABLE "event_booking" (
	"id" uuid PRIMARY KEY NOT NULL,
	"user_id" uuid NOT NULL,
	"event_slot_id" uuid NOT NULL,
	"created_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
	"updated_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);
--> statement-breakpoint
CREATE TABLE "event_airspace" (
	"id" uuid PRIMARY KEY NOT NULL,
	"event_id" uuid NOT NULL,
	"name" text NOT NULL,
	"created_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
	"updated_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
	"icao_codes" text[] DEFAULT '{"RAY"}' NOT NULL,
	"description" text DEFAULT '' NOT NULL
);
--> statement-breakpoint
CREATE TABLE "event" (
	"id" uuid PRIMARY KEY NOT NULL,
	"title" text NOT NULL,
	"start_at" timestamp with time zone NOT NULL,
	"end_at" timestamp with time zone NOT NULL,
	"created_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
	"updated_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
	"end_booking_at" timestamp with time zone DEFAULT '-infinity' NOT NULL,
	"start_booking_at" timestamp with time zone DEFAULT '-infinity' NOT NULL,
	"image_url" text,
	"description" text DEFAULT '' NOT NULL
);
--> statement-breakpoint
CREATE TABLE "event_slot" (
	"id" uuid PRIMARY KEY NOT NULL,
	"event_airspace_id" uuid NOT NULL,
	"enter_at" timestamp with time zone NOT NULL,
	"created_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
	"updated_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
	"leave_at" timestamp with time zone,
	"aircraft_type_icao" text,
	"callsign" text
);
--> statement-breakpoint
CREATE TABLE "user" (
	"id" uuid PRIMARY KEY NOT NULL,
	"cid" text NOT NULL,
	"full_name" text NOT NULL,
	"created_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
	"updated_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
	"roles" text[] NOT NULL,
	"email" text
);
--> statement-breakpoint
CREATE TABLE "device_authorization" (
	"device_code" uuid PRIMARY KEY NOT NULL,
	"user_code" text NOT NULL,
	"expires_at" timestamp with time zone NOT NULL,
	"created_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
	"client_id" text NOT NULL,
	"user_id" uuid
);
--> statement-breakpoint
CREATE TABLE "session" (
	"token" uuid PRIMARY KEY NOT NULL,
	"user_id" uuid NOT NULL,
	"user_updated_at" timestamp with time zone NOT NULL,
	"expires_in" timestamp with time zone NOT NULL,
	"created_at" timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
	"code" uuid,
	"client_id" text DEFAULT '' NOT NULL
);
--> statement-breakpoint
CREATE TABLE "flight" (
	"callsign" text NOT NULL,
	"cid" text NOT NULL,
	"last_observed_at" timestamp with time zone NOT NULL,
	"latitude" double precision NOT NULL,
	"longitude" double precision NOT NULL,
	"altitude" bigint NOT NULL,
	"departure" text NOT NULL,
	"departure_gate" text,
	"arrival" text NOT NULL,
	"arrival_gate" text,
	"cruise_tas" bigint NOT NULL,
	"raw_route" text NOT NULL,
	"state" text NOT NULL,
	"id" uuid PRIMARY KEY DEFAULT '00000000-0000-0000-0000-000000000000' NOT NULL,
	"arrival_runway" text,
	"departure_runway" text,
	"finalized_at" timestamp with time zone,
	"aircraft" text DEFAULT '' NOT NULL,
	"equipment" text DEFAULT '' NOT NULL,
	"navigation_performance" text DEFAULT '' NOT NULL,
	"transponder" text DEFAULT '' NOT NULL
);
--> statement-breakpoint
ALTER TABLE "event_booking" ADD CONSTRAINT "fk_event_booking_event_slot_event_slot_id" FOREIGN KEY ("event_slot_id") REFERENCES "public"."event_slot"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "event_booking" ADD CONSTRAINT "fk_event_booking_user_user_id" FOREIGN KEY ("user_id") REFERENCES "public"."user"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "event_airspace" ADD CONSTRAINT "fk_event_airspace_event_event_id" FOREIGN KEY ("event_id") REFERENCES "public"."event"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "event_slot" ADD CONSTRAINT "fk_event_slot_event_airspace_event_airspace_id" FOREIGN KEY ("event_airspace_id") REFERENCES "public"."event_airspace"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "device_authorization" ADD CONSTRAINT "fk_device_authorization_user_user_id" FOREIGN KEY ("user_id") REFERENCES "public"."user"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "session" ADD CONSTRAINT "fk_session_user_user_id" FOREIGN KEY ("user_id") REFERENCES "public"."user"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
CREATE UNIQUE INDEX "ix_event_booking_event_slot_id" ON "event_booking" USING btree ("event_slot_id" uuid_ops);--> statement-breakpoint
CREATE INDEX "ix_event_booking_user_id" ON "event_booking" USING btree ("user_id" uuid_ops);--> statement-breakpoint
CREATE INDEX "ix_event_airspace_event_id" ON "event_airspace" USING btree ("event_id" uuid_ops);--> statement-breakpoint
CREATE INDEX "ix_event_slot_event_airspace_id" ON "event_slot" USING btree ("event_airspace_id" uuid_ops);--> statement-breakpoint
CREATE UNIQUE INDEX "ix_user_cid" ON "user" USING btree ("cid" text_ops);--> statement-breakpoint
CREATE UNIQUE INDEX "ix_user_email" ON "user" USING btree ("email" text_ops);--> statement-breakpoint
CREATE UNIQUE INDEX "ix_device_authorization_user_code" ON "device_authorization" USING btree ("user_code" text_ops);--> statement-breakpoint
CREATE INDEX "ix_device_authorization_user_id" ON "device_authorization" USING btree ("user_id" uuid_ops);--> statement-breakpoint
CREATE INDEX "ix_session_user_id" ON "session" USING btree ("user_id" uuid_ops);--> statement-breakpoint
CREATE INDEX "ix_flight_callsign" ON "flight" USING btree ("callsign" text_ops);--> statement-breakpoint
CREATE INDEX "ix_flight_cid" ON "flight" USING btree ("cid" text_ops);
