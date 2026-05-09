import { expect, test as baseTest } from "vitest";
import { getClient } from "../../lib/backend.js";
import { components } from "../../lib/api/schema.js";

const test = baseTest
  .extend("coordinator", async ({}) => {
    return await getClient(["event-coordinator"]);
  })
  .extend("user", async ({}) => {
    return await getClient([]);
  });

test("GET /api/events lists current events", async ({ coordinator }) => {
  const suffix = Date.now().toString();
  const event: components["schemas"]["EventSaveRequest"] = {
    title: `E2E Listed Event ${suffix}`,
    title_en: `E2E Listed Event EN ${suffix}`,
    description: "",
    start_at: "2031-02-20T10:00:00Z",
    end_at: "2031-02-20T12:00:00Z",
  };

  const createdEvent = await coordinator.POST("/api/events", {
    body: event,
  });

  const { data, error, response } = await coordinator.GET("/api/events");

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        id: createdEvent.data.id,
        ...event,
      }),
    ]),
  );
});

test("GET /api/events excludes past events", async ({ coordinator }) => {
  const suffix = Date.now().toString();
  const event: components["schemas"]["EventSaveRequest"] = {
    title: `E2E Past Event ${suffix}`,
    title_en: `E2E Past Event EN ${suffix}`,
    description: "",
    start_at: "2020-02-20T10:00:00Z",
    end_at: "2020-02-20T12:00:00Z",
  };

  const createdPastEvent = await coordinator.POST("/api/events", {
    body: event,
  });

  const { data, error, response } = await coordinator.GET("/api/events");

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).not.toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        id: createdPastEvent.data.id,
      }),
    ]),
  );
});

test("GET /api/events/past lists past events", async ({ coordinator }) => {
  const suffix = Date.now().toString();
  const event: components["schemas"]["EventSaveRequest"] = {
    title: `E2E Past Event ${suffix}`,
    title_en: `E2E Past Event EN ${suffix}`,
    description: "",
    start_at: "2020-02-20T10:00:00Z",
    end_at: "2020-02-20T12:00:00Z",
  };

  const createdPastEvent = await coordinator.POST("/api/events", {
    body: event,
  });

  const { data, error, response } = await coordinator.GET("/api/events/past", {
    params: {
      query: {
        until: "2020-03-31T00:00:00Z",
      },
    },
  });

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        id: createdPastEvent.data.id,
        ...createdPastEvent.data,
      }),
    ]),
  );
});

test("POST /api/events creates an event", async ({ coordinator }) => {
  const suffix = Date.now().toString();
  const event: components["schemas"]["EventSaveRequest"] = {
    title: `E2E Past Event ${suffix}`,
    title_en: `E2E Past Event EN ${suffix}`,
    description: "Created by the e2e POST /api/events test.",
    start_at: "2030-01-15T10:00:00Z",
    end_at: "2030-01-15T12:00:00Z",
    start_booking_at: "2030-01-01T00:00:00Z",
    end_booking_at: "2030-01-14T00:00:00Z",
    start_atc_booking_at: "2030-01-02T00:00:00Z",
    image_url: "https://example.test/event.png",
    community_link: "https://community.example.test/events/e2e",
    vatsim_link: "https://my.vatsim.net/events/e2e",
  };

  const { data, error, response } = await coordinator.POST("/api/events", {
    body: event,
  });

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toBeTruthy();
  expect(data.id).toMatch(/^[0-9A-HJKMNP-TV-Z]{26}$/);
  expect(data.title).toBe(event.title);
  expect(data.title_en).toBe(event.title_en);
  expect(data.description).toBe(event.description);
  expect(data.start_at).toBe(event.start_at);
  expect(data.end_at).toBe(event.end_at);
  expect(data.start_booking_at).toBe(event.start_booking_at);
  expect(data.end_booking_at).toBe(event.end_booking_at);
  expect(data.start_atc_booking_at).toBe(event.start_atc_booking_at);
  expect(data.image_url).toBe(event.image_url);
  expect(data.community_link).toBe(event.community_link);
  expect(data.vatsim_link).toBe(event.vatsim_link);
});

test("POST /api/events rejects users without event coordinator permission", async ({
  user,
}) => {
  const { data, error, response } = await user.POST("/api/events", {
    body: {
      title: `E2E Past Event`,
      title_en: `E2E Past Event EN`,
      description: "Created by the e2e POST /api/events test.",
      start_at: "2030-01-15T10:00:00Z",
      end_at: "2030-01-15T12:00:00Z",
    },
  });

  expect(response.status).toBe(403);
  expect(data).toBeFalsy();
  expect(error).toEqual({
    detail: "forbidden",
    status: 403,
    title: "Forbidden",
    type: "about:blank",
  });
});
