import { expect, test as baseTest } from "vitest";
import { getClient } from "../../../../lib/backend.js";

const test = baseTest
  .extend("mentor", async () => getClient(["controller-training-mentor"]))
  .extend("trainee", async () => getClient([]))
  .extend("training", async ({ mentor, trainee }) => {
    const trainerSession = await mentor.GET("/api/session");
    const traineeSession = await trainee.GET("/api/session");
    const result = await mentor.POST("/api/atc/trainings", {
      body: {
        name: "Self reflection test",
        trainer_id: trainerSession.data!.user!.id,
        trainee_id: traineeSession.data!.user!.id,
        start_at: "2031-06-01T10:00:00Z",
        end_at: "2031-06-01T11:00:00Z",
      },
    });
    expect(result.response.status).toBe(200);
    return result.data!;
  });
const body = (answer: string) => ({
  request_answers: [{ id: "reflection", answer }],
});

test("trainee saves before training and edits after training without changing mentor feedback", async ({
  trainee,
  mentor,
  training,
}) => {
  const params = { path: { id: training.id } };
  const sheet = await trainee.GET(
    "/api/atc/trainings/{id}/self-reflection-sheet",
    { params: { path: { id: training.id } } },
  );
  expect(sheet.data?.fields).toEqual([
    expect.objectContaining({ id: "reflection", kind: "long-text" }),
  ]);
  expect(training.self_reflection_sheet_filing).toBeNull();
  const first = await trainee.PUT("/api/atc/trainings/{id}/self-reflection", {
    params,
    body: body("初次反思\n需要改进协调"),
  });
  expect(first.response.status).toBe(200);
  expect(first.data?.self_reflection_sheet_filing?.[0].answer).toBe(
    "初次反思\n需要改进协调",
  );
  const ended = await mentor.PUT("/api/atc/trainings/{id}", {
    params,
    body: {
      ...training,
      start_at: "2020-06-01T10:00:00Z",
      end_at: "2020-06-01T11:00:00Z",
    },
  });
  expect(ended.response.status).toBe(200);
  const second = await trainee.PUT("/api/atc/trainings/{id}/self-reflection", {
    params,
    body: body("修改后的反思"),
  });
  expect(second.response.status).toBe(200);
  expect(second.data?.self_reflection_sheet_filing_id).toBe(
    first.data?.self_reflection_sheet_filing_id,
  );
  const loaded = await trainee.GET("/api/atc/trainings/{id}", { params });
  expect(loaded.data?.self_reflection_sheet_filing?.[0].answer).toBe(
    "修改后的反思",
  );
  expect(loaded.data?.record_sheet_filing).toBeNull();
});

test("reflection access is limited to the trainee, assigned trainer and training director assistant", async ({
  trainee,
  mentor,
  training,
}) => {
  const params = { path: { id: training.id } };
  const admin = await getClient(["controller-training-director-assistant"]);
  const first = await admin.PUT("/api/atc/trainings/{id}/self-reflection", {
    params,
    body: body("Admin draft"),
  });
  expect(first.response.status).toBe(200);
  const saved = await trainee.PUT("/api/atc/trainings/{id}/self-reflection", {
    params,
    body: body("Private reflection"),
  });
  expect(saved.response.status).toBe(200);
  expect(saved.data?.self_reflection_sheet_filing_id).toBe(
    first.data?.self_reflection_sheet_filing_id,
  );
  for (const viewer of [trainee, mentor, admin]) {
    expect(
      (
        await viewer.GET("/api/atc/trainings/{id}/self-reflection-sheet", {
          params,
        })
      ).response.status,
    ).toBe(200);
    const read = await viewer.GET("/api/atc/trainings/{id}", { params });
    expect(read.data?.self_reflection_sheet_filing?.[0].answer).toBe(
      "Private reflection",
    );
  }
  const otherMentor = await getClient(["controller-training-mentor"]);
  const stranger = await getClient([]);
  const staff = await getClient(["staff"]);
  for (const denied of [otherMentor, stranger, staff]) {
    expect(
      (
        await denied.GET("/api/atc/trainings/{id}/self-reflection-sheet", {
          params,
        })
      ).response.status,
    ).toBe(403);
  }
  for (const denied of [mentor, otherMentor, stranger, staff]) {
    expect(
      (
        await denied.PUT("/api/atc/trainings/{id}/self-reflection", {
          params,
          body: body("Denied"),
        })
      ).response.status,
    ).toBe(403);
  }
  const hidden = await otherMentor.GET("/api/atc/trainings/{id}", { params });
  expect(hidden.response.status).toBe(200);
  expect(hidden.data?.self_reflection_sheet_filing).toBeNull();
  expect(hidden.data?.self_reflection_sheet_filing_id).toBeNull();
  const active = await otherMentor.GET("/api/atc/trainings/active");
  expect(
    active.data?.find((row) => row.id === training.id)
      ?.self_reflection_sheet_filing,
  ).toBeNull();
  const history = await otherMentor.GET("/api/atc/trainings/by-user/{userId}", {
    params: { path: { userId: training.trainee_id } },
  });
  expect(
    history.data?.find((row) => row.id === training.id)
      ?.self_reflection_sheet_filing,
  ).toBeNull();
  const updated = await otherMentor.PUT("/api/atc/trainings/{id}", {
    params,
    body: training,
  });
  expect(updated.response.status).toBe(200);
  expect(updated.data?.self_reflection_sheet_filing).toBeNull();
  await mentor.GET("/api/atc/trainings/record-sheet");
  const recorded = await otherMentor.PUT("/api/atc/trainings/{id}/record", {
    params,
    body: { request_answers: [] },
  });
  expect(recorded.response.status).toBe(200);
  expect(recorded.data?.self_reflection_sheet_filing).toBeNull();
  const finished = await otherMentor.GET("/api/atc/trainings/finished");
  expect(
    finished.data?.find((row) => row.id === training.id)
      ?.self_reflection_sheet_filing,
  ).toBeNull();
  const anonymous = await getClient();
  expect(
    (
      await anonymous.GET("/api/atc/trainings/{id}/self-reflection-sheet", {
        params,
      })
    ).response.status,
  ).toBe(401);
  expect(
    (
      await anonymous.PUT("/api/atc/trainings/{id}/self-reflection", {
        params,
        body: body("Denied"),
      })
    ).response.status,
  ).toBe(401);
  const edited = await admin.PUT("/api/atc/trainings/{id}/self-reflection", {
    params,
    body: body("Admin edit"),
  });
  expect(edited.response.status).toBe(200);
  expect(edited.data?.self_reflection_sheet_filing_id).toBe(
    first.data?.self_reflection_sheet_filing_id,
  );
  expect(
    (await trainee.GET("/api/atc/trainings/{id}", { params })).data
      ?.self_reflection_sheet_filing?.[0].answer,
  ).toBe("Admin edit");
});

test("concurrent initial saves share a filing; invalid fields do not overwrite answers", async ({
  trainee,
  training,
}) => {
  const params = { path: { id: training.id } };
  const results = await Promise.all(
    ["First", "Second"].map((answer) =>
      trainee.PUT("/api/atc/trainings/{id}/self-reflection", {
        params,
        body: body(answer),
      }),
    ),
  );
  results.forEach((result) => expect(result.response.status).toBe(200));
  expect(results[0].data?.self_reflection_sheet_filing_id).toBe(
    results[1].data?.self_reflection_sheet_filing_id,
  );
  const before = await trainee.GET("/api/atc/trainings/{id}", { params });
  const invalid = await trainee.PUT("/api/atc/trainings/{id}/self-reflection", {
    params,
    body: { request_answers: [{ id: "unknown", answer: "Bad field" }] },
  });
  expect(invalid.response.ok).toBe(false);
  const after = await trainee.GET("/api/atc/trainings/{id}", { params });
  expect(after.data?.self_reflection_sheet_filing).toEqual(
    before.data?.self_reflection_sheet_filing,
  );
});

test("uses configured fields and excludes deleted fields", async ({
  trainee,
  training,
}) => {
  const staff = await getClient(["staff"]);
  const params = { path: { sheetId: "training-self-reflection" } };
  const original = await staff.GET("/api/sheets/{sheetId}", { params });
  expect(original.response.status).toBe(200);
  const custom = {
    ...original.data!.fields[0],
    id: "next-steps",
    name_zh: "改进计划",
    name_en: "Next steps",
  };
  try {
    const configured = await staff.PUT("/api/sheets/{sheetId}", {
      params,
      body: { name: original.data!.name, fields: [custom] },
    });
    expect(configured.response.status).toBe(200);
    const sheet = await trainee.GET(
      "/api/atc/trainings/{id}/self-reflection-sheet",
      { params: { path: { id: training.id } } },
    );
    expect(sheet.data?.fields.map((field) => field.id)).toEqual(["next-steps"]);
    const saved = await trainee.PUT("/api/atc/trainings/{id}/self-reflection", {
      params: { path: { id: training.id } },
      body: {
        request_answers: [
          { id: "next-steps", answer: "Practice coordination" },
        ],
      },
    });
    expect(saved.response.status).toBe(200);
    expect(saved.data?.self_reflection_sheet_filing?.[0]).toMatchObject({
      field: { id: "next-steps" },
      answer: "Practice coordination",
    });
  } finally {
    const restored = await staff.PUT("/api/sheets/{sheetId}", {
      params,
      body: {
        name: original.data!.name,
        fields: original.data!.fields.filter((field) => !field.is_deleted),
      },
    });
    expect(restored.response.status).toBe(200);
  }
});
