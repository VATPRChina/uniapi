import { inject } from "vitest";
import { createApiClient, createApiClientWithRoles } from "./api/client.js";

export const getBaseUrl = (): string => {
  const baseUrl =
    process.env.E2E_BACKEND_BASE_URL ??
    ((inject as (key: string) => unknown)("backendBaseUrl") as
      | string
      | undefined);

  if (baseUrl === undefined) {
    throw new Error("E2E_BACKEND_BASE_URL is not set");
  }

  return baseUrl;
};

export const getClient = async (roles: string[] | null = null) => {
  const baseUrl = getBaseUrl();

  if (roles !== null) {
    return createApiClientWithRoles(baseUrl, {
      cid: Math.floor(10000000 + Math.random() * 9000000).toString(),
      roles,
    });
  }

  return createApiClient(baseUrl);
};
