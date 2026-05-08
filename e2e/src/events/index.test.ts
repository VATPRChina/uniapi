import { expect, test } from "vitest";
import { issueUserTokenWithRoles } from "../../lib/api/client.js";
import { getBackend } from "../../lib/backend.js";

test("POST /api/events creates an event", async () => {
  const client = await getBackend();
  const accessToken = await issueUserTokenWithRoles(client, {
    cid: "910001",
    fullName: "E2E Event Coordinator",
    email: "e2e-event-coordinator@example.test",
    roles: ["event-coordinator"],
  });
  const startAt = "2030-01-15T10:00:00Z";
  const endAt = "2030-01-15T12:00:00Z";
  const startBookingAt = "2030-01-01T00:00:00Z";
  const endBookingAt = "2030-01-14T00:00:00Z";
  const startAtcBookingAt = "2030-01-02T00:00:00Z";

  const { data, error, response } = await client.POST("/api/events", {
    body: {
      title: "E2E Event",
      title_en: "E2E Event EN",
      start_at: startAt,
      end_at: endAt,
      start_booking_at: startBookingAt,
      end_booking_at: endBookingAt,
      start_atc_booking_at: startAtcBookingAt,
      image_url: "https://example.test/event.png",
      community_link: "https://community.example.test/events/e2e",
      vatsim_link: "https://my.vatsim.net/events/e2e",
      description: "Created by the e2e POST /api/events test.",
    },
    headers: {
      authorization: `Bearer ${accessToken}`,
    },
  });

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toBeTruthy();
  expect(data.id).toMatch(/^[0-9A-HJKMNP-TV-Z]{26}$/);
  expect(data.title).toBe("E2E Event");
  expect(data.title_en).toBe("E2E Event EN");
  expect(data.start_at).toBe(startAt);
  expect(data.end_at).toBe(endAt);
  expect(data.start_booking_at).toBe(startBookingAt);
  expect(data.end_booking_at).toBe(endBookingAt);
  expect(data.start_atc_booking_at).toBe(startAtcBookingAt);
  expect(data.image_url).toBe("https://example.test/event.png");
  expect(data.community_link).toBe("https://community.example.test/events/e2e");
  expect(data.vatsim_link).toBe("https://my.vatsim.net/events/e2e");
  expect(data.description).toBe("Created by the e2e POST /api/events test.");
});

test("GET /api/events lists current events", async () => {
  const client = await getBackend();
  const accessToken = await issueUserTokenWithRoles(client, {
    cid: "910002",
    fullName: "E2E Event List Coordinator",
    email: "e2e-event-list-coordinator@example.test",
    roles: ["event-coordinator"],
  });
  const suffix = Date.now().toString();
  const title = `E2E Listed Event ${suffix}`;
  const titleEn = `E2E Listed Event EN ${suffix}`;
  const startAt = "2031-02-20T10:00:00Z";
  const endAt = "2031-02-20T12:00:00Z";
  const startBookingAt = "2031-02-01T00:00:00Z";
  const endBookingAt = "2031-02-19T00:00:00Z";
  const startAtcBookingAt = "2031-02-02T00:00:00Z";

  const createdEvent = await client.POST("/api/events", {
    body: {
      title,
      title_en: titleEn,
      start_at: startAt,
      end_at: endAt,
      start_booking_at: startBookingAt,
      end_booking_at: endBookingAt,
      start_atc_booking_at: startAtcBookingAt,
      image_url: "https://example.test/listed-event.png",
      community_link: "https://community.example.test/events/e2e-listed",
      vatsim_link: "https://my.vatsim.net/events/e2e-listed",
      description: "Created by the e2e GET /api/events test.",
    },
    headers: {
      authorization: `Bearer ${accessToken}`,
    },
  });

  expect(createdEvent.error).toBeFalsy();
  expect(createdEvent.response.status).toBe(200);
  expect(createdEvent.data).toBeTruthy();
  if (!createdEvent.data) {
    throw new Error("Expected event creation to return an event");
  }

  const { data, error, response } = await client.GET("/api/events");

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        id: createdEvent.data.id,
        title,
        title_en: titleEn,
        start_at: startAt,
        end_at: endAt,
        start_booking_at: startBookingAt,
        end_booking_at: endBookingAt,
        start_atc_booking_at: startAtcBookingAt,
      }),
    ]),
  );
});
