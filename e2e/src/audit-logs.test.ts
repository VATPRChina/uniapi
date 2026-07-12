import { expect, test as baseTest } from "vitest";
import { getClient } from "../lib/backend.js";

const test = baseTest
  .extend("coordinator", async ({}) => {
    return await getClient(["event-coordinator"]);
  })
  .extend("user", async ({}) => {
    return await getClient([]);
  })
  .extend("atcAdmin", async ({}) => {
    return await getClient(["controller-training-director-assistant"]);
  })
  .extend("staff", async ({}) => {
    return await getClient(["staff"]);
  })
  .extend("event", async ({ coordinator }) => {
    const suffix = Date.now().toString();
    const event = await coordinator.POST("/api/events", {
      body: {
        title: `Audit Route Test ${suffix}`,
        description: "Created by the audit route test.",
        start_at: "2031-10-20T10:00:00Z",
        end_at: "2031-10-20T13:00:00Z",
      },
    });

    expect(event.error).toBeFalsy();
    return event.data;
  })
  .extend("auditedUser", async ({ atcAdmin, staff }) => {
    const target = await getClient([]);
    const session = await target.GET("/api/session");
    expect(session.error).toBeFalsy();

    const userId = session.data.user.id;
    const status = await atcAdmin.PUT("/api/users/{id}/atc/status", {
      params: { path: { id: userId } },
      body: {
        is_absent: false,
        is_visiting: false,
        rating: "S3",
        permissions: [],
      },
    });
    expect(status.error).toBeFalsy();

    const roles = await staff.PUT("/api/users/{id}/roles", {
      params: { path: { id: userId } },
      body: ["event-coordinator"],
    });
    expect(roles.error).toBeFalsy();

    return session.data.user;
  });

test("event audit routes list logs and require volunteer permission", async ({
  coordinator,
  user,
  event,
}) => {
  const allResponse = await coordinator.GET("/api/events/audit");
  const eventResponse = await coordinator.GET("/api/events/{id}/audit", {
    params: { path: { id: event.id } },
  });
  const forbiddenResponse = await user.GET("/api/events/audit");

  expect(allResponse.error).toBeFalsy();
  expect(eventResponse.error).toBeFalsy();
  expect(forbiddenResponse.response.status).toBe(403);

  const expected = expect.objectContaining({
    entity: { kind: "event", id: event.id },
    child_entity: null,
    before: null,
    operated_by: expect.any(String),
    created_at: expect.any(String),
  });
  expect(allResponse.data).toEqual(expect.arrayContaining([expected]));
  expect(eventResponse.data).toEqual([expected]);
});

test("user audit routes include role and ATC status logs", async ({
  atcAdmin,
  auditedUser,
}) => {
  const allResponse = await atcAdmin.GET("/api/users/audit");
  const userResponse = await atcAdmin.GET("/api/users/{id}/audit", {
    params: { path: { id: auditedUser.id } },
  });
  const expected = [
    expect.objectContaining({
      entity: { kind: "user", id: auditedUser.id },
      child_entity: { kind: "user-atc-permission", id: auditedUser.id },
    }),
    expect.objectContaining({
      entity: { kind: "user", id: auditedUser.id },
      child_entity: { kind: "user-role", id: auditedUser.id },
    }),
  ];

  expect(allResponse.error).toBeFalsy();
  expect(userResponse.error).toBeFalsy();
  expect(allResponse.data).toEqual(expect.arrayContaining(expected));
  expect(userResponse.data).toEqual(expect.arrayContaining(expected));

  const atcResponse = await atcAdmin.GET("/api/users/{id}/atc/status/audit", {
    params: { path: { id: auditedUser.id } },
  });
  expect(atcResponse.error).toBeFalsy();
  expect(atcResponse.data).toEqual([
    expect.objectContaining({
      entity: { kind: "user", id: auditedUser.id },
      child_entity: {
        kind: "user-atc-permission",
        id: auditedUser.id,
      },
    }),
  ]);
});
