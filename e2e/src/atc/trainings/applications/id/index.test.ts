import { expect, test as baseTest } from "vitest";
import { getClient } from "../../../../../lib/backend.js";

const test = baseTest
  .extend("traineeEmail", async ({}) => {
    return `e2e-training-application-id-${Date.now()}-${Math.random()}@example.test`;
  })
  .extend("admin", async ({}) => {
    return await getClient(["controller-training-director-assistant"]);
  })
  .extend("trainee", async ({ traineeEmail }) => {
    return await getClient([], {
      email: traineeEmail,
    });
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
        name: `E2E Training Application ID ${suffix}`,
        slots: [
          {
            start_at: "2031-08-01T10:00:00Z",
            end_at: "2031-08-01T11:00:00Z",
          },
        ],
      },
    });

    expect(controllerTrainee).toBeTruthy();
    expect(application.response.status).toBe(200);
    return application.data;
  });

test("GET /api/atc/trainings/applications/{id} returns a training application", async ({
  trainee,
  admin,
  application,
  traineeEmail,
}) => {
  const { data, error, response } = await trainee.GET(
    "/api/atc/trainings/applications/{id}",
    {
      params: {
        path: {
          id: application.id,
        },
      },
    },
  );

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toEqual(
    expect.objectContaining({
      id: application.id,
      trainee_id: application.trainee_id,
      name: application.name,
      status: "pending",
      train_id: null,
      slots: [
        expect.objectContaining({
          application_id: application.id,
          start_at: "2031-08-01T10:00:00Z",
          end_at: "2031-08-01T11:00:00Z",
        }),
      ],
    }),
  );

  const adminApplication = await admin.GET(
    "/api/atc/trainings/applications/{id}",
    {
      params: {
        path: {
          id: application.id,
        },
      },
    },
  );

  expect(adminApplication.error).toBeFalsy();
  expect(adminApplication.response.status).toBe(200);
  expect(adminApplication.data).toEqual(
    expect.objectContaining({
      id: application.id,
      trainee_id: application.trainee_id,
      trainee_email: traineeEmail,
    }),
  );
});

test("PUT /api/atc/trainings/applications/{id} updates a training application", async ({
  trainee,
  application,
}) => {
  const { data, error, response } = await trainee.PUT(
    "/api/atc/trainings/applications/{id}",
    {
      params: {
        path: {
          id: application.id,
        },
      },
      body: {
        name: `E2E Updated Training Application ${Date.now()}`,
        slots: [
          {
            start_at: "2031-08-02T10:00:00Z",
            end_at: "2031-08-02T11:30:00Z",
          },
        ],
      },
    },
  );

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toEqual(
    expect.objectContaining({
      id: application.id,
      trainee_id: application.trainee_id,
      status: "pending",
      train_id: null,
      slots: [
        expect.objectContaining({
          application_id: application.id,
          start_at: "2031-08-02T10:00:00Z",
          end_at: "2031-08-02T11:30:00Z",
        }),
      ],
    }),
  );
  expect(data.updated_at).not.toBe(application.updated_at);
});

test("DELETE /api/atc/trainings/applications/{id} cancels a training application", async ({
  trainee,
  application,
}) => {
  const { data, error, response } = await trainee.DELETE(
    "/api/atc/trainings/applications/{id}",
    {
      params: {
        path: {
          id: application.id,
        },
      },
    },
  );

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toEqual(
    expect.objectContaining({
      id: application.id,
      trainee_id: application.trainee_id,
      status: "cancelled",
    }),
  );
});
