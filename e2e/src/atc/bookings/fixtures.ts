import { expect, test as baseTest } from "vitest";
import { createApiClientWithRoles } from "../../../lib/api/client.js";
import type { components } from "../../../lib/api/schema.js";
import { getBaseUrl, getClient } from "../../../lib/backend.js";

type ApiClient = Awaited<ReturnType<typeof getClient>>;

export type Actor = {
  client: ApiClient;
  id: string;
};

export const defaultBooking: components["schemas"]["AtcBookingSaveRequest"] = {
  callsign: "ZBAA_TWR",
  start_at: "2099-08-26T10:00:00Z",
  end_at: "2099-08-26T12:00:00Z",
  remarks: "E2E ATC booking.",
};

async function currentActor(client: ApiClient): Promise<Actor> {
  const session = await client.GET("/api/session");

  expect(session.error).toBeFalsy();
  expect(session.response.status).toBe(200);
  expect(session.data).toBeTruthy();

  return { client, id: session.data.user.id };
}

async function grantControllerPermission(admin: ApiClient, userId: string) {
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
}

async function createController(admin: ApiClient): Promise<Actor> {
  const client = await createApiClientWithRoles(getBaseUrl(), {
    cid: Math.floor(10000000 + Math.random() * 9000000).toString(),
    roles: [],
  });
  const actor = await currentActor(client);
  await grantControllerPermission(admin, actor.id);
  return actor;
}

export async function createBooking(
  actor: Actor,
  body: components["schemas"]["AtcBookingSaveRequest"] = defaultBooking,
) {
  const response = await actor.client.PUT("/api/atc/bookings", { body });

  expect(response.error).toBeFalsy();
  expect(response.response.status).toBe(200);
  expect(response.data).toBeTruthy();

  return response.data;
}

export const test = baseTest
  .extend("admin", async ({}) => {
    return currentActor(
      await getClient([
        "controller-training-director-assistant",
        "event-coordinator",
      ]),
    );
  })
  .extend("controller", async ({ admin }) => {
    return createController(admin.client);
  })
  .extend("otherController", async ({ admin }) => {
    return createController(admin.client);
  })
  .extend("user", async ({}) => {
    return currentActor(await getClient([]));
  })
  .extend("booking", async ({ controller }) => {
    return createBooking(controller);
  })
  .extend("event", async ({ admin }) => {
    const suffix = `${Date.now()}-${Math.random()}`;
    const response = await admin.client.POST("/api/events", {
      body: {
        title: `E2E ATC Booking Event ${suffix}`,
        title_en: `E2E ATC Booking Event EN ${suffix}`,
        description: "Created by the ATC booking E2E suite.",
        start_at: "2099-09-26T10:00:00Z",
        end_at: "2099-09-26T13:00:00Z",
        start_booking_at: "2099-09-01T00:00:00Z",
        end_booking_at: "2099-09-25T00:00:00Z",
        start_atc_booking_at: "2020-01-01T00:00:00Z",
      },
    });

    expect(response.error).toBeFalsy();
    expect(response.response.status).toBe(200);
    expect(response.data).toBeTruthy();
    return response.data;
  })
  .extend("position", async ({ admin, event }) => {
    const response = await admin.client.POST(
      "/api/events/{event_id}/controllers",
      {
        params: { path: { event_id: event.id } },
        body: {
          callsign: "ZBAA_TWR",
          start_at: "2099-09-26T10:30:00Z",
          end_at: "2099-09-26T12:30:00Z",
          remarks: "E2E event-linked ATC booking.",
          position_kind_id: "TWR",
          minimum_controller_state: "student",
        },
      },
    );

    expect(response.error).toBeFalsy();
    expect(response.response.status).toBe(200);
    expect(response.data).toBeTruthy();
    return response.data;
  })
  .extend("eventBooking", async ({ controller, event, position }) => {
    const response = await controller.client.PUT(
      "/api/events/{event_id}/controllers/{position_id}/booking",
      {
        params: {
          path: { event_id: event.id, position_id: position.id },
        },
        body: {},
      },
    );

    expect(response.error).toBeFalsy();
    expect(response.response.status).toBe(200);
    expect(response.data).toBeTruthy();
    return response.data;
  });
