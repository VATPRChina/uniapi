import { sql } from "drizzle-orm";
import {
  text,
  timestamp,
  uniqueIndex,
  index,
  foreignKey,
  pgTable,
  uuid,
} from "drizzle-orm/pg-core";

export const user = pgTable(
  "user",
  {
    id: uuid().primaryKey().notNull(),
    cid: text().notNull(),
    fullName: text("full_name").notNull(),
    createdAt: timestamp("created_at", { withTimezone: true, mode: "string" })
      .defaultNow()
      .notNull(),
    updatedAt: timestamp("updated_at", { withTimezone: true, mode: "string" })
      .defaultNow()
      .notNull(),
    roles: text().array().notNull(),
    email: text(),
  },
  (table) => [
    uniqueIndex("ix_user_cid").using(
      "btree",
      table.cid.asc().nullsLast().op("text_ops")
    ),
    uniqueIndex("ix_user_email").using(
      "btree",
      table.email.asc().nullsLast().op("text_ops")
    ),
  ]
);

export const deviceAuthorization = pgTable(
  "device_authorization",
  {
    deviceCode: uuid("device_code").primaryKey().notNull(),
    userCode: text("user_code").notNull(),
    expiresAt: timestamp("expires_at", { withTimezone: true }).notNull(),
    createdAt: timestamp("created_at", { withTimezone: true, mode: "string" })
      .defaultNow()
      .notNull(),
    clientId: text("client_id").notNull(),
    userId: uuid("user_id"),
  },
  (table) => [
    uniqueIndex("ix_device_authorization_user_code").using(
      "btree",
      table.userCode.asc().nullsLast().op("text_ops")
    ),
    index("ix_device_authorization_user_id").using(
      "btree",
      table.userId.asc().nullsLast().op("uuid_ops")
    ),
    foreignKey({
      columns: [table.userId],
      foreignColumns: [user.id],
      name: "fk_device_authorization_user_user_id",
    }),
  ]
);

export const session = pgTable(
  "session",
  {
    token: uuid().primaryKey().notNull(),
    userId: uuid("user_id").notNull(),
    userUpdatedAt: timestamp("user_updated_at", {
      withTimezone: true,
    }).notNull(),
    expiresIn: timestamp("expires_in", { withTimezone: true }).notNull(),
    createdAt: timestamp("created_at", { withTimezone: true, mode: "string" })
      .defaultNow()
      .notNull(),
    code: uuid(),
    clientId: text("client_id").default("").notNull(),
  },
  (table) => [
    index("ix_session_user_id").using(
      "btree",
      table.userId.asc().nullsLast().op("uuid_ops")
    ),
    foreignKey({
      columns: [table.userId],
      foreignColumns: [user.id],
      name: "fk_session_user_user_id",
    }).onDelete("cascade"),
  ]
);
