import { components, paths } from "./api";
import { authMiddleware } from "./services/auth";
import createClient, { Middleware } from "openapi-fetch";

export class ApiError extends Error {}

const throwMiddleware: Middleware = {
  async onResponse({ response }) {
    if (!response.ok) {
      const body = (await response.clone().json()) as components["schemas"]["ErrorProdResponse"];
      throw new ApiError(body.message);
    }
  },
};

export const client = createClient<paths>({ baseUrl: "/" });
client.use(authMiddleware);
client.use(throwMiddleware);

export default client;
