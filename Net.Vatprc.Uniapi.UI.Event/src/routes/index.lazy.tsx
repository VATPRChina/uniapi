import client from "@/client";
import { Alert, Card, Image, Stack, Text } from "@mantine/core";
import { useQuery } from "@tanstack/react-query";
import { createLazyFileRoute } from "@tanstack/react-router";
import { formatRelative } from "date-fns";

const Index = () => {
  const { error, data } = useQuery({
    queryKey: ["repoData"],
    queryFn: () => client.GET("/api/events"),
  });

  return (
    <>
      {error?.message && <Alert title={error?.message} />}

      <Stack>
        {data?.data?.map((event) => (
          <Card key={event.id} shadow="sm" padding="lg" withBorder>
            <Card.Section>
              <Image src="https://community.vatprc.net/uploads/default/optimized/2X/3/35599eef688f188dc6325654461f2b4353576346_2_1380x776.jpeg" />
            </Card.Section>
            <Text fw={500} my="sm">
              {event.title}
            </Text>
            <Text>Start Time: {formatRelative(event.start_at, Date.now())}</Text>
            <Text>End Time: {formatRelative(event.end_at, Date.now())}</Text>
          </Card>
        ))}
      </Stack>
    </>
  );
};

export const Route = createLazyFileRoute("/")({
  component: Index,
});
