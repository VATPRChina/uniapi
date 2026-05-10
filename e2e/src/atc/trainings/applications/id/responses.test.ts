import { expect, test as baseTest } from "vitest";
import { getClient } from "../../../../../lib/backend.js";

const test = baseTest
  .extend("admin", async ({}) => {
    return await getClient(["controller-training-director-assistant"]);
  })
  .extend("mentor", async ({}) => {
    return await getClient(["controller-training-mentor"]);
  })
  .extend("trainee", async ({}) => {
    return await getClient([]);
  })
  .extend("traineeSession", async ({ trainee }) => {
    const session = await trainee.GET("/api/session");
    expect(session.response.status).toBe(200);
    return session.data;
  })
  .extend("controllerTrainee", async ({ admin, traineeSession }) => {
    const status = await admin.PUT("/api/users/{id}/atc/status", {
      params: {
        path: {
          id: traineeSession.user.id,
        },
      },
      body: {
        is_absent: false,
        is_visiting: false,
        rating: "S2",
        permissions: [
          {
            position_kind_id: "TWR",
            state: "certified",
            solo_expires_at: null,
          },
        ],
      },
    });

    expect(status.response.status).toBe(200);
    return traineeSession;
  })
  .extend("application", async ({ trainee, controllerTrainee }) => {
    const suffix = Date.now().toString();
    const application = await trainee.POST("/api/atc/trainings/applications", {
      body: {
        name: `E2E Training Application Responses ${suffix}`,
        slots: [
          {
            start_at: "2031-09-01T10:00:00Z",
            end_at: "2031-09-01T11:00:00Z",
          },
        ],
      },
    });

    expect(controllerTrainee).toBeTruthy();
    expect(application.response.status).toBe(200);
    return application.data;
  })
  .extend("response", async ({ mentor, application }) => {
    const response = await mentor.PUT(
      "/api/atc/trainings/applications/{id}/response",
      {
        params: {
          path: {
            id: application.id,
          },
        },
        body: {
          slot_id: null,
          comment: "Available as a backup trainer.",
        },
      },
    );

    expect(response.response.status).toBe(200);
    return response.data;
  });

test("GET /api/atc/trainings/applications/{id}/responses lists responses", async ({
  trainee,
  application,
  response,
}) => {
  const { data, error, response: httpResponse } = await trainee.GET(
    "/api/atc/trainings/applications/{id}/responses",
    {
      params: {
        path: {
          id: application.id,
        },
      },
    },
  );

  expect(error).toBeFalsy();
  expect(httpResponse.status).toBe(200);
  expect(data).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        id: response.id,
        application_id: application.id,
        trainer_id: response.trainer_id,
        is_accepted: false,
        comment: "Available as a backup trainer.",
      }),
    ]),
  );
});
