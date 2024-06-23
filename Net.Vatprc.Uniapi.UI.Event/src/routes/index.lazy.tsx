import { createLazyFileRoute } from "@tanstack/react-router";
import React from "react";

export const Route = createLazyFileRoute("/")({
  component: () => <div>Hello /!</div>,
});
