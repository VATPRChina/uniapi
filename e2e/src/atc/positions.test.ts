import { expect, test as baseTest } from "vitest";
import { getClient } from "../../lib/backend.js";
import type { components } from "../../lib/api/schema.js";

const test = baseTest
  .extend("facilityEngineer", async ({}) => {
    return await getClient(["tech-afv-facility-engineer"]);
  })
  .extend("user", async ({}) => {
    return await getClient([]);
  });

test("GET /api/atc/positions publicly lists positions in operational order", async () => {
  const client = await getClient();
  const { data, error, response } = await client.GET("/api/atc/positions");

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data.length).toBeGreaterThanOrEqual(665);

  const callsigns = data.map((position) => position.callsign);
  const orderedExamples = [
    "ZBAA_CTR",
    "ZBAA_APP",
    "ZBAA_TWR",
    "ZBAA_GND",
    "ZBAA_RMP",
    "ZBAA_DEL",
    "ZBAA_ATIS",
  ];
  const indexes = orderedExamples.map((callsign) => callsigns.indexOf(callsign));

  expect(indexes.every((index) => index >= 0)).toBe(true);
  expect(indexes).toEqual([...indexes].sort((left, right) => left - right));
  expect(callsigns.indexOf("ZBAA_TWR")).toBeLessThan(
    callsigns.indexOf("ZBAA_W_TWR"),
  );
});

test("facility engineers can manage an ATC position by callsign with audit history", async ({
  facilityEngineer,
  user,
}) => {
  const callsign = `ZE2E${Date.now().toString(36).toUpperCase()}_TWR`;
  const createBody: components["schemas"]["AtcPositionSaveRequest"] = {
    category: "standard",
    callsign: ` ${callsign.toLowerCase()} `,
    is_tier_2: true,
    callsign_zh: " 测试塔台 ",
    callsign_en: " E2E Tower ",
    frequency: 123.45,
    cpdlc_code: " E2ET ",
    remarks: " Created by the E2E test. ",
  };

  const forbidden = await user.POST("/api/atc/positions", {
    body: createBody,
  });
  expect(forbidden.response.status).toBe(403);

  const created = await facilityEngineer.POST("/api/atc/positions", {
    body: createBody,
  });
  expect(created.error).toBeFalsy();
  expect(created.response.status).toBe(200);
  expect(created.data).toEqual(
    expect.objectContaining({
      category: "standard",
      callsign,
      is_tier_2: true,
      callsign_zh: "测试塔台",
      callsign_en: "E2E Tower",
      frequency: 123.45,
      frequency_khz: 123450,
      cpdlc_code: "E2ET",
      remarks: "Created by the E2E test.",
    }),
  );

  const found = await facilityEngineer.GET(
    "/api/atc/positions/{callsign}",
    {
      params: { path: { callsign: callsign.toLowerCase() } },
    },
  );
  expect(found.error).toBeFalsy();
  expect(found.data?.callsign).toBe(callsign);

  const updateBody: components["schemas"]["AtcPositionSaveRequest"] = {
    ...createBody,
    callsign,
    category: "chengdu-low-area",
    frequency: 124.35,
    remarks: " Updated by the E2E test. ",
  };
  const updated = await facilityEngineer.PUT(
    "/api/atc/positions/{callsign}",
    {
      params: { path: { callsign } },
      body: updateBody,
    },
  );
  expect(updated.error).toBeFalsy();
  expect(updated.data).toEqual(
    expect.objectContaining({
      callsign,
      category: "chengdu-low-area",
      frequency: 124.35,
      frequency_khz: 124350,
      remarks: "Updated by the E2E test.",
    }),
  );

  const audit = await facilityEngineer.GET(
    "/api/atc/positions/{callsign}/audit",
    { params: { path: { callsign } } },
  );
  expect(audit.error).toBeFalsy();
  expect(audit.data).toHaveLength(2);
  expect(audit.data).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        entity: { kind: "atc-position", id: callsign },
        before: null,
        after: expect.objectContaining({ callsign }),
      }),
      expect.objectContaining({
        entity: { kind: "atc-position", id: callsign },
        before: expect.objectContaining({ category: "standard", callsign }),
        after: expect.objectContaining({
          category: "chengdu-low-area",
          callsign,
        }),
      }),
    ]),
  );

  const deleted = await facilityEngineer.DELETE(
    "/api/atc/positions/{callsign}",
    { params: { path: { callsign } } },
  );
  expect(deleted.error).toBeFalsy();
  expect(deleted.response.status).toBe(204);

  const missing = await facilityEngineer.GET(
    "/api/atc/positions/{callsign}",
    { params: { path: { callsign } } },
  );
  expect(missing.response.status).toBe(404);
});
