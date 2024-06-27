import "@/index.css";
import { routeTree } from "@/routeTree.gen";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider, createRouter } from "@tanstack/react-router";
import React from "react";
import { createRoot } from "react-dom/client";

const queryClient = new QueryClient();

const router = createRouter({ routeTree });

const container = document.getElementById("root");
// eslint-disable-next-line @typescript-eslint/no-non-null-assertion
const root = createRoot(container!);
root.render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  </React.StrictMode>,
);
