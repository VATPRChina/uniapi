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
