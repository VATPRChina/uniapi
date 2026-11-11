import { createServer } from "node:http";

const timestamp = "2026-11-11T00:00:00Z";
const pilot = (callsign: string, departure: string, arrival: string) => ({
  cid: 1234567,
  name: "E2E Pilot",
  callsign,
  altitude: 0,
  last_updated: timestamp,
  flight_plan: {
    flight_rules: "I",
    departure,
    arrival,
    aircraft: "B738/M-SDE2E3FGHIRWY/LB1",
    altitude: "FL330",
    route: "DCT",
  },
});

export async function startVatsimServer() {
  const payload = JSON.stringify({
    general: { update_timestamp: timestamp },
    pilots: [
      pilot("E2E001", "ZBAA", "ZSPD"),
      pilot("E2E002", "ZSPD", "ZBAA"),
    ],
    prefiles: [pilot("E2E003", "ZBAA", "ZSPD")],
    controllers: [],
  });
  const server = createServer((request, response) => {
    if (request.url !== "/vatsim-data.json") {
      response.writeHead(404).end();
      return;
    }
    response.writeHead(200, { "Content-Type": "application/json" });
    response.end(payload);
  });
  await new Promise<void>((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolve);
  });
  const address = server.address();
  if (address === null || typeof address === "string") {
    throw new Error("VATSIM fixture server did not bind a TCP port");
  }
  return {
    url: `http://127.0.0.1:${address.port}/vatsim-data.json`,
    stop: () => new Promise<void>((resolve, reject) => {
      server.close((error) => error ? reject(error) : resolve());
      server.closeIdleConnections();
    }),
  };
}
