import { serve } from "@hono/node-server";
import { Hono } from "hono";
import { logger } from "./log";
import { logger as honoLog } from "hono/logger";

const app = new Hono();

app.use(honoLog((msg, ...rest) => logger.info(msg, ...rest)));

app.get("/", (c) => {
  return c.text("Hello Hono!");
});

serve(
  {
    fetch: app.fetch,
    port: 3000,
  },
  (info) => {
    logger.info(`Server is running on http://localhost:${info.port}`);
  }
);
