import { expect, test as baseTest } from "vitest";
import { getClient } from "../../../lib/backend.js";

const applicantEmail = `e2e-atc-application-${Date.now()}@example.test`;

const test = baseTest
  .extend("applicant", async ({}) => {
    return await getClient([], {
      email: applicantEmail,
    });
  })
  .extend("otherApplicant", async ({}) => {
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
  })
  .extend("otherApplication", async ({ otherApplicant }) => {
    const sheet = await otherApplicant.GET("/api/atc/applications/sheet");
    expect(sheet.response.status).toBe(200);

    const application = await otherApplicant.POST("/api/atc/applications", {
      body: {
        request_answers: [],
      },
    });

    expect(application.response.status).toBe(200);
    return application.data;
  });

test("POST /api/atc/applications creates an ATC application", async ({
  applicant,
}) => {
  const sheet = await applicant.GET("/api/atc/applications/sheet");
  expect(sheet.response.status).toBe(200);

  const { data, error, response } = await applicant.POST(
    "/api/atc/applications",
    {
      body: {
        request_answers: [],
      },
    },
  );

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toBeTruthy();
  expect(data.id).toMatch(/^[0-9A-HJKMNP-TV-Z]{26}$/);
  expect(data.user_id).toMatch(/^[0-9A-HJKMNP-TV-Z]{26}$/);
  expect(data.applied_at).toBeTruthy();
  expect(data.status).toBe("submitted");
  expect(data.user.id).toBe(data.user_id);
  expect(data.user.full_name).toBeTruthy();
});

test("POST /api/atc/applications rejects a duplicate active application", async ({
  applicant,
  application,
}) => {
  expect(application).toBeTruthy();

  const { data, error, response } = await applicant.POST(
    "/api/atc/applications",
    {
      body: {
        request_answers: [],
      },
    },
  );

  expect(response.status).toBe(409);
  expect(data).toBeFalsy();
  expect(error).toEqual({
    detail: "ATC application already exists",
    status: 409,
    title: "ATC application already exists",
    type: "urn:vatprc-uniapi-error:application-already-exists",
  });
});

test("GET /api/atc/applications lists only the current user's applications for normal users", async ({
  applicant,
  application,
  otherApplication,
}) => {
  const { data, error, response } = await applicant.GET(
    "/api/atc/applications",
  );

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        id: application.id,
        user_id: application.user_id,
        status: "submitted",
        user: expect.objectContaining({
          full_name: expect.any(String),
        }),
      }),
    ]),
  );
  expect(data).not.toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        id: otherApplication.id,
      }),
    ]),
  );
});

test("GET /api/atc/applications lists all applications for ATC application reviewers", async ({
  reviewer,
  application,
}) => {
  const { data, error, response } = await reviewer.GET("/api/atc/applications");

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        id: application.id,
        user_id: application.user_id,
        user_email: applicantEmail,
        status: "submitted",
        user: expect.objectContaining({
          full_name: expect.any(String),
        }),
      }),
    ]),
  );
});

test("PUT /api/atc/applications/{id}/review reviews an ATC application", async ({
  reviewer,
  application,
}) => {
  const sheet = await reviewer.GET("/api/atc/applications/review-sheet");
  expect(sheet.response.status).toBe(200);

  const { data, error, response } = await reviewer.PUT(
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

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toEqual(
    expect.objectContaining({
      id: application.id,
      user_id: application.user_id,
      status: "approved",
      application_filing_answers: [],
      review_filing_answers: [],
      user: expect.objectContaining({
        full_name: expect.any(String),
      }),
    }),
  );
});
