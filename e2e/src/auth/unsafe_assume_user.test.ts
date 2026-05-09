import { expect, test } from "vitest";
import { getClient } from "../../lib/backend.js";

const clientId = "01K7X723S04R1F5P2P8NJV5QT5";
const clientSecret = "urn:vatprc:client_secret:01K7X71YQQXWSDT2XK1ZNYMS2J";
const unauthorizedClientId = "01KKBQN3MEYA0D33VBX1QW0Y7B";
const unauthorizedClientSecret =
  "urn:vatprc:client_secret:01KKBQNKEM70WGF785J7HS1DWR";

test("issues a user token for an allowed client", async () => {
  const client = await getClient();
  const { data: clientToken } = await client.POST("/auth/token", {
    body: {
      grant_type: "client_credentials",
      client_id: clientId,
      client_secret: clientSecret,
    },
    headers: {
      "content-type": "application/x-www-form-urlencoded",
    },
  });

  expect(clientToken.access_token).toBeTruthy();

  const assumedUserId = "01KKBMA3EGXS5N7XDY4KV7TN80";
  const { data, error, response } = await client.POST(
    "/auth/__unsafe_assume_user",
    {
      body: {
        id: assumedUserId,
        cid: "900001",
        full_name: "E2E Assumed User",
        email: "e2e-assumed-user@example.test",
        roles: ["controller", "event-coordinator"],
      },
      headers: {
        authorization: `Bearer ${clientToken?.access_token}`,
      },
    },
  );

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toBeTruthy();

  expect(data.access_token).toMatch(
    /^[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+$/,
  );
  expect(data.token_type).toBe("Bearer");
  expect(data.expires_in).toBeGreaterThan(0);
  expect(data.refresh_token).toMatch(/^[0-9A-HJKMNP-TV-Z]{26}$/);
  expect(data.scope).toBe("");

  const { data: session } = await client.GET("/api/session", {
    headers: {
      authorization: `Bearer ${data.access_token}`,
    },
  });
  expect(session).toBeTruthy();
  expect(session.user.id).toBe(assumedUserId);
  expect(session.user.full_name).toBe("E2E Assumed User");
  expect(session.user.direct_roles).toEqual([
    "controller",
    "event-coordinator",
  ]);
  expect(session.user.roles).toEqual(
    expect.arrayContaining(["controller", "event-coordinator", "volunteer"]),
  );
});

test("returns unauthorized without a bearer token", async () => {
  const client = await getClient();
  const { data, error, response } = await client.POST(
    "/auth/__unsafe_assume_user",
    {
      body: {
        cid: "900002",
      },
    },
  );

  expect(response.status).toBe(401);
  expect(data).toBeFalsy();
  expect(error).toEqual({
    detail: "missing bearer token",
    status: 401,
    title: "Unauthorized",
    type: "about:blank",
  });
});

test("returns unauthorized for a client without unsafe assume permission", async () => {
  const client = await getClient();
  const { data: clientToken } = await client.POST("/auth/token", {
    body: {
      grant_type: "client_credentials",
      client_id: unauthorizedClientId,
      client_secret: unauthorizedClientSecret,
    },
    headers: {
      "content-type": "application/x-www-form-urlencoded",
    },
  });

  expect(clientToken.access_token).toBeTruthy();

  const { data, error, response } = await client.POST(
    "/auth/__unsafe_assume_user",
    {
      body: {
        cid: "900003",
      },
      headers: {
        authorization: `Bearer ${clientToken.access_token}`,
      },
    },
  );

  expect(response.status).toBe(401);
  expect(data).toBeFalsy();
  expect(error).toEqual({
    detail: "client is not allowed to assume users",
    status: 401,
    title: "Unauthorized",
    type: "about:blank",
  });
});
