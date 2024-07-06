import { useApi } from "@/client";
import { Alert, Anchor, Card, Image, Stack, Text, Tooltip } from "@mantine/core";
import { Link, createLazyFileRoute } from "@tanstack/react-router";
import { format, formatRelative } from "date-fns";
import { formatInTimeZone } from "date-fns-tz";

const Index = () => {
  const { error, data: events } = useApi("/api/events", {});

  return (
    <>
      {error?.message && <Alert title={error?.message} />}

      <Stack>
        {events?.map((event) => (
          <Card key={event.id} shadow="sm" padding="lg" withBorder>
            <Card.Section>
              <Image src="https://community.vatprc.net/uploads/default/optimized/2X/3/35599eef688f188dc6325654461f2b4353576346_2_1380x776.jpeg" />
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
