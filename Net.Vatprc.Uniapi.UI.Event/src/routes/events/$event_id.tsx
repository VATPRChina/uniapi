import client from "@/client";
import { useUser } from "@/services/auth";
import { promiseWithToast } from "@/utils";
import { Button, Group, Image, Stack, Table, Title } from "@mantine/core";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { format } from "date-fns";

const EventComponent = () => {
  const { event_id } = Route.useParams();
  const queryClient = useQueryClient();
  const { data: event } = useQuery({
    queryKey: ["/api/events/{eid}", event_id],
    queryFn: () => client.GET("/api/events/{eid}", { params: { path: { eid: event_id } } }),
  });
  const { data: slots } = useQuery({
    queryKey: ["/api/events/{eid}/slots", event_id],
    queryFn: () => client.GET("/api/events/{eid}/slots", { params: { path: { eid: event_id } } }),
  });
  const user = useUser();

  const rows = slots?.data?.map((element, id) => (
    <Table.Tr key={id}>
      <Table.Td>{element.airspace.name}</Table.Td>
      <Table.Td>{format(element.enter_at, "yyyy-MM-dd HH:mm zzzz")}</Table.Td>
      <Table.Td>
        <Group>
          {!element.booking && (
            <Button
              variant="subtle"
              onClick={() => {
                promiseWithToast(
                  client
                    .PUT("/api/events/{eid}/slots/{sid}/booking", {
                      params: { path: { eid: event_id, sid: element.id } },
                    })
                    .then(() => queryClient.invalidateQueries({ queryKey: ["/api/events/{eid}/slots", event_id] })),
                );
              }}
            >
              Book
            </Button>
          )}
          {element.booking?.user_id == user?.id && (
            <Button
              variant="subtle"
              onClick={() => {
                promiseWithToast(
                  client
                    .DELETE("/api/events/{eid}/slots/{sid}/booking", {
                      params: { path: { eid: event_id, sid: element.id } },
                    })
                    .then(() => queryClient.invalidateQueries({ queryKey: ["/api/events/{eid}/slots", event_id] })),
                );
              }}
            >
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
        alt={event?.data?.title}
        radius="md"
      />
      <Title order={1}>{event?.data?.title}</Title>
      <Title order={2}>Slots</Title>
      <Table>
        <Table.Thead>
          <Table.Tr>
            <Table.Th>Airspace Name</Table.Th>
            <Table.Th>Enter at</Table.Th>
            <Table.Th>Book</Table.Th>
          </Table.Tr>
        </Table.Thead>
        <Table.Tbody>{rows}</Table.Tbody>
      </Table>
    </Stack>
  );
};

export const Route = createFileRoute("/events/$event_id")({
  component: EventComponent,
});
