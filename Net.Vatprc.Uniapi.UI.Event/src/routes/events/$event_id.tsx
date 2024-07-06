import { useApi, useApiDelete, useApiPut } from "@/client";
import { CreateAirspace } from "@/components/create-airspace";
import { CreateSlot } from "@/components/create-slot";
import { useUser } from "@/services/auth";
import { Button, Card, Group, Image, Stack, Table, Text, Title } from "@mantine/core";
import { useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { format } from "date-fns";

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
  const postMutate = () => queryClient.invalidateQueries({ queryKey: ["api", "events", eventId, "slots"] });
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
    </Group>
  );
};

const EVENT_BOOKING_LIMIT = 1;

const EventComponent = () => {
  const { event_id } = Route.useParams();
  const { data: event } = useApi("/api/events/{eid}", { path: { eid: event_id } });
  const { data: slots } = useApi("/api/events/{eid}/slots", { path: { eid: event_id } });
  const { data: airspaces } = useApi("/api/events/{eid}/airspaces", { path: { eid: event_id } });
  const user = useUser();

  const rows = slots?.map((element, id) => (
    <Table.Tr key={id}>
      <Table.Td>{element.airspace.name}</Table.Td>
      <Table.Td>{format(element.enter_at, "yyyy-MM-dd HH:mm zzzz")}</Table.Td>
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
      <Image src="https://cdn.sa.net/2024/07/06/OSoUsbluV69nhCw.png" alt={event?.title} radius="md" />
      <Title order={1}>{event?.title}</Title>
      <Title order={2}>
        Slots
        <CreateSlot ml={4} eventId={event_id} />
      </Title>
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
      <Title order={2}>
        Airspaces
        <CreateAirspace ml={4} eventId={event_id} />
      </Title>
      <Group>
        {airspaces?.map((airspace) => (
          <Card shadow="sm" padding="lg" radius="md" withBorder key={airspace.id}>
            <Text>{airspace.name}</Text>
          </Card>
        ))}
      </Group>
    </Stack>
  );
};

export const Route = createFileRoute("/events/$event_id")({
  component: EventComponent,
});
