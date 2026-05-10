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
        name: `E2E Training Record ${suffix}`,
        trainer_id: mentorSession.user.id,
        trainee_id: traineeSession.user.id,
        start_at: "2031-06-01T10:00:00Z",
        end_at: "2031-06-01T11:00:00Z",
      },
    });

    expect(training.response.status).toBe(200);
    return training.data;
  });

test("PUT /api/atc/trainings/{id}/record sets a training record", async ({
  mentor,
  training,
}) => {
  const sheet = await mentor.GET("/api/atc/trainings/record-sheet");
  expect(sheet.response.status).toBe(200);

  const { data, error, response } = await mentor.PUT(
    "/api/atc/trainings/{id}/record",
    {
      params: {
        path: {
          id: training.id,
        },
      },
      body: {
        request_answers: [],
      },
    },
  );

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toEqual(
    expect.objectContaining({
      id: training.id,
      record_sheet_filing_id: expect.stringMatching(/^[0-9A-HJKMNP-TV-Z]{26}$/),
      record_sheet_filing: [],
    }),
  );
});
