import {
  pgSchema,
  uuid,
  doublePrecision,
  integer,
  text,
  index,
  foreignKey,
  uniqueIndex,
  bigint,
  char,
} from "drizzle-orm/pg-core";

export const navdata = pgSchema("navdata");

export const airportGate = navdata.table(
  "airport_gate",
  {
    id: uuid().primaryKey().notNull(),
    airportId: uuid("airport_id").notNull(),
    identifier: text().notNull(),
    latitude: doublePrecision().notNull(),
    longitude: doublePrecision().notNull(),
  },
  (table) => [
    index("ix_airport_gate_airport_id").using(
      "btree",
      table.airportId.asc().nullsLast().op("uuid_ops")
    ),
    foreignKey({
      columns: [table.airportId],
      foreignColumns: [airport.id],
      name: "fk_airport_gate_airport_airport_id",
    }).onDelete("cascade"),
  ]
);

export const airport = navdata.table(
  "airport",
  {
    id: uuid().primaryKey().notNull(),
    identifier: text().notNull(),
    latitude: doublePrecision().notNull(),
    longitude: doublePrecision().notNull(),
    elevation: integer().notNull(),
  },
  (table) => [
    uniqueIndex("ix_airport_identifier").using(
      "btree",
      table.identifier.asc().nullsLast().op("text_ops")
    ),
  ]
);

export const runway = navdata.table(
  "runway",
  {
    id: uuid().primaryKey().notNull(),
    airportId: uuid("airport_id").notNull(),
    identifier: text().notNull(),
    latitude: doublePrecision().notNull(),
    longitude: doublePrecision().notNull(),
  },
  (table) => [
    index("ix_runway_airport_id").using(
      "btree",
      table.airportId.asc().nullsLast().op("uuid_ops")
    ),
    foreignKey({
      columns: [table.airportId],
      foreignColumns: [airport.id],
      name: "fk_runway_airport_airport_id",
    }).onDelete("cascade"),
  ]
);

export const preferredRoute = navdata.table("preferred_route", {
  id: uuid().primaryKey().notNull(),
  departure: text().notNull(),
  arrival: text().notNull(),
  rawRoute: text("raw_route").notNull(),
});

export const ndbNavaid = navdata.table("ndb_navaid", {
  id: uuid().primaryKey().notNull(),
  sectionCode: text("section_code").notNull(),
  airportIcaoIdent: text("airport_icao_ident"),
  icaoCode: text("icao_code").notNull(),
  identifier: text().notNull(),
  latitude: doublePrecision().notNull(),
  longitude: doublePrecision().notNull(),
});

export const procedure = navdata.table(
  "procedure",
  {
    id: uuid().primaryKey().notNull(),
    airportId: uuid("airport_id").notNull(),
    identifier: text().notNull(),
    subsectionCode: char("subsection_code", { length: 1 }).notNull(),
  },
  (table) => [
    index("ix_procedure_airport_id").using(
      "btree",
      table.airportId.asc().nullsLast().op("uuid_ops")
    ),
    foreignKey({
      columns: [table.airportId],
      foreignColumns: [airport.id],
      name: "fk_procedure_airport_airport_id",
    }).onDelete("cascade"),
  ]
);

export const vhfNavaid = navdata.table("vhf_navaid", {
  id: uuid().primaryKey().notNull(),
  icaoCode: text("icao_code").notNull(),
  vorIdentifier: text("vor_identifier").notNull(),
  vorLatitude: doublePrecision("vor_latitude"),
  vorLongitude: doublePrecision("vor_longitude"),
  dmeIdentifier: text("dme_identifier"),
  dmeLatitude: doublePrecision("dme_latitude"),
  dmeLongitude: doublePrecision("dme_longitude"),
});

export const waypoint = navdata.table("waypoint", {
  id: uuid().primaryKey().notNull(),
  sectionCode: text("section_code").notNull(),
  regionCode: text("region_code").notNull(),
  icaoCode: text("icao_code").notNull(),
  identifier: text().notNull(),
  latitude: doublePrecision().notNull(),
  longitude: doublePrecision().notNull(),
});

export const airwayFix = navdata.table(
  "airway_fix",
  {
    id: uuid().primaryKey().notNull(),
    airwayId: uuid("airway_id").notNull(),
    // You can use { mode: "bigint" } if numbers are exceeding js number limitations
    sequenceNumber: bigint("sequence_number", { mode: "number" }).notNull(),
    fixIdentifier: text("fix_identifier").notNull(),
    fixIcaoCode: text("fix_icao_code").notNull(),
    descriptionCode: text("description_code").notNull(),
  },
  (table) => [
    index("ix_airway_fix_airway_id").using(
      "btree",
      table.airwayId.asc().nullsLast().op("uuid_ops")
    ),
    foreignKey({
      columns: [table.airwayId],
      foreignColumns: [airway.id],
      name: "fk_airway_fix_airway_airway_id",
    }).onDelete("cascade"),
  ]
);

export const airway = navdata.table("airway", {
  id: uuid().primaryKey().notNull(),
  identifier: text().notNull(),
});
