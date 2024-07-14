import { paths } from "@/api";
import { invalidatePath, useApi, useApiDelete } from "@/client";
import { useUser } from "@/services/auth";
import { Button, Tooltip } from "@mantine/core";
import { isFuture, isPast } from "date-fns";

export const SlotReleaseButton = ({
  eventId,
  slotId,
  slot,
}: {
  eventId: string;
  slotId: string;
  slot: paths["/api/events/{eid}/slots/{sid}"]["get"]["responses"]["200"]["content"]["application/json"];
}) => {
  const { data: event } = useApi("/api/events/{eid}", { path: { eid: eventId } });
  const { mutate: release } = useApiDelete(
    "/api/events/{eid}/slots/{sid}/booking",
    {
      path: { eid: eventId, sid: slotId },
    },
    () => invalidatePath("/api/events/{eid}/slots", { eid: eventId }),
  );
  const user = useUser();

  let disableMessage = "";
  if (event) {
    if (!isPast(event.start_booking_at) || !isFuture(event.end_booking_at)) {
      disableMessage += "Event is not in booking period.";
    }
    if (slot.booking && slot.booking.user_id !== user?.id && !user?.roles.includes("ec")) {
      disableMessage += "Slot is booked by someone else.";
    }
  }

  // slot cannot be released if not booked
  if (!slot.booking) return null;

  const button = (
    <Button
      variant="subtle"
      onClick={() => release({})}
      disabled={!!disableMessage}
      color={user?.roles.includes("ec") ? "red" : "yellow"}
    >
      Release
    </Button>
  );

  return disableMessage ? <Tooltip label={disableMessage}>{button}</Tooltip> : button;
};
