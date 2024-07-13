import { formatPath, useApi, useApiDelete, useApiPut } from "@/client";
import { CreateAirspace } from "@/components/create-airspace";
import { CreateEvent } from "@/components/create-event";
import { CreateSlot } from "@/components/create-slot";
import { DeleteAirspace } from "@/components/delete-airspace";
import { DeleteEvent } from "@/components/delete-event";
import { DeleteSlot } from "@/components/delete-slot";
import { useUser } from "@/services/auth";
import {
  ActionIcon,
  Alert,
  Button,
  Card,
  Group,
  Image,
  LoadingOverlay,
  Stack,
  Table,
  Text,
  Title,
  Tooltip,
} from "@mantine/core";
import { useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { format, formatRelative } from "date-fns";
import { formatInTimeZone } from "date-fns-tz";

const EventBookingButtons = ({
  eventId,
  slotId,
  canBook,
  disableBook,
  canUnbook,
}: {
  eventId: string;
  slotId: string;
  canBook: boolean;
  disableBook: boolean;
  canUnbook: boolean;
}) => {
  const queryClient = useQueryClient();
  const postMutate = () =>
    queryClient.invalidateQueries({ queryKey: formatPath("/api/events/{eid}/slots", { eid: eventId }) });
  const { mutate: book } = useApiPut(
    "/api/events/{eid}/slots/{sid}/booking",
    {
      path: { eid: eventId, sid: slotId },
    },
    postMutate,
  );
  const { mutate: unbook } = useApiDelete(
    "/api/events/{eid}/slots/{sid}/booking",
    {
      path: { eid: eventId, sid: slotId },
    },
    postMutate,
  );

  return (
    <Group>
      {canBook && (
        <Button variant="subtle" onClick={() => book({})} disabled={disableBook}>
          Book
        </Button>
      )}
      {canUnbook && (
        <Button variant="subtle" onClick={() => unbook({})}>
          Unbook
        </Button>
      )}
      <DeleteSlot eventId={eventId} slotId={slotId} />
    </Group>
  );
};

const EVENT_BOOKING_LIMIT = 1;

const EventComponent = () => {
  const { event_id } = Route.useParams();
  const { data: event, isLoading } = useApi("/api/events/{eid}", { path: { eid: event_id } });
  const { data: slots, isLoading: isLoadingSlots } = useApi("/api/events/{eid}/slots", { path: { eid: event_id } });
  const { data: airspaces, isLoading: isLoadingAirspaces } = useApi("/api/events/{eid}/airspaces", {
    path: { eid: event_id },
  });
  const user = useUser();

  const rows = slots?.map((element, id) => (
    <Table.Tr key={id}>
      <Table.Td>{element.airspace.name}</Table.Td>
      <Table.Td>
        <Tooltip label={format(element.enter_at, "yyyy-MM-dd HH:mm zzzz")} position="top-start">
          <Text>{formatInTimeZone(element.enter_at, "UTC", "yyyy-MM-dd HH:mm")}Z</Text>
        </Tooltip>
      </Table.Td>
      <Table.Td>
        <EventBookingButtons
          eventId={event_id}
          slotId={element.id}
          canBook={!element.booking}
          canUnbook={element.booking?.user_id == user?.id}
          disableBook={
            slots.filter((slot) => slot.booking?.user_id == user?.id).length >= EVENT_BOOKING_LIMIT &&
            element.booking?.user_id != user?.id
          }
        />
      </Table.Td>
    </Table.Tr>
  ));

  return (
    <Stack>
      <LoadingOverlay
        visible={isLoading || isLoadingAirspaces || isLoadingSlots}
        overlayProps={{ radius: "sm", blur: 2 }}
      />
      <Image src="https://cdn.sa.net/2024/07/06/OSoUsbluV69nhCw.png" alt={event?.title} radius="md" />
      <Group gap="xs">
        <Title order={1}>{event?.title}</Title>
        <ActionIcon.Group>
          <CreateEvent eventId={event_id} />
          <DeleteEvent eventId={event_id} />
        </ActionIcon.Group>
      </Group>
      {event && (
        <>
          <Text>
            Start Time:
            <Tooltip label={format(event.start_at, "yyyy-MM-dd HH:mm zzzz")} position="top-start">
              <span> {formatInTimeZone(event.start_at, "UTC", "yyyy-MM-dd HH:mm")}Z </span>
            </Tooltip>
            ({formatRelative(event.start_at, Date.now())})
          </Text>
          <Text>
            End Time:
            <Tooltip label={format(event.end_at, "yyyy-MM-dd HH:mm zzzz")} position="top-start">
              <span> {formatInTimeZone(event.end_at, "UTC", "yyyy-MM-dd HH:mm")}Z </span>
            </Tooltip>
            ({formatRelative(event.end_at, Date.now())})
          </Text>
          <Text>
            Start Booking Time:
            <Tooltip label={format(event.start_booking_at, "yyyy-MM-dd HH:mm zzzz")} position="top-start">
              <span> {formatInTimeZone(event.start_booking_at, "UTC", "yyyy-MM-dd HH:mm")}Z </span>
            </Tooltip>
            ({formatRelative(event.start_booking_at, Date.now())})
          </Text>
          <Text>
            End Booking Time:
            <Tooltip label={format(event.end_booking_at, "yyyy-MM-dd HH:mm zzzz")} position="top-start">
              <span> {formatInTimeZone(event.end_booking_at, "UTC", "yyyy-MM-dd HH:mm")}Z </span>
            </Tooltip>
            ({formatRelative(event.end_booking_at, Date.now())})
          </Text>
        </>
      )}
      <Title order={2}>
        Slots
        <CreateSlot ml={4} eventId={event_id} />
      </Title>
      {slots?.length === 0 && <Alert title="No available slot now." />}
      {(slots?.length ?? 0) > 0 && (
        <Table highlightOnHover>
          <Table.Thead>
            <Table.Tr>
              <Table.Th>Airspace Name</Table.Th>
              <Table.Th>Enter at</Table.Th>
              <Table.Th></Table.Th>
            </Table.Tr>
          </Table.Thead>
          <Table.Tbody>{rows}</Table.Tbody>
        </Table>
      )}
      <Title order={2}>
        Airspaces
        <CreateAirspace ml={4} eventId={event_id} />
      </Title>
      {airspaces?.length === 0 && <Alert title="No available airspace now." />}
      <Group>
        {airspaces?.map((airspace) => (
          <Card shadow="sm" padding="lg" radius="md" withBorder key={airspace.id}>
            <Group>
              <Text>{airspace.name}</Text>
              <DeleteAirspace eventId={event_id} airspaceId={airspace.id} />
            </Group>
          </Card>
        ))}
      </Group>
    </Stack>
  );
};

export const Route = createFileRoute("/events/$event_id")({
  component: EventComponent,
});
