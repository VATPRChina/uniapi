import { expect, test } from "vitest";
import { getClient } from "../../lib/backend.js";

test("GET /api/users/me/atc/online-time returns the current quarter total", async () => {
  const controller = await getClient(["controller"], { cid: "1573922" });

  const { data, error, response } = await controller.GET(
    "/api/users/me/atc/online-time",
  );

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);

  const asOf = new Date(data.as_of);
  const quarter = Math.floor(asOf.getUTCMonth() / 3) + 1;
  const periodStart = new Date(
    Date.UTC(asOf.getUTCFullYear(), (quarter - 1) * 3, 1),
  );

  expect(data).toEqual({
    period: `${asOf.getUTCFullYear()}Q${quarter}`,
    period_start: expect.any(String),
    as_of: expect.any(String),
    total_seconds: expect.any(Number),
  });
  expect(Number.isNaN(asOf.getTime())).toBe(false);
  expect(new Date(data.period_start).getTime()).toBe(periodStart.getTime());
  expect(data.total_seconds).toBeGreaterThan(0);
});
