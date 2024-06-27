import logo from "../assets/standard.svg";
import { Link, Outlet, createRootRoute } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/router-devtools";
import React from "react";

export const Route = createRootRoute({
  component: () => (
    <div className="my-4 container flex flex-col gap-2">
      <div className="p-2 flex gap-2 items-center">
        <img src={logo} alt="logo" className="h-8" />
        <Link to="/" className="[&.active]:font-bold">
          Home
        </Link>
        <Link to="/about" className="[&.active]:font-bold">
          About
        </Link>
      </div>
      <hr />
      <Outlet />
      <TanStackRouterDevtools />
    </div>
  ),
});
