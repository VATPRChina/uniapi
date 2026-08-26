import { expect, test } from "vitest";
import { getBaseUrl } from "../lib/backend.js";

test("unknown endpoints return RFC 9457 problem details", async () => {
  const response = await fetch(`${getBaseUrl()}/api/does-not-exist`);
  const body = await response.json();

  expect(response.status).toBe(404);
  expect(response.headers.get("content-type")).toContain(
    "application/problem+json",
  );
  expect(body).toEqual({
    type: "urn:vatprc-uniapi-error:not-found",
    title: "endpoint with id /api/does-not-exist not found",
    status: 404,
    detail: "endpoint with id /api/does-not-exist not found",
  });
});
