import { expect, test } from "vitest";
import { getClient } from "../../lib/backend.js";

test("issues a device authorization for a valid client", async () => {
  const client = await getClient();
  const { data, error, response } = await client.POST(
    "/auth/device_authorization",
    {
      body: {
        client_id: "01J2ED6BDX9J9BTYS2RVW83ME7",
      },
      headers: {
        "content-type": "application/x-www-form-urlencoded",
      },
    },
  );

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toBeTruthy();
  if (data === undefined) {
    throw new Error("Expected device authorization response data");
  }

  expect(data.device_code).toMatch(/^[0-9A-HJKMNP-TV-Z]{26}$/);
  expect(data.user_code).toMatch(
    /^[BCDFGHJKLMNPQRSTVWXZ]{4}-[BCDFGHJKLMNPQRSTVWXZ]{4}$/,
  );
  expect(data.verification_uri).toBe("http://127.0.0.1:3010/auth/device");
  expect(data.verification_uri_complete).toBe(
    "http://127.0.0.1:3010/auth/device?user_code=" +
      data.user_code.replace("-", ""),
  );
  expect(data.expires_in).toBeGreaterThan(0);
});

test("returns error for device authorization with an invalid client", async () => {
  const client = await getClient();
  const { data, error, response } = await client.POST(
    "/auth/device_authorization",
    {
      body: {
        client_id: "01KR1CMM7G26J7J4HSQS7N7J8M",
      },
      headers: {
        "content-type": "application/x-www-form-urlencoded",
      },
    },
  );

  expect(response.status).toBe(401);
  expect(data).toBeFalsy();
  expect(error).toBe("invalid_client: client_id not found");
});
