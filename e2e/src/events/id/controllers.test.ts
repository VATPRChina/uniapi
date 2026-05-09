import { expect, test as baseTest } from "vitest";
import { getClient } from "../../../lib/backend.js";

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
        title: `E2E Controller Test Event ${suffix}`,
        title_en: `E2E Controller Test Event EN ${suffix}`,
        description: "Created by the e2e controller test.",
        start_at: "2031-10-20T10:00:00Z",
        end_at: "2031-10-20T13:00:00Z",
        start_booking_at: "2031-10-01T00:00:00Z",
        end_booking_at: "2031-10-19T00:00:00Z",
        start_atc_booking_at: "2020-01-01T00:00:00Z",
      },
    });

    return event.data;
  });

test("POST /api/events/{event_id}/controllers creates a controller position", async ({
  coordinator,
  event,
}) => {
  const position = {
    callsign: "ZBAA_TWR",
    start_at: "2031-10-20T10:30:00Z",
    end_at: "2031-10-20T12:30:00Z",
    remarks: "E2E controller position.",
    position_kind_id: "TWR",
    minimum_controller_state: "student" as const,
  };

  const createdPosition = await coordinator.POST(
    "/api/events/{event_id}/controllers",
    {
      params: { path: { event_id: event.id } },
      body: position,
    },
  );

  expect(createdPosition.error).toBeFalsy();
  expect(createdPosition.response.status).toBe(200);
  expect(createdPosition.data).toEqual(
    expect.objectContaining({
      id: expect.stringMatching(/^[0-9A-HJKMNP-TV-Z]{26}$/),
      callsign: position.callsign,
      start_at: position.start_at,
      end_at: position.end_at,
      remarks: position.remarks,
      position_kind_id: position.position_kind_id,
      minimum_controller_state: position.minimum_controller_state,
      booking: null,
      event: expect.objectContaining({
        id: event.id,
      }),
    }),
  );
});

test("POST /api/events/{event_id}/controllers cannot be invoked by user", async ({
  user,
  event,
}) => {
  const response = await user.POST("/api/events/{event_id}/controllers", {
    params: { path: { event_id: event.id } },
    body: {
      callsign: "ZBAA_GND",
      start_at: "2031-10-20T10:30:00Z",
      end_at: "2031-10-20T12:30:00Z",
      remarks: "E2E forbidden controller position.",
      position_kind_id: "GND",
      minimum_controller_state: "student",
    },
  });

  expect(response.error).toBeTruthy();
  expect(response.response.status).toBe(403);
});

test("GET /api/events/{event_id}/controllers lists controller positions", async ({
  coordinator,
  user,
  event,
}) => {
  const position = {
    callsign: "ZBAA_APP",
    start_at: "2031-10-20T10:30:00Z",
    end_at: "2031-10-20T12:30:00Z",
    remarks: "E2E listed controller position.",
    position_kind_id: "APP",
    minimum_controller_state: "under-mentor" as const,
  };

  const createdPosition = await coordinator.POST(
    "/api/events/{event_id}/controllers",
    {
      params: { path: { event_id: event.id } },
      body: position,
    },
  );

  expect(createdPosition.error).toBeFalsy();
  expect(createdPosition.response.status).toBe(200);
  expect(createdPosition.data).toBeTruthy();

  const { data, error, response } = await user.GET(
    "/api/events/{event_id}/controllers",
    {
      params: { path: { event_id: event.id } },
    },
  );

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        id: createdPosition.data.id,
        callsign: position.callsign,
        start_at: position.start_at,
        end_at: position.end_at,
        remarks: position.remarks,
        position_kind_id: position.position_kind_id,
        minimum_controller_state: position.minimum_controller_state,
        booking: null,
        event: expect.objectContaining({
          id: event.id,
        }),
      }),
    ]),
  );
});
