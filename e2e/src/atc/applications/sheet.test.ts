import { expect, test as baseTest } from "vitest";
import { getClient } from "../../../lib/backend.js";

const test = baseTest.extend("applicant", async ({}) => {
  return await getClient([]);
});

test("GET /api/atc/applications/sheet returns the ATC application sheet", async ({
  applicant,
}) => {
  const { data, error, response } = await applicant.GET(
    "/api/atc/applications/sheet",
  );

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toEqual({
    id: "atc-application",
    name: "ATC Application Sheet",
    fields: expect.any(Array),
  });
});

test("GET /api/atc/applications/review-sheet returns the ATC application review sheet", async ({
  applicant,
}) => {
  const { data, error, response } = await applicant.GET(
    "/api/atc/applications/review-sheet",
  );

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toEqual({
    id: "atc-application-review",
    name: "ATC Application Review Sheet",
    fields: expect.any(Array),
  });
});
