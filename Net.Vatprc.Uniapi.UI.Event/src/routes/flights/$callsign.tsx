import { useApi } from "@/client";
import { FlightWarnings } from "@/components/flight/warnings";
import { Alert, Card, Group, LoadingOverlay, Stack, Text, Title } from "@mantine/core";
import { createFileRoute } from "@tanstack/react-router";

const Flight = () => {
  const { callsign } = Route.useParams();
  const { isLoading, error, data: flight } = useApi("/api/flights/by-callsign/{callsign}", { path: { callsign } });

  return (
    <Card shadow="sm" withBorder>
      <LoadingOverlay visible={isLoading} />
      {error?.message && <Alert title={error?.message} color="red" />}
      {!error && (
        <Stack>
          <Title order={2}>{callsign}</Title>
          <Title order={3}>
            {flight?.departure} - {flight?.arrival}
          </Title>
          <Group c="gray" gap="xs">
            <Text component="span" ff="monospace">
              {flight?.aircraft}
            </Text>
            <Text fw={300} component="span">
              Equipment
            </Text>
            <Text component="span" ff="monospace">
              {flight?.equipment}
            </Text>
            <Text fw={300} component="span">
              Navigation Performance
            </Text>
            <Text component="span" ff="monospace">
              {flight?.navigation_performance}
            </Text>
            <Text fw={300} component="span">
              Transpoder
            </Text>
            <Text component="span" ff="monospace">
              {flight?.transponder}
            </Text>
          </Group>
          <Group c="gray" gap="xs">
            <Text fw={300} component="span">
              Route
            </Text>
            <Text component="span" ff="monospace">
              {flight?.__simplified_route}
            </Text>
          </Group>
          <FlightWarnings callsign={callsign} />
        </Stack>
      )}
    </Card>
  );
};

export const Route = createFileRoute("/flights/$callsign")({
  component: Flight,
});
