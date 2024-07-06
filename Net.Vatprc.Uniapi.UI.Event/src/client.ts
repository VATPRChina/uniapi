/* eslint-disable @typescript-eslint/ban-types */
import { components, paths } from "./api";
import { authMiddleware } from "./services/auth";
import { errorToast } from "./utils";
import { QueryClient, UseMutationResult, UseQueryResult, useMutation, useQuery } from "@tanstack/react-query";
import createClient, { Middleware, defaultPathSerializer } from "openapi-fetch";
import { Any, Object } from "ts-toolbelt";

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
      const body = (await response.clone().json()) as components["schemas"]["ErrorProdResponse"];
      const err = new ApiError(body.message, response.status, body.error_code);
      throw err;
    }
  },
};

export const client = createClient<paths>({ baseUrl: "/" });
client.use(authMiddleware);
client.use(throwMiddleware);

export default client;
export const queryClient = new QueryClient();

type Equals<A1, A2> = (<A>() => A extends A2 ? 1 : 0) extends <A>() => A extends A1 ? 1 : 0 ? 1 : 0;

type MethodOn<Path, Method extends string> = Path extends { [M in Method]: unknown } ? Path[Method] : never;
type ParameterOf<Operation> = Operation extends { parameters: infer P } ? P : never;
type ResponsesOf<Operation> = Operation extends { responses: infer P } ? P : never;
type SuccessResponseOf<Responses> = Responses extends { [200]: infer P } ? P : never;
type ContentOf<Response> = Response extends { content: Record<string, infer P> } ? P : unknown;
type RequestBodyOf<Operation> = Operation extends { requestBody?: infer P } ? P : unknown;
// type OptionalParametersOf<Parameters> = Parameters extends {
//   query?: infer Q;
//   header?: infer H;
//   path?: infer P;
//   cookie?: infer C;
// }
//   ? { query?: Partial<Q>; header?: Partial<H>; path?: Partial<P>; cookie?: Partial<C> }
//   : never;
interface OptionalParametersOf<Parameters> {
  query?: Partial<QueryOf<Parameters>>;
  header?: Partial<HeaderOf<Parameters>>;
  path?: Partial<PathOf<Parameters>>;
  cookie?: Partial<CookieOf<Parameters>>;
}
type QueryOf<Parameters> = Parameters extends { query: infer V } ? V : unknown;
type HeaderOf<Parameters> = Parameters extends { header: infer V } ? V : unknown;
type PathOf<Parameters> = Parameters extends { path: infer V } ? V : unknown;
type CookieOf<Parameters> = Parameters extends { cookie: infer V } ? V : unknown;
type Subtract<T, V> = Pick<T, Exclude<keyof T, keyof V>>;
type OptionalParameters<Parameters, Key extends string> = Parameters extends { [K in Key]?: infer T }
  ? { [K in Key]?: Partial<T> }
  : never;

type GetParameterOf<Parameters, Key extends string> = Parameters extends { [K in Key]?: infer T extends object }
  ? T
  : never;
type RestParameters<Parameters, Key extends string, Existing extends OptionalParameters<Parameters, Key>> = {
  [K in Key]: Object.Exclude<GetParameterOf<Parameters, Key>, GetParameterOf<Existing, Key>>;
};
type OmitKeyIfEmpty<T, K extends string> = T extends { [Key in K]: infer V }
  ? Any.Equals<keyof V, never> extends 1
    ? {}
    : { [Key in K]: V }
  : {};
type RestParameterOf<
  Parameters,
  Existing extends OptionalParameters<Parameters, "query"> &
    OptionalParameters<Parameters, "header"> &
    OptionalParameters<Parameters, "path"> &
    OptionalParameters<Parameters, "cookie">,
> = unknown & // eslint-disable-line @typescript-eslint/no-redundant-type-constituents
  OmitKeyIfEmpty<RestParameters<Parameters, "query", Existing>, "query"> &
  OmitKeyIfEmpty<RestParameters<Parameters, "header", Existing>, "header"> &
  OmitKeyIfEmpty<RestParameters<Parameters, "path", Existing>, "path"> &
  OmitKeyIfEmpty<RestParameters<Parameters, "cookie", Existing>, "cookie">;

export const createHooks = <Paths>() => {
  const useApi = <Path extends keyof Paths & string, Method extends string = "get">(
    url: Path,
    params: ParameterOf<MethodOn<Paths[Path], Method>>,
  ): UseQueryResult<ContentOf<SuccessResponseOf<ResponsesOf<MethodOn<Paths[Path], Method>>>>, Error> => {
    // @ts-expect-error params.path cannot be inferred
    // otherwise we need to write a long runtime check
    // (typeof params === "object" && !!params && "path" in params &&
    //   typeof params.path === "object" && !!params.path
    // which is always true for having other type assertions
    // eslint-disable-next-line @typescript-eslint/no-unsafe-argument
    const queryKey = defaultPathSerializer(url, params?.path ?? {})
      .split("/")
      .filter((s) => !!s);
    return useQuery({
      queryKey,
      // @ts-expect-error - URL cannot be inferred here
      queryFn: () => client.GET(url, { params }).then((data) => data.data),
    });
  };

  const createApiMutate =
    <Method extends string>(clientMethod: keyof typeof client) =>
    <
      Path extends keyof Paths & string,
      Params extends OptionalParametersOf<ParameterOf<MethodOn<Paths[Path], Method>>>,
    >(
      url: Path,
      params: Params,
      onSuccess?: () => unknown,
    ): UseMutationResult<
      ContentOf<SuccessResponseOf<ResponsesOf<MethodOn<Paths[Path], Method>>>>,
      Error,
      ContentOf<RequestBodyOf<MethodOn<Paths[Path], Method>>> &
        RestParameterOf<OptionalParametersOf<ParameterOf<MethodOn<Paths[Path], Method>>>, Params>
    > => {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-argument
      const mutationKey = defaultPathSerializer(url, params.path ?? {})
        .split("/")
        .filter((s) => !!s);
      return useMutation({
        mutationKey,
        // @ts-expect-error - URL cannot be inferred here
        // eslint-disable-next-line
        mutationFn: (body) => client[clientMethod](url, { params, body }).then((data) => data.data),
        async onSuccess() {
          onSuccess?.();
          await queryClient.invalidateQueries({ queryKey: mutationKey });
        },
        onError(err) {
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
