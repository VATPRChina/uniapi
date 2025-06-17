import { DateTime } from "./datetime";
import { paths } from "@/api";
import { invalidatePath, useApi, useApiDelete } from "@/client";
import { useUser } from "@/services/auth";
import { Button, Group, Popover, Stack, Text, Tooltip } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
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
  const [opened, { toggle, close }] = useDisclosure(false);

  const { data: event } = useApi("/api/events/{eid}", { path: { eid: eventId } });
  const { mutate: release, isPending } = useApiDelete(
    "/api/events/{eid}/slots/{sid}/booking",
    {
      path: { eid: eventId, sid: slotId },
    },
    () => (close(), invalidatePath("/api/events/{eid}/slots", { eid: eventId })),
  );
  const user = useUser();

  let disableMessage = "";
  if (event) {
    if (!isPast(event.start_booking_at) || !isFuture(event.end_booking_at)) {
      disableMessage += "Event is not in booking period.";
    }
    if (slot.booking && slot.booking.user_id !== user.id && !user.roles.includes("event_coordinator")) {
      disableMessage += "Slot is booked by someone else.";
    }
  }

  // slot cannot be released if not booked
  if (!slot.booking) return null;

  const button = (
    <Popover opened={opened} onClose={close} shadow="lg">
      <Popover.Target>
        <Button
          variant="subtle"
          onClick={toggle}
          disabled={!!disableMessage}
          color={user.roles.includes("event_coordinator") && user.id !== slot.booking?.user_id ? "red" : "yellow"}
        >
          Release
        </Button>
      </Popover.Target>
      <Popover.Dropdown>
        <Stack>
          <Text>
            Do you want to release the slot for {slot?.airspace?.name} at{" "}
            <DateTime noDate noDistance>
              {slot.enter_at}
            </DateTime>
            ?
          </Text>
          <Group>
            <Button variant="outline" onClick={toggle}>
              Cancel
            </Button>
            <Button color="red" onClick={() => release({})} disabled={!!disableMessage} loading={isPending}>
              Yes
            </Button>
          </Group>
        </Stack>
      </Popover.Dropdown>
    </Popover>
  );

  return disableMessage ? <Tooltip label={disableMessage}>{button}</Tooltip> : button;
};
