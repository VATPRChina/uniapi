import { expect, test as baseTest } from "vitest";
import { createApiClientWithRoles } from "../../../../../lib/api/client.js";
import { getBaseUrl, getClient } from "../../../../../lib/backend.js";

const test = baseTest
  .extend("coordinator", async ({}) => {
    return await getClient(["event-coordinator"]);
  })
  .extend("user", async ({}) => {
    return await getClient([]);
  })
  .extend("assignee", async ({}) => {
    const client = await createApiClientWithRoles(getBaseUrl(), {
      cid: Math.floor(10000000 + Math.random() * 9000000).toString(),
      roles: [],
    });
    const session = await client.GET("/api/session");

    expect(session.error).toBeFalsy();
    expect(session.response.status).toBe(200);
    expect(session.data).toBeTruthy();

    return {
      client,
      id: session.data.user.id,
    };
  })
  .extend("event", async ({ coordinator }) => {
    const suffix = Date.now().toString();

    const event = await coordinator.POST("/api/events", {
      body: {
        title: `E2E Slot Booking Event ${suffix}`,
        title_en: `E2E Slot Booking Event EN ${suffix}`,
        description: "Created by the e2e slot booking test.",
        start_at: "2031-10-20T10:00:00Z",
        end_at: "2031-10-20T13:00:00Z",
        start_booking_at: "2020-01-01T00:00:00Z",
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
          name: `E2E Slot Booking Airspace ${suffix}`,
          icao_codes: ["ZBAA", "ZSSS"],
          description: "Created by the e2e slot booking test.",
        },
      },
    );

    return airspace.data;
  })
  .extend("slot", async ({ coordinator, event, airspace }) => {
    const slot = await coordinator.POST("/api/events/{event_id}/slots", {
      params: { path: { event_id: event.id } },
      body: {
        airspace_id: airspace.id,
        enter_at: "2031-10-20T10:30:00Z",
        leave_at: "2031-10-20T12:30:00Z",
        callsign: "CCA789",
        aircraft_type_icao: "B789",
      },
    });

    expect(slot.error).toBeFalsy();
    expect(slot.response.status).toBe(200);
    expect(slot.data).toBeTruthy();

    return slot.data;
  })
  .extend("closedEvent", async ({ coordinator }) => {
    const suffix = Date.now().toString();

    const event = await coordinator.POST("/api/events", {
      body: {
        title: `E2E Closed Slot Booking Event ${suffix}`,
        title_en: `E2E Closed Slot Booking Event EN ${suffix}`,
        description: "Created by the e2e closed slot booking test.",
        start_at: "2031-11-20T10:00:00Z",
        end_at: "2031-11-20T13:00:00Z",
        start_booking_at: "2020-01-01T00:00:00Z",
        end_booking_at: "2021-01-01T00:00:00Z",
      },
    });

    return event.data;
  })
  .extend("closedAirspace", async ({ coordinator, closedEvent }) => {
    const suffix = Date.now().toString();
    const airspace = await coordinator.POST(
      "/api/events/{event_id}/airspaces",
      {
        params: {
          path: {
            event_id: closedEvent.id,
          },
        },
        body: {
          name: `E2E Closed Slot Booking Airspace ${suffix}`,
          icao_codes: ["ZBAA", "ZSSS"],
          description: "Created by the e2e closed slot booking test.",
        },
      },
    );

    return airspace.data;
  })
  .extend("closedSlot", async ({ coordinator, closedEvent, closedAirspace }) => {
    const slot = await coordinator.POST("/api/events/{event_id}/slots", {
      params: { path: { event_id: closedEvent.id } },
      body: {
        airspace_id: closedAirspace.id,
        enter_at: "2031-11-20T10:30:00Z",
        leave_at: "2031-11-20T12:30:00Z",
        callsign: "CCA790",
        aircraft_type_icao: "B789",
      },
    });

    expect(slot.error).toBeFalsy();
    expect(slot.response.status).toBe(200);
    expect(slot.data).toBeTruthy();

    return slot.data;
  });

test("PUT /api/events/{event_id}/slots/{slot_id}/booking books a slot", async ({
  user,
  event,
  slot,
}) => {
  const booking = await user.PUT(
    "/api/events/{event_id}/slots/{slot_id}/booking",
    {
      params: {
        path: {
          event_id: event.id,
          slot_id: slot.id,
        },
      },
      body: {},
    },
  );

  expect(booking.error).toBeFalsy();
  expect(booking.response.status).toBe(200);
  expect(booking.data).toBeTruthy();

  expect(booking.data).toEqual(
    expect.objectContaining({
      id: expect.stringMatching(/^[0-9A-HJKMNP-TV-Z]{26}$/),
      user_id: expect.stringMatching(/^[0-9A-HJKMNP-TV-Z]{26}$/),
      user: null,
      created_at: expect.any(String),
      updated_at: expect.any(String),
    }),
  );
});

test("DELETE /api/events/{event_id}/slots/{slot_id}/booking cancels a booking", async ({
  user,
  event,
  slot,
}) => {
  const createdBooking = await user.PUT(
    "/api/events/{event_id}/slots/{slot_id}/booking",
    {
      params: {
        path: {
          event_id: event.id,
          slot_id: slot.id,
        },
      },
      body: {},
    },
  );

  expect(createdBooking.error).toBeFalsy();
  expect(createdBooking.response.status).toBe(200);
  expect(createdBooking.data).toBeTruthy();

  const deletedBooking = await user.DELETE(
    "/api/events/{event_id}/slots/{slot_id}/booking",
    {
      params: {
        path: {
          event_id: event.id,
          slot_id: slot.id,
        },
      },
    },
  );

  expect(deletedBooking.error).toBeFalsy();
  expect(deletedBooking.response.status).toBe(200);
  expect(deletedBooking.data).toEqual(
    expect.objectContaining({
      id: createdBooking.data.id,
      user_id: createdBooking.data.user_id,
      user: null,
      created_at: createdBooking.data.created_at,
      updated_at: createdBooking.data.updated_at,
    }),
  );

  const slots = await user.GET("/api/events/{event_id}/slots", {
    params: { path: { event_id: event.id } },
  });

  expect(slots.error).toBeFalsy();
  expect(slots.response.status).toBe(200);
  expect(slots.data).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        id: slot.id,
        booking: null,
      }),
    ]),
  );
});

test("already booked slot cannot be booked by another user", async ({
  user,
  assignee,
  event,
  slot,
}) => {
  const createdBooking = await user.PUT(
    "/api/events/{event_id}/slots/{slot_id}/booking",
    {
      params: {
        path: {
          event_id: event.id,
          slot_id: slot.id,
        },
      },
      body: {},
    },
  );

  expect(createdBooking.error).toBeFalsy();
  expect(createdBooking.response.status).toBe(200);
  expect(createdBooking.data).toBeTruthy();

  const booking = await assignee.client.PUT(
    "/api/events/{event_id}/slots/{slot_id}/booking",
    {
      params: {
        path: {
          event_id: event.id,
          slot_id: slot.id,
        },
      },
      body: {},
    },
  );

  expect(booking.error).toBeTruthy();
  expect(booking.response.status).toBe(409);
  expect(booking.data).toBeFalsy();
});

test("booked slot cannot be released by another user", async ({
  user,
  assignee,
  event,
  slot,
}) => {
  const createdBooking = await user.PUT(
    "/api/events/{event_id}/slots/{slot_id}/booking",
    {
      params: {
        path: {
          event_id: event.id,
          slot_id: slot.id,
        },
      },
      body: {},
    },
  );

  expect(createdBooking.error).toBeFalsy();
  expect(createdBooking.response.status).toBe(200);
  expect(createdBooking.data).toBeTruthy();

  const deletedBooking = await assignee.client.DELETE(
    "/api/events/{event_id}/slots/{slot_id}/booking",
    {
      params: {
        path: {
          event_id: event.id,
          slot_id: slot.id,
        },
      },
    },
  );

  expect(deletedBooking.error).toBeTruthy();
  expect(deletedBooking.response.status).toBe(409);
  expect(deletedBooking.data).toBeFalsy();
});

test("slot can be assigned and unassigned by event coordinator", async ({
  coordinator,
  assignee,
  event,
  slot,
}) => {
  const createdBooking = await coordinator.PUT(
    "/api/events/{event_id}/slots/{slot_id}/booking",
    {
      params: {
        path: {
          event_id: event.id,
          slot_id: slot.id,
        },
      },
      body: {
        user_id: assignee.id,
      },
    },
  );

  expect(createdBooking.error).toBeFalsy();
  expect(createdBooking.response.status).toBe(200);
  expect(createdBooking.data).toEqual(
    expect.objectContaining({
      id: expect.stringMatching(/^[0-9A-HJKMNP-TV-Z]{26}$/),
      user_id: assignee.id,
      user: expect.objectContaining({
        id: assignee.id,
      }),
      created_at: expect.any(String),
      updated_at: expect.any(String),
    }),
  );

  const deletedBooking = await coordinator.DELETE(
    "/api/events/{event_id}/slots/{slot_id}/booking",
    {
      params: {
        path: {
          event_id: event.id,
          slot_id: slot.id,
        },
      },
    },
  );

  expect(deletedBooking.error).toBeFalsy();
  expect(deletedBooking.response.status).toBe(200);
  expect(deletedBooking.data).toEqual(
    expect.objectContaining({
      id: createdBooking.data.id,
      user_id: assignee.id,
    }),
  );
});

test("slot cannot be assigned by normal user", async ({
  user,
  assignee,
  event,
  slot,
}) => {
  const booking = await user.PUT(
    "/api/events/{event_id}/slots/{slot_id}/booking",
    {
      params: {
        path: {
          event_id: event.id,
          slot_id: slot.id,
        },
      },
      body: {
        user_id: assignee.id,
      },
    },
  );

  expect(booking.error).toEqual({
    detail: "only user with roles {EventCoordinator} can perform this action",
    status: 403,
    title: "only user with roles {EventCoordinator} can perform this action",
    type: "urn:vatprc-uniapi-error:forbidden",
  });
  expect(booking.response.status).toBe(403);
  expect(booking.data).toBeFalsy();
});

test("slot can be assigned and unassigned by event coordinator outside booking time", async ({
  coordinator,
  assignee,
  closedEvent,
  closedSlot,
}) => {
  const createdBooking = await coordinator.PUT(
    "/api/events/{event_id}/slots/{slot_id}/booking",
    {
      params: {
        path: {
          event_id: closedEvent.id,
          slot_id: closedSlot.id,
        },
      },
      body: {
        user_id: assignee.id,
      },
    },
  );

  expect(createdBooking.error).toBeFalsy();
  expect(createdBooking.response.status).toBe(200);
  expect(createdBooking.data).toEqual(
    expect.objectContaining({
      id: expect.stringMatching(/^[0-9A-HJKMNP-TV-Z]{26}$/),
      user_id: assignee.id,
      user: expect.objectContaining({
        id: assignee.id,
      }),
      created_at: expect.any(String),
      updated_at: expect.any(String),
    }),
  );

  const deletedBooking = await coordinator.DELETE(
    "/api/events/{event_id}/slots/{slot_id}/booking",
    {
      params: {
        path: {
          event_id: closedEvent.id,
          slot_id: closedSlot.id,
        },
      },
    },
  );

  expect(deletedBooking.error).toBeFalsy();
  expect(deletedBooking.response.status).toBe(200);
  expect(deletedBooking.data).toEqual(
    expect.objectContaining({
      id: createdBooking.data.id,
      user_id: assignee.id,
    }),
  );
});
