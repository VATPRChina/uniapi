import createClient from "openapi-fetch";
import type { paths } from "./schema.js";

export const createApiClient = (baseUrl: string) =>
  createClient<paths>({
    baseUrl,
  });
