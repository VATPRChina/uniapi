import test from "ava";
import { getBackend } from "../lib/backend.js";

test("GET /health returns healthy status", async (t) => {
  const client = await getBackend();
  const { data, error, response } = await client.GET("/health");

  t.falsy(error);
  t.is(response.status, 200);
  t.deepEqual(data, {
    status: "ok",
    database: "ok",
  });
});
