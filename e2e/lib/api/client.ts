import createClient from "openapi-fetch";
import type { paths } from "./schema.js";

export const createApiClient = (baseUrl: string) => {
  const client = createClient<paths>({
    baseUrl,
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
