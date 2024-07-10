import { login } from "@/services/auth";
import { createFileRoute } from "@tanstack/react-router";
import { redirect } from "@tanstack/react-router";

interface AuthCallbackSearch {
  code: string;
}

export const Route = createFileRoute("/auth/callback")({
  validateSearch: (search: Record<string, unknown>): AuthCallbackSearch => {
    return {
      code: ("code" in search && typeof search["code"] === "string" ? search["code"] : undefined) ?? "",
    };
  },
  component: () => <div>Please wait while being logged in.</div>,
  beforeLoad: async (match) => {
    await login(match.search.code);
    redirect({ to: "/", throw: true });
  },
});
