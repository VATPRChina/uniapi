import { expect, test as baseTest } from "vitest";
import { getClient } from "../../lib/backend.js";

const test = baseTest
  .extend("admin", async ({}) => {
    return await getClient(["controller-training-director-assistant"]);
  })
  .extend("controller", async ({}) => {
    return await getClient([]);
  })
  .extend("listedController", async ({ admin, controller }) => {
    const session = await controller.GET("/api/session");
    expect(session.response.status).toBe(200);

    const status = await admin.PUT("/api/users/{id}/atc/status", {
      params: {
        path: {
          id: session.data.user.id,
        },
      },
      body: {
        is_absent: false,
        is_visiting: true,
        rating: "C1",
        permissions: [
          {
            position_kind_id: "APP",
            state: "certified",
            solo_expires_at: null,
          },
          {
            position_kind_id: "TWR",
            state: "solo",
            solo_expires_at: "2031-01-01T00:00:00Z",
          },
        ],
      },
    });

    expect(status.response.status).toBe(200);
    return status.data;
  });

test("GET /api/atc/controllers lists ATC controllers", async ({
  listedController,
}) => {
  const client = await getClient();

  const { data, error, response } = await client.GET("/api/atc/controllers");

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        user_id: listedController.user_id,
        is_absent: false,
        is_visiting: true,
        rating: "C1",
        user: expect.objectContaining({
          id: listedController.user_id,
          full_name: expect.any(String),
        }),
        permissions: [
          {
            position_kind_id: "APP",
            state: "certified",
            solo_expires_at: null,
          },
          {
            position_kind_id: "TWR",
            state: "solo",
            solo_expires_at: "2031-01-01T00:00:00Z",
          },
        ],
      }),
    ]),
  );
});

test("GET /api/users/{id}/atc/status returns a user ATC status", async ({
  admin,
  listedController,
}) => {
  const { data, error, response } = await admin.GET(
    "/api/users/{id}/atc/status",
    {
      params: {
        path: {
          id: listedController.user_id,
        },
      },
    },
  );

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toEqual(listedController);
});

test("PUT /api/users/{id}/atc/status rejects solo without expiration", async ({
  admin,
  controller,
}) => {
  const session = await controller.GET("/api/session");
  expect(session.response.status).toBe(200);

  const { data, error, response } = await admin.PUT(
    "/api/users/{id}/atc/status",
    {
      params: {
        path: {
          id: session.data.user.id,
        },
      },
      body: {
        is_absent: false,
        is_visiting: false,
        rating: "S2",
        permissions: [
          {
            position_kind_id: "TWR",
            state: "solo",
            solo_expires_at: null,
          },
        ],
      },
    },
  );

  expect(response.status).toBe(400);
  expect(data).toBeFalsy();
  expect(error).toEqual({
    detail: "solo expiration not provided",
    status: 400,
    title: "solo expiration not provided",
    type: "urn:vatprc-uniapi-error:solo-expiration-not-provided",
  });
});
