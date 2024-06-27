import { paths } from "./api";
import { authMiddleware } from "./services/auth";
import createClient from "openapi-fetch";

export const client = createClient<paths>({ baseUrl: "/" });
client.use(authMiddleware);

export default client;
