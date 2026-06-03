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

/**
 * Get an API client.
 *
 * If roles are provided, the client will be authenticated with a token that has the specified roles.
 *
 * If roles are not provided, the client will be unauthenticated.
 *
 * @param roles List of roles
 * @returns API client
 */
type GetClientOptions = {
  cid?: string;
  email?: string;
  fullName?: string;
  id?: string;
};

export const getClient = async (
  roles: string[] | null = null,
  options: GetClientOptions = {},
) => {
  const baseUrl = getBaseUrl();

  if (roles !== null) {
    return createApiClientWithRoles(baseUrl, {
      cid:
        options.cid ?? Math.floor(10000000 + Math.random() * 9000000).toString(),
      email: options.email,
      fullName: options.fullName,
      id: options.id,
      roles,
    });
  }

  return createApiClient(baseUrl);
};
