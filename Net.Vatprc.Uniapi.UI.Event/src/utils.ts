import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

type PromiseOrFunction = Promise<unknown> | (() => Promise<unknown>);

export const promiseWithLog = (promise: PromiseOrFunction, final?: () => unknown) => {
  // eslint-disable-next-line no-console
  (typeof promise === "function" ? promise() : promise).catch((err) => console.error(err)).finally(final);
};

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}
