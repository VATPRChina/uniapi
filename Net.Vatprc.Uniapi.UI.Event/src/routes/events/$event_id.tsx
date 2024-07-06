import { useApi, useApiDelete, useApiPut } from "@/client";
import { CreateAirspace } from "@/components/create-airspace";
import { CreateSlot } from "@/components/create-slot";
import { useUser } from "@/services/auth";
import { Button, Card, Group, Image, Stack, Table, Text, Title } from "@mantine/core";
import { createFileRoute } from "@tanstack/react-router";
import { format } from "date-fns";

const EventComponent = () => {
  const { event_id } = Route.useParams();
  const { data: event } = useApi("/api/events/{eid}", { path: { eid: event_id } });
  const { data: slots } = useApi("/api/events/{eid}/slots", { path: { eid: event_id } });
  const { data: airspaces } = useApi("/api/events/{eid}/airspaces", { path: { eid: event_id } });
  const user = useUser();

  const { mutate: book } = useApiPut("/api/events/{eid}/slots/{sid}/booking", {
    path: { eid: event_id },
  });
  const { mutate: unbook } = useApiDelete("/api/events/{eid}/slots/{sid}/booking", {
    path: { eid: event_id },
  });

  const rows = slots?.map((element, id) => (
    <Table.Tr key={id}>
      <Table.Td>{element.airspace.name}</Table.Td>
      <Table.Td>{format(element.enter_at, "yyyy-MM-dd HH:mm zzzz")}</Table.Td>
      <Table.Td>
        <Group>
          {!element.booking && (
            <Button variant="subtle" onClick={() => book({ path: { sid: element.id } })}>
              Book
            </Button>
          )}
          {element.booking?.user_id == user?.id && (
            <Button variant="subtle" onClick={() => unbook({ path: { sid: element.id } })}>
              Unbook
            </Button>
          )}
        </Group>
      </Table.Td>
    </Table.Tr>
  ));

  return (
    <Stack>
      <Image
        src="https://community.vatprc.net/uploads/default/optimized/2X/3/35599eef688f188dc6325654461f2b4353576346_2_1380x776.jpeg"
        alt={event?.title}
        radius="md"
      />
      <Title order={1}>{event?.title}</Title>
      <Title order={2}>
        Slots
        <CreateSlot ml={4} eventId={event_id} />
      </Title>
      <Table>
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
