import { startBackend } from "./lib/backend-server.js";
import { startVatsimServer } from "./lib/vatsim-server.js";

export default async function setup(context: {
  provide: (key: string, value: unknown) => void;
}) {
  const vatsim = await startVatsimServer();
  try {
    const backend = await startBackend({ APP_UTILS__VATSIM_DATA_URL: vatsim.url });
    context.provide("backendBaseUrl", backend.baseUrl);
    process.env.E2E_BACKEND_BASE_URL = backend.baseUrl;

    return async () => {
      try {
        await backend.stop();
      } finally {
        await vatsim.stop();
      }
    };
  } catch (error) {
    await vatsim.stop();
    throw error;
  }
}
