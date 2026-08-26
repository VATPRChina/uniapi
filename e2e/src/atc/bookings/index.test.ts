import { expect } from "vitest";
import { defaultBooking, test } from "./fixtures.js";

test("PUT /api/atc/bookings creates and normalizes a free ATC booking", async ({
  controller,
}) => {
  const body = {
    ...defaultBooking,
    callsign: " zbaa_twr ",
    remarks: "  controller training  ",
  };
  const { data, error, response } = await controller.client.PUT(
    "/api/atc/bookings",
    { body },
  );

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toEqual(
    expect.objectContaining({
      id: expect.stringMatching(/^[0-9A-HJKMNP-TV-Z]{26}$/),
      callsign: "ZBAA_TWR",
      start_at: body.start_at,
      end_at: body.end_at,
      remarks: "controller training",
      event_position: null,
      created_at: expect.any(String),
      updated_at: expect.any(String),
      deleted_at: null,
      user: expect.objectContaining({
        id: controller.id,
        cid: expect.any(String),
        full_name: expect.any(String),
      }),
    }),
  );
});

test("PUT /api/atc/bookings requires a controller", async ({ user }) => {
  const { data, error, response } = await user.client.PUT(
    "/api/atc/bookings",
    { body: defaultBooking },
  );

  expect(response.status).toBe(403);
  expect(data).toBeFalsy();
  expect(error).toBeTruthy();
});

test("PUT /api/atc/bookings rejects an invalid time range", async ({
  controller,
}) => {
  const { data, error, response } = await controller.client.PUT(
    "/api/atc/bookings",
    {
      body: {
        ...defaultBooking,
        start_at: "2099-08-26T12:00:00Z",
        end_at: "2099-08-26T10:00:00Z",
      },
    },
  );

  expect(response.status).toBe(400);
  expect(data).toBeFalsy();
  expect(error).toEqual(
    expect.objectContaining({
      status: 400,
      type: "urn:vatprc-uniapi-error:bad-request",
    }),
  );
});
