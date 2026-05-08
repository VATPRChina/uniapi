import { startBackend } from "./lib/backend-server.js";

export default async function setup(context: {
  provide: (key: string, value: unknown) => void;
}) {
  const backend = await startBackend();
  context.provide("backendBaseUrl", backend.baseUrl);
  process.env.E2E_BACKEND_BASE_URL = backend.baseUrl;

  return async () => {
    await backend.stop();
  };
}
