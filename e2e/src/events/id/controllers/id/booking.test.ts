import { expect, test as baseTest } from "vitest";
import { createApiClientWithRoles } from "../../../../../lib/api/client.js";
import { getBaseUrl, getClient } from "../../../../../lib/backend.js";

type ApiClient = Awaited<ReturnType<typeof getClient>>;

async function grantTowerPermission(admin: ApiClient, userId: string) {
  const status = await admin.PUT("/api/users/{id}/atc/status", {
    params: { path: { id: userId } },
    body: {
      is_visiting: false,
      is_absent: false,
      rating: "C1",
      permissions: [
        {
          position_kind_id: "TWR",
          state: "certified",
          solo_expires_at: null,
        },
      ],
    },
  });

  expect(status.error).toBeFalsy();
  expect(status.response.status).toBe(200);
  expect(status.data).toEqual(
    expect.objectContaining({
      user_id: userId,
      rating: "C1",
      permissions: expect.arrayContaining([
        expect.objectContaining({
          position_kind_id: "TWR",
          state: "certified",
        }),
      ]),
    }),
  );
}

const test = baseTest
  .extend("bookingAdmin", async ({}) => {
    const client = await getClient([
      "event-coordinator",
      "controller-training-director-assistant",
    ]);
    const session = await client.GET("/api/session");

    expect(session.error).toBeFalsy();
    expect(session.response.status).toBe(200);
    expect(session.data).toBeTruthy();

    await grantTowerPermission(client, session.data.user.id);

    return {
      client,
      id: session.data.user.id,
    };
  })
  .extend("controller", async ({ bookingAdmin }) => {
    const client = await createApiClientWithRoles(getBaseUrl(), {
      cid: Math.floor(10000000 + Math.random() * 9000000).toString(),
      roles: [],
    });
    const session = await client.GET("/api/session");

    expect(session.error).toBeFalsy();
    expect(session.response.status).toBe(200);
    expect(session.data).toBeTruthy();

    await grantTowerPermission(bookingAdmin.client, session.data.user.id);

    return {
      client,
      id: session.data.user.id,
    };
  })
  .extend("assignee", async ({ bookingAdmin }) => {
    const client = await createApiClientWithRoles(getBaseUrl(), {
      cid: Math.floor(10000000 + Math.random() * 9000000).toString(),
      roles: [],
    });
    const session = await client.GET("/api/session");

    expect(session.error).toBeFalsy();
    expect(session.response.status).toBe(200);
    expect(session.data).toBeTruthy();

    await grantTowerPermission(bookingAdmin.client, session.data.user.id);

    return {
      client,
      id: session.data.user.id,
    };
  })
  .extend("user", async ({}) => {
    const client = await getClient([]);
    const session = await client.GET("/api/session");

    expect(session.error).toBeFalsy();
    expect(session.response.status).toBe(200);
    expect(session.data).toBeTruthy();

    return {
      client,
      id: session.data.user.id,
    };
  })
  .extend("event", async ({ bookingAdmin }) => {
    const suffix = Date.now().toString();

    const event = await bookingAdmin.client.POST("/api/events", {
      body: {
        title: `E2E Controller Booking Event ${suffix}`,
        title_en: `E2E Controller Booking Event EN ${suffix}`,
        description: "Created by the e2e controller booking test.",
        start_at: "2031-10-20T10:00:00Z",
        end_at: "2031-10-20T13:00:00Z",
        start_booking_at: "2031-10-01T00:00:00Z",
        end_booking_at: "2031-10-19T00:00:00Z",
        start_atc_booking_at: "2020-01-01T00:00:00Z",
      },
    });

    return event.data;
  })
  .extend("position", async ({ bookingAdmin, event }) => {
    const position = await bookingAdmin.client.POST(
      "/api/events/{event_id}/controllers",
      {
        params: { path: { event_id: event.id } },
        body: {
          callsign: "ZBAA_TWR",
          start_at: "2031-10-20T10:30:00Z",
          end_at: "2031-10-20T12:30:00Z",
          remarks: "E2E controller booking position.",
          position_kind_id: "TWR",
          minimum_controller_state: "student",
        },
      },
    );

    expect(position.error).toBeFalsy();
    expect(position.response.status).toBe(200);
    expect(position.data).toBeTruthy();

    return position.data;
  })
  .extend("closedEvent", async ({ bookingAdmin }) => {
    const suffix = Date.now().toString();

    const event = await bookingAdmin.client.POST("/api/events", {
      body: {
        title: `E2E Closed Controller Booking Event ${suffix}`,
        title_en: `E2E Closed Controller Booking Event EN ${suffix}`,
        description: "Created by the e2e closed controller booking test.",
        start_at: "2031-11-20T10:00:00Z",
        end_at: "2031-11-20T13:00:00Z",
        start_booking_at: "2031-11-01T00:00:00Z",
        end_booking_at: "2031-11-19T00:00:00Z",
        start_atc_booking_at: "2031-11-01T00:00:00Z",
      },
    });

    return event.data;
  })
  .extend("closedPosition", async ({ bookingAdmin, closedEvent }) => {
    const position = await bookingAdmin.client.POST(
      "/api/events/{event_id}/controllers",
      {
        params: { path: { event_id: closedEvent.id } },
        body: {
          callsign: "ZBAA_TWR",
          start_at: "2031-11-20T10:30:00Z",
          end_at: "2031-11-20T12:30:00Z",
          remarks: "E2E closed controller booking position.",
          position_kind_id: "TWR",
          minimum_controller_state: "student",
        },
      },
    );

    expect(position.error).toBeFalsy();
    expect(position.response.status).toBe(200);
    expect(position.data).toBeTruthy();

    return position.data;
  });

test("PUT /api/events/{event_id}/controllers/{position_id}/booking books a controller position", async ({
  controller,
  event,
  position,
}) => {
  const booking = await controller.client.PUT(
    "/api/events/{event_id}/controllers/{position_id}/booking",
    {
      params: {
        path: {
          event_id: event.id,
          position_id: position.id,
        },
      },
      body: {},
    },
  );

  expect(booking.error).toBeFalsy();
  expect(booking.response.status).toBe(200);
  expect(booking.data).toEqual(
    expect.objectContaining({
      user_id: controller.id,
      user: expect.objectContaining({
        id: controller.id,
      }),
      booked_at: expect.any(String),
    }),
  );
});

test("DELETE /api/events/{event_id}/controllers/{position_id}/booking cancels a controller position booking", async ({
  controller,
  event,
  position,
}) => {
  const createdBooking = await controller.client.PUT(
    "/api/events/{event_id}/controllers/{position_id}/booking",
    {
      params: {
        path: {
          event_id: event.id,
          position_id: position.id,
        },
      },
      body: {},
    },
  );

  expect(createdBooking.error).toBeFalsy();
  expect(createdBooking.response.status).toBe(200);
  expect(createdBooking.data).toBeTruthy();

  const deletedBooking = await controller.client.DELETE(
    "/api/events/{event_id}/controllers/{position_id}/booking",
    {
      params: {
        path: {
          event_id: event.id,
          position_id: position.id,
        },
      },
    },
  );

  expect(deletedBooking.error).toBeFalsy();
  expect(deletedBooking.response.status).toBe(200);
  expect(deletedBooking.data).toEqual(
    expect.objectContaining({
      user_id: createdBooking.data.user_id,
      user: expect.objectContaining({
        id: createdBooking.data.user_id,
      }),
      booked_at: createdBooking.data.booked_at,
    }),
  );

  const positions = await controller.client.GET(
    "/api/events/{event_id}/controllers",
    {
      params: { path: { event_id: event.id } },
    },
  );

  expect(positions.error).toBeFalsy();
  expect(positions.response.status).toBe(200);
  expect(positions.data).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        id: position.id,
        booking: null,
      }),
    ]),
  );
});

test("already booked controller position cannot be booked by another controller", async ({
  controller,
  assignee,
  event,
  position,
}) => {
  const createdBooking = await controller.client.PUT(
    "/api/events/{event_id}/controllers/{position_id}/booking",
    {
      params: {
        path: {
          event_id: event.id,
          position_id: position.id,
        },
      },
      body: {},
    },
  );

  expect(createdBooking.error).toBeFalsy();
  expect(createdBooking.response.status).toBe(200);
  expect(createdBooking.data).toBeTruthy();

  const booking = await assignee.client.PUT(
    "/api/events/{event_id}/controllers/{position_id}/booking",
    {
      params: {
        path: {
          event_id: event.id,
          position_id: position.id,
        },
      },
      body: {},
    },
  );

  expect(booking.error).toBeTruthy();
  expect(booking.response.status).toBe(409);
  expect(booking.data).toBeFalsy();
});

test("booked controller position cannot be released by another controller", async ({
  controller,
  assignee,
  event,
  position,
}) => {
  const createdBooking = await controller.client.PUT(
    "/api/events/{event_id}/controllers/{position_id}/booking",
    {
      params: {
        path: {
          event_id: event.id,
          position_id: position.id,
        },
      },
      body: {},
    },
  );

  expect(createdBooking.error).toBeFalsy();
  expect(createdBooking.response.status).toBe(200);
  expect(createdBooking.data).toBeTruthy();

  const deletedBooking = await assignee.client.DELETE(
    "/api/events/{event_id}/controllers/{position_id}/booking",
    {
      params: {
        path: {
          event_id: event.id,
          position_id: position.id,
        },
      },
    },
  );

  expect(deletedBooking.error).toBeTruthy();
  expect(deletedBooking.response.status).toBe(409);
  expect(deletedBooking.data).toBeFalsy();
});

test("controller position can be assigned and unassigned by event coordinator", async ({
  bookingAdmin,
  assignee,
  event,
  position,
}) => {
  const createdBooking = await bookingAdmin.client.PUT(
    "/api/events/{event_id}/controllers/{position_id}/booking",
    {
      params: {
        path: {
          event_id: event.id,
          position_id: position.id,
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
      user_id: assignee.id,
      user: expect.objectContaining({
        id: assignee.id,
      }),
      booked_at: expect.any(String),
    }),
  );

  const deletedBooking = await bookingAdmin.client.DELETE(
    "/api/events/{event_id}/controllers/{position_id}/booking",
    {
      params: {
        path: {
          event_id: event.id,
          position_id: position.id,
        },
      },
    },
  );

  expect(deletedBooking.error).toBeFalsy();
  expect(deletedBooking.response.status).toBe(200);
  expect(deletedBooking.data).toEqual(
    expect.objectContaining({
      user_id: assignee.id,
      user: expect.objectContaining({
        id: assignee.id,
      }),
      booked_at: createdBooking.data.booked_at,
    }),
  );
});

test("controller position cannot be assigned by normal user", async ({
  user,
  assignee,
  event,
  position,
}) => {
  const booking = await user.client.PUT(
    "/api/events/{event_id}/controllers/{position_id}/booking",
    {
      params: {
        path: {
          event_id: event.id,
          position_id: position.id,
        },
      },
      body: {
        user_id: assignee.id,
      },
    },
  );

  expect(booking.error).toBeTruthy();
  expect(booking.response.status).toBe(403);
  expect(booking.data).toBeFalsy();
});

test("controller position can be assigned and unassigned regardless of booking time by event coordinator", async ({
  bookingAdmin,
  assignee,
  closedEvent,
  closedPosition,
}) => {
  const createdBooking = await bookingAdmin.client.PUT(
    "/api/events/{event_id}/controllers/{position_id}/booking",
    {
      params: {
        path: {
          event_id: closedEvent.id,
          position_id: closedPosition.id,
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
      user_id: assignee.id,
      user: expect.objectContaining({
        id: assignee.id,
      }),
      booked_at: expect.any(String),
    }),
  );

  const deletedBooking = await bookingAdmin.client.DELETE(
    "/api/events/{event_id}/controllers/{position_id}/booking",
    {
      params: {
        path: {
          event_id: closedEvent.id,
          position_id: closedPosition.id,
        },
      },
    },
  );

  expect(deletedBooking.error).toBeFalsy();
  expect(deletedBooking.response.status).toBe(200);
  expect(deletedBooking.data).toEqual(
    expect.objectContaining({
      user_id: assignee.id,
      user: expect.objectContaining({
        id: assignee.id,
      }),
      booked_at: createdBooking.data.booked_at,
    }),
  );
});
