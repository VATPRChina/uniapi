import { useApi } from "@/client";
import { FlightWarnings } from "@/components/flight/warnings";
import { Alert, Anchor, Card, Stack } from "@mantine/core";
import { Link, createLazyFileRoute } from "@tanstack/react-router";

const Index = () => {
  const { error, data: flights, isLoading } = useApi("/api/flights/active", {});

  return (
    <>
      {error?.message && <Alert title={error?.message} color="red" />}
      <Stack>
        {flights?.length === 0 && !isLoading && <Alert title="No active flight now." />}
        {flights?.map((flight) => (
          <Card key={flight.id} shadow="sm" withBorder>
            <Anchor fw={500} my="sm" component={Link} to={"/flights/" + flight.callsign}>
              {flight.callsign}
            </Anchor>
            <FlightWarnings callsign={flight.callsign} />
          </Card>
        ))}
      </Stack>
    </>
  );
};

export const Route = createLazyFileRoute("/flights/")({
  component: Index,
});
