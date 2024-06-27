import client from "@/client";
import { CreateEvent } from "@/components/create-event";
import { DevLogin } from "@/components/dev-login";
import { Alert, AlertTitle } from "@/components/ui/alert";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
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
      {error?.message && (
        <Alert>
          <AlertTitle>{error?.message}</AlertTitle>
        </Alert>
      )}

      <div className="flex gap-x-2">
        <CreateEvent />
        <DevLogin />
      </div>

      {data?.data?.map((event) => (
        <Card key={event.id} className="max-w-2xl">
          <CardHeader>
            <CardTitle>{event.title}</CardTitle>
          </CardHeader>
          <CardContent>
            <p>Start Time: {formatRelative(event.start_at, Date.now())}</p>
            <p>End Time: {formatRelative(event.end_at, Date.now())}</p>
          </CardContent>
        </Card>
      ))}
    </>
  );
};

export const Route = createLazyFileRoute("/")({
  component: Index,
});
