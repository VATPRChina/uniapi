import { expect, test } from "vitest";
import { getClient } from "../../lib/backend.js";

test("GET /api/users/me/atc/online-time returns quarter and lifetime totals by position", async () => {
  const controller = await getClient(["controller"], {
    cid: process.env.E2E_CONTROLLER_CID ?? "1573922",
  });

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
    by_position: {
      S1: expect.any(Number),
      S2: expect.any(Number),
      S3: expect.any(Number),
      "C1+": expect.any(Number),
    },
    lifetime: {
      total_seconds: expect.any(Number),
      by_position: {
        S1: expect.any(Number),
        S2: expect.any(Number),
        S3: expect.any(Number),
        "C1+": expect.any(Number),
      },
    },
  });
  expect(Number.isNaN(asOf.getTime())).toBe(false);
  expect(new Date(data.period_start).getTime()).toBe(periodStart.getTime());
  expect(data.total_seconds).toBeGreaterThanOrEqual(0);
  expect(data.lifetime.total_seconds).toBeGreaterThanOrEqual(
    data.total_seconds,
  );
  for (const key of ["S1", "S2", "S3", "C1+"] as const) {
    expect(data.by_position[key]).toBeGreaterThanOrEqual(0);
    expect(Number.isInteger(data.by_position[key])).toBe(true);
    expect(Number.isInteger(data.lifetime.by_position[key])).toBe(true);
    expect(data.lifetime.by_position[key]).toBeGreaterThanOrEqual(
      data.by_position[key],
    );
  }
  for (const summary of [data, data.lifetime]) {
    expect(
      Object.values(summary.by_position).reduce((a, b) => a + b, 0),
    ).toBe(summary.total_seconds);
  }
});
