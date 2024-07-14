import NoEventImage from "@/assets/no-event-image.svg";
import { useApi } from "@/client";
import { DateTime } from "@/components/datetime";
import { Alert, Anchor, Card, Image, Stack, Text } from "@mantine/core";
import { Link, createLazyFileRoute } from "@tanstack/react-router";

const Index = () => {
  const { error, data: events, isLoading } = useApi("/api/events", {});

  return (
    <>
      <Alert
        title="VATPRC Events is under construction and is not available to public now. Contents are subject to changes."
        color="yellow"
      />
      {error?.message && <Alert title={error?.message} color="red" />}

      <Stack>
        {events?.length === 0 && !isLoading && <Alert title="No available event now." />}
        {events?.map((event) => (
          <Card key={event.id} shadow="sm" padding="lg" withBorder>
            <Card.Section>
              <Image src={event?.image_url ?? NoEventImage} />
            </Card.Section>
            <Anchor fw={500} my="sm" component={Link} to={"/events/" + event.id}>
              {event.title}
            </Anchor>
            <Text>
              Start Time: <DateTime>{event.start_at}</DateTime>
            </Text>
            <Text>
              End Time: <DateTime>{event.end_at}</DateTime>
            </Text>
            <Text>
              Start Booking Time: <DateTime>{event.start_booking_at}</DateTime>
            </Text>
            <Text>
              End Booking Time: <DateTime>{event.end_booking_at}</DateTime>
            </Text>
          </Card>
        ))}
      </Stack>
    </>
  );
};

export const Route = createLazyFileRoute("/")({
  component: Index,
});
