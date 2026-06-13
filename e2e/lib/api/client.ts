import createClient, { ClientOptions } from "openapi-fetch";
import type { paths } from "./schema.js";

const unsafeAssumeClientId = "01K7X723S04R1F5P2P8NJV5QT5";
const unsafeAssumeClientSecret =
  "urn:vatprc:client_secret:01K7X71YQQXWSDT2XK1ZNYMS2J";

type ApiClient = ReturnType<typeof createApiClient>;

type IssueUserTokenOptions = {
  cid: string;
  roles: string[];
  email?: string;
  fullName?: string;
  id?: string;
};

export const createApiClient = (
  baseUrl: string,
  options: Omit<ClientOptions, "baseUrl"> = {},
) => {
  const client = createClient<paths>({
    baseUrl,
    ...options,
  });
  client.use({
    onResponse: async ({ response }) => {
      console.log("Response:", {
        url: response.url,
        status: response.status,
        headers: Object.fromEntries(response.headers.entries()),
        body: await response.clone().text(),
      });
    },
  });
  return client;
};

export async function issueUserTokenWithRoles(
  client: ApiClient,
  options: IssueUserTokenOptions,
): Promise<string> {
  const { data: clientToken, error: clientTokenError } = await client.POST(
    "/auth/token",
    {
      body: {
        grant_type: "client_credentials",
        client_id: unsafeAssumeClientId,
        client_secret: unsafeAssumeClientSecret,
      },
      headers: {
        "content-type": "application/x-www-form-urlencoded",
      },
    },
  );

  if (clientTokenError || !clientToken?.access_token) {
    throw new Error("Failed to issue unsafe assume client token");
  }

  const { data, error } = await client.POST("/auth/__unsafe_assume_user", {
    body: {
      id: options.id,
      cid: options.cid,
      full_name: options.fullName,
      email: options.email,
      roles: options.roles,
    },
    headers: {
      authorization: `Bearer ${clientToken.access_token}`,
    },
  });

  if (error || !data?.access_token) {
    throw new Error(
      `Failed to issue user token with roles: ${JSON.stringify(error)}`,
    );
  }

  return data.access_token;
}

export const createApiClientWithRoles = async (
  baseUrl: string,
  options: IssueUserTokenOptions,
) => {
  const client = createApiClient(baseUrl);
  const token = await issueUserTokenWithRoles(client, options);
  const userClient = createApiClient(baseUrl, {
    headers: {
      Authorization: `Bearer ${token}`,
    },
  });
  return userClient;
};
