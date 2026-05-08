import { inject } from "vitest";
import { createApiClient } from "./api/client.js";

export const getBackend = async () => {
  const baseUrl =
    process.env.E2E_BACKEND_BASE_URL ??
    ((inject as (key: string) => unknown)("backendBaseUrl") as
      | string
      | undefined);

  if (baseUrl === undefined) {
    throw new Error("E2E_BACKEND_BASE_URL is not set");
  }

  return createApiClient(baseUrl);
};
