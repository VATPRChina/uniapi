import { expect } from "vitest";
import { defaultBooking, test } from "./fixtures.js";

test("PUT /api/atc/bookings/{id} updates an owned free booking", async ({
  controller,
  booking,
}) => {
  const body = {
    callsign: " zbaa_app ",
    start_at: "2099-08-26T11:00:00Z",
    end_at: "2099-08-26T13:00:00Z",
    remarks: "  updated booking  ",
  };
  const { data, error, response } = await controller.client.PUT(
    "/api/atc/bookings/{id}",
    { params: { path: { id: booking.id } }, body },
  );

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toEqual(
    expect.objectContaining({
      id: booking.id,
      callsign: "ZBAA_APP",
      start_at: body.start_at,
      end_at: body.end_at,
      remarks: "updated booking",
      event_position: null,
      user: expect.objectContaining({ id: controller.id }),
    }),
  );
  expect(new Date(data.updated_at).getTime()).toBeGreaterThanOrEqual(
    new Date(booking.updated_at).getTime(),
  );
});

test("a controller cannot update or delete another controller's booking", async ({
  booking,
  otherController,
}) => {
  const update = await otherController.client.PUT("/api/atc/bookings/{id}", {
    params: { path: { id: booking.id } },
    body: { ...defaultBooking, callsign: "ZBAA_APP" },
  });
  const cancellation = await otherController.client.DELETE(
    "/api/atc/bookings/{id}",
    { params: { path: { id: booking.id } } },
  );

  expect(update.response.status).toBe(403);
  expect(update.error).toBeTruthy();
  expect(cancellation.response.status).toBe(403);
  expect(cancellation.error).toBeTruthy();
});

test("DELETE /api/atc/bookings/{id} soft-deletes an owned booking", async ({
  controller,
  booking,
}) => {
  const { data, error, response } = await controller.client.DELETE(
    "/api/atc/bookings/{id}",
    { params: { path: { id: booking.id } } },
  );

  expect(error).toBeFalsy();
  expect(response.status).toBe(200);
  expect(data).toEqual(
    expect.objectContaining({
      id: booking.id,
      deleted_at: expect.any(String),
      user: expect.objectContaining({ id: controller.id }),
    }),
  );

  const mine = await controller.client.GET(
    "/api/atc/bookings/mine/upcoming",
  );
  expect(mine.data).not.toEqual(
    expect.arrayContaining([expect.objectContaining({ id: booking.id })]),
  );
});

test("event-linked bookings reject direct changes and follow event position changes", async ({
  admin,
  controller,
  event,
  position,
  eventBooking,
}) => {
  expect(eventBooking.user_id).toBe(controller.id);

  const mine = await controller.client.GET(
    "/api/atc/bookings/mine/upcoming",
  );
  const linked = mine.data.find(
    (candidate) => candidate.event_position?.id === position.id,
  );
  expect(linked).toEqual(
    expect.objectContaining({
      callsign: position.callsign,
      remarks: position.remarks,
      event_position: expect.objectContaining({
        id: position.id,
        event: expect.objectContaining({ id: event.id }),
      }),
    }),
  );

  const directUpdate = await controller.client.PUT(
    "/api/atc/bookings/{id}",
    {
      params: { path: { id: linked.id } },
      body: { ...defaultBooking, callsign: "ZBAA_APP" },
    },
  );
  const directDelete = await controller.client.DELETE(
    "/api/atc/bookings/{id}",
    { params: { path: { id: linked.id } } },
  );
  expect(directUpdate.response.status).toBe(409);
  expect(directDelete.response.status).toBe(409);

  const updatedPosition = {
    callsign: "ZBAA_APP",
    start_at: "2099-09-26T11:00:00Z",
    end_at: "2099-09-26T12:45:00Z",
    remarks: "Updated event-linked booking.",
    position_kind_id: "APP",
    minimum_controller_state: "student" as const,
  };
  const update = await admin.client.PUT(
    "/api/events/{event_id}/controllers/{position_id}",
    {
      params: { path: { event_id: event.id, position_id: position.id } },
      body: updatedPosition,
    },
  );
  expect(update.response.status).toBe(200);

  const afterUpdate = await controller.client.GET(
    "/api/atc/bookings/mine/upcoming",
  );
  expect(afterUpdate.data).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        id: linked.id,
        callsign: updatedPosition.callsign,
        start_at: updatedPosition.start_at,
        end_at: updatedPosition.end_at,
        remarks: updatedPosition.remarks,
      }),
    ]),
  );

  const deletion = await admin.client.DELETE(
    "/api/events/{event_id}/controllers/{position_id}",
    { params: { path: { event_id: event.id, position_id: position.id } } },
  );
  expect(deletion.response.status).toBe(204);

  const afterDeletion = await controller.client.GET(
    "/api/atc/bookings/mine/upcoming",
  );
  expect(afterDeletion.data).not.toEqual(
    expect.arrayContaining([expect.objectContaining({ id: linked.id })]),
  );
});
