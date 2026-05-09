import { expect, test } from "vitest";
import { getClient } from "../lib/backend.js";

test("GET /health returns healthy status", async () => {
  const client = await getClient();
  const { data, error, response } = await client.GET("/health");

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toEqual({
    status: "ok",
    database: "ok",
  });
});
