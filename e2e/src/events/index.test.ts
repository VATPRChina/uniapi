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

test("POST /api/events rejects users without event coordinator permission", async () => {
  const client = await getBackend();
  const accessToken = await issueUserTokenWithRoles(client, {
    cid: "910005",
    fullName: "E2E Non Event Coordinator",
    email: "e2e-non-event-coordinator@example.test",
    roles: ["controller"],
  });

  const { data, error, response } = await client.POST("/api/events", {
    body: {
      title: "E2E Forbidden Event",
      title_en: "E2E Forbidden Event EN",
      start_at: "2030-04-15T10:00:00Z",
      end_at: "2030-04-15T12:00:00Z",
      start_booking_at: "2030-04-01T00:00:00Z",
      end_booking_at: "2030-04-14T00:00:00Z",
      start_atc_booking_at: "2030-04-02T00:00:00Z",
      image_url: "https://example.test/forbidden-event.png",
      community_link: "https://community.example.test/events/e2e-forbidden",
      vatsim_link: "https://my.vatsim.net/events/e2e-forbidden",
      description: "Created by the e2e forbidden POST /api/events test.",
    },
    headers: {
      authorization: `Bearer ${accessToken}`,
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

test("GET /api/events excludes past events", async () => {
  const client = await getBackend();
  const accessToken = await issueUserTokenWithRoles(client, {
    cid: "910004",
    fullName: "E2E Current Event Filter Coordinator",
    email: "e2e-current-event-filter-coordinator@example.test",
    roles: ["event-coordinator"],
  });
  const suffix = Date.now().toString();
  const pastTitle = `E2E Excluded Past Event ${suffix}`;
  const pastTitleEn = `E2E Excluded Past Event EN ${suffix}`;
  const pastStartAt = "2020-02-20T10:00:00Z";
  const pastEndAt = "2020-02-20T12:00:00Z";
  const pastStartBookingAt = "2020-02-01T00:00:00Z";
  const pastEndBookingAt = "2020-02-19T00:00:00Z";
  const pastStartAtcBookingAt = "2020-02-02T00:00:00Z";

  const createdPastEvent = await client.POST("/api/events", {
    body: {
      title: pastTitle,
      title_en: pastTitleEn,
      start_at: pastStartAt,
      end_at: pastEndAt,
      start_booking_at: pastStartBookingAt,
      end_booking_at: pastEndBookingAt,
      start_atc_booking_at: pastStartAtcBookingAt,
      image_url: "https://example.test/excluded-past-event.png",
      community_link: "https://community.example.test/events/e2e-excluded-past",
      vatsim_link: "https://my.vatsim.net/events/e2e-excluded-past",
      description: "Created to verify GET /api/events excludes past events.",
    },
    headers: {
      authorization: `Bearer ${accessToken}`,
    },
  });

  expect(createdPastEvent.error).toBeFalsy();
  expect(createdPastEvent.response.status).toBe(200);
  expect(createdPastEvent.data).toBeTruthy();
  if (!createdPastEvent.data) {
    throw new Error("Expected past event creation to return an event");
  }

  const { data, error, response } = await client.GET("/api/events");

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

test("GET /api/events/{id} returns an event", async () => {
  const client = await getBackend();
  const accessToken = await issueUserTokenWithRoles(client, {
    cid: "910006",
    fullName: "E2E Event Detail Coordinator",
    email: "e2e-event-detail-coordinator@example.test",
    roles: ["event-coordinator"],
  });
  const suffix = Date.now().toString();
  const title = `E2E Event Detail ${suffix}`;
  const titleEn = `E2E Event Detail EN ${suffix}`;
  const startAt = "2031-05-20T10:00:00Z";
  const endAt = "2031-05-20T12:00:00Z";
  const startBookingAt = "2031-05-01T00:00:00Z";
  const endBookingAt = "2031-05-19T00:00:00Z";
  const startAtcBookingAt = "2031-05-02T00:00:00Z";

  const createdEvent = await client.POST("/api/events", {
    body: {
      title,
      title_en: titleEn,
      start_at: startAt,
      end_at: endAt,
      start_booking_at: startBookingAt,
      end_booking_at: endBookingAt,
      start_atc_booking_at: startAtcBookingAt,
      image_url: "https://example.test/event-detail.png",
      community_link: "https://community.example.test/events/e2e-detail",
      vatsim_link: "https://my.vatsim.net/events/e2e-detail",
      description: "Created by the e2e GET /api/events/{id} test.",
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

  const { data, error, response } = await client.GET("/api/events/{id}", {
    params: {
      path: {
        id: createdEvent.data.id,
      },
    },
  });

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toEqual(
    expect.objectContaining({
      id: createdEvent.data.id,
      title,
      title_en: titleEn,
      start_at: startAt,
      end_at: endAt,
      start_booking_at: startBookingAt,
      end_booking_at: endBookingAt,
      start_atc_booking_at: startAtcBookingAt,
      image_url: "https://example.test/event-detail.png",
      community_link: "https://community.example.test/events/e2e-detail",
      vatsim_link: "https://my.vatsim.net/events/e2e-detail",
      description: "Created by the e2e GET /api/events/{id} test.",
    }),
  );
});

test("PUT /api/events/{id} updates an event", async () => {
  const client = await getBackend();
  const accessToken = await issueUserTokenWithRoles(client, {
    cid: "910007",
    fullName: "E2E Event Update Coordinator",
    email: "e2e-event-update-coordinator@example.test",
    roles: ["event-coordinator"],
  });
  const suffix = Date.now().toString();
  const originalTitle = `E2E Event Before Update ${suffix}`;
  const updatedTitle = `E2E Event After Update ${suffix}`;
  const updatedTitleEn = `E2E Event After Update EN ${suffix}`;

  const createdEvent = await client.POST("/api/events", {
    body: {
      title: originalTitle,
      title_en: `E2E Event Before Update EN ${suffix}`,
      start_at: "2031-06-20T10:00:00Z",
      end_at: "2031-06-20T12:00:00Z",
      start_booking_at: "2031-06-01T00:00:00Z",
      end_booking_at: "2031-06-19T00:00:00Z",
      start_atc_booking_at: "2031-06-02T00:00:00Z",
      image_url: "https://example.test/event-before-update.png",
      community_link: "https://community.example.test/events/e2e-before-update",
      vatsim_link: "https://my.vatsim.net/events/e2e-before-update",
      description: "Created by the e2e PUT /api/events/{id} test.",
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

  const updatedEvent = await client.PUT("/api/events/{id}", {
    params: {
      path: {
        id: createdEvent.data.id,
      },
    },
    body: {
      title: updatedTitle,
      title_en: updatedTitleEn,
      start_at: "2031-06-21T10:00:00Z",
      end_at: "2031-06-21T12:00:00Z",
      start_booking_at: "2031-06-03T00:00:00Z",
      end_booking_at: "2031-06-20T00:00:00Z",
      start_atc_booking_at: "2031-06-04T00:00:00Z",
      image_url: "https://example.test/event-after-update.png",
      community_link: "https://community.example.test/events/e2e-after-update",
      vatsim_link: "https://my.vatsim.net/events/e2e-after-update",
      description: "Updated by the e2e PUT /api/events/{id} test.",
    },
    headers: {
      authorization: `Bearer ${accessToken}`,
    },
  });

  expect(updatedEvent.error).toBeFalsy();
  expect(updatedEvent.response.status).toBe(200);
  expect(updatedEvent.data).toEqual(
    expect.objectContaining({
      id: createdEvent.data.id,
      title: updatedTitle,
      title_en: updatedTitleEn,
      start_at: "2031-06-21T10:00:00Z",
      end_at: "2031-06-21T12:00:00Z",
      start_booking_at: "2031-06-03T00:00:00Z",
      end_booking_at: "2031-06-20T00:00:00Z",
      start_atc_booking_at: "2031-06-04T00:00:00Z",
      image_url: "https://example.test/event-after-update.png",
      community_link: "https://community.example.test/events/e2e-after-update",
      vatsim_link: "https://my.vatsim.net/events/e2e-after-update",
      description: "Updated by the e2e PUT /api/events/{id} test.",
    }),
  );
});

test("PUT /api/events/{id} rejects users without event coordinator permission", async () => {
  const client = await getBackend();
  const eventCoordinatorToken = await issueUserTokenWithRoles(client, {
    cid: "910009",
    fullName: "E2E Event Update Permission Coordinator",
    email: "e2e-event-update-permission-coordinator@example.test",
    roles: ["event-coordinator"],
  });
  const nonEventCoordinatorToken = await issueUserTokenWithRoles(client, {
    cid: "910010",
    fullName: "E2E Event Update Non Coordinator",
    email: "e2e-event-update-non-coordinator@example.test",
    roles: ["controller"],
  });
  const suffix = Date.now().toString();
  const title = `E2E Event Update Permission ${suffix}`;

  const createdEvent = await client.POST("/api/events", {
    body: {
      title,
      title_en: `E2E Event Update Permission EN ${suffix}`,
      start_at: "2031-08-20T10:00:00Z",
      end_at: "2031-08-20T12:00:00Z",
      start_booking_at: "2031-08-01T00:00:00Z",
      end_booking_at: "2031-08-19T00:00:00Z",
      start_atc_booking_at: "2031-08-02T00:00:00Z",
      image_url: "https://example.test/event-update-permission.png",
      community_link: "https://community.example.test/events/e2e-update-permission",
      vatsim_link: "https://my.vatsim.net/events/e2e-update-permission",
      description: "Created by the e2e PUT /api/events/{id} permission test.",
    },
    headers: {
      authorization: `Bearer ${eventCoordinatorToken}`,
    },
  });

  expect(createdEvent.error).toBeFalsy();
  expect(createdEvent.response.status).toBe(200);
  expect(createdEvent.data).toBeTruthy();
  if (!createdEvent.data) {
    throw new Error("Expected event creation to return an event");
  }

  const updatedEvent = await client.PUT("/api/events/{id}", {
    params: {
      path: {
        id: createdEvent.data.id,
      },
    },
    body: {
      title: `E2E Event Update Permission Denied ${suffix}`,
      title_en: `E2E Event Update Permission Denied EN ${suffix}`,
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
    headers: {
      authorization: `Bearer ${nonEventCoordinatorToken}`,
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

test("DELETE /api/events/{id} deletes an event", async () => {
  const client = await getBackend();
  const accessToken = await issueUserTokenWithRoles(client, {
    cid: "910008",
    fullName: "E2E Event Delete Coordinator",
    email: "e2e-event-delete-coordinator@example.test",
    roles: ["event-coordinator"],
  });
  const suffix = Date.now().toString();
  const title = `E2E Event To Delete ${suffix}`;

  const createdEvent = await client.POST("/api/events", {
    body: {
      title,
      title_en: `E2E Event To Delete EN ${suffix}`,
      start_at: "2031-07-20T10:00:00Z",
      end_at: "2031-07-20T12:00:00Z",
      start_booking_at: "2031-07-01T00:00:00Z",
      end_booking_at: "2031-07-19T00:00:00Z",
      start_atc_booking_at: "2031-07-02T00:00:00Z",
      image_url: "https://example.test/event-to-delete.png",
      community_link: "https://community.example.test/events/e2e-to-delete",
      vatsim_link: "https://my.vatsim.net/events/e2e-to-delete",
      description: "Created by the e2e DELETE /api/events/{id} test.",
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

  const deletedEvent = await client.DELETE("/api/events/{id}", {
    params: {
      path: {
        id: createdEvent.data.id,
      },
    },
    headers: {
      authorization: `Bearer ${accessToken}`,
    },
  });

  expect(deletedEvent.error).toBeFalsy();
  expect(deletedEvent.response.status).toBe(200);
  expect(deletedEvent.data).toEqual(
    expect.objectContaining({
      id: createdEvent.data.id,
      title,
    }),
  );

  const fetchedDeletedEvent = await client.GET("/api/events/{id}", {
    params: {
      path: {
        id: createdEvent.data.id,
      },
    },
  });

  expect(fetchedDeletedEvent.response.status).toBe(404);
  expect(fetchedDeletedEvent.data).toBeFalsy();
  expect(fetchedDeletedEvent.error).toEqual({
    detail: "event not found",
    status: 404,
    title: "Not Found",
    type: "about:blank",
  });
});

test("DELETE /api/events/{id} rejects users without event coordinator permission", async () => {
  const client = await getBackend();
  const eventCoordinatorToken = await issueUserTokenWithRoles(client, {
    cid: "910011",
    fullName: "E2E Event Delete Permission Coordinator",
    email: "e2e-event-delete-permission-coordinator@example.test",
    roles: ["event-coordinator"],
  });
  const nonEventCoordinatorToken = await issueUserTokenWithRoles(client, {
    cid: "910012",
    fullName: "E2E Event Delete Non Coordinator",
    email: "e2e-event-delete-non-coordinator@example.test",
    roles: ["controller"],
  });
  const suffix = Date.now().toString();
  const title = `E2E Event Delete Permission ${suffix}`;

  const createdEvent = await client.POST("/api/events", {
    body: {
      title,
      title_en: `E2E Event Delete Permission EN ${suffix}`,
      start_at: "2031-09-20T10:00:00Z",
      end_at: "2031-09-20T12:00:00Z",
      start_booking_at: "2031-09-01T00:00:00Z",
      end_booking_at: "2031-09-19T00:00:00Z",
      start_atc_booking_at: "2031-09-02T00:00:00Z",
      image_url: "https://example.test/event-delete-permission.png",
      community_link: "https://community.example.test/events/e2e-delete-permission",
      vatsim_link: "https://my.vatsim.net/events/e2e-delete-permission",
      description: "Created by the e2e DELETE /api/events/{id} permission test.",
    },
    headers: {
      authorization: `Bearer ${eventCoordinatorToken}`,
    },
  });

  expect(createdEvent.error).toBeFalsy();
  expect(createdEvent.response.status).toBe(200);
  expect(createdEvent.data).toBeTruthy();
  if (!createdEvent.data) {
    throw new Error("Expected event creation to return an event");
  }

  const deletedEvent = await client.DELETE("/api/events/{id}", {
    params: {
      path: {
        id: createdEvent.data.id,
      },
    },
    headers: {
      authorization: `Bearer ${nonEventCoordinatorToken}`,
    },
  });

  expect(deletedEvent.response.status).toBe(403);
  expect(deletedEvent.data).toBeFalsy();
  expect(deletedEvent.error).toEqual({
    detail: "forbidden",
    status: 403,
    title: "Forbidden",
    type: "about:blank",
  });

  const fetchedEvent = await client.GET("/api/events/{id}", {
    params: {
      path: {
        id: createdEvent.data.id,
      },
    },
  });

  expect(fetchedEvent.error).toBeFalsy();
  expect(fetchedEvent.response.status).toBe(200);
  expect(fetchedEvent.data).toEqual(
    expect.objectContaining({
      id: createdEvent.data.id,
      title,
    }),
  );
});

test("GET /api/events/past lists past events", async () => {
  const client = await getBackend();
  const accessToken = await issueUserTokenWithRoles(client, {
    cid: "910003",
    fullName: "E2E Past Event Coordinator",
    email: "e2e-past-event-coordinator@example.test",
    roles: ["event-coordinator"],
  });
  const suffix = Date.now().toString();
  const title = `E2E Past Event ${suffix}`;
  const titleEn = `E2E Past Event EN ${suffix}`;
  const startAt = "2020-03-10T10:00:00Z";
  const endAt = "2020-03-10T12:00:00Z";
  const startBookingAt = "2020-03-01T00:00:00Z";
  const endBookingAt = "2020-03-09T00:00:00Z";
  const startAtcBookingAt = "2020-03-02T00:00:00Z";

  const createdEvent = await client.POST("/api/events", {
    body: {
      title,
      title_en: titleEn,
      start_at: startAt,
      end_at: endAt,
      start_booking_at: startBookingAt,
      end_booking_at: endBookingAt,
      start_atc_booking_at: startAtcBookingAt,
      image_url: "https://example.test/past-event.png",
      community_link: "https://community.example.test/events/e2e-past",
      vatsim_link: "https://my.vatsim.net/events/e2e-past",
      description: "Created by the e2e GET /api/events/past test.",
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

  const { data, error, response } = await client.GET("/api/events/past", {
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
