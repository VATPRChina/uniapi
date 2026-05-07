import { registerSharedWorker, type SharedWorker } from "ava/plugin";
import type { BackendWorkerMessage } from "./backend-worker.js";
import { createApiClient } from "./api/client.js";

const backend: SharedWorker.Plugin.Protocol<BackendWorkerMessage> =
  registerSharedWorker<BackendWorkerMessage>({
    filename: new URL("./backend-worker.js", import.meta.url),
    supportedProtocols: ["ava-4"],
    teardown: async () => {
      if (!backend.currentlyAvailable) {
        return;
      }

      const message = backend.publish({ type: "release" });
      for await (const reply of message.replies()) {
        if (reply.data.type === "released") {
          return;
        }
      }
    },
  });

export const getBackend = async () => {
  await backend.available;

  const message = backend.publish({ type: "getBaseUrl" });
  for await (const reply of message.replies()) {
    if (reply.data.type === "baseUrl") {
      return createApiClient(reply.data.baseUrl);
    }
  }

  throw new Error("Backend shared worker did not return a base URL");
};
