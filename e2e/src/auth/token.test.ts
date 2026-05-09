import { describe, expect, test } from "vitest";
import { getClient } from "../../lib/backend.js";

describe("client_credentials", () => {
  test("issues a client credentials access token", async () => {
    const client = await getClient();
    const { data, error, response } = await client.POST("/auth/token", {
      body: {
        grant_type: "client_credentials",
        client_id: "01K7X723S04R1F5P2P8NJV5QT5",
        client_secret: "urn:vatprc:client_secret:01K7X71YQQXWSDT2XK1ZNYMS2J",
      },
      headers: {
        "content-type": "application/x-www-form-urlencoded",
      },
    });

    expect(error).toBeFalsy();
    expect(response.status).toBe(200);
    expect(data).toBeTruthy();
    if (data === undefined) {
      throw new Error("Expected token response data");
    }

    expect(data.access_token).toMatch(
      /^[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+$/,
    );
    expect(data.token_type).toBe("Bearer");
    expect(data.expires_in).toBeGreaterThan(0);
    expect(data.refresh_token).toBeUndefined();
    expect(data.scope).toBe("");
  });

  test("returns error for client credentials with an invalid secret", async () => {
    const client = await getClient();
    const { data, error, response } = await client.POST("/auth/token", {
      body: {
        grant_type: "client_credentials",
        client_id: "01K7X723S04R1F5P2P8NJV5QT5",
        client_secret: "invalid-secret",
      },
      headers: {
        "content-type": "application/x-www-form-urlencoded",
      },
    });

    expect(response.status).toBe(400);
    expect(data).toBeFalsy();
    expect(error).toBe("invalid_grant: client_id or client_secret is invalid");
  });
});

describe.todo("device_code");
describe.todo("refresh_token");
describe.todo("authorization_code");
describe.todo("client_credentials");
describe("unsupported", () => {
  test("returns error for unsupported grant type", async () => {
    const client = await getClient();
    const { data, error, response } = await client.POST("/auth/token", {
      body: {
        grant_type: "unsupported_grant_type",
      },
      headers: {
        "content-type": "application/x-www-form-urlencoded",
      },
    });

    expect(response.status).toBe(400);
    expect(data).toBeFalsy();
    expect(error).toBe(
      "unsupported_grant_type: The authorization grant type is not supported by the authorization server.",
    );
  });
});
