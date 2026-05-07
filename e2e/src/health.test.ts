import test from "ava";
import { getBackend } from "../lib/backend.js";

test("GET /health returns healthy status", async (t) => {
  const baseUrl = await getBackend();
  const response = await fetch(`${baseUrl}/health`);

  t.is(response.status, 200);
  t.deepEqual(await response.json(), {
    status: "ok",
    database: "ok",
  });
});
