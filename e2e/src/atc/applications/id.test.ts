import { expect, test as baseTest } from "vitest";
import { getClient } from "../../../lib/backend.js";

const test = baseTest
  .extend("applicant", async ({}) => {
    return await getClient([]);
  })
  .extend("reviewer", async ({}) => {
    return await getClient(["controller-training-director-assistant"]);
  })
  .extend("application", async ({ applicant }) => {
    const sheet = await applicant.GET("/api/atc/applications/sheet");
    expect(sheet.response.status).toBe(200);

    const application = await applicant.POST("/api/atc/applications", {
      body: {
        request_answers: [],
      },
    });

    expect(application.response.status).toBe(200);
    return application.data;
  });

test("GET /api/atc/applications/{id} returns an ATC application", async ({
  applicant,
  application,
}) => {
  const { data, error, response } = await applicant.GET(
    "/api/atc/applications/{id}",
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
      user_id: application.user_id,
      status: "submitted",
      application_filing_answers: [],
      review_filing_answers: null,
      user: expect.objectContaining({
        full_name: expect.any(String),
      }),
    }),
  );
});

test("PUT /api/atc/applications/{id} updates an ATC application", async ({
  applicant,
  application,
}) => {
  const { data, error, response } = await applicant.PUT(
    "/api/atc/applications/{id}",
    {
      params: {
        path: {
          id: application.id,
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
      id: application.id,
      user_id: application.user_id,
      status: "submitted",
      application_filing_answers: [],
      review_filing_answers: null,
      user: expect.objectContaining({
        full_name: expect.any(String),
      }),
    }),
  );
});

test("PUT /api/atc/applications/{id} rejects updates after review", async ({
  applicant,
  reviewer,
  application,
}) => {
  const sheet = await reviewer.GET("/api/atc/applications/review-sheet");
  expect(sheet.response.status).toBe(200);

  const reviewedApplication = await reviewer.PUT(
    "/api/atc/applications/{id}/review",
    {
      params: {
        path: {
          id: application.id,
        },
      },
      body: {
        status: "approved",
        review_answers: [],
      },
    },
  );
  expect(reviewedApplication.response.status).toBe(200);

  const { data, error, response } = await applicant.PUT(
    "/api/atc/applications/{id}",
    {
      params: {
        path: {
          id: application.id,
        },
      },
      body: {
        request_answers: [],
      },
    },
  );

  expect(response.status).toBe(409);
  expect(data).toBeFalsy();
  expect(error).toEqual({
    detail: "ATC application cannot be updated at current status",
    status: 409,
    title: "ATC application cannot be updated at current status",
    type: "urn:vatprc-uniapi-error:application-cannot-update",
  });
});
