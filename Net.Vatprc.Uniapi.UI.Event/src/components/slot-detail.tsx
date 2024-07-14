import { DateTime } from "./datetime";
import { SlotBookButton } from "./slot-button-book";
import { SlotReleaseButton } from "./slot-button-release";
import { useApi } from "@/client";
import { useUser } from "@/services/auth";
import { Alert, Button, Grid, Group, Modal, Pill, Stack, Text, Title, useMantineTheme } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { IconCircleCheck, IconCircleDashed, IconCircleMinus } from "@tabler/icons-react";
import { addMinutes } from "date-fns";
import React from "react";

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
  const theme = useMantineTheme();

  return (
    <>
      <Button variant="subtle" aria-label="Details" onClick={toggle}>
        Details
      </Button>
      <Modal opened={opened} onClose={close} title="Slot Information" size="xl">
        <Stack>
          {slot?.booking?.user_id === user?.id && (
            <Alert variant="light" color="green" title="Slot is booked by you." icon={<IconCircleCheck />} />
          )}
          {slot?.booking && slot?.booking?.user_id !== user?.id && (
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
          <Group gap={0}>
            <Text component="span" fw={700} mr="xs">
              Enter at:
            </Text>
            <Pill mr="xs">CTOT</Pill>
            <DateTime>{slot?.enter_at}</DateTime>
          </Group>
          <Title order={3}>Time detail</Title>
          <Grid columns={8}>
            {slot?.enter_at &&
              TIMES.map((time, i) => (
                <React.Fragment key={i}>
                  <Grid.Col span={time.span} offset={time.offset} bg={theme.colors[time.color][1]}>
                    <Stack gap={4} align="center">
                      <Text component="span" fw={700}>
                        {time.name}
                      </Text>
                      <Text>
                        <DateTime noDistance noDate position="bottom">
                          {addMinutes(slot?.enter_at, time.startMin)}
                        </DateTime>
                        -
                        <DateTime noDistance noDate position="bottom">
                          {addMinutes(slot?.enter_at, time.endMin)}
                        </DateTime>
                      </Text>
                    </Stack>
                  </Grid.Col>
                  <Grid.Col span={8 - (time.offset + time.span)}></Grid.Col>
                </React.Fragment>
              ))}
          </Grid>
        </Stack>
      </Modal>
    </>
  );
};
