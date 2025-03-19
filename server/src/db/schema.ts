import {
  pgTable,
  varchar,
  uniqueIndex,
  index,
  foreignKey,
  uuid,
  timestamp,
  text,
  pgSchema,
  doublePrecision,
  integer,
  bigint,
  char,
} from "drizzle-orm/pg-core";
import { sql } from "drizzle-orm";

export const flight = pgTable(
  "flight",
  {
    callsign: text().notNull(),
    cid: text().notNull(),
    lastObservedAt: timestamp("last_observed_at", {
      withTimezone: true,
    }).notNull(),
    latitude: doublePrecision().notNull(),
    longitude: doublePrecision().notNull(),
    // You can use { mode: "bigint" } if numbers are exceeding js number limitations
    altitude: bigint({ mode: "number" }).notNull(),
    departure: text().notNull(),
    departureGate: text("departure_gate"),
    arrival: text().notNull(),
    arrivalGate: text("arrival_gate"),
    // You can use { mode: "bigint" } if numbers are exceeding js number limitations
    cruiseTas: bigint("cruise_tas", { mode: "number" }).notNull(),
    rawRoute: text("raw_route").notNull(),
    state: text().notNull(),
    id: uuid()
      .default(sql`'00000000-0000-0000-0000-000000000000'`)
      .primaryKey()
      .notNull(),
    arrivalRunway: text("arrival_runway"),
    departureRunway: text("departure_runway"),
    finalizedAt: timestamp("finalized_at", { withTimezone: true }),
    aircraft: text().default("").notNull(),
    equipment: text().default("").notNull(),
    navigationPerformance: text("navigation_performance").default("").notNull(),
    transponder: text().default("").notNull(),
  },
  (table) => [
    index("ix_flight_callsign").using(
      "btree",
      table.callsign.asc().nullsLast().op("text_ops")
    ),
    index("ix_flight_cid").using(
      "btree",
      table.cid.asc().nullsLast().op("text_ops")
    ),
  ]
);
