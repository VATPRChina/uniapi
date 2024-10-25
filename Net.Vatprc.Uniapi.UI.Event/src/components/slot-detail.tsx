import { DateTime } from "./datetime";
import { Markdown } from "./markdown";
import { SlotBookButton } from "./slot-button-book";
import { SlotReleaseButton } from "./slot-button-release";
import { useApi } from "@/client";
import { useUser } from "@/services/auth";
import { Alert, Button, Group, Modal, Pill, Stack, Text, Timeline, Title } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { IconCircleCheck, IconCircleDashed, IconCircleMinus } from "@tabler/icons-react";
import { addMinutes } from "date-fns";

const TIMES = [
  { name: "Request for Clearance Delivery", color: "red", span: 3, offset: 0, startMin: -30, endMin: -15 },
  { name: "Request for Pushback", color: "yellow", span: 4, offset: 2, startMin: -20, endMin: 0 },
  { name: "Expect Takeoff", color: "green", span: 3, offset: 5, startMin: -5, endMin: 10 },
] as const;

export const SlotDetail = ({ eventId, slotId }: { eventId: string; slotId: string }) => {
  const user = useUser();
  const [opened, { toggle, close }] = useDisclosure(false);
  const { data: slot } = useApi("/api/events/{eid}/slots/{sid}", {
    path: { eid: eventId, sid: slotId },
    enabled: opened,
  });
  const { data: event } = useApi("/api/events/{eid}", {
    path: { eid: eventId },
    enabled: opened,
  });

  return (
    <>
      <Button variant="subtle" aria-label="Details" onClick={toggle}>
        Details
      </Button>
      <Modal opened={opened} onClose={close} title="Slot Information" size="xl">
        <Stack>
          {slot?.booking?.user_id === user.id && (
            <Alert variant="light" color="green" title="Slot is booked by you." icon={<IconCircleCheck />} />
          )}
          {slot?.booking && slot?.booking?.user_id !== user.id && (
            <Alert variant="light" color="red" title="Slot is booked by someone else." icon={<IconCircleMinus />} />
          )}
          {!slot?.booking?.user_id && (
            <Alert variant="light" color="yellow" title="Slot is not booked." icon={<IconCircleDashed />} />
          )}
          {slot && (
            <Group>
              <SlotBookButton eventId={eventId} slotId={slotId} slot={slot} />
              <SlotReleaseButton eventId={eventId} slotId={slotId} slot={slot} />
            </Group>
          )}
          <Text>
            <Text component="span" fw={700} mr="xs">
              Area:
            </Text>
            {slot?.airspace?.name}
          </Text>
          {slot?.callsign && (
            <Text>
              <Text component="span" fw={700} mr="xs">
                Callsign:
              </Text>
              {slot?.callsign}
            </Text>
          )}
          {slot?.aircraft_type_icao && (
            <Text>
              <Text component="span" fw={700} mr="xs">
                Aircraft Type:
              </Text>
              {slot?.aircraft_type_icao}
            </Text>
          )}
          <Group gap={0}>
            <Text component="span" fw={700} mr="xs">
              Times:
            </Text>
            <Stack>
              <Group>
                <Pill>CTOT</Pill>
                <DateTime>{slot?.enter_at}</DateTime>
              </Group>
              {slot?.leave_at && (
                <Group>
                  <Pill>ELDT</Pill>
                  <DateTime>{slot?.leave_at}</DateTime>
                </Group>
              )}
            </Stack>
          </Group>
          <Title order={3}>Time detail</Title>
          <Timeline>
            {slot?.enter_at &&
              TIMES.map((time, i) => (
                <Timeline.Item key={i}>
                  <Group>
                    <Text fw={700}>{time.name}</Text>
                    <DateTime noDistance noDate position="bottom">
                      {addMinutes(slot?.enter_at, time.startMin)}
                    </DateTime>
                    -
                    <DateTime noDistance noDate position="bottom">
                      {addMinutes(slot?.enter_at, time.endMin)}
                    </DateTime>
                  </Group>
                </Timeline.Item>
              ))}
            {slot?.leave_at && (
              <Timeline.Item key={TIMES.length}>
                <Group>
                  <Text fw={700}>Expect arrival</Text>
                  <DateTime noDistance noDate position="bottom">
                    {slot.leave_at}
                  </DateTime>
                </Group>
              </Timeline.Item>
            )}
          </Timeline>
          <Title order={3}>Event briefing</Title>
          {!event?.description && <Text>No briefing available.</Text>}
          <Markdown>{event?.description}</Markdown>
          <Title order={3}>Area briefing</Title>
          {!slot?.airspace.description && <Text>No briefing available.</Text>}
          <Markdown>{slot?.airspace.description}</Markdown>
        </Stack>
      </Modal>
    </>
  );
};
