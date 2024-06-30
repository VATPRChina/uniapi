import { notifications } from "@mantine/notifications";
import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export const errorToast = (err: Error) => {
  notifications.show({
    title: "An error occurred.",
    message: err.message,
    color: "red",
  });
};

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
