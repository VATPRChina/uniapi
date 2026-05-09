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

test("GET /api/events/{id} returns an event", async ({ user, event }) => {
  const { data, error, response } = await user.GET("/api/events/{id}", {
    params: { path: { id: event.id } },
  });

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toEqual(
    expect.objectContaining({
      id: event.id,
      title: event.title,
      title_en: event.title_en,
      start_at: event.start_at,
      end_at: event.end_at,
      start_booking_at: event.start_booking_at,
      end_booking_at: event.end_booking_at,
      start_atc_booking_at: event.start_atc_booking_at,
      image_url: event.image_url,
      community_link: event.community_link,
      vatsim_link: event.vatsim_link,
      description: event.description,
      created_at: event.created_at,
      updated_at: event.updated_at,
    }),
  );
});

test("PUT /api/events/{id} updates an event", async ({
  coordinator,
  event,
}) => {
  const update = {
    title: `E2E Event After Update ${Date.now()}`,
    title_en: `E2E Event After Update EN ${Date.now()}`,
    start_at: "2031-06-21T10:00:00Z",
    end_at: "2031-06-21T12:00:00Z",
    start_booking_at: "2031-06-03T00:00:00Z",
    end_booking_at: "2031-06-20T00:00:00Z",
    start_atc_booking_at: "2031-06-04T00:00:00Z",
    image_url: "https://example.test/event-after-update.png",
    community_link: "https://community.example.test/events/e2e-after-update",
    vatsim_link: "https://my.vatsim.net/events/e2e-after-update",
    description: "Updated by the e2e PUT /api/events/{id} test.",
  };

  const updatedEvent = await coordinator.PUT("/api/events/{id}", {
    params: { path: { id: event.id } },
    body: update,
  });

  expect(updatedEvent.error).toBeFalsy();
  expect(updatedEvent.response.status).toBe(200);
  expect(updatedEvent.data).toEqual(
    expect.objectContaining({
      id: event.id,
      ...update,
      created_at: event.created_at,
    }),
  );
  expect(updatedEvent.data.updated_at).not.toEqual(event.updated_at);
});

test("PUT /api/events/{id} rejects users without event coordinator permission", async ({
  user,
  event,
}) => {
  const updatedEvent = await user.PUT("/api/events/{id}", {
    params: {
      path: {
        id: event.id,
      },
    },
    body: {
      title: `E2E Event Update Permission Denied ${Date.now()}`,
      title_en: `E2E Event Update Permission Denied EN ${Date.now()}`,
      start_at: "2031-08-21T10:00:00Z",
      end_at: "2031-08-21T12:00:00Z",
      start_booking_at: "2031-08-03T00:00:00Z",
      end_booking_at: "2031-08-20T00:00:00Z",
      start_atc_booking_at: "2031-08-04T00:00:00Z",
      image_url: "https://example.test/event-update-permission-denied.png",
      community_link:
        "https://community.example.test/events/e2e-update-permission-denied",
      vatsim_link: "https://my.vatsim.net/events/e2e-update-permission-denied",
      description: "This update should be rejected by the e2e permission test.",
    },
  });

  expect(updatedEvent.response.status).toBe(403);
  expect(updatedEvent.data).toBeFalsy();
  expect(updatedEvent.error).toEqual({
    detail: "forbidden",
    status: 403,
    title: "Forbidden",
    type: "about:blank",
  });
});
