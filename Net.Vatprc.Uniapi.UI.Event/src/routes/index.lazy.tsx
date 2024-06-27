import client from "@/client";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { useQuery } from "@tanstack/react-query";
import { createLazyFileRoute } from "@tanstack/react-router";
import React from "react";

const Index = () => {
  const { isPending, error, data } = useQuery({
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

      {data?.data?.map((event) => (
        <Card key={event.id} className="max-w-2xl">
          <CardHeader>
            <CardTitle>{event.title}</CardTitle>
          </CardHeader>
          <CardContent>
            <p>
              {event.start_at} {event.end_at}
            </p>
          </CardContent>
        </Card>
      ))}
    </>
  );
};

export const Route = createLazyFileRoute("/")({
  component: Index,
});
