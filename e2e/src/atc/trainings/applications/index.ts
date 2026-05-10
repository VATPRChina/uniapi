import { expect, test as baseTest } from "vitest";
import { getClient } from "../../../../lib/backend.js";

const test = baseTest
  .extend("admin", async ({}) => {
    return await getClient(["controller-training-director-assistant"]);
  })
  .extend("trainee", async ({}) => {
    return await getClient([]);
  })
  .extend("otherTrainee", async ({}) => {
    return await getClient([]);
  })
  .extend("traineeSession", async ({ trainee }) => {
    const session = await trainee.GET("/api/session");
    expect(session.response.status).toBe(200);
    return session.data;
  })
  .extend("otherTraineeSession", async ({ otherTrainee }) => {
    const session = await otherTrainee.GET("/api/session");
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
  .extend("otherControllerTrainee", async ({ admin, otherTraineeSession }) => {
    const status = await admin.PUT("/api/users/{id}/atc/status", {
      params: {
        path: {
          id: otherTraineeSession.user.id,
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
    return otherTraineeSession;
  })
  .extend("application", async ({ trainee, controllerTrainee }) => {
    const suffix = Date.now().toString();
    const application = await trainee.POST("/api/atc/trainings/applications", {
      body: {
        name: `E2E Training Application ${suffix}`,
        slots: [
          {
            start_at: "2031-07-01T10:00:00Z",
            end_at: "2031-07-01T11:00:00Z",
          },
        ],
      },
    });

    expect(controllerTrainee).toBeTruthy();
    expect(application.response.status).toBe(200);
    return application.data;
  })
  .extend(
    "otherApplication",
    async ({ otherTrainee, otherControllerTrainee }) => {
      const suffix = Date.now().toString();
      const application = await otherTrainee.POST(
        "/api/atc/trainings/applications",
        {
          body: {
            name: `E2E Other Training Application ${suffix}`,
            slots: [
              {
                start_at: "2031-07-02T10:00:00Z",
                end_at: "2031-07-02T11:00:00Z",
              },
            ],
          },
        },
      );

      expect(otherControllerTrainee).toBeTruthy();
      expect(application.response.status).toBe(200);
      return application.data;
    },
  );

test("POST /api/atc/trainings/applications creates a training application", async ({
  trainee,
  controllerTrainee,
}) => {
  const suffix = Date.now().toString();

  const { data, error, response } = await trainee.POST(
    "/api/atc/trainings/applications",
    {
      body: {
        name: `E2E Created Training Application ${suffix}`,
        slots: [
          {
            start_at: "2031-07-03T10:00:00Z",
            end_at: "2031-07-03T11:00:00Z",
          },
        ],
      },
    },
  );

  expect(controllerTrainee).toBeTruthy();
  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toEqual(
    expect.objectContaining({
      name: `E2E Created Training Application ${suffix}`,
      trainee_id: controllerTrainee.user.id,
      status: "pending",
      train_id: null,
      slots: [
        expect.objectContaining({
          start_at: "2031-07-03T10:00:00Z",
          end_at: "2031-07-03T11:00:00Z",
        }),
      ],
    }),
  );
  expect(data.id).toMatch(/^[0-9A-HJKMNP-TV-Z]{26}$/);
});

test("GET /api/atc/trainings/applications lists visible training applications", async ({
  trainee,
  admin,
  application,
  otherApplication,
}) => {
  const ownApplications = await trainee.GET("/api/atc/trainings/applications");

  expect(ownApplications.error).toBeFalsy();
  expect(ownApplications.response.status).toBe(200);
  expect(ownApplications.data).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        id: application.id,
      }),
    ]),
  );
  expect(ownApplications.data).not.toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        id: otherApplication.id,
      }),
    ]),
  );

  const adminApplications = await admin.GET("/api/atc/trainings/applications");

  expect(adminApplications.error).toBeFalsy();
  expect(adminApplications.response.status).toBe(200);
  expect(adminApplications.data).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        id: application.id,
      }),
      expect.objectContaining({
        id: otherApplication.id,
      }),
    ]),
  );
});
