import { expect } from "vitest";
import { createBooking, defaultBooking, test } from "../fixtures.js";

test("GET /api/atc/bookings/mine/upcoming lists only the current controller's bookings", async ({
  controller,
  otherController,
}) => {
  const mine = await createBooking(controller);
  const theirs = await createBooking(otherController, {
    ...defaultBooking,
    callsign: "ZSPD_APP",
  });

  const { data, error, response } = await controller.client.GET(
    "/api/atc/bookings/mine/upcoming",
  );

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toEqual(
    expect.arrayContaining([expect.objectContaining({ id: mine.id })]),
  );
  expect(data).not.toEqual(
    expect.arrayContaining([expect.objectContaining({ id: theirs.id })]),
  );
});

test("GET /api/atc/bookings/mine/upcoming requires a controller", async ({
  user,
}) => {
  const { data, error, response } = await user.client.GET(
    "/api/atc/bookings/mine/upcoming",
  );

  expect(response.status).toBe(403);
  expect(data).toBeFalsy();
  expect(error).toBeTruthy();
});
