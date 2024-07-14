import NoEventImage from "@/assets/no-event-image.svg";
import { useApi } from "@/client";
import { Alert, Anchor, Card, Image, Stack, Text, Tooltip } from "@mantine/core";
import { Link, createLazyFileRoute } from "@tanstack/react-router";
import { format, formatRelative } from "date-fns";
import { formatInTimeZone } from "date-fns-tz";

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
          </Card>
        ))}
      </Stack>
    </>
  );
};

export const Route = createLazyFileRoute("/")({
  component: Index,
});
