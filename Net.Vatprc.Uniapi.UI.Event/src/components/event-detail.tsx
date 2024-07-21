import { DateTime } from "./datetime";
import { Markdown } from "./markdown";
import { useApi } from "@/client";
import { Group, Text } from "@mantine/core";
import { IconArrowRight } from "@tabler/icons-react";

export const EventDetail = ({ eventId }: { eventId: string }) => {
  const { data: event } = useApi("/api/events/{eid}", { path: { eid: eventId } });

  return (
    <>
      <Group gap="xs">
        <Text>Time:</Text>
        <DateTime>{event?.start_at}</DateTime>
        <IconArrowRight size={12} />
        <DateTime>{event?.end_at}</DateTime>
      </Group>
      <Group gap="xs">
        <Text>Booking:</Text>
        <DateTime>{event?.start_booking_at}</DateTime>
        <IconArrowRight size={12} />
        <DateTime>{event?.end_booking_at}</DateTime>
      </Group>
      <Markdown>{event?.description}</Markdown>
    </>
  );
};
