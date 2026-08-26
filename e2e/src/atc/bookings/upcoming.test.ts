import { expect } from "vitest";
import { getClient } from "../../../lib/backend.js";
import { createBooking, defaultBooking, test } from "./fixtures.js";

test("GET /api/atc/bookings/upcoming publicly lists active future bookings", async ({
  controller,
  otherController,
}) => {
  const upcoming = await createBooking(controller);
  const past = await createBooking(otherController, {
    ...defaultBooking,
    callsign: "ZSSS_GND",
    start_at: "2020-08-26T10:00:00Z",
    end_at: "2020-08-26T12:00:00Z",
  });
  const cancelled = await createBooking(otherController, {
    ...defaultBooking,
    callsign: "ZGGG_APP",
  });
  const cancellation = await otherController.client.DELETE(
    "/api/atc/bookings/{id}",
    { params: { path: { id: cancelled.id } } },
  );
  expect(cancellation.response.status).toBe(200);

  const { data, error, response } = await (await getClient()).GET(
    "/api/atc/bookings/upcoming",
  );

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        id: upcoming.id,
        user: expect.objectContaining({ id: controller.id }),
      }),
    ]),
  );
  expect(data).not.toEqual(
    expect.arrayContaining([
      expect.objectContaining({ id: past.id }),
      expect.objectContaining({ id: cancelled.id }),
    ]),
  );
});
