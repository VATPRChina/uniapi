/* eslint-disable */

/* tslint:disable */

/*
 * ---------------------------------------------------------------
 * ## THIS FILE WAS GENERATED VIA SWAGGER-TYPESCRIPT-API        ##
 * ##                                                           ##
 * ## AUTHOR: acacode                                           ##
 * ## SOURCE: https://github.com/acacode/swagger-typescript-api ##
 * ---------------------------------------------------------------
 */
import { apiSecurityWorker } from "./services/auth";
import useSWR, { MutatorOptions, SWRConfiguration, mutate } from "swr";
import useSWRMutation, { SWRMutationConfiguration } from "swr/mutation";

export interface CreateEventAirspaceDto {
  name: string;
}

export interface CreateEventDto {
  title: string;
  /** @format date-time */
  start_at: string;
  /** @format date-time */
  end_at: string;
}

export interface CreateEventSlotDto {
  airspace_id: string;
  /** @format date-time */
  enter_at: string;
}

export interface ErrorProdResponse {
  error_code: string;
  message: string;
  [key: string]: any;
}

export interface EventAirspaceDto {
  id: string;
  event_id: string;
  name: string;
  /** @format date-time */
  created_at: string;
  /** @format date-time */
  updated_at: string;
}

export interface EventBookingDto {
  id: string;
  user_id: string;
  /** @format date-time */
  created_at: string;
  /** @format date-time */
  updated_at: string;
}

export interface EventDto {
  id: string;
  /** @format date-time */
  created_at: string;
  /** @format date-time */
  updated_at: string;
  title: string;
  /** @format date-time */
  start_at: string;
  /** @format date-time */
  end_at: string;
}

export interface EventSlotDto {
  id: string;
  event_id: string;
  event_airspace_id: string;
  /** @format date-time */
  enter_at: string;
  /** @format date-time */
  created_at: string;
  /** @format date-time */
  updated_at: string;
  booking?: EventBookingDto;
}

export interface LoginResDto {
  access_token: string;
  /** @format int32 */
  expires_in: number;
  refresh_token: string;
  scope: string;
  token_type: string;
  issued_token_type: string;
}

export interface TokenDto {
  user: UserDto;
  /** @format date-time */
  issued_at: string;
  /** @format date-time */
  expires_at: string;
}

export interface UpdateEventAirspaceDto {
  name: string;
}

export interface UpdateEventDto {
  title: string;
  /** @format date-time */
  start_at: string;
  /** @format date-time */
  end_at: string;
}

export interface UpdateEventSlotDto {
  /** @format date-time */
  enter_at: string;
}

export interface UserDto {
  id: string;
  cid: string;
  full_name: string;
  /** @format date-time */
  created_at: string;
  /** @format date-time */
  updated_at: string;
  /** @uniqueItems true */
  roles: string[];
}

export type QueryParamsType = Record<string | number, any>;
export type ResponseFormat = keyof Omit<Body, "body" | "bodyUsed">;

export interface FullRequestParams extends Omit<RequestInit, "body"> {
  /** set parameter to `true` for call `securityWorker` for this request */
  secure?: boolean;
  /** request path */
  path: string;
  /** content type of request body */
  type?: ContentType;
  /** query params */
  query?: QueryParamsType;
  /** format of response (i.e. response.json() -> format: "json") */
  format?: ResponseFormat;
  /** request body */
  body?: unknown;
  /** base url */
  baseUrl?: string;
  /** request cancellation token */
  cancelToken?: CancelToken;
}

export type RequestParams = Omit<FullRequestParams, "body" | "method" | "query" | "path">;

export interface ApiConfig<SecurityDataType = unknown> {
  baseUrl?: string;
  baseApiParams?: Omit<RequestParams, "baseUrl" | "cancelToken" | "signal">;
  securityWorker?: (securityData: SecurityDataType | null) => Promise<RequestParams | void> | RequestParams | void;
  customFetch?: typeof fetch;
}

export interface HttpResponse<D extends unknown, E extends unknown = unknown> extends Response {
  data: D;
  error: E;
}

type CancelToken = Symbol | string | number;

export enum ContentType {
  Json = "application/json",
  FormData = "multipart/form-data",
  UrlEncoded = "application/x-www-form-urlencoded",
  Text = "text/plain",
}

export class HttpClient<SecurityDataType = unknown> {
  public baseUrl: string = "https://uniapi.vatprc.net";
  private securityData: SecurityDataType | null = null;
  private securityWorker?: ApiConfig<SecurityDataType>["securityWorker"];
  private abortControllers = new Map<CancelToken, AbortController>();
  private customFetch = (...fetchParams: Parameters<typeof fetch>) => fetch(...fetchParams);

  private baseApiParams: RequestParams = {
    credentials: "same-origin",
    headers: {},
    redirect: "follow",
    referrerPolicy: "no-referrer",
  };

  constructor(apiConfig: ApiConfig<SecurityDataType> = {}) {
    Object.assign(this, apiConfig);
  }

  public setSecurityData = (data: SecurityDataType | null) => {
    this.securityData = data;
  };

  protected encodeQueryParam(key: string, value: any) {
    const encodedKey = encodeURIComponent(key);
    return `${encodedKey}=${encodeURIComponent(typeof value === "number" ? value : `${value}`)}`;
  }

  protected addQueryParam(query: QueryParamsType, key: string) {
    return this.encodeQueryParam(key, query[key]);
  }

  protected addArrayQueryParam(query: QueryParamsType, key: string) {
    const value = query[key];
    return value.map((v: any) => this.encodeQueryParam(key, v)).join("&");
  }

  protected toQueryString(rawQuery?: QueryParamsType): string {
    const query = rawQuery || {};
    const keys = Object.keys(query).filter((key) => "undefined" !== typeof query[key]);
    return keys
      .map((key) => (Array.isArray(query[key]) ? this.addArrayQueryParam(query, key) : this.addQueryParam(query, key)))
      .join("&");
  }

  protected addQueryParams(rawQuery?: QueryParamsType): string {
    const queryString = this.toQueryString(rawQuery);
    return queryString ? `?${queryString}` : "";
  }

  private contentFormatters: Record<ContentType, (input: any) => any> = {
    [ContentType.Json]: (input: any) =>
      input !== null && (typeof input === "object" || typeof input === "string") ? JSON.stringify(input) : input,
    [ContentType.Text]: (input: any) => (input !== null && typeof input !== "string" ? JSON.stringify(input) : input),
    [ContentType.FormData]: (input: any) =>
      Object.keys(input || {}).reduce((formData, key) => {
        const property = input[key];
        formData.append(
          key,
          property instanceof Blob
            ? property
            : typeof property === "object" && property !== null
              ? JSON.stringify(property)
              : `${property}`,
        );
        return formData;
      }, new FormData()),
    [ContentType.UrlEncoded]: (input: any) => this.toQueryString(input),
  };

  protected mergeRequestParams(params1: RequestParams, params2?: RequestParams): RequestParams {
    return {
      ...this.baseApiParams,
      ...params1,
      ...(params2 || {}),
      headers: {
        ...(this.baseApiParams.headers || {}),
        ...(params1.headers || {}),
        ...((params2 && params2.headers) || {}),
      },
    };
  }

  protected createAbortSignal = (cancelToken: CancelToken): AbortSignal | undefined => {
    if (this.abortControllers.has(cancelToken)) {
      const abortController = this.abortControllers.get(cancelToken);
      if (abortController) {
        return abortController.signal;
      }
      return void 0;
    }

    const abortController = new AbortController();
    this.abortControllers.set(cancelToken, abortController);
    return abortController.signal;
  };

  public abortRequest = (cancelToken: CancelToken) => {
    const abortController = this.abortControllers.get(cancelToken);

    if (abortController) {
      abortController.abort();
      this.abortControllers.delete(cancelToken);
    }
  };

  public request = async <T = any, E = any>({
    body,
    secure,
    path,
    type,
    query,
    format,
    baseUrl,
    cancelToken,
    ...params
  }: FullRequestParams): Promise<HttpResponse<T, E>> => {
    const secureParams =
      ((typeof secure === "boolean" ? secure : this.baseApiParams.secure) &&
        this.securityWorker &&
        (await this.securityWorker(this.securityData))) ||
      {};
    const requestParams = this.mergeRequestParams(params, secureParams);
    const queryString = query && this.toQueryString(query);
    const payloadFormatter = this.contentFormatters[type || ContentType.Json];
    const responseFormat = format || requestParams.format;

    return this.customFetch(`${baseUrl || this.baseUrl || ""}${path}${queryString ? `?${queryString}` : ""}`, {
      ...requestParams,
      headers: {
        ...(requestParams.headers || {}),
        ...(type && type !== ContentType.FormData ? { "Content-Type": type } : {}),
      },
      signal: (cancelToken ? this.createAbortSignal(cancelToken) : requestParams.signal) || null,
      body: typeof body === "undefined" || body === null ? null : payloadFormatter(body),
    }).then(async (response) => {
      const r = response as HttpResponse<T, E>;
      r.data = null as unknown as T;
      r.error = null as unknown as E;

      const data = !responseFormat
        ? r
        : await response[responseFormat]()
            .then((data) => {
              if (r.ok) {
                r.data = data;
              } else {
                r.error = data;
              }
              return r;
            })
            .catch((e) => {
              r.error = e;
              return r;
            });

      if (cancelToken) {
        this.abortControllers.delete(cancelToken);
      }

      if (!response.ok) throw data;
      return data;
    });
  };
}

export class ApiError extends Error {}

/**
 * @title VATPRC UniAPI
 * @version v1
 * @baseUrl https://uniapi.vatprc.net
 *
 * # Error Handling
 *
 * VATPRC UniAPI returns normalized error responses. The response body is a JSON object with the following fields:
 *
 * | Field           | Type     | Description     |
 * | --------------- | -------- | --------------- |
 * | `error_code`    | `string` | Error code.     |
 * | `message`       | `string` | Error message.  |
 * | `connection_id` | `string` | Connection ID.     |
 * | `request_id`    | `string` | Request ID. |
 *
 * It may contain additional fields depending on the error code.
 *
 * For details, see the examples on each API endpoint. The additional fields is denoted like `{field}` in the
 * error message example.
 */
export class Api<SecurityDataType extends unknown> extends HttpClient<SecurityDataType> {
  api = {
    /**
     * No description
     *
     * @tags Event
     * @name EventList
     * @request GET:/api/events
     * @secure
     */
    eventList: (params: RequestParams = {}) =>
      this.request<EventDto[], ErrorProdResponse>({
        path: `/api/events`,
        method: "GET",
        secure: true,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags Event
     * @name EventList
     * @request GET:/api/events
     * @secure
     */
    useEventList: (options?: SWRConfiguration, doFetch: boolean = true) =>
      useSWR<EventDto[], ErrorProdResponse>(doFetch ? `/api/events` : null, options),

    /**
     * No description
     *
     * @tags Event
     * @name EventList
     * @request GET:/api/events
     * @secure
     */
    mutateEventList: (data?: EventDto[] | Promise<EventDto[]>, options?: MutatorOptions) =>
      mutate<EventDto[]>(`/api/events`, data, options),

    /**
     * No description
     *
     * @tags Event
     * @name EventCreate
     * @request POST:/api/events
     * @secure
     */
    eventCreate: (data: CreateEventDto, params: RequestParams = {}) =>
      this.request<EventDto, ErrorProdResponse>({
        path: `/api/events`,
        method: "POST",
        body: data,
        secure: true,
        type: ContentType.Json,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags Event
     * @name EventCreate
     * @request POST:/api/events
     * @secure
     */
    useEventCreate: (options?: SWRMutationConfiguration<EventDto, ErrorProdResponse, string, CreateEventDto>) =>
      useSWRMutation(
        `/api/events`,
        (_url: string, { arg }: { arg: CreateEventDto }) =>
          this.api.eventCreate(arg).then(
            (x) => x.data,
            (x) => Promise.reject(x.error),
          ),
        options,
      ),

    /**
     * No description
     *
     * @tags Event
     * @name EventGet
     * @request GET:/api/events/{eid}
     * @secure
     */
    eventGet: (eid: string, params: RequestParams = {}) =>
      this.request<EventDto, ErrorProdResponse>({
        path: `/api/events/${eid}`,
        method: "GET",
        secure: true,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags Event
     * @name EventGet
     * @request GET:/api/events/{eid}
     * @secure
     */
    useEventGet: (eid: string, options?: SWRConfiguration, doFetch: boolean = true) =>
      useSWR<EventDto, ErrorProdResponse>(doFetch ? `/api/events/${eid}` : null, options),

    /**
     * No description
     *
     * @tags Event
     * @name EventGet
     * @request GET:/api/events/{eid}
     * @secure
     */
    mutateEventGet: (eid: string, data?: EventDto | Promise<EventDto>, options?: MutatorOptions) =>
      mutate<EventDto>(`/api/events/${eid}`, data, options),

    /**
     * No description
     *
     * @tags Event
     * @name EventUpdate
     * @request POST:/api/events/{eid}
     * @secure
     */
    eventUpdate: (eid: string, data: UpdateEventDto, params: RequestParams = {}) =>
      this.request<EventDto, ErrorProdResponse>({
        path: `/api/events/${eid}`,
        method: "POST",
        body: data,
        secure: true,
        type: ContentType.Json,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags Event
     * @name EventUpdate
     * @request POST:/api/events/{eid}
     * @secure
     */
    useEventUpdate: (
      eid: string,
      options?: SWRMutationConfiguration<EventDto, ErrorProdResponse, string, UpdateEventDto>,
    ) =>
      useSWRMutation(
        `/api/events/${eid}`,
        (_url: string, { arg }: { arg: UpdateEventDto }) =>
          this.api.eventUpdate(eid, arg).then(
            (x) => x.data,
            (x) => Promise.reject(x.error),
          ),
        options,
      ),

    /**
     * No description
     *
     * @tags Event
     * @name EventDelete
     * @request DELETE:/api/events/{eid}
     * @secure
     */
    eventDelete: (eid: string, params: RequestParams = {}) =>
      this.request<EventDto, ErrorProdResponse>({
        path: `/api/events/${eid}`,
        method: "DELETE",
        secure: true,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags Event
     * @name EventDelete
     * @request DELETE:/api/events/{eid}
     * @secure
     */
    useEventDelete: (eid: string, options?: SWRMutationConfiguration<EventDto, ErrorProdResponse, string, never>) =>
      useSWRMutation(
        `/api/events/${eid}`,
        (_url: string, { arg }: { arg: never }) =>
          this.api.eventDelete(eid, arg).then(
            (x) => x.data,
            (x) => Promise.reject(x.error),
          ),
        options,
      ),

    /**
     * No description
     *
     * @tags EventAirspace
     * @name EventAirspaceList
     * @request GET:/api/events/{eid}/airspaces
     * @secure
     */
    eventAirspaceList: (eid: string, params: RequestParams = {}) =>
      this.request<EventAirspaceDto[], ErrorProdResponse>({
        path: `/api/events/${eid}/airspaces`,
        method: "GET",
        secure: true,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags EventAirspace
     * @name EventAirspaceList
     * @request GET:/api/events/{eid}/airspaces
     * @secure
     */
    useEventAirspaceList: (eid: string, options?: SWRConfiguration, doFetch: boolean = true) =>
      useSWR<EventAirspaceDto[], ErrorProdResponse>(doFetch ? `/api/events/${eid}/airspaces` : null, options),

    /**
     * No description
     *
     * @tags EventAirspace
     * @name EventAirspaceList
     * @request GET:/api/events/{eid}/airspaces
     * @secure
     */
    mutateEventAirspaceList: (
      eid: string,
      data?: EventAirspaceDto[] | Promise<EventAirspaceDto[]>,
      options?: MutatorOptions,
    ) => mutate<EventAirspaceDto[]>(`/api/events/${eid}/airspaces`, data, options),

    /**
     * No description
     *
     * @tags EventAirspace
     * @name EventAirspaceCreate
     * @request POST:/api/events/{eid}/airspaces
     * @secure
     */
    eventAirspaceCreate: (eid: string, data: CreateEventAirspaceDto, params: RequestParams = {}) =>
      this.request<EventAirspaceDto, ErrorProdResponse>({
        path: `/api/events/${eid}/airspaces`,
        method: "POST",
        body: data,
        secure: true,
        type: ContentType.Json,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags EventAirspace
     * @name EventAirspaceCreate
     * @request POST:/api/events/{eid}/airspaces
     * @secure
     */
    useEventAirspaceCreate: (
      eid: string,
      options?: SWRMutationConfiguration<EventAirspaceDto, ErrorProdResponse, string, CreateEventAirspaceDto>,
    ) =>
      useSWRMutation(
        `/api/events/${eid}/airspaces`,
        (_url: string, { arg }: { arg: CreateEventAirspaceDto }) =>
          this.api.eventAirspaceCreate(eid, arg).then(
            (x) => x.data,
            (x) => Promise.reject(x.error),
          ),
        options,
      ),

    /**
     * No description
     *
     * @tags EventAirspace
     * @name EventAirspaceGet
     * @request GET:/api/events/{eid}/airspaces/{aid}
     * @secure
     */
    eventAirspaceGet: (eid: string, aid: string, params: RequestParams = {}) =>
      this.request<EventAirspaceDto, ErrorProdResponse>({
        path: `/api/events/${eid}/airspaces/${aid}`,
        method: "GET",
        secure: true,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags EventAirspace
     * @name EventAirspaceGet
     * @request GET:/api/events/{eid}/airspaces/{aid}
     * @secure
     */
    useEventAirspaceGet: (eid: string, aid: string, options?: SWRConfiguration, doFetch: boolean = true) =>
      useSWR<EventAirspaceDto, ErrorProdResponse>(doFetch ? `/api/events/${eid}/airspaces/${aid}` : null, options),

    /**
     * No description
     *
     * @tags EventAirspace
     * @name EventAirspaceGet
     * @request GET:/api/events/{eid}/airspaces/{aid}
     * @secure
     */
    mutateEventAirspaceGet: (
      eid: string,
      aid: string,
      data?: EventAirspaceDto | Promise<EventAirspaceDto>,
      options?: MutatorOptions,
    ) => mutate<EventAirspaceDto>(`/api/events/${eid}/airspaces/${aid}`, data, options),

    /**
     * No description
     *
     * @tags EventAirspace
     * @name EventAirspaceUpdate
     * @request PUT:/api/events/{eid}/airspaces/{aid}
     * @secure
     */
    eventAirspaceUpdate: (eid: string, aid: string, data: UpdateEventAirspaceDto, params: RequestParams = {}) =>
      this.request<EventAirspaceDto, ErrorProdResponse>({
        path: `/api/events/${eid}/airspaces/${aid}`,
        method: "PUT",
        body: data,
        secure: true,
        type: ContentType.Json,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags EventAirspace
     * @name EventAirspaceUpdate
     * @request PUT:/api/events/{eid}/airspaces/{aid}
     * @secure
     */
    useEventAirspaceUpdate: (
      eid: string,
      aid: string,
      options?: SWRMutationConfiguration<EventAirspaceDto, ErrorProdResponse, string, UpdateEventAirspaceDto>,
    ) =>
      useSWRMutation(
        `/api/events/${eid}/airspaces/${aid}`,
        (_url: string, { arg }: { arg: UpdateEventAirspaceDto }) =>
          this.api.eventAirspaceUpdate(eid, aid, arg).then(
            (x) => x.data,
            (x) => Promise.reject(x.error),
          ),
        options,
      ),

    /**
     * No description
     *
     * @tags EventAirspace
     * @name EventAirspaceDelete
     * @request DELETE:/api/events/{eid}/airspaces/{aid}
     * @secure
     */
    eventAirspaceDelete: (eid: string, aid: string, params: RequestParams = {}) =>
      this.request<EventAirspaceDto, ErrorProdResponse>({
        path: `/api/events/${eid}/airspaces/${aid}`,
        method: "DELETE",
        secure: true,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags EventAirspace
     * @name EventAirspaceDelete
     * @request DELETE:/api/events/{eid}/airspaces/{aid}
     * @secure
     */
    useEventAirspaceDelete: (
      eid: string,
      aid: string,
      options?: SWRMutationConfiguration<EventAirspaceDto, ErrorProdResponse, string, never>,
    ) =>
      useSWRMutation(
        `/api/events/${eid}/airspaces/${aid}`,
        (_url: string, { arg }: { arg: never }) =>
          this.api.eventAirspaceDelete(eid, aid, arg).then(
            (x) => x.data,
            (x) => Promise.reject(x.error),
          ),
        options,
      ),

    /**
     * No description
     *
     * @tags EventSlot
     * @name EventSlotList
     * @request GET:/api/events/{eid}/slots
     * @secure
     */
    eventSlotList: (eid: string, params: RequestParams = {}) =>
      this.request<EventSlotDto[], ErrorProdResponse>({
        path: `/api/events/${eid}/slots`,
        method: "GET",
        secure: true,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags EventSlot
     * @name EventSlotList
     * @request GET:/api/events/{eid}/slots
     * @secure
     */
    useEventSlotList: (eid: string, options?: SWRConfiguration, doFetch: boolean = true) =>
      useSWR<EventSlotDto[], ErrorProdResponse>(doFetch ? `/api/events/${eid}/slots` : null, options),

    /**
     * No description
     *
     * @tags EventSlot
     * @name EventSlotList
     * @request GET:/api/events/{eid}/slots
     * @secure
     */
    mutateEventSlotList: (eid: string, data?: EventSlotDto[] | Promise<EventSlotDto[]>, options?: MutatorOptions) =>
      mutate<EventSlotDto[]>(`/api/events/${eid}/slots`, data, options),

    /**
     * No description
     *
     * @tags EventSlot
     * @name EventSlotCreate
     * @request POST:/api/events/{eid}/slots
     * @secure
     */
    eventSlotCreate: (eid: string, data: CreateEventSlotDto, params: RequestParams = {}) =>
      this.request<EventSlotDto, ErrorProdResponse>({
        path: `/api/events/${eid}/slots`,
        method: "POST",
        body: data,
        secure: true,
        type: ContentType.Json,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags EventSlot
     * @name EventSlotCreate
     * @request POST:/api/events/{eid}/slots
     * @secure
     */
    useEventSlotCreate: (
      eid: string,
      options?: SWRMutationConfiguration<EventSlotDto, ErrorProdResponse, string, CreateEventSlotDto>,
    ) =>
      useSWRMutation(
        `/api/events/${eid}/slots`,
        (_url: string, { arg }: { arg: CreateEventSlotDto }) =>
          this.api.eventSlotCreate(eid, arg).then(
            (x) => x.data,
            (x) => Promise.reject(x.error),
          ),
        options,
      ),

    /**
     * No description
     *
     * @tags EventSlot
     * @name EventSlotGet
     * @request GET:/api/events/{eid}/slots/{sid}
     * @secure
     */
    eventSlotGet: (eid: string, sid: string, params: RequestParams = {}) =>
      this.request<EventSlotDto, ErrorProdResponse>({
        path: `/api/events/${eid}/slots/${sid}`,
        method: "GET",
        secure: true,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags EventSlot
     * @name EventSlotGet
     * @request GET:/api/events/{eid}/slots/{sid}
     * @secure
     */
    useEventSlotGet: (eid: string, sid: string, options?: SWRConfiguration, doFetch: boolean = true) =>
      useSWR<EventSlotDto, ErrorProdResponse>(doFetch ? `/api/events/${eid}/slots/${sid}` : null, options),

    /**
     * No description
     *
     * @tags EventSlot
     * @name EventSlotGet
     * @request GET:/api/events/{eid}/slots/{sid}
     * @secure
     */
    mutateEventSlotGet: (
      eid: string,
      sid: string,
      data?: EventSlotDto | Promise<EventSlotDto>,
      options?: MutatorOptions,
    ) => mutate<EventSlotDto>(`/api/events/${eid}/slots/${sid}`, data, options),

    /**
     * No description
     *
     * @tags EventSlot
     * @name EventSlotUpdate
     * @request PUT:/api/events/{eid}/slots/{sid}
     * @secure
     */
    eventSlotUpdate: (eid: string, sid: string, data: UpdateEventSlotDto, params: RequestParams = {}) =>
      this.request<EventSlotDto, ErrorProdResponse>({
        path: `/api/events/${eid}/slots/${sid}`,
        method: "PUT",
        body: data,
        secure: true,
        type: ContentType.Json,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags EventSlot
     * @name EventSlotUpdate
     * @request PUT:/api/events/{eid}/slots/{sid}
     * @secure
     */
    useEventSlotUpdate: (
      eid: string,
      sid: string,
      options?: SWRMutationConfiguration<EventSlotDto, ErrorProdResponse, string, UpdateEventSlotDto>,
    ) =>
      useSWRMutation(
        `/api/events/${eid}/slots/${sid}`,
        (_url: string, { arg }: { arg: UpdateEventSlotDto }) =>
          this.api.eventSlotUpdate(eid, sid, arg).then(
            (x) => x.data,
            (x) => Promise.reject(x.error),
          ),
        options,
      ),

    /**
     * No description
     *
     * @tags EventSlot
     * @name EventSlotDelete
     * @request DELETE:/api/events/{eid}/slots/{sid}
     * @secure
     */
    eventSlotDelete: (eid: string, sid: string, params: RequestParams = {}) =>
      this.request<EventSlotDto, ErrorProdResponse>({
        path: `/api/events/${eid}/slots/${sid}`,
        method: "DELETE",
        secure: true,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags EventSlot
     * @name EventSlotDelete
     * @request DELETE:/api/events/{eid}/slots/{sid}
     * @secure
     */
    useEventSlotDelete: (
      eid: string,
      sid: string,
      options?: SWRMutationConfiguration<EventSlotDto, ErrorProdResponse, string, never>,
    ) =>
      useSWRMutation(
        `/api/events/${eid}/slots/${sid}`,
        (_url: string, { arg }: { arg: never }) =>
          this.api.eventSlotDelete(eid, sid, arg).then(
            (x) => x.data,
            (x) => Promise.reject(x.error),
          ),
        options,
      ),

    /**
     * No description
     *
     * @tags EventSlotBooking
     * @name EventSlotBookingGet
     * @request GET:/api/events/{eid}/slots/{sid}/booking
     * @secure
     */
    eventSlotBookingGet: (eid: string, sid: string, params: RequestParams = {}) =>
      this.request<EventBookingDto, ErrorProdResponse>({
        path: `/api/events/${eid}/slots/${sid}/booking`,
        method: "GET",
        secure: true,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags EventSlotBooking
     * @name EventSlotBookingGet
     * @request GET:/api/events/{eid}/slots/{sid}/booking
     * @secure
     */
    useEventSlotBookingGet: (eid: string, sid: string, options?: SWRConfiguration, doFetch: boolean = true) =>
      useSWR<EventBookingDto, ErrorProdResponse>(doFetch ? `/api/events/${eid}/slots/${sid}/booking` : null, options),

    /**
     * No description
     *
     * @tags EventSlotBooking
     * @name EventSlotBookingGet
     * @request GET:/api/events/{eid}/slots/{sid}/booking
     * @secure
     */
    mutateEventSlotBookingGet: (
      eid: string,
      sid: string,
      data?: EventBookingDto | Promise<EventBookingDto>,
      options?: MutatorOptions,
    ) => mutate<EventBookingDto>(`/api/events/${eid}/slots/${sid}/booking`, data, options),

    /**
     * No description
     *
     * @tags EventSlotBooking
     * @name EventSlotBookingPut
     * @request PUT:/api/events/{eid}/slots/{sid}/booking
     * @secure
     */
    eventSlotBookingPut: (eid: string, sid: string, params: RequestParams = {}) =>
      this.request<EventBookingDto, ErrorProdResponse>({
        path: `/api/events/${eid}/slots/${sid}/booking`,
        method: "PUT",
        secure: true,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags EventSlotBooking
     * @name EventSlotBookingPut
     * @request PUT:/api/events/{eid}/slots/{sid}/booking
     * @secure
     */
    useEventSlotBookingPut: (
      eid: string,
      sid: string,
      options?: SWRMutationConfiguration<EventBookingDto, ErrorProdResponse, string, never>,
    ) =>
      useSWRMutation(
        `/api/events/${eid}/slots/${sid}/booking`,
        (_url: string, { arg }: { arg: never }) =>
          this.api.eventSlotBookingPut(eid, sid, arg).then(
            (x) => x.data,
            (x) => Promise.reject(x.error),
          ),
        options,
      ),

    /**
     * No description
     *
     * @tags EventSlotBooking
     * @name EventSlotBookingDelete
     * @request DELETE:/api/events/{eid}/slots/{sid}/booking
     * @secure
     */
    eventSlotBookingDelete: (eid: string, sid: string, params: RequestParams = {}) =>
      this.request<EventBookingDto, ErrorProdResponse>({
        path: `/api/events/${eid}/slots/${sid}/booking`,
        method: "DELETE",
        secure: true,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags EventSlotBooking
     * @name EventSlotBookingDelete
     * @request DELETE:/api/events/{eid}/slots/{sid}/booking
     * @secure
     */
    useEventSlotBookingDelete: (
      eid: string,
      sid: string,
      options?: SWRMutationConfiguration<EventBookingDto, ErrorProdResponse, string, never>,
    ) =>
      useSWRMutation(
        `/api/events/${eid}/slots/${sid}/booking`,
        (_url: string, { arg }: { arg: never }) =>
          this.api.eventSlotBookingDelete(eid, sid, arg).then(
            (x) => x.data,
            (x) => Promise.reject(x.error),
          ),
        options,
      ),

    /**
     * @description Login with username and password. This API does not comply with OAuth 2.1, and only supports first-party applications (the built-in web frontend). It is based on `grant_type` `password` (which has been drooped in OAuth 2.1) or `refresh_token`. It requires additional parameters for security control. **Request with password** It requires `username`, `password`, `captcha`. ```text username=alice&password=foobar&captcha=foobar&grant_type=password ``` **Request with refresh token** It requires `refresh_token`. ```text grant_type=refresh_token&refresh_token=507f0155-577e-448d-870b-5abe98a41d3f ```
     *
     * @tags Session
     * @name SessionLogin
     * @summary Login
     * @request POST:/api/session
     */
    sessionLogin: (
      data: {
        username?: string;
        password?: string;
        grant_type?: string;
        refresh_token?: string;
      },
      params: RequestParams = {},
    ) =>
      this.request<LoginResDto, ErrorProdResponse>({
        path: `/api/session`,
        method: "POST",
        body: data,
        type: ContentType.UrlEncoded,
        format: "json",
        ...params,
      }),
    /**
     * @description Login with username and password. This API does not comply with OAuth 2.1, and only supports first-party applications (the built-in web frontend). It is based on `grant_type` `password` (which has been drooped in OAuth 2.1) or `refresh_token`. It requires additional parameters for security control. **Request with password** It requires `username`, `password`, `captcha`. ```text username=alice&password=foobar&captcha=foobar&grant_type=password ``` **Request with refresh token** It requires `refresh_token`. ```text grant_type=refresh_token&refresh_token=507f0155-577e-448d-870b-5abe98a41d3f ```
     *
     * @tags Session
     * @name SessionLogin
     * @summary Login
     * @request POST:/api/session
     */
    useSessionLogin: (
      options?: SWRMutationConfiguration<
        LoginResDto,
        ErrorProdResponse,
        string,
        {
          username?: string;
          password?: string;
          grant_type?: string;
          refresh_token?: string;
        }
      >,
    ) =>
      useSWRMutation(
        `/api/session`,
        (
          _url: string,
          {
            arg,
          }: {
            arg: {
              username?: string;
              password?: string;
              grant_type?: string;
              refresh_token?: string;
            };
          },
        ) =>
          this.api.sessionLogin(arg).then(
            (x) => x.data,
            (x) => Promise.reject(x.error),
          ),
        options,
      ),

    /**
     * No description
     *
     * @tags Session
     * @name SessionGet
     * @summary Get Current
     * @request GET:/api/session
     * @secure
     */
    sessionGet: (params: RequestParams = {}) =>
      this.request<TokenDto, ErrorProdResponse>({
        path: `/api/session`,
        method: "GET",
        secure: true,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags Session
     * @name SessionGet
     * @summary Get Current
     * @request GET:/api/session
     * @secure
     */
    useSessionGet: (options?: SWRConfiguration, doFetch: boolean = true) =>
      useSWR<TokenDto, ErrorProdResponse>(doFetch ? `/api/session` : null, options),

    /**
     * No description
     *
     * @tags Session
     * @name SessionGet
     * @summary Get Current
     * @request GET:/api/session
     * @secure
     */
    mutateSessionGet: (data?: TokenDto | Promise<TokenDto>, options?: MutatorOptions) =>
      mutate<TokenDto>(`/api/session`, data, options),

    /**
     * No description
     *
     * @tags Session
     * @name SessionLogout
     * @summary Logout
     * @request DELETE:/api/session
     * @secure
     */
    sessionLogout: (params: RequestParams = {}) =>
      this.request<void, ErrorProdResponse>({
        path: `/api/session`,
        method: "DELETE",
        secure: true,
        ...params,
      }),
    /**
     * No description
     *
     * @tags Session
     * @name SessionLogout
     * @summary Logout
     * @request DELETE:/api/session
     * @secure
     */
    useSessionLogout: (options?: SWRMutationConfiguration<void, ErrorProdResponse, string, never>) =>
      useSWRMutation(
        `/api/session`,
        (_url: string, { arg }: { arg: never }) =>
          this.api.sessionLogout(arg).then(
            (x) => x.data,
            (x) => Promise.reject(x.error),
          ),
        options,
      ),

    /**
     * No description
     *
     * @tags User
     * @name UserList
     * @request GET:/api/users
     * @secure
     */
    userList: (params: RequestParams = {}) =>
      this.request<UserDto[], ErrorProdResponse>({
        path: `/api/users`,
        method: "GET",
        secure: true,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags User
     * @name UserList
     * @request GET:/api/users
     * @secure
     */
    useUserList: (options?: SWRConfiguration, doFetch: boolean = true) =>
      useSWR<UserDto[], ErrorProdResponse>(doFetch ? `/api/users` : null, options),

    /**
     * No description
     *
     * @tags User
     * @name UserList
     * @request GET:/api/users
     * @secure
     */
    mutateUserList: (data?: UserDto[] | Promise<UserDto[]>, options?: MutatorOptions) =>
      mutate<UserDto[]>(`/api/users`, data, options),

    /**
     * No description
     *
     * @tags User
     * @name UserGet
     * @request GET:/api/users/{id}
     * @secure
     */
    userGet: (id: string, params: RequestParams = {}) =>
      this.request<UserDto, ErrorProdResponse>({
        path: `/api/users/${id}`,
        method: "GET",
        secure: true,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags User
     * @name UserGet
     * @request GET:/api/users/{id}
     * @secure
     */
    useUserGet: (id: string, options?: SWRConfiguration, doFetch: boolean = true) =>
      useSWR<UserDto, ErrorProdResponse>(doFetch ? `/api/users/${id}` : null, options),

    /**
     * No description
     *
     * @tags User
     * @name UserGet
     * @request GET:/api/users/{id}
     * @secure
     */
    mutateUserGet: (id: string, data?: UserDto | Promise<UserDto>, options?: MutatorOptions) =>
      mutate<UserDto>(`/api/users/${id}`, data, options),
  };
}

export const client = new Api({
  baseUrl: "",
  securityWorker: apiSecurityWorker,
});
export default client.api;

export const fetcher = async (arg: string | [string, Record<string, unknown>?]) => {
  const { path, query } = typeof arg === "string" ? { path: arg, query: undefined } : { path: arg[0], query: arg[1] };
  return await client
    .request({ path, query, secure: true })
    .then((res) => res.json())
    .catch(async (err) => {
      if (err.json) throw await err.json();
      throw err;
    });
};
