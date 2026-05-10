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
        name: `E2E Training Application Response ${suffix}`,
        slots: [
          {
            start_at: "2031-10-01T10:00:00Z",
            end_at: "2031-10-01T11:00:00Z",
          },
        ],
      },
    });

    expect(controllerTrainee).toBeTruthy();
    expect(application.response.status).toBe(200);
    return application.data;
  })
  .extend("acceptedResponse", async ({ mentor, application }) => {
    const response = await mentor.PUT(
      "/api/atc/trainings/applications/{id}/response",
      {
        params: {
          path: {
            id: application.id,
          },
        },
        body: {
          slot_id: application.slots[0].id,
          comment: "I can take this training slot.",
        },
      },
    );

    expect(response.response.status).toBe(200);
    return response.data;
  });

test("PUT /api/atc/trainings/applications/{id}/response responds to a training application", async ({
  mentor,
  application,
}) => {
  const { data, error, response } = await mentor.PUT(
    "/api/atc/trainings/applications/{id}/response",
    {
      params: {
        path: {
          id: application.id,
        },
      },
      body: {
        slot_id: application.slots[0].id,
        comment: "I can take this training slot.",
      },
    },
  );

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toEqual(
    expect.objectContaining({
      application_id: application.id,
      is_accepted: true,
      comment: "I can take this training slot.",
      trainer: expect.objectContaining({
        full_name: expect.any(String),
      }),
    }),
  );
  expect(data.id).toMatch(/^[0-9A-HJKMNP-TV-Z]{26}$/);
});

test("PUT /api/atc/trainings/applications/{id}/response creates a training when accepted", async ({
  mentor,
  application,
  acceptedResponse,
}) => {
  const { data, error, response } = await mentor.GET(
    "/api/atc/trainings/active",
  );

  expect(acceptedResponse.is_accepted).toBe(true);
  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        name: application.name,
        trainer_id: acceptedResponse.trainer_id,
        trainee_id: application.trainee_id,
        start_at: application.slots[0].start_at,
        end_at: application.slots[0].end_at,
        deleted_at: null,
        record_sheet_filing_id: null,
      }),
    ]),
  );
});

test("PUT /api/atc/trainings/applications/{id}/response rejects accepting an accepted application", async ({
  mentor,
  application,
  acceptedResponse,
}) => {
  expect(acceptedResponse.is_accepted).toBe(true);

  const { data, error, response } = await mentor.PUT(
    "/api/atc/trainings/applications/{id}/response",
    {
      params: {
        path: {
          id: application.id,
        },
      },
      body: {
        slot_id: application.slots[0].id,
        comment: "Trying to accept again.",
      },
    },
  );

  expect(response.status).toBe(409);
  expect(data).toBeFalsy();
  expect(error).toEqual({
    detail: "training application already accepted",
    status: 409,
    title: "training application already accepted",
    type: "urn:vatprc-uniapi-error:training-application-already-accepted",
  });
});
