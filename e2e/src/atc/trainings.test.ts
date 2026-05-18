import { expect, test as baseTest } from "vitest";
import { getClient } from "../../lib/backend.js";

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
  .extend("activeTraining", async ({ mentor, mentorSession, traineeSession }) => {
    const suffix = Date.now().toString();
    const training = await mentor.POST("/api/atc/trainings", {
      body: {
        name: `E2E Active Training ${suffix}`,
        trainer_id: mentorSession.user.id,
        trainee_id: traineeSession.user.id,
        start_at: "2031-03-01T10:00:00Z",
        end_at: "2031-03-01T11:00:00Z",
      },
    });

    expect(training.response.status).toBe(200);
    return training.data;
  })
  .extend("finishedTraining", async ({ mentor, mentorSession, traineeSession }) => {
    const suffix = Date.now().toString();
    const training = await mentor.POST("/api/atc/trainings", {
      body: {
        name: `E2E Finished Training ${suffix}`,
        trainer_id: mentorSession.user.id,
        trainee_id: traineeSession.user.id,
        start_at: "2031-04-01T10:00:00Z",
        end_at: "2031-04-01T11:00:00Z",
      },
    });
    expect(training.response.status).toBe(200);

    const sheet = await mentor.GET("/api/atc/trainings/record-sheet");
    expect(sheet.response.status).toBe(200);

    const recordedTraining = await mentor.PUT("/api/atc/trainings/{id}/record", {
      params: {
        path: {
          id: training.data.id,
        },
      },
      body: {
        request_answers: [],
      },
    });

    expect(recordedTraining.response.status).toBe(200);
    return recordedTraining.data;
  });

test("GET /api/atc/trainings/record-sheet returns the training record sheet", async ({
  mentor,
}) => {
  const { data, error, response } = await mentor.GET(
    "/api/atc/trainings/record-sheet",
  );

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toEqual({
    id: "training-record",
    name: "Training Record Sheet",
    fields: expect.any(Array),
  });
});

test("POST /api/atc/trainings creates a training", async ({
  mentor,
  mentorSession,
  traineeSession,
}) => {
  const suffix = Date.now().toString();

  const { data, error, response } = await mentor.POST("/api/atc/trainings", {
    body: {
      name: `E2E Created Training ${suffix}`,
      trainer_id: mentorSession.user.id,
      trainee_id: traineeSession.user.id,
      start_at: "2031-02-01T10:00:00Z",
      end_at: "2031-02-01T11:00:00Z",
    },
  });

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toEqual(
    expect.objectContaining({
      name: `E2E Created Training ${suffix}`,
      trainer_id: mentorSession.user.id,
      trainee_id: traineeSession.user.id,
      start_at: "2031-02-01T10:00:00Z",
      end_at: "2031-02-01T11:00:00Z",
      deleted_at: null,
      record_sheet_filing_id: null,
      record_sheet_filing: null,
    }),
  );
  expect(data.id).toMatch(/^[0-9A-HJKMNP-TV-Z]{26}$/);
});

test("GET /api/atc/trainings/active lists active trainings", async ({
  mentor,
  activeTraining,
}) => {
  const { data, error, response } = await mentor.GET(
    "/api/atc/trainings/active",
  );

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        id: activeTraining.id,
        name: activeTraining.name,
        record_sheet_filing_id: null,
        deleted_at: null,
      }),
    ]),
  );
});

test("GET /api/atc/trainings/finished lists finished trainings", async ({
  mentor,
  finishedTraining,
}) => {
  const { data, error, response } = await mentor.GET(
    "/api/atc/trainings/finished",
  );

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        id: finishedTraining.id,
        name: finishedTraining.name,
        record_sheet_filing_id: finishedTraining.record_sheet_filing_id,
        record_sheet_filing: [],
      }),
    ]),
  );
});

test("GET /api/atc/trainings/by-user/{userId} lists trainings for a user", async ({
  mentor,
  traineeSession,
  activeTraining,
  finishedTraining,
}) => {
  const { data, error, response } = await mentor.GET(
    "/api/atc/trainings/by-user/{userId}",
    {
      params: {
        path: {
          userId: traineeSession.user.id,
        },
      },
    },
  );

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        id: activeTraining.id,
        trainee_id: traineeSession.user.id,
      }),
      expect.objectContaining({
        id: finishedTraining.id,
        trainee_id: traineeSession.user.id,
      }),
    ]),
  );
});
