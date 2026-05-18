import { expect, test as baseTest } from "vitest";
import { components } from "../lib/api/schema.js";
import { getClient } from "../lib/backend.js";

const test = baseTest
  .extend("techDirector", async ({}) => {
    return await getClient(["staff"]);
  })
  .extend("user", async ({}) => {
    return await getClient([]);
  })
  .extend("anonymous", async ({}) => {
    return await getClient(null);
  });

const sheetBody = (
  suffix: string,
): components["schemas"]["SheetSaveRequest"] => ({
  name: `E2E Sheet ${suffix}`,
  fields: [
    {
      id: "callsign",
      sequence: 1,
      name_zh: "呼号",
      name_en: "Callsign",
      kind: "short-text",
      single_choice_options: [],
      description_zh: "请输入呼号",
      description_en: "Enter callsign",
    },
    {
      id: "remarks",
      sequence: 2,
      name_zh: "备注",
      name_en: "Remarks",
      kind: "long-text",
      single_choice_options: [],
    },
    {
      id: "rating",
      sequence: 3,
      name_zh: "等级",
      kind: "single-choice",
      single_choice_options: ["S2", "S3", "C1"],
    },
  ],
});

test("PUT /api/sheets/{sheetId} upserts a sheet", async ({ techDirector }) => {
  const suffix = Date.now().toString();
  const sheetId = `e2e-sheet-${suffix}`;
  const body = sheetBody(suffix);

  const { data, error, response } = await techDirector.PUT(
    "/api/sheets/{sheetId}",
    {
      params: {
        path: {
          sheetId,
        },
      },
      body,
    },
  );

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toEqual({
    id: sheetId,
    name: body.name,
    fields: body.fields.map((field) => ({
      ...field,
      sheet_id: sheetId,
      is_deleted: false,
      description_zh: field.description_zh ?? null,
      description_en: field.description_en ?? null,
      name_en: field.name_en ?? null,
    })),
  });
});

test("GET /api/sheets/{sheetId} returns a sheet", async ({
  techDirector,
  anonymous,
}) => {
  const suffix = Date.now().toString();
  const sheetId = `e2e-sheet-get-${suffix}`;
  const body = sheetBody(suffix);

  const created = await techDirector.PUT("/api/sheets/{sheetId}", {
    params: {
      path: {
        sheetId,
      },
    },
    body,
  });
  expect(created.response.status).toBe(200);

  const { data, error, response } = await anonymous.GET(
    "/api/sheets/{sheetId}",
    {
      params: {
        path: {
          sheetId,
        },
      },
    },
  );

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toEqual(created.data);
});

test("GET /api/sheets lists sheets with fields", async ({
  techDirector,
  anonymous,
}) => {
  const suffix = Date.now().toString();
  const sheetId = `e2e-sheet-list-${suffix}`;
  const body = sheetBody(suffix);

  const created = await techDirector.PUT("/api/sheets/{sheetId}", {
    params: {
      path: {
        sheetId,
      },
    },
    body,
  });
  expect(created.response.status).toBe(200);

  const { data, error, response } = await anonymous.GET("/api/sheets");

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toEqual(expect.arrayContaining([created.data]));
});

test("PUT /api/sheets/{sheetId} updates fields and marks omitted fields deleted", async ({
  techDirector,
}) => {
  const suffix = Date.now().toString();
  const sheetId = `e2e-sheet-update-${suffix}`;

  const created = await techDirector.PUT("/api/sheets/{sheetId}", {
    params: {
      path: {
        sheetId,
      },
    },
    body: sheetBody(suffix),
  });
  expect(created.response.status).toBe(200);

  const updateBody: components["schemas"]["SheetSaveRequest"] = {
    name: `E2E Sheet Updated ${suffix}`,
    fields: [
      {
        id: "callsign",
        sequence: 10,
        name_zh: "管制席位",
        name_en: "Position",
        kind: "short-text",
        single_choice_options: [],
      },
    ],
  };

  const { data, error, response } = await techDirector.PUT(
    "/api/sheets/{sheetId}",
    {
      params: {
        path: {
          sheetId,
        },
      },
      body: updateBody,
    },
  );

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data.name).toBe(updateBody.name);
  expect(data.fields).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        id: "callsign",
        sequence: 10,
        name_zh: "管制席位",
        is_deleted: false,
      }),
      expect.objectContaining({
        id: "remarks",
        is_deleted: true,
      }),
      expect.objectContaining({
        id: "rating",
        is_deleted: true,
      }),
    ]),
  );
});

test("PUT /api/sheets/{sheetId} rejects users without tech director permission", async ({
  user,
}) => {
  const { data, error, response } = await user.PUT("/api/sheets/{sheetId}", {
    params: {
      path: {
        sheetId: `e2e-sheet-forbidden-${Date.now()}`,
      },
    },
    body: sheetBody(Date.now().toString()),
  });

  expect(response.status).toBe(403);
  expect(data).toBeFalsy();
  expect(error).toEqual({
    detail: "only user with roles {Staff} can perform this action",
    status: 403,
    title: "only user with roles {Staff} can perform this action",
    type: "urn:vatprc-uniapi-error:forbidden",
  });
});

test("GET /api/sheets/{sheetId} returns 404 for unknown sheets", async ({
  anonymous,
}) => {
  const { data, error, response } = await anonymous.GET(
    "/api/sheets/{sheetId}",
    {
      params: {
        path: {
          sheetId: `e2e-sheet-missing-${Date.now()}`,
        },
      },
    },
  );

  expect(response.status).toBe(404);
  expect(data).toBeFalsy();
  expect(error).toEqual({
    detail: expect.stringContaining("not found"),
    status: 404,
    title: expect.stringContaining("not found"),
    type: "urn:vatprc-uniapi-error:not-found",
  });
});
