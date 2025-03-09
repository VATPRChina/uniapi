import { components } from "@/api";
import { useApi } from "@/client";
import { Alert, Loader, Stack } from "@mantine/core";

const messages: Record<string, string> = {
  no_rvsm: "The aircraft does not specify RVSM capability.",
  no_rnav1: "The aircraft does not specify RNAV1 capability.",
  rnp_ar: "The aircraft specifies RNP AR capability with RF.",
  rnp_ar_without_rf: "The aircraft specifies RNP AR capability without RF.",
  no_transponder: "The aircraft does not specify transponder capability.",
  no_preferred_route: "There is no CAAC preferred route for the aircraft.",
  not_preferred_route: "The aircraft does not follow the CAAC preferred route.",
};

const descriptions: Record<
  string,
  (flight: components["schemas"]["FlightDto"], warning: components["schemas"]["WarningMessage"]) => React.ReactNode
> = {
  no_rvsm: (flight) => `Equipment code "${flight.equipment}" does not contain "W" for RVSM.`,
  no_rnav1: (flight) =>
    `Equipment code "${flight.equipment}" does not contain "R" for RNAV1, or navigation performance code "${flight.navigation_performance}" does not contain "D1" or "D2" for RNAV1.`,
  rnp_ar: () => "",
  rnp_ar_without_rf: () => "",
  no_transponder: () => "Transponder field is empty.",
  no_preferred_route: (flight) =>
    `Our database does not contain a preferred route for the flight ${flight.departure}-${flight.arrival}. Please follow controller instructions.`,
  not_preferred_route: (flight, message) =>
    `The route in flight plan "${flight.__simplified_route}" does not match the preferred route "${(
      message.parameter?.split(";") ?? []
    ).join('" or "')}".`,
};

export const FlightWarnings = ({ callsign }: { callsign: string }) => {
  const { data: flight } = useApi("/api/flights/by-callsign/{callsign}", { path: { callsign } });
  const {
    isLoading,
    error,
    data: warnings,
  } = useApi("/api/flights/by-callsign/{callsign}/warnings", { path: { callsign } });

  if (isLoading) return <Loader />;
  return (
    <>
      {error?.message && <Alert title={error?.message} color="red" />}
      <Stack>
        {(warnings?.length ?? 0) === 0 && <Alert title={"Flight looks good."} color="green"></Alert>}
        {warnings?.map((warning) => (
          <Alert
            key={warning.message_code}
            title={messages[warning.message_code] ?? warning.message_code}
            color={warning.message_code === "no_preferred_route" ? "green" : "yellow"}
          >
            {flight && descriptions[warning.message_code]?.(flight, warning)}
          </Alert>
        ))}
      </Stack>
    </>
  );
};
