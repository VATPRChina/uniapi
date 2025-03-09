import { paths } from "./api";
import { errorToast } from "./utils";
import { QueryClient, UseMutationResult, UseQueryResult, useMutation, useQuery } from "@tanstack/react-query";
import createClient, { Middleware, defaultPathSerializer } from "openapi-fetch";

export class ApiError extends Error {
  constructor(
    message: string,
    public readonly status?: number,
    public readonly errorCode?: string,
  ) {
    super(message);
    this.name = "ApiError";
  }
}

const throwMiddleware: Middleware = {
  async onResponse({ response }) {
    if (!response.ok) {
      const body = (await response.clone().json()) as { message: string; error_code: string };
      const err = new ApiError(body.message, response.status, body.error_code);
      if (err.errorCode !== "INVALID_TOKEN") throw err;
    }
  },
};

export const client = createClient<paths>({ baseUrl: import.meta.env.VITE_API_ENDPOINT });
client.use(throwMiddleware);

export default client;
export const queryClient = new QueryClient();

type MethodOn<Path, Method extends string> = Path extends { [M in Method]: unknown } ? Path[Method] : never;
type ParameterOf<Operation> = Operation extends { parameters: infer P } ? P : never;
type ResponsesOf<Operation> = Operation extends { responses: infer P } ? P : never;
type SuccessResponseOf<Responses> = Responses extends { [200]: infer P } ? P : never;
type ContentOf<Response> = Response extends { content: Record<string, infer P> } ? P : unknown;
type RequestBodyOf<Operation> = Operation extends { requestBody?: infer P } ? P : unknown;
export const createHooks = <Paths>() => {
  const useApi = <Path extends keyof Paths & string, Method extends string = "get">(
    url: Path,
    params: ParameterOf<MethodOn<Paths[Path], Method>> & { enabled?: boolean },
  ): UseQueryResult<ContentOf<SuccessResponseOf<ResponsesOf<MethodOn<Paths[Path], Method>>>>, Error> => {
    // @ts-expect-error - Path cannot be inferred here
    // eslint-disable-next-line @typescript-eslint/no-unsafe-argument
    const queryKey = defaultPathSerializer(url, params?.path ?? {})
      .split("/")
      .filter((s) => !!s);
    return useQuery({
      queryKey,
      // @ts-expect-error - URL cannot be inferred here
      queryFn: () => client.GET(url, { params }).then((data) => data.data ?? null),
      enabled: params.enabled ?? true,
    });
  };

  const createApiMutate =
    <Method extends string>(clientMethod: keyof typeof client) =>
    <Path extends keyof Paths & string>(
      url: Path,
      params: ParameterOf<MethodOn<Paths[Path], Method>>,
      onSuccess?: () => unknown,
    ): UseMutationResult<
      ContentOf<SuccessResponseOf<ResponsesOf<MethodOn<Paths[Path], Method>>>>,
      Error,
      ContentOf<RequestBodyOf<MethodOn<Paths[Path], Method>>>
    > => {
      // @ts-expect-error - Path cannot be inferred here
      // otherwise we need to write a long runtime check
      // (typeof params === "object" && !!params && "path" in params &&
      //   typeof params.path === "object" && !!params.path
      // which is always true for having other type assertions
      // eslint-disable-next-line @typescript-eslint/no-unsafe-argument
      const mutationKey = defaultPathSerializer(url, params.path ?? {})
        .split("/")
        .filter((s) => !!s);
      return useMutation({
        mutationKey,
        // @ts-expect-error - URL cannot be inferred here
        // eslint-disable-next-line
        mutationFn: (body) => client[clientMethod](url, { params, body }).then((data) => data.data ?? null),
        async onSuccess() {
          onSuccess?.();
          await queryClient.invalidateQueries({ queryKey: mutationKey });
        },
        onError(err) {
          console.error(err); // eslint-disable-line no-console
          errorToast(err);
        },
      });
    };

  const useApiPost = createApiMutate<"post">("POST");
  const useApiPut = createApiMutate<"put">("PUT");
  const useApiDelete = createApiMutate<"delete">("DELETE");
  const useApiPatch = createApiMutate<"patch">("PATCH");

  return { useApi, useApiPost, useApiPut, useApiDelete, useApiPatch };
};
export const { useApi, useApiPost, useApiPut, useApiDelete, useApiPatch } = createHooks<paths>();

export const formatPath = <Path extends keyof paths>(
  url: Path,
  path: paths[Path] extends { get: { parameters: { path: infer G } } } ? G : never,
) => {
  return defaultPathSerializer(url, path ?? {})
    .split("/")
    .filter((s) => !!s);
};

export const invalidatePath = <Path extends keyof paths>(
  url: Path,
  path: paths[Path] extends { get: { parameters: { path: infer G } } } ? G : never,
) => {
  return queryClient.invalidateQueries({ queryKey: formatPath(url, path) });
};
