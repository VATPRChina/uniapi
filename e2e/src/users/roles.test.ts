import { expect, test as baseTest } from "vitest";
import { getClient } from "../../lib/backend.js";

const test = baseTest
  .extend("staff", async ({}) => {
    return await getClient(["staff"]);
  })
  .extend("user", async ({}) => {
    return await getClient([]);
  });

test("PUT /api/users/{id}/roles updates user roles", async ({ staff, user }) => {
  const session = await user.GET("/api/session");
  expect(session.response.status).toBe(200);

  const { data, error, response } = await staff.PUT("/api/users/{id}/roles", {
    params: {
      path: {
        id: session.data.user.id,
      },
    },
    body: ["event-coordinator"],
  });

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data.direct_roles).toEqual(["event-coordinator"]);
});

test("Tech Director Assistant can assume roles for one request", async () => {
  const client = await getClient(["tech-director-assistant"]);

  const assumedSession = await client.GET("/api/session", {
    headers: {
      "X-Role-Assume": "event-director, operation-director-assistant",
    },
  });

  expect(assumedSession.error).toBeFalsy();
  expect(assumedSession.data.user.roles).toEqual(
    expect.arrayContaining([
      "software-engineer",
      "event-director",
      "event-coordinator",
      "operation-director-assistant",
    ]),
  );

  const nextSession = await client.GET("/api/session");

  expect(nextSession.error).toBeFalsy();
  expect(nextSession.data.user.roles).not.toContain("event-director");
  expect(nextSession.data.user.roles).not.toContain(
    "operation-director-assistant",
  );
});

test("Division Director inherits AFV Facility Engineer across session and user responses", async () => {
  const client = await getClient(["division-director"]);
  const session = await client.GET("/api/session");

  expect(session.error).toBeFalsy();
  expect(session.data.user.direct_roles).toEqual(["division-director"]);
  expect(session.data.user.roles).toEqual(
    expect.arrayContaining([
      "division-director",
      "tech-director",
      "tech-director-assistant",
      "tech-afv-facility-engineer",
      "software-engineer",
      "volunteer",
    ]),
  );

  const me = await client.GET("/api/users/me");
  expect(me.error).toBeFalsy();
  expect(me.data.direct_roles).toEqual(["division-director"]);
  expect(me.data.roles).toContain("tech-afv-facility-engineer");

  const audit = await client.GET("/api/atc/positions/{callsign}/audit", {
    params: { path: { callsign: "ZBAA_TWR" } },
  });
  expect(audit.error).toBeFalsy();
  expect(audit.response.status).toBe(200);
});
