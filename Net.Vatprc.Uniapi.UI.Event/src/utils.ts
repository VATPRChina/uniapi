import { paths } from "./api";
import client from "./client";
import { notifications } from "@mantine/notifications";
import { useQuery } from "@tanstack/react-query";
import { type ClassValue, clsx } from "clsx";
import { FetchOptions, FetchResponse, defaultPathSerializer } from "openapi-fetch";
import type { FilterKeys, HasRequiredKeys, HttpMethod, MediaType, PathsWithMethod } from "openapi-typescript-helpers";
import { twMerge } from "tailwind-merge";

type PromiseOrFunction = Promise<unknown> | (() => Promise<unknown>);

export const promiseWithLog = (promise: PromiseOrFunction, final?: () => unknown) => {
  // eslint-disable-next-line no-console
  (typeof promise === "function" ? promise() : promise).catch((err) => console.error(err)).finally(final);
};

export const promiseWithToast = (promise: PromiseOrFunction, final?: () => unknown) => {
  (typeof promise === "function" ? promise() : promise)
    .catch((err) => {
      // eslint-disable-next-line no-console
      console.error(err);
      notifications.show({
        title: "An error occurred.",
        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access
        message: err.message ?? err?.error?.message ?? (err as Error).message ?? "Unknown error",
        color: "red",
      });
    })
    .finally(final);
};

export const wrapPromiseWithToast = (promise: PromiseOrFunction) => () => promiseWithToast(promise);

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export type MaybeOptionalInit<Params extends Partial<Record<HttpMethod, object>>, Location extends keyof Params> =
  HasRequiredKeys<FetchOptions<FilterKeys<Params, Location>>> extends never
    ? FetchOptions<FilterKeys<Params, Location>> | undefined
    : FetchOptions<FilterKeys<Params, Location>>;

export const useClientQuery = <
  // Paths extends Record<string, Record<HttpMethod, unknown>>,
  // Method extends HttpMethod,
  Media extends MediaType,
  Path extends PathsWithMethod<paths, "get">,
  Init extends MaybeOptionalInit<paths[Path], "get">,
>(
  url: Path,
  init: Init,
) => {
  /* eslint-disable */
  const queryKey = defaultPathSerializer(url as any, {
    ...((init?.params as any)?.path ?? {}),
    ...((init as any)?.path ?? {}),
  })
    .split("/")
    .filter((s) => !!s);
  return useQuery<FetchResponse<paths[Path]["get"], Init, Media>["data"]>({
    queryKey,
    queryFn: () => client.GET(url as any, init as any).then((d) => d.data),
  });
  /* eslint-enable */
};
