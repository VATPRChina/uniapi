import { expect, test } from "vitest";
import { getBaseUrl, getClient } from "../../lib/backend.js";

test("redirects to login for a valid client", async () => {
  const client = await getClient();
  const { response } = await client.GET("/auth/authorize", {
    params: {
      query: {
        response_type: "code",
        client_id: "01J2ED6BDX9J9BTYS2RVW83ME7",
        redirect_uri: "http://localhost:3000/auth/callback",
        state: "e2e-state",
      },
    },
    redirect: "manual",
  });

  expect(response.status).toBe(307);
  expect(response.headers.get("location") ?? "").toMatch(
    /^\/auth\/login\?state=/,
  );
  expect(response.headers.get("set-cookie") ?? "").toMatch(
    /^auth-[0-9A-HJKMNP-TV-Z]{26}=/,
  );
  const location = response.headers.get("location") ?? "";
  const stateId = new URLSearchParams(location.split("?", 2)[1]).get("state");
  const [cookiePair] = (response.headers.get("set-cookie") ?? "").split(";", 1);
  const [cookieName, cookieValue] = cookiePair.split("=", 2);
  expect(cookieName).toBe(`auth-${stateId}`);
  expect(JSON.parse(decodeURIComponent(cookieValue))).toEqual({
    auth_type: "code",
    client_id: "01J2ED6BDX9J9BTYS2RVW83ME7",
    redirect_uri: "http://localhost:3000/auth/callback",
    user_code: null,
    state: "e2e-state",
  });
});

test("returns error for an invalid client", async () => {
  const response = await fetch(authorizeUrl({
    response_type: "code",
    client_id: "01KR1CMM7G26J7J4HSQS7N7J8M",
    redirect_uri: "http://localhost:3000/auth/callback",
    state: "e2e-state",
  }), {
    redirect: "manual",
  });
  const body = await response.text();

  expect(response.status).toBe(200);
  expect(response.headers.get("content-type") ?? "").toContain("text/html");
  expect(body).toContain("Invalid client");
  expect(body).toContain("The client or redirect URI is invalid.");
});

test("returns error for a client with an invalid redirect uri", async () => {
  const response = await fetch(authorizeUrl({
    response_type: "code",
    client_id: "01J2ED6BDX9J9BTYS2RVW83ME7",
    redirect_uri: "http://localhost:3000/auth/callback2",
    state: "e2e-state",
  }), {
    redirect: "manual",
  });
  const body = await response.text();

  expect(response.status).toBe(200);
  expect(response.headers.get("content-type") ?? "").toContain("text/html");
  expect(body).toContain("Invalid client");
  expect(body).toContain("The client or redirect URI is invalid.");
});

function authorizeUrl(query: Record<string, string>): URL {
  const url = new URL("/auth/authorize", getBaseUrl());
  for (const [key, value] of Object.entries(query)) {
    url.searchParams.set(key, value);
  }
  return url;
}
