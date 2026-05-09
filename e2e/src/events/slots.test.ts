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
  })
  .extend("airspace", async ({ coordinator, event }) => {
    const suffix = Date.now().toString();
    const airspace = await coordinator.POST(
      "/api/events/{event_id}/airspaces",
      {
        params: {
          path: {
            event_id: event.id,
          },
        },
        body: {
          name: `E2E Slot Airspace ${suffix}`,
          icao_codes: ["ZBAA", "ZSSS"],
          description: "Created by the e2e slot creation test.",
        },
      },
    );
    return airspace.data;
  });

test("POST /api/events/{event_id}/slots creates a slot", async ({
  coordinator,
  event,
  airspace,
}) => {
  const slot = {
    airspace_id: airspace.id,
    enter_at: "2031-10-20T10:30:00Z",
    leave_at: "2031-10-20T12:30:00Z",
    callsign: "CCA123",
    aircraft_type_icao: "B738",
  };

  const createdSlot = await coordinator.POST("/api/events/{event_id}/slots", {
    params: { path: { event_id: event.id } },
    body: slot,
  });

  expect(createdSlot.error).toBeFalsy();
  expect(createdSlot.response.status).toBe(200);
  expect(createdSlot.data).toEqual(
    expect.objectContaining({
      id: expect.stringMatching(/^[0-9A-HJKMNP-TV-Z]{26}$/),
      event_id: event.id,
      airspace_id: airspace.id,
      enter_at: slot.enter_at,
      leave_at: slot.leave_at,
      callsign: slot.callsign,
      aircraft_type_icao: slot.aircraft_type_icao,
      booking: null,
      airspace: expect.objectContaining({
        id: airspace.id,
        event_id: event.id,
        name: airspace.name,
        icao_codes: airspace.icao_codes,
        description: airspace.description,
      }),
    }),
  );
});

test("POST /api/events/{event_id}/slots cannot be invoked by user", async ({
  user,
  event,
  airspace,
}) => {
  const response = await user.POST("/api/events/{event_id}/slots", {
    params: { path: { event_id: event.id } },
    body: {
      airspace_id: airspace.id,
      enter_at: "2031-10-20T10:30:00Z",
      leave_at: "2031-10-20T12:30:00Z",
      callsign: "CCA123",
      aircraft_type_icao: "B738",
    },
  });

  expect(response.error).toBeTruthy();
  expect(response.response.status).toBe(403);
});
