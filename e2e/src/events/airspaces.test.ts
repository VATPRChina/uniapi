import { expect, test as baseTest } from "vitest";
import { getClient } from "../../lib/backend.js";

const test = baseTest
  .extend("coordinator", async ({}) => {
    return await getClient(["event-coordinator"]);
  })
  .extend("user", async ({}) => {
    return await getClient([]);
  })
  .extend("event", async ({ coordinator }) => {
    const suffix = Date.now().toString();

    const event = await coordinator.POST("/api/events", {
      body: {
        title: `E2E Slot Test Event ${suffix}`,
        title_en: `E2E Slot Test Event EN ${suffix}`,
        description: "Created by the e2e slot test.",
        start_at: "2031-10-20T10:00:00Z",
        end_at: "2031-10-20T13:00:00Z",
        start_booking_at: "2031-10-01T00:00:00Z",
        end_booking_at: "2031-10-19T00:00:00Z",
      },
    });

    return event.data;
  });

test("POST /api/events/{event_id}/airspaces creates an airspace", async ({
  coordinator,
  event,
}) => {
  const suffix = Date.now().toString();
  const airspace = {
    name: `E2E Airspace ${suffix}`,
    icao_codes: ["ZBAA", "ZGGG"],
    description:
      "Created by the e2e POST /api/events/{event_id}/airspaces test.",
  };

  const createdAirspace = await coordinator.POST(
    "/api/events/{event_id}/airspaces",
    {
      params: { path: { event_id: event.id } },
      body: airspace,
    },
  );

  expect(createdAirspace.error).toBeFalsy();
  expect(createdAirspace.response.status).toBe(200);
  expect(createdAirspace.data).toEqual(
    expect.objectContaining({
      id: expect.stringMatching(/^[0-9A-HJKMNP-TV-Z]{26}$/),
      event_id: event.id,
      name: airspace.name,
      icao_codes: airspace.icao_codes,
      description: airspace.description,
    }),
  );
});
