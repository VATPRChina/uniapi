import { sql } from "drizzle-orm";
import {
  timestamp,
  uniqueIndex,
  index,
  foreignKey,
  text,
  pgTable,
  uuid,
} from "drizzle-orm/pg-core";
import { user } from "./user";

export const eventBooking = pgTable(
  "event_booking",
  {
    id: uuid().primaryKey().notNull(),
    userId: uuid("user_id").notNull(),
    eventSlotId: uuid("event_slot_id").notNull(),
    createdAt: timestamp("created_at", { withTimezone: true, mode: "string" })
      .defaultNow()
      .notNull(),
    updatedAt: timestamp("updated_at", { withTimezone: true, mode: "string" })
      .defaultNow()
      .notNull(),
  },
  (table) => [
    uniqueIndex("ix_event_booking_event_slot_id").using(
      "btree",
      table.eventSlotId.asc().nullsLast().op("uuid_ops")
    ),
    index("ix_event_booking_user_id").using(
      "btree",
      table.userId.asc().nullsLast().op("uuid_ops")
    ),
    foreignKey({
      columns: [table.eventSlotId],
      foreignColumns: [eventSlot.id],
      name: "fk_event_booking_event_slot_event_slot_id",
    }).onDelete("cascade"),
    foreignKey({
      columns: [table.userId],
      foreignColumns: [user.id],
      name: "fk_event_booking_user_user_id",
    }).onDelete("cascade"),
  ]
);

export const eventAirspace = pgTable(
  "event_airspace",
  {
    id: uuid().primaryKey().notNull(),
    eventId: uuid("event_id").notNull(),
    name: text().notNull(),
    createdAt: timestamp("created_at", { withTimezone: true, mode: "string" })
      .defaultNow()
      .notNull(),
    updatedAt: timestamp("updated_at", { withTimezone: true, mode: "string" })
      .defaultNow()
      .notNull(),
    icaoCodes: text("icao_codes").array().default(["RAY"]).notNull(),
    description: text().default("").notNull(),
  },
  (table) => [
    index("ix_event_airspace_event_id").using(
      "btree",
      table.eventId.asc().nullsLast().op("uuid_ops")
    ),
    foreignKey({
      columns: [table.eventId],
      foreignColumns: [event.id],
      name: "fk_event_airspace_event_event_id",
    }).onDelete("cascade"),
  ]
);

export const event = pgTable("event", {
  id: uuid().primaryKey().notNull(),
  title: text().notNull(),
  startAt: timestamp("start_at", {
    withTimezone: true,
  }).notNull(),
  endAt: timestamp("end_at", { withTimezone: true, mode: "string" }).notNull(),
  createdAt: timestamp("created_at", { withTimezone: true, mode: "string" })
    .defaultNow()
    .notNull(),
  updatedAt: timestamp("updated_at", { withTimezone: true, mode: "string" })
    .defaultNow()
    .notNull(),
  endBookingAt: timestamp("end_booking_at", {
    withTimezone: true,
  })
    .default(sql`'-infinity'`)
    .notNull(),
  startBookingAt: timestamp("start_booking_at", {
    withTimezone: true,
  })
    .default(sql`'-infinity'`)
    .notNull(),
  imageUrl: text("image_url"),
  description: text().default("").notNull(),
});

export const eventSlot = pgTable(
  "event_slot",
  {
    id: uuid().primaryKey().notNull(),
    eventAirspaceId: uuid("event_airspace_id").notNull(),
    enterAt: timestamp("enter_at", { withTimezone: true }).notNull(),
    createdAt: timestamp("created_at", { withTimezone: true, mode: "string" })
      .defaultNow()
      .notNull(),
    updatedAt: timestamp("updated_at", { withTimezone: true, mode: "string" })
      .defaultNow()
      .notNull(),
    leaveAt: timestamp("leave_at", { withTimezone: true, mode: "string" }),
    aircraftTypeIcao: text("aircraft_type_icao"),
    callsign: text(),
  },
  (table) => [
    index("ix_event_slot_event_airspace_id").using(
      "btree",
      table.eventAirspaceId.asc().nullsLast().op("uuid_ops")
    ),
    foreignKey({
      columns: [table.eventAirspaceId],
      foreignColumns: [eventAirspace.id],
      name: "fk_event_slot_event_airspace_event_airspace_id",
    }).onDelete("cascade"),
  ]
);
