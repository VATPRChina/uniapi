import NoEventImage from "@/assets/no-event-image.svg";
import { useApi } from "@/client";
import { CreateAirspace } from "@/components/airspace-create";
import { DeleteAirspace } from "@/components/airspace-delete";
import { DateTime } from "@/components/datetime";
import { CreateEvent } from "@/components/event-create";
import { DeleteEvent } from "@/components/event-delete";
import { EventDetail } from "@/components/event-detail";
import { SlotBookButton } from "@/components/slot-button-book";
import { SlotReleaseButton } from "@/components/slot-button-release";
import { CreateSlot } from "@/components/slot-create";
import { DeleteSlot } from "@/components/slot-delete";
import { SlotDetail } from "@/components/slot-detail";
import { ExportSlot } from "@/components/slot-export";
import { ImportSlot } from "@/components/slot-import";
import { useUser } from "@/services/auth";
import { ActionIcon, Alert, Card, Group, Image, LoadingOverlay, Pill, Stack, Table, Text, Title } from "@mantine/core";
import { createFileRoute } from "@tanstack/react-router";

const EventComponent = () => {
  const { event_id } = Route.useParams();
  const { data: event, isLoading } = useApi("/api/events/{eid}", { path: { eid: event_id } });
  const { data: slots, isLoading: isLoadingSlots } = useApi("/api/events/{eid}/slots", { path: { eid: event_id } });
  const { data: airspaces, isLoading: isLoadingAirspaces } = useApi("/api/events/{eid}/airspaces", {
    path: { eid: event_id },
  });
  const user = useUser();

  const rows = slots?.map((slot) => (
    <Table.Tr key={slot.id} bg={slot?.booking?.user_id === user.id ? "green.0" : undefined}>
      <Table.Td>{slot.airspace.name}</Table.Td>
      <Table.Td>
        <Stack gap="xs">
          <Text>
            <Pill mr="xs">CTOT</Pill>
            <DateTime>{slot.enter_at}</DateTime>
          </Text>
          {slot.leave_at && (
            <Text>
              <Pill mr="xs">TTA</Pill>
              <DateTime>{slot.leave_at}</DateTime>
            </Text>
          )}
        </Stack>
      </Table.Td>
      <Table.Td>
        <Group>
          <SlotDetail eventId={event_id} slotId={slot.id} />
          <SlotBookButton eventId={event_id} slotId={slot.id} slot={slot} />
          <SlotReleaseButton eventId={event_id} slotId={slot.id} slot={slot} />
          <DeleteSlot eventId={event_id} slotId={slot.id} />
        </Group>
      </Table.Td>
    </Table.Tr>
  ));

  return (
    <Stack>
      <LoadingOverlay
        visible={isLoading || isLoadingAirspaces || isLoadingSlots}
        overlayProps={{ radius: "sm", blur: 2 }}
      />
      <Image src={event?.image_url ?? NoEventImage} alt={event?.title} radius="md" />
      <Group gap="xs">
        <Title order={1}>{event?.title}</Title>
        <ActionIcon.Group>
          <CreateEvent eventId={event_id} />
          <DeleteEvent eventId={event_id} />
        </ActionIcon.Group>
      </Group>
      <EventDetail eventId={event_id} />
      <Title order={2}>
        Slots
        <CreateSlot ml={4} eventId={event_id} />
        <ImportSlot eventId={event_id} />
        <ExportSlot eventId={event_id} />
      </Title>
      {slots?.length === 0 && <Alert title="No available slot now." />}
      {(slots?.length ?? 0) > 0 && (
        <Table highlightOnHover>
          <Table.Thead>
            <Table.Tr>
              <Table.Th>Area</Table.Th>
              <Table.Th>Time</Table.Th>
              <Table.Th></Table.Th>
            </Table.Tr>
          </Table.Thead>
          <Table.Tbody>{rows}</Table.Tbody>
        </Table>
      )}
      <Title order={2}>
        Areas
        <CreateAirspace ml={4} eventId={event_id} />
      </Title>
      {airspaces?.length === 0 && <Alert title="No available airspace now." />}
      <Group align="stretch">
        {airspaces?.map((airspace) => (
          <Card shadow="sm" padding="lg" radius="md" withBorder key={airspace.id}>
            <Stack>
              <Group>
                <Text>{airspace.name}</Text>
                <DeleteAirspace eventId={event_id} airspaceId={airspace.id} />
              </Group>
              {airspace.icao_codes.length > 0 && (
                <Group>
                  {airspace.icao_codes.map((icao) => (
                    <Pill key={icao}>{icao}</Pill>
                  ))}
                </Group>
              )}
            </Stack>
          </Card>
        ))}
      </Group>
    </Stack>
  );
};

export const Route = createFileRoute("/events/$event_id")({
  component: EventComponent,
});
