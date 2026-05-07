import test from "ava";
import { getBackend } from "../../lib/backend.js";

test("redirects to login for a valid client", async (t) => {
  const client = await getBackend();
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

  t.is(response.status, 307);
  t.regex(response.headers.get("location") ?? "", /^\/auth\/login\?state=/);
  t.regex(
    response.headers.get("set-cookie") ?? "",
    /^auth-[0-9A-HJKMNP-TV-Z]{26}=/,
  );
});

test("returns error for an invalid client", async (t) => {
  const client = await getBackend();
  const { data, error, response } = await client.GET("/auth/authorize", {
    params: {
      query: {
        response_type: "code",
        client_id: "01KR1CMM7G26J7J4HSQS7N7J8M",
        redirect_uri: "http://localhost:3000/auth/callback",
        state: "e2e-state",
      },
    },
    redirect: "manual",
  });

  t.is(response.status, 401);
  t.falsy(data);
  t.deepEqual(error, {
    detail: "client is invalid",
    status: 401,
    title: "Unauthorized",
    type: "about:blank",
  });
});

test("returns error for a client with an invalid redirect uri", async (t) => {
  const client = await getBackend();
  const { data, error, response } = await client.GET("/auth/authorize", {
    params: {
      query: {
        response_type: "code",
        client_id: "01J2ED6BDX9J9BTYS2RVW83ME7",
        redirect_uri: "http://localhost:3000/auth/callback2",
        state: "e2e-state",
      },
    },
    redirect: "manual",
  });

  t.is(response.status, 401);
  t.falsy(data);
  t.deepEqual(error, {
    detail: "client is invalid",
    status: 401,
    title: "Unauthorized",
    type: "about:blank",
  });
});
