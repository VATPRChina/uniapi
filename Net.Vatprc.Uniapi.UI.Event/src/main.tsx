import { fetcher } from "./api";
import { routeTree } from "./routeTree.gen";
import "@/index.css";
import { RouterProvider, createRouter } from "@tanstack/react-router";
import React from "react";
import { createRoot } from "react-dom/client";
import { SWRConfig } from "swr";
import { defaultConfig } from "swr/_internal";

const router = createRouter({ routeTree });

const container = document.getElementById("root");
// eslint-disable-next-line @typescript-eslint/no-non-null-assertion
const root = createRoot(container!);
root.render(
  <React.StrictMode>
    <SWRConfig
      value={{
        fetcher,
        onErrorRetry(err, key, config, revalidate, revalidateOpts) {
          // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
          if (err.error_code === "INVALID_TOKEN") {
            return;
          }
          defaultConfig.onErrorRetry(err, key, config, revalidate, revalidateOpts);
        },
      }}
    >
      <RouterProvider router={router} />
    </SWRConfig>
  </React.StrictMode>,
);
