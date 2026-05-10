import { expect, test as baseTest } from "vitest";
import { getClient } from "../../../../lib/backend.js";

const test = baseTest
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
  .extend("mentorSession", async ({ mentor }) => {
    const session = await mentor.GET("/api/session");
    expect(session.response.status).toBe(200);
    return session.data;
  })
  .extend("training", async ({ mentor, mentorSession, traineeSession }) => {
    const suffix = Date.now().toString();
    const training = await mentor.POST("/api/atc/trainings", {
      body: {
        name: `E2E Training ID ${suffix}`,
        trainer_id: mentorSession.user.id,
        trainee_id: traineeSession.user.id,
        start_at: "2031-05-01T10:00:00Z",
        end_at: "2031-05-01T11:00:00Z",
      },
    });

    expect(training.response.status).toBe(200);
    return training.data;
  });

test("GET /api/atc/trainings/{id} returns a training", async ({
  mentor,
  training,
}) => {
  const { data, error, response } = await mentor.GET(
    "/api/atc/trainings/{id}",
    {
      params: {
        path: {
          id: training.id,
        },
      },
    },
  );

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toEqual(
    expect.objectContaining({
      id: training.id,
      name: training.name,
      trainer_id: training.trainer_id,
      trainee_id: training.trainee_id,
      start_at: training.start_at,
      end_at: training.end_at,
      deleted_at: null,
      record_sheet_filing_id: null,
      record_sheet_filing: null,
    }),
  );
});

test("PUT /api/atc/trainings/{id} updates a training", async ({
  mentor,
  mentorSession,
  traineeSession,
  training,
}) => {
  const update = {
    name: `E2E Updated Training ${Date.now()}`,
    trainer_id: mentorSession.user.id,
    trainee_id: traineeSession.user.id,
    start_at: "2031-05-02T10:00:00Z",
    end_at: "2031-05-02T11:30:00Z",
  };

  const { data, error, response } = await mentor.PUT(
    "/api/atc/trainings/{id}",
    {
      params: {
        path: {
          id: training.id,
        },
      },
      body: update,
    },
  );

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toEqual(
    expect.objectContaining({
      id: training.id,
      ...update,
      deleted_at: null,
      record_sheet_filing_id: null,
      record_sheet_filing: null,
    }),
  );
  expect(data.updated_at).not.toBe(training.updated_at);
});

test("DELETE /api/atc/trainings/{id} deletes a future training", async ({
  mentor,
  training,
}) => {
  const { data, error, response } = await mentor.DELETE(
    "/api/atc/trainings/{id}",
    {
      params: {
        path: {
          id: training.id,
        },
      },
    },
  );

  expect(error).toBeFalsy();
  expect(response.status).toBe(204);
  expect(data).toBeFalsy();

  const deletedTraining = await mentor.GET("/api/atc/trainings/{id}", {
    params: {
      path: {
        id: training.id,
      },
    },
  });
  expect(deletedTraining.response.status).toBe(200);
  expect(deletedTraining.data.deleted_at).toBeTruthy();
});
