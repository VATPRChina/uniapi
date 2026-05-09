import { expect, test as baseTest } from "vitest";
import { getClient } from "../../../../lib/backend.js";

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
        title: `E2E Controller ID Test Event ${suffix}`,
        title_en: `E2E Controller ID Test Event EN ${suffix}`,
        description: "Created by the e2e controller id test.",
        start_at: "2031-10-20T10:00:00Z",
        end_at: "2031-10-20T13:00:00Z",
        start_booking_at: "2031-10-01T00:00:00Z",
        end_booking_at: "2031-10-19T00:00:00Z",
        start_atc_booking_at: "2020-01-01T00:00:00Z",
      },
    });

    return event.data;
  })
  .extend("position", async ({ coordinator, event }) => {
    const position = await coordinator.POST("/api/events/{event_id}/controllers", {
      params: { path: { event_id: event.id } },
      body: {
        callsign: "ZBAA_TWR",
        start_at: "2031-10-20T10:30:00Z",
        end_at: "2031-10-20T12:30:00Z",
        remarks: "E2E controller id position.",
        position_kind_id: "TWR",
        minimum_controller_state: "student",
      },
    });

    expect(position.error).toBeFalsy();
    expect(position.response.status).toBe(200);
    expect(position.data).toBeTruthy();

    return position.data;
  });

test("PUT /api/events/{event_id}/controllers/{position_id} updates a controller position", async ({
  coordinator,
  event,
  position,
}) => {
  const updatedPosition = {
    callsign: "ZBAA_APP",
    start_at: "2031-10-20T11:00:00Z",
    end_at: "2031-10-20T12:45:00Z",
    remarks: "E2E updated controller position.",
    position_kind_id: "APP",
    minimum_controller_state: "under-mentor" as const,
  };

  const response = await coordinator.PUT(
    "/api/events/{event_id}/controllers/{position_id}",
    {
      params: {
        path: {
          event_id: event.id,
          position_id: position.id,
        },
      },
      body: updatedPosition,
    },
  );

  expect(response.error).toBeFalsy();
  expect(response.response.status).toBe(200);
  expect(response.data).toEqual(
    expect.objectContaining({
      id: position.id,
      callsign: updatedPosition.callsign,
      start_at: updatedPosition.start_at,
      end_at: updatedPosition.end_at,
      remarks: updatedPosition.remarks,
      position_kind_id: updatedPosition.position_kind_id,
      minimum_controller_state: updatedPosition.minimum_controller_state,
      booking: null,
      event: expect.objectContaining({
        id: event.id,
      }),
    }),
  );
});

test("PUT /api/events/{event_id}/controllers/{position_id} requires event coordinator role", async ({
  user,
  event,
  position,
}) => {
  const response = await user.PUT(
    "/api/events/{event_id}/controllers/{position_id}",
    {
      params: {
        path: {
          event_id: event.id,
          position_id: position.id,
        },
      },
      body: {
        callsign: "ZBAA_APP",
        start_at: "2031-10-20T11:00:00Z",
        end_at: "2031-10-20T12:45:00Z",
        remarks: "E2E forbidden controller position update.",
        position_kind_id: "APP",
        minimum_controller_state: "student",
      },
    },
  );

  expect(response.error).toBeTruthy();
  expect(response.response.status).toBe(403);
});

test("DELETE /api/events/{event_id}/controllers/{position_id} deletes a controller position", async ({
  coordinator,
  user,
  event,
  position,
}) => {
  const deletedPosition = await coordinator.DELETE(
    "/api/events/{event_id}/controllers/{position_id}",
    {
      params: {
        path: {
          event_id: event.id,
          position_id: position.id,
        },
      },
    },
  );

  expect(deletedPosition.error).toBeFalsy();
  expect(deletedPosition.response.status).toBe(204);

  const positions = await user.GET("/api/events/{event_id}/controllers", {
    params: { path: { event_id: event.id } },
  });

  expect(positions.error).toBeFalsy();
  expect(positions.response.status).toBe(200);
  expect(positions.data).not.toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        id: position.id,
      }),
    ]),
  );
});

test("DELETE /api/events/{event_id}/controllers/{position_id} requires event coordinator role", async ({
  user,
  event,
  position,
}) => {
  const response = await user.DELETE(
    "/api/events/{event_id}/controllers/{position_id}",
    {
      params: {
        path: {
          event_id: event.id,
          position_id: position.id,
        },
      },
    },
  );

  expect(response.error).toBeTruthy();
  expect(response.response.status).toBe(403);
});
