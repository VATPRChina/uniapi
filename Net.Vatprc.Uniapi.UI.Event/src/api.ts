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

/** Category information. */
export interface CategoryDto {
  /** id */
  id: string;
  /** user-friendly name */
  title: string;
  /**
   * created time
   * @format date-time
   */
  created_at: string;
  /**
   * last updated time
   * @format date-time
   */
  updated_at: string;
}

export interface ConfigurationDto {
  turnstile_site_key: string;
}

export interface CreateItemDto {
  category_id: string;
  title: string;
  annotations: Record<string, string>;
  parent_item_id?: string | null;
}

export interface CreateJobDto {
  /** @format int32 */
  bangumi: number;
  target: string;
  regex: string;
  /** @format int32 */
  match_group_ep: number;
}

/** Information for creating */
export interface CreateLinkDto {
  /**
   * the url this link points to
   * @format uri
   */
  address: string;
  /** the MIME type of the target of this link */
  mime_type: string;
  /** extra information for this link */
  annotations: Record<string, string>;
  /** id of parent link, if exists */
  parent_link_id?: string | null;
}

export interface CreateOrUpdateCategoryDto {
  /** user-friendly name */
  title: string;
}

export interface CreateUserDto {
  username: string;
  password: string;
  captcha: string;
}

export interface ErrorProdResponse {
  error_code: string;
  message: string;
  [key: string]: any;
}

export interface FetchPageResponseDto {
  has_new_items: boolean;
}

export interface FileDto {
  name: string;
  type: FileType;
}

export enum FileType {
  Folder = "Folder",
  File = "File",
}

export interface ImportFolderDto {
  path: string;
  regex: string;
  /** @format int32 */
  match_group_ep: number;
  /** @format int32 */
  bangumi: number;
}

export interface ImportSubjectDto {
  /**
   * bangumi subject ID
   * @format int32
   */
  id: number;
}

/** An item, like an anime, a manga, a episode in an anime, etc. */
export interface ItemDto {
  id: string;
  category_id: string;
  title: string;
  annotations: Record<string, string>;
  parent_item_id?: string | null;
  /** @format date-time */
  created_at: string;
  /** @format date-time */
  updated_at: string;
  image_url?: string | null;
}

export interface JobDto {
  id: string;
  /** @format int32 */
  bangumi: number;
  target: string;
  regex: string;
  /** @format int32 */
  match_group_ep: number;
  enabled: boolean;
  /** @format date-time */
  last_fetched_at: string;
}

/** Link. */
export interface LinkDto {
  /** id */
  id: string;
  /** id of the item this link belongs to */
  item_id: string;
  /**
   * the url this link points to
   * @format uri
   */
  address: string;
  /** the MIME type of the target of this link */
  mime_type: string;
  /** extra information for this link */
  annotations: Record<string, string>;
  /** id of parent link, if exists */
  parent_link_id?: string | null;
  /**
   * created time
   * @format date-time
   */
  created_at: string;
  /**
   * last updated time
   * @format date-time
   */
  updated_at: string;
}

export interface ListFolderDto {
  path: string;
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

export interface SearchRequestDto {
  query: string;
}

export interface SearchResultItemDto {
  /** @format int32 */
  id: number;
  name: string;
  name_cn: string;
}

export interface TokenDto {
  user: UserDto;
  /** @format date-time */
  issued_at: string;
  /** @format date-time */
  expires_at: string;
}

export interface TorrentDto {
  id: string;
  origin_site: string;
  origin_id: string;
  title: string;
  /** @format date-time */
  published_at: string;
  link_torrent?: string | null;
  link_magnet?: string | null;
}

export interface UpdateItemDto {
  category_id?: string | null;
  title?: string | null;
  annotations?: Record<string, string>;
}

export interface UpdateJobDto {
  /** @format int32 */
  bangumi: number;
  target: string;
  regex: string;
  /** @format int32 */
  match_group_ep: number;
  enabled: boolean;
}

/** Information for updating */
export interface UpdateLinkDto {
  /**
   * the url this link points to
   * @format uri
   */
  address: string;
  /** the MIME type of the target of this link */
  mime_type: string;
  /** extra information for this link */
  annotations: Record<string, string>;
}

export interface UserDto {
  id: string;
  username: string;
  /** @format date-time */
  created_at: string;
  /** @format date-time */
  updated_at: string;
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
  public baseUrl: string = "https://zhuiani.me";
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
 * @title ZhuiAni.me API
 * @version v1
 * @baseUrl https://zhuiani.me
 *
 * # Error Handling
 *
 * ZhuiAni.me returns normalized error responses. The response body is a JSON object with the following fields:
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
     * @description Import a subject from https://bgm.tv .
     *
     * @tags Bangumi
     * @name BangumiImportSubject
     * @summary Import Subject
     * @request POST:/api/modules/bangumi/import_subject
     * @secure
     */
    bangumiImportSubject: (data: ImportSubjectDto, params: RequestParams = {}) =>
      this.request<ItemDto, ErrorProdResponse>({
        path: `/api/modules/bangumi/import_subject`,
        method: "POST",
        body: data,
        secure: true,
        type: ContentType.Json,
        format: "json",
        ...params,
      }),
    /**
     * @description Import a subject from https://bgm.tv .
     *
     * @tags Bangumi
     * @name BangumiImportSubject
     * @summary Import Subject
     * @request POST:/api/modules/bangumi/import_subject
     * @secure
     */
    useBangumiImportSubject: (
      options?: SWRMutationConfiguration<ItemDto, ErrorProdResponse, string, ImportSubjectDto>,
    ) =>
      useSWRMutation(
        `/api/modules/bangumi/import_subject`,
        (_url: string, { arg }: { arg: ImportSubjectDto }) =>
          this.api.bangumiImportSubject(arg).then(
            (x) => x.data,
            (x) => Promise.reject(x.error),
          ),
        options,
      ),

    /**
     * No description
     *
     * @tags Bangumi
     * @name BangumiSearchSubject
     * @request POST:/api/modules/bangumi/search_subject
     * @secure
     */
    bangumiSearchSubject: (data: SearchRequestDto, params: RequestParams = {}) =>
      this.request<SearchResultItemDto[], ErrorProdResponse>({
        path: `/api/modules/bangumi/search_subject`,
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
     * @tags Bangumi
     * @name BangumiSearchSubject
     * @request POST:/api/modules/bangumi/search_subject
     * @secure
     */
    useBangumiSearchSubject: (
      options?: SWRMutationConfiguration<SearchResultItemDto[], ErrorProdResponse, string, SearchRequestDto>,
    ) =>
      useSWRMutation(
        `/api/modules/bangumi/search_subject`,
        (_url: string, { arg }: { arg: SearchRequestDto }) =>
          this.api.bangumiSearchSubject(arg).then(
            (x) => x.data,
            (x) => Promise.reject(x.error),
          ),
        options,
      ),

    /**
     * No description
     *
     * @tags Bangumi
     * @name BangumiGetSubject
     * @request GET:/api/modules/bangumi/subjects/{id}
     * @secure
     */
    bangumiGetSubject: (id: number, params: RequestParams = {}) =>
      this.request<SearchResultItemDto, ErrorProdResponse>({
        path: `/api/modules/bangumi/subjects/${id}`,
        method: "GET",
        secure: true,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags Bangumi
     * @name BangumiGetSubject
     * @request GET:/api/modules/bangumi/subjects/{id}
     * @secure
     */
    useBangumiGetSubject: (id: number, options?: SWRConfiguration, doFetch: boolean = true) =>
      useSWR<SearchResultItemDto, ErrorProdResponse>(doFetch ? `/api/modules/bangumi/subjects/${id}` : null, options),

    /**
     * No description
     *
     * @tags Bangumi
     * @name BangumiGetSubject
     * @request GET:/api/modules/bangumi/subjects/{id}
     * @secure
     */
    mutateBangumiGetSubject: (
      id: number,
      data?: SearchResultItemDto | Promise<SearchResultItemDto>,
      options?: MutatorOptions,
    ) => mutate<SearchResultItemDto>(`/api/modules/bangumi/subjects/${id}`, data, options),

    /**
     * No description
     *
     * @tags Category
     * @name CategoryList
     * @summary List
     * @request GET:/api/categories
     * @secure
     */
    categoryList: (params: RequestParams = {}) =>
      this.request<CategoryDto[], ErrorProdResponse>({
        path: `/api/categories`,
        method: "GET",
        secure: true,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags Category
     * @name CategoryList
     * @summary List
     * @request GET:/api/categories
     * @secure
     */
    useCategoryList: (options?: SWRConfiguration, doFetch: boolean = true) =>
      useSWR<CategoryDto[], ErrorProdResponse>(doFetch ? `/api/categories` : null, options),

    /**
     * No description
     *
     * @tags Category
     * @name CategoryList
     * @summary List
     * @request GET:/api/categories
     * @secure
     */
    mutateCategoryList: (data?: CategoryDto[] | Promise<CategoryDto[]>, options?: MutatorOptions) =>
      mutate<CategoryDto[]>(`/api/categories`, data, options),

    /**
     * No description
     *
     * @tags Category
     * @name CategoryCreate
     * @summary Create
     * @request POST:/api/categories
     * @secure
     */
    categoryCreate: (data: CreateOrUpdateCategoryDto, params: RequestParams = {}) =>
      this.request<CategoryDto, ErrorProdResponse>({
        path: `/api/categories`,
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
     * @tags Category
     * @name CategoryCreate
     * @summary Create
     * @request POST:/api/categories
     * @secure
     */
    useCategoryCreate: (
      options?: SWRMutationConfiguration<CategoryDto, ErrorProdResponse, string, CreateOrUpdateCategoryDto>,
    ) =>
      useSWRMutation(
        `/api/categories`,
        (_url: string, { arg }: { arg: CreateOrUpdateCategoryDto }) =>
          this.api.categoryCreate(arg).then(
            (x) => x.data,
            (x) => Promise.reject(x.error),
          ),
        options,
      ),

    /**
     * No description
     *
     * @tags Category
     * @name CategoryGet
     * @summary Get
     * @request GET:/api/categories/{id}
     * @secure
     */
    categoryGet: (id: number, params: RequestParams = {}) =>
      this.request<CategoryDto, ErrorProdResponse>({
        path: `/api/categories/${id}`,
        method: "GET",
        secure: true,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags Category
     * @name CategoryGet
     * @summary Get
     * @request GET:/api/categories/{id}
     * @secure
     */
    useCategoryGet: (id: number, options?: SWRConfiguration, doFetch: boolean = true) =>
      useSWR<CategoryDto, ErrorProdResponse>(doFetch ? `/api/categories/${id}` : null, options),

    /**
     * No description
     *
     * @tags Category
     * @name CategoryGet
     * @summary Get
     * @request GET:/api/categories/{id}
     * @secure
     */
    mutateCategoryGet: (id: number, data?: CategoryDto | Promise<CategoryDto>, options?: MutatorOptions) =>
      mutate<CategoryDto>(`/api/categories/${id}`, data, options),

    /**
     * No description
     *
     * @tags Category
     * @name CategoryUpdate
     * @summary Update
     * @request PATCH:/api/categories/{id}
     * @secure
     */
    categoryUpdate: (id: number, data: CreateOrUpdateCategoryDto, params: RequestParams = {}) =>
      this.request<CategoryDto, ErrorProdResponse>({
        path: `/api/categories/${id}`,
        method: "PATCH",
        body: data,
        secure: true,
        type: ContentType.Json,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags Category
     * @name CategoryUpdate
     * @summary Update
     * @request PATCH:/api/categories/{id}
     * @secure
     */
    useCategoryUpdate: (
      id: number,
      options?: SWRMutationConfiguration<CategoryDto, ErrorProdResponse, string, CreateOrUpdateCategoryDto>,
    ) =>
      useSWRMutation(
        `/api/categories/${id}`,
        (_url: string, { arg }: { arg: CreateOrUpdateCategoryDto }) =>
          this.api.categoryUpdate(id, arg).then(
            (x) => x.data,
            (x) => Promise.reject(x.error),
          ),
        options,
      ),

    /**
     * No description
     *
     * @tags Category
     * @name CategoryDelete
     * @summary Delete
     * @request DELETE:/api/categories/{id}
     * @secure
     */
    categoryDelete: (id: number, params: RequestParams = {}) =>
      this.request<CategoryDto, ErrorProdResponse>({
        path: `/api/categories/${id}`,
        method: "DELETE",
        secure: true,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags Category
     * @name CategoryDelete
     * @summary Delete
     * @request DELETE:/api/categories/{id}
     * @secure
     */
    useCategoryDelete: (
      id: number,
      options?: SWRMutationConfiguration<CategoryDto, ErrorProdResponse, string, never>,
    ) =>
      useSWRMutation(
        `/api/categories/${id}`,
        (_url: string, { arg }: { arg: never }) =>
          this.api.categoryDelete(id, arg).then(
            (x) => x.data,
            (x) => Promise.reject(x.error),
          ),
        options,
      ),

    /**
     * @description This API will only return those are top-level, i.e. do not have a parent item. The result will be ordered by id descendingly.
     *
     * @tags Category
     * @name CategoryGetItems
     * @summary Get Child Items
     * @request GET:/api/categories/{id}/items
     * @secure
     */
    categoryGetItems: (id: number, params: RequestParams = {}) =>
      this.request<ItemDto[], ErrorProdResponse>({
        path: `/api/categories/${id}/items`,
        method: "GET",
        secure: true,
        format: "json",
        ...params,
      }),
    /**
     * @description This API will only return those are top-level, i.e. do not have a parent item. The result will be ordered by id descendingly.
     *
     * @tags Category
     * @name CategoryGetItems
     * @summary Get Child Items
     * @request GET:/api/categories/{id}/items
     * @secure
     */
    useCategoryGetItems: (id: number, options?: SWRConfiguration, doFetch: boolean = true) =>
      useSWR<ItemDto[], ErrorProdResponse>(doFetch ? `/api/categories/${id}/items` : null, options),

    /**
     * @description This API will only return those are top-level, i.e. do not have a parent item. The result will be ordered by id descendingly.
     *
     * @tags Category
     * @name CategoryGetItems
     * @summary Get Child Items
     * @request GET:/api/categories/{id}/items
     * @secure
     */
    mutateCategoryGetItems: (id: number, data?: ItemDto[] | Promise<ItemDto[]>, options?: MutatorOptions) =>
      mutate<ItemDto[]>(`/api/categories/${id}/items`, data, options),

    /**
     * @description This API will only return those are top-level, i.e. do not have a parent item. The result will be ordered by id descendingly.
     *
     * @tags Item
     * @name ItemList
     * @summary List
     * @request GET:/api/items
     * @secure
     */
    itemList: (params: RequestParams = {}) =>
      this.request<ItemDto[], ErrorProdResponse>({
        path: `/api/items`,
        method: "GET",
        secure: true,
        format: "json",
        ...params,
      }),
    /**
     * @description This API will only return those are top-level, i.e. do not have a parent item. The result will be ordered by id descendingly.
     *
     * @tags Item
     * @name ItemList
     * @summary List
     * @request GET:/api/items
     * @secure
     */
    useItemList: (options?: SWRConfiguration, doFetch: boolean = true) =>
      useSWR<ItemDto[], ErrorProdResponse>(doFetch ? `/api/items` : null, options),

    /**
     * @description This API will only return those are top-level, i.e. do not have a parent item. The result will be ordered by id descendingly.
     *
     * @tags Item
     * @name ItemList
     * @summary List
     * @request GET:/api/items
     * @secure
     */
    mutateItemList: (data?: ItemDto[] | Promise<ItemDto[]>, options?: MutatorOptions) =>
      mutate<ItemDto[]>(`/api/items`, data, options),

    /**
     * No description
     *
     * @tags Item
     * @name ItemCreate
     * @summary Create
     * @request POST:/api/items
     * @secure
     */
    itemCreate: (data: CreateItemDto, params: RequestParams = {}) =>
      this.request<ItemDto, ErrorProdResponse>({
        path: `/api/items`,
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
     * @tags Item
     * @name ItemCreate
     * @summary Create
     * @request POST:/api/items
     * @secure
     */
    useItemCreate: (options?: SWRMutationConfiguration<ItemDto, ErrorProdResponse, string, CreateItemDto>) =>
      useSWRMutation(
        `/api/items`,
        (_url: string, { arg }: { arg: CreateItemDto }) =>
          this.api.itemCreate(arg).then(
            (x) => x.data,
            (x) => Promise.reject(x.error),
          ),
        options,
      ),

    /**
     * No description
     *
     * @tags Item
     * @name ItemGet
     * @summary Get
     * @request GET:/api/items/{id}
     * @secure
     */
    itemGet: (id: string, params: RequestParams = {}) =>
      this.request<ItemDto, ErrorProdResponse>({
        path: `/api/items/${id}`,
        method: "GET",
        secure: true,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags Item
     * @name ItemGet
     * @summary Get
     * @request GET:/api/items/{id}
     * @secure
     */
    useItemGet: (id: string, options?: SWRConfiguration, doFetch: boolean = true) =>
      useSWR<ItemDto, ErrorProdResponse>(doFetch ? `/api/items/${id}` : null, options),

    /**
     * No description
     *
     * @tags Item
     * @name ItemGet
     * @summary Get
     * @request GET:/api/items/{id}
     * @secure
     */
    mutateItemGet: (id: string, data?: ItemDto | Promise<ItemDto>, options?: MutatorOptions) =>
      mutate<ItemDto>(`/api/items/${id}`, data, options),

    /**
     * No description
     *
     * @tags Item
     * @name ItemUpdate
     * @summary Update
     * @request PATCH:/api/items/{id}
     * @secure
     */
    itemUpdate: (id: string, data: UpdateItemDto, params: RequestParams = {}) =>
      this.request<ItemDto, ErrorProdResponse>({
        path: `/api/items/${id}`,
        method: "PATCH",
        body: data,
        secure: true,
        type: ContentType.Json,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags Item
     * @name ItemUpdate
     * @summary Update
     * @request PATCH:/api/items/{id}
     * @secure
     */
    useItemUpdate: (
      id: string,
      options?: SWRMutationConfiguration<ItemDto, ErrorProdResponse, string, UpdateItemDto>,
    ) =>
      useSWRMutation(
        `/api/items/${id}`,
        (_url: string, { arg }: { arg: UpdateItemDto }) =>
          this.api.itemUpdate(id, arg).then(
            (x) => x.data,
            (x) => Promise.reject(x.error),
          ),
        options,
      ),

    /**
     * No description
     *
     * @tags Item
     * @name ItemDelete
     * @summary Delete
     * @request DELETE:/api/items/{id}
     * @secure
     */
    itemDelete: (id: string, params: RequestParams = {}) =>
      this.request<ItemDto, ErrorProdResponse>({
        path: `/api/items/${id}`,
        method: "DELETE",
        secure: true,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags Item
     * @name ItemDelete
     * @summary Delete
     * @request DELETE:/api/items/{id}
     * @secure
     */
    useItemDelete: (id: string, options?: SWRMutationConfiguration<ItemDto, ErrorProdResponse, string, never>) =>
      useSWRMutation(
        `/api/items/${id}`,
        (_url: string, { arg }: { arg: never }) =>
          this.api.itemDelete(id, arg).then(
            (x) => x.data,
            (x) => Promise.reject(x.error),
          ),
        options,
      ),

    /**
     * No description
     *
     * @tags Item
     * @name ItemGetChildItems
     * @summary Get Child Items
     * @request GET:/api/items/{id}/items
     * @secure
     */
    itemGetChildItems: (id: string, params: RequestParams = {}) =>
      this.request<ItemDto[], ErrorProdResponse>({
        path: `/api/items/${id}/items`,
        method: "GET",
        secure: true,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags Item
     * @name ItemGetChildItems
     * @summary Get Child Items
     * @request GET:/api/items/{id}/items
     * @secure
     */
    useItemGetChildItems: (id: string, options?: SWRConfiguration, doFetch: boolean = true) =>
      useSWR<ItemDto[], ErrorProdResponse>(doFetch ? `/api/items/${id}/items` : null, options),

    /**
     * No description
     *
     * @tags Item
     * @name ItemGetChildItems
     * @summary Get Child Items
     * @request GET:/api/items/{id}/items
     * @secure
     */
    mutateItemGetChildItems: (id: string, data?: ItemDto[] | Promise<ItemDto[]>, options?: MutatorOptions) =>
      mutate<ItemDto[]>(`/api/items/${id}/items`, data, options),

    /**
     * No description
     *
     * @tags ItemLink
     * @name ItemLinkList
     * @summary List
     * @request GET:/api/items/{item_id}/links
     * @secure
     */
    itemLinkList: (itemId: string, params: RequestParams = {}) =>
      this.request<LinkDto[], ErrorProdResponse>({
        path: `/api/items/${itemId}/links`,
        method: "GET",
        secure: true,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags ItemLink
     * @name ItemLinkList
     * @summary List
     * @request GET:/api/items/{item_id}/links
     * @secure
     */
    useItemLinkList: (itemId: string, options?: SWRConfiguration, doFetch: boolean = true) =>
      useSWR<LinkDto[], ErrorProdResponse>(doFetch ? `/api/items/${itemId}/links` : null, options),

    /**
     * No description
     *
     * @tags ItemLink
     * @name ItemLinkList
     * @summary List
     * @request GET:/api/items/{item_id}/links
     * @secure
     */
    mutateItemLinkList: (itemId: string, data?: LinkDto[] | Promise<LinkDto[]>, options?: MutatorOptions) =>
      mutate<LinkDto[]>(`/api/items/${itemId}/links`, data, options),

    /**
     * No description
     *
     * @tags ItemLink
     * @name ItemLinkCreate
     * @summary Create
     * @request POST:/api/items/{item_id}/links
     * @secure
     */
    itemLinkCreate: (itemId: string, data: CreateLinkDto, params: RequestParams = {}) =>
      this.request<LinkDto, ErrorProdResponse>({
        path: `/api/items/${itemId}/links`,
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
     * @tags ItemLink
     * @name ItemLinkCreate
     * @summary Create
     * @request POST:/api/items/{item_id}/links
     * @secure
     */
    useItemLinkCreate: (
      itemId: string,
      options?: SWRMutationConfiguration<LinkDto, ErrorProdResponse, string, CreateLinkDto>,
    ) =>
      useSWRMutation(
        `/api/items/${itemId}/links`,
        (_url: string, { arg }: { arg: CreateLinkDto }) =>
          this.api.itemLinkCreate(itemId, arg).then(
            (x) => x.data,
            (x) => Promise.reject(x.error),
          ),
        options,
      ),

    /**
     * No description
     *
     * @tags ItemLink
     * @name ItemLinkGet
     * @summary Get
     * @request GET:/api/items/{item_id}/links/{id}
     * @secure
     */
    itemLinkGet: (itemId: string, id: string, params: RequestParams = {}) =>
      this.request<LinkDto, ErrorProdResponse>({
        path: `/api/items/${itemId}/links/${id}`,
        method: "GET",
        secure: true,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags ItemLink
     * @name ItemLinkGet
     * @summary Get
     * @request GET:/api/items/{item_id}/links/{id}
     * @secure
     */
    useItemLinkGet: (itemId: string, id: string, options?: SWRConfiguration, doFetch: boolean = true) =>
      useSWR<LinkDto, ErrorProdResponse>(doFetch ? `/api/items/${itemId}/links/${id}` : null, options),

    /**
     * No description
     *
     * @tags ItemLink
     * @name ItemLinkGet
     * @summary Get
     * @request GET:/api/items/{item_id}/links/{id}
     * @secure
     */
    mutateItemLinkGet: (itemId: string, id: string, data?: LinkDto | Promise<LinkDto>, options?: MutatorOptions) =>
      mutate<LinkDto>(`/api/items/${itemId}/links/${id}`, data, options),

    /**
     * No description
     *
     * @tags ItemLink
     * @name ItemLinkUpdate
     * @summary Update
     * @request PATCH:/api/items/{item_id}/links/{id}
     * @secure
     */
    itemLinkUpdate: (itemId: string, id: string, data: UpdateLinkDto, params: RequestParams = {}) =>
      this.request<LinkDto, ErrorProdResponse>({
        path: `/api/items/${itemId}/links/${id}`,
        method: "PATCH",
        body: data,
        secure: true,
        type: ContentType.Json,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags ItemLink
     * @name ItemLinkUpdate
     * @summary Update
     * @request PATCH:/api/items/{item_id}/links/{id}
     * @secure
     */
    useItemLinkUpdate: (
      itemId: string,
      id: string,
      options?: SWRMutationConfiguration<LinkDto, ErrorProdResponse, string, UpdateLinkDto>,
    ) =>
      useSWRMutation(
        `/api/items/${itemId}/links/${id}`,
        (_url: string, { arg }: { arg: UpdateLinkDto }) =>
          this.api.itemLinkUpdate(itemId, id, arg).then(
            (x) => x.data,
            (x) => Promise.reject(x.error),
          ),
        options,
      ),

    /**
     * No description
     *
     * @tags ItemLink
     * @name ItemLinkDelete
     * @summary Delete
     * @request DELETE:/api/items/{item_id}/links/{id}
     * @secure
     */
    itemLinkDelete: (itemId: string, id: string, params: RequestParams = {}) =>
      this.request<LinkDto, ErrorProdResponse>({
        path: `/api/items/${itemId}/links/${id}`,
        method: "DELETE",
        secure: true,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags ItemLink
     * @name ItemLinkDelete
     * @summary Delete
     * @request DELETE:/api/items/{item_id}/links/{id}
     * @secure
     */
    useItemLinkDelete: (
      itemId: string,
      id: string,
      options?: SWRMutationConfiguration<LinkDto, ErrorProdResponse, string, never>,
    ) =>
      useSWRMutation(
        `/api/items/${itemId}/links/${id}`,
        (_url: string, { arg }: { arg: never }) =>
          this.api.itemLinkDelete(itemId, id, arg).then(
            (x) => x.data,
            (x) => Promise.reject(x.error),
          ),
        options,
      ),

    /**
     * No description
     *
     * @tags PikPak
     * @name PikPakList
     * @request GET:/api/modules/pikpak/jobs
     * @secure
     */
    pikPakList: (params: RequestParams = {}) =>
      this.request<JobDto[], ErrorProdResponse>({
        path: `/api/modules/pikpak/jobs`,
        method: "GET",
        secure: true,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags PikPak
     * @name PikPakList
     * @request GET:/api/modules/pikpak/jobs
     * @secure
     */
    usePikPakList: (options?: SWRConfiguration, doFetch: boolean = true) =>
      useSWR<JobDto[], ErrorProdResponse>(doFetch ? `/api/modules/pikpak/jobs` : null, options),

    /**
     * No description
     *
     * @tags PikPak
     * @name PikPakList
     * @request GET:/api/modules/pikpak/jobs
     * @secure
     */
    mutatePikPakList: (data?: JobDto[] | Promise<JobDto[]>, options?: MutatorOptions) =>
      mutate<JobDto[]>(`/api/modules/pikpak/jobs`, data, options),

    /**
     * No description
     *
     * @tags PikPak
     * @name PikPakCreate
     * @request POST:/api/modules/pikpak/jobs
     * @secure
     */
    pikPakCreate: (data: CreateJobDto, params: RequestParams = {}) =>
      this.request<JobDto, ErrorProdResponse>({
        path: `/api/modules/pikpak/jobs`,
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
     * @tags PikPak
     * @name PikPakCreate
     * @request POST:/api/modules/pikpak/jobs
     * @secure
     */
    usePikPakCreate: (options?: SWRMutationConfiguration<JobDto, ErrorProdResponse, string, CreateJobDto>) =>
      useSWRMutation(
        `/api/modules/pikpak/jobs`,
        (_url: string, { arg }: { arg: CreateJobDto }) =>
          this.api.pikPakCreate(arg).then(
            (x) => x.data,
            (x) => Promise.reject(x.error),
          ),
        options,
      ),

    /**
     * No description
     *
     * @tags PikPak
     * @name PikPakGet
     * @request GET:/api/modules/pikpak/jobs/{id}
     * @secure
     */
    pikPakGet: (id: string, params: RequestParams = {}) =>
      this.request<JobDto, ErrorProdResponse>({
        path: `/api/modules/pikpak/jobs/${id}`,
        method: "GET",
        secure: true,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags PikPak
     * @name PikPakGet
     * @request GET:/api/modules/pikpak/jobs/{id}
     * @secure
     */
    usePikPakGet: (id: string, options?: SWRConfiguration, doFetch: boolean = true) =>
      useSWR<JobDto, ErrorProdResponse>(doFetch ? `/api/modules/pikpak/jobs/${id}` : null, options),

    /**
     * No description
     *
     * @tags PikPak
     * @name PikPakGet
     * @request GET:/api/modules/pikpak/jobs/{id}
     * @secure
     */
    mutatePikPakGet: (id: string, data?: JobDto | Promise<JobDto>, options?: MutatorOptions) =>
      mutate<JobDto>(`/api/modules/pikpak/jobs/${id}`, data, options),

    /**
     * No description
     *
     * @tags PikPak
     * @name PikPakUpdate
     * @request POST:/api/modules/pikpak/jobs/{id}
     * @secure
     */
    pikPakUpdate: (id: string, data: UpdateJobDto, params: RequestParams = {}) =>
      this.request<JobDto, ErrorProdResponse>({
        path: `/api/modules/pikpak/jobs/${id}`,
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
     * @tags PikPak
     * @name PikPakUpdate
     * @request POST:/api/modules/pikpak/jobs/{id}
     * @secure
     */
    usePikPakUpdate: (
      id: string,
      options?: SWRMutationConfiguration<JobDto, ErrorProdResponse, string, UpdateJobDto>,
    ) =>
      useSWRMutation(
        `/api/modules/pikpak/jobs/${id}`,
        (_url: string, { arg }: { arg: UpdateJobDto }) =>
          this.api.pikPakUpdate(id, arg).then(
            (x) => x.data,
            (x) => Promise.reject(x.error),
          ),
        options,
      ),

    /**
     * No description
     *
     * @tags PikPak
     * @name PikPakDelete
     * @request DELETE:/api/modules/pikpak/jobs/{id}
     * @secure
     */
    pikPakDelete: (id: string, params: RequestParams = {}) =>
      this.request<void, ErrorProdResponse>({
        path: `/api/modules/pikpak/jobs/${id}`,
        method: "DELETE",
        secure: true,
        ...params,
      }),
    /**
     * No description
     *
     * @tags PikPak
     * @name PikPakDelete
     * @request DELETE:/api/modules/pikpak/jobs/{id}
     * @secure
     */
    usePikPakDelete: (id: string, options?: SWRMutationConfiguration<void, ErrorProdResponse, string, never>) =>
      useSWRMutation(
        `/api/modules/pikpak/jobs/${id}`,
        (_url: string, { arg }: { arg: never }) =>
          this.api.pikPakDelete(id, arg).then(
            (x) => x.data,
            (x) => Promise.reject(x.error),
          ),
        options,
      ),

    /**
     * No description
     *
     * @tags PikPak
     * @name PikPakListFolder
     * @request POST:/api/modules/pikpak/list_folder
     * @secure
     */
    pikPakListFolder: (data: ListFolderDto, params: RequestParams = {}) =>
      this.request<FileDto[], ErrorProdResponse>({
        path: `/api/modules/pikpak/list_folder`,
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
     * @tags PikPak
     * @name PikPakListFolder
     * @request POST:/api/modules/pikpak/list_folder
     * @secure
     */
    usePikPakListFolder: (options?: SWRMutationConfiguration<FileDto[], ErrorProdResponse, string, ListFolderDto>) =>
      useSWRMutation(
        `/api/modules/pikpak/list_folder`,
        (_url: string, { arg }: { arg: ListFolderDto }) =>
          this.api.pikPakListFolder(arg).then(
            (x) => x.data,
            (x) => Promise.reject(x.error),
          ),
        options,
      ),

    /**
     * No description
     *
     * @tags PikPak
     * @name PikPakImportFolder
     * @request POST:/api/modules/pikpak/import_folder
     * @secure
     */
    pikPakImportFolder: (data: ImportFolderDto, params: RequestParams = {}) =>
      this.request<LinkDto[], ErrorProdResponse>({
        path: `/api/modules/pikpak/import_folder`,
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
     * @tags PikPak
     * @name PikPakImportFolder
     * @request POST:/api/modules/pikpak/import_folder
     * @secure
     */
    usePikPakImportFolder: (
      options?: SWRMutationConfiguration<LinkDto[], ErrorProdResponse, string, ImportFolderDto>,
    ) =>
      useSWRMutation(
        `/api/modules/pikpak/import_folder`,
        (_url: string, { arg }: { arg: ImportFolderDto }) =>
          this.api.pikPakImportFolder(arg).then(
            (x) => x.data,
            (x) => Promise.reject(x.error),
          ),
        options,
      ),

    /**
     * No description
     *
     * @tags Session
     * @name SessionGetConfig
     * @request GET:/api/session/config
     */
    sessionGetConfig: (params: RequestParams = {}) =>
      this.request<ConfigurationDto, ErrorProdResponse>({
        path: `/api/session/config`,
        method: "GET",
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags Session
     * @name SessionGetConfig
     * @request GET:/api/session/config
     */
    useSessionGetConfig: (options?: SWRConfiguration, doFetch: boolean = true) =>
      useSWR<ConfigurationDto, ErrorProdResponse>(doFetch ? `/api/session/config` : null, options),

    /**
     * No description
     *
     * @tags Session
     * @name SessionGetConfig
     * @request GET:/api/session/config
     */
    mutateSessionGetConfig: (data?: ConfigurationDto | Promise<ConfigurationDto>, options?: MutatorOptions) =>
      mutate<ConfigurationDto>(`/api/session/config`, data, options),

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
        captcha?: string;
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
          captcha?: string;
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
              captcha?: string;
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
     * @tags Torrent
     * @name TorrentList
     * @request GET:/api/modules/torrent_directory/torrents
     * @secure
     */
    torrentList: (
      query?: {
        query?: string;
        /** @format int32 */
        count?: number;
        /** @format date-time */
        until?: string;
      },
      params: RequestParams = {},
    ) =>
      this.request<TorrentDto[], ErrorProdResponse>({
        path: `/api/modules/torrent_directory/torrents`,
        method: "GET",
        query: query,
        secure: true,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags Torrent
     * @name TorrentList
     * @request GET:/api/modules/torrent_directory/torrents
     * @secure
     */
    useTorrentList: (
      query?: {
        query?: string;
        /** @format int32 */
        count?: number;
        /** @format date-time */
        until?: string;
      },
      options?: SWRConfiguration,
      doFetch: boolean = true,
    ) =>
      useSWR<TorrentDto[], ErrorProdResponse>(
        doFetch ? [`/api/modules/torrent_directory/torrents`, query] : null,
        options,
      ),

    /**
     * No description
     *
     * @tags Torrent
     * @name TorrentList
     * @request GET:/api/modules/torrent_directory/torrents
     * @secure
     */
    mutateTorrentList: (
      query?: {
        query?: string;
        /** @format int32 */
        count?: number;
        /** @format date-time */
        until?: string;
      },
      data?: TorrentDto[] | Promise<TorrentDto[]>,
      options?: MutatorOptions,
    ) => mutate<TorrentDto[]>([`/api/modules/torrent_directory/torrents`, query], data, options),

    /**
     * No description
     *
     * @tags Torrent
     * @name TorrentFetchPage
     * @request POST:/api/modules/torrent_directory/providers/{name}/fetch/{page}
     * @secure
     */
    torrentFetchPage: (name: string, page: number, params: RequestParams = {}) =>
      this.request<FetchPageResponseDto, ErrorProdResponse>({
        path: `/api/modules/torrent_directory/providers/${name}/fetch/${page}`,
        method: "POST",
        secure: true,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags Torrent
     * @name TorrentFetchPage
     * @request POST:/api/modules/torrent_directory/providers/{name}/fetch/{page}
     * @secure
     */
    useTorrentFetchPage: (
      name: string,
      page: number,
      options?: SWRMutationConfiguration<FetchPageResponseDto, ErrorProdResponse, string, never>,
    ) =>
      useSWRMutation(
        `/api/modules/torrent_directory/providers/${name}/fetch/${page}`,
        (_url: string, { arg }: { arg: never }) =>
          this.api.torrentFetchPage(name, page, arg).then(
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
     * @name UserRegister
     * @request POST:/api/users
     */
    userRegister: (data: CreateUserDto, params: RequestParams = {}) =>
      this.request<UserDto, ErrorProdResponse>({
        path: `/api/users`,
        method: "POST",
        body: data,
        type: ContentType.Json,
        format: "json",
        ...params,
      }),
    /**
     * No description
     *
     * @tags User
     * @name UserRegister
     * @request POST:/api/users
     */
    useUserRegister: (options?: SWRMutationConfiguration<UserDto, ErrorProdResponse, string, CreateUserDto>) =>
      useSWRMutation(
        `/api/users`,
        (_url: string, { arg }: { arg: CreateUserDto }) =>
          this.api.userRegister(arg).then(
            (x) => x.data,
            (x) => Promise.reject(x.error),
          ),
        options,
      ),

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
