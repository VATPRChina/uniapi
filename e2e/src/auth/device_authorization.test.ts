import test from "ava";
import { getBackend } from "../../lib/backend.js";

test("issues a device authorization for a valid client", async (t) => {
  const client = await getBackend();
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

  t.falsy(error);
  t.is(response.status, 200);
  if (!t.truthy(data)) return;
  t.regex(data.device_code, /^[0-9A-HJKMNP-TV-Z]{26}$/);
  t.regex(
    data.user_code,
    /^[BCDFGHJKLMNPQRSTVWXZ]{4}-[BCDFGHJKLMNPQRSTVWXZ]{4}$/,
  );
  t.is(data.verification_uri, "http://127.0.0.1:3010/auth/device");
  t.is(
    data.verification_uri_complete,
    "http://127.0.0.1:3010/auth/device?user_code=" +
      data.user_code.replace("-", ""),
  );
  t.true(data.expires_in > 0);
});

test("returns error for device authorization with an invalid client", async (t) => {
  const client = await getBackend();
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

  t.is(response.status, 401);
  t.falsy(data);
  t.deepEqual(error, {
    detail: "invalid_client: client_id not found",
    status: 401,
    title: "Unauthorized",
    type: "about:blank",
  });
});
