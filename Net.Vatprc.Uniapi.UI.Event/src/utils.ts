import { ErrorProdResponse, HttpResponse } from "./api";
import { createStandaloneToast } from "@chakra-ui/react";

export const { ToastContainer, toast } = createStandaloneToast();

type PromiseOrFunction = Promise<unknown> | (() => Promise<unknown>);

export const promiseWithLog = (promise: PromiseOrFunction, final?: () => unknown) => {
  (typeof promise === "function" ? promise() : promise).catch((err) => console.error(err)).finally(final);
};
export const promiseWithToast = (promise: PromiseOrFunction, final?: () => unknown) => {
  (typeof promise === "function" ? promise() : promise)
    .catch((err) => {
      console.error(err);
      toast({
        title: "An error occurred.",
        description:
          (err as ErrorProdResponse).message ??
          (err as HttpResponse<void, ErrorProdResponse>).error?.message ??
          (err as Error).message ??
          "Unknown error",
        status: "error",
        duration: 5000,
        isClosable: true,
      });
    })
    .finally(final);
};
