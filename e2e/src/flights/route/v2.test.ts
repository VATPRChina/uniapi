import { expect, test } from "vitest";
import { getClient } from "../../../lib/backend.js";

const params = { query: { route: "ZBAA DCT ZSPD" } };

test("route diagnostics require authentication", async () => {
  const client = await getClient();
  const { response } = await client.GET("/api/flights/route/v2", { params });
  expect(response.status).toBe(401);
});

test("route diagnostics reject users without an internal role", async () => {
  const client = await getClient(["user"]);
  const { response } = await client.GET("/api/flights/route/v2", { params });
  expect(response.status).toBe(403);
});

test("engineers can inspect expanded route geometry", async () => {
  const client = await getClient(["software-engineer"]);
  const { data, error, response } = await client.GET("/api/flights/route/v2", { params });
  expect(response.status).toBe(200);
  expect(error).toBeFalsy();
  expect(data?.legs.length).toBeGreaterThan(0);
  expect(data?.legs[0].from.identifier).toBe("ZBAA");
  expect(data?.legs.at(-1)?.to.identifier).toBe("ZSPD");
  expect(Array.isArray(data?.segments)).toBe(true);
  expect(Array.isArray(data?.sid_candidates)).toBe(true);
});
