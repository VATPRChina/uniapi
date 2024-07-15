import { paths } from "@/api";
import { invalidatePath, useApi, useApiPut } from "@/client";
import { useUser } from "@/services/auth";
import { Button, Tooltip } from "@mantine/core";
import { isFuture, isPast } from "date-fns";

const EVENT_BOOKING_LIMIT = 1;

export const SlotBookButton = ({
  eventId,
  slotId,
  slot,
}: {
  eventId: string;
  slotId: string;
  slot: paths["/api/events/{eid}/slots/{sid}"]["get"]["responses"]["200"]["content"]["application/json"];
}) => {
  const { data: event } = useApi("/api/events/{eid}", { path: { eid: eventId } });
  const { mutate: book } = useApiPut(
    "/api/events/{eid}/slots/{sid}/booking",
    {
      path: { eid: eventId, sid: slotId },
    },
    () => invalidatePath("/api/events/{eid}/slots", { eid: eventId }),
  );
  const { data: slots } = useApi("/api/events/{eid}/slots", { path: { eid: eventId } });
  const user = useUser();

  let disableMessage = "";
  if (event) {
    if (!isPast(event.start_booking_at) || !isFuture(event.end_booking_at)) {
      disableMessage += "Event is not in booking period.";
    }
    if (slot.booking && slot.booking.user_id !== user.id) {
      disableMessage += "Slot is booked by someone else.";
    }
    if ((slots?.filter((slot) => slot.booking?.user_id === user.id).length ?? 0) >= EVENT_BOOKING_LIMIT) {
      disableMessage += "Cannot book twice on the same event.";
    }
  }

  // slot cannot be booked again
  if (slot.booking && slot.booking.user_id === user.id) return null;

  const button = (
    <Button variant="subtle" onClick={() => book({})} disabled={!!disableMessage} color="green">
      Book
    </Button>
  );

  return disableMessage ? <Tooltip label={disableMessage}>{button}</Tooltip> : button;
};
