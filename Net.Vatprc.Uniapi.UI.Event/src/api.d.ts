/**
 * This file was auto-generated by openapi-typescript.
 * Do not make direct changes to the file.
 */

export interface paths {
  "/auth/authorize": {
    parameters: {
      query?: never;
      header?: never;
      path?: never;
      cookie?: never;
    };
    get: operations["Auth_Authorize"];
    put?: never;
    post?: never;
    delete?: never;
    options?: never;
    head?: never;
    patch?: never;
    trace?: never;
  };
  "/auth/login": {
    parameters: {
      query?: never;
      header?: never;
      path?: never;
      cookie?: never;
    };
    get: operations["Auth_Login"];
    put?: never;
    post?: never;
    delete?: never;
    options?: never;
    head?: never;
    patch?: never;
    trace?: never;
  };
  "/auth/callback/vatsim": {
    parameters: {
      query?: never;
      header?: never;
      path?: never;
      cookie?: never;
    };
    get: operations["Auth_VatsimCallback"];
    put?: never;
    post?: never;
    delete?: never;
    options?: never;
    head?: never;
    patch?: never;
    trace?: never;
  };
  "/api/events": {
    parameters: {
      query?: never;
      header?: never;
      path?: never;
      cookie?: never;
    };
    get: operations["Event_List"];
    put?: never;
    post: operations["Event_Create"];
    delete?: never;
    options?: never;
    head?: never;
    patch?: never;
    trace?: never;
  };
  "/api/events/{eid}": {
    parameters: {
      query?: never;
      header?: never;
      path?: never;
      cookie?: never;
    };
    get: operations["Event_Get"];
    put?: never;
    post: operations["Event_Update"];
    delete: operations["Event_Delete"];
    options?: never;
    head?: never;
    patch?: never;
    trace?: never;
  };
  "/api/events/{eid}/airspaces": {
    parameters: {
      query?: never;
      header?: never;
      path?: never;
      cookie?: never;
    };
    get: operations["EventAirspace_List"];
    put?: never;
    post: operations["EventAirspace_Create"];
    delete?: never;
    options?: never;
    head?: never;
    patch?: never;
    trace?: never;
  };
  "/api/events/{eid}/airspaces/{aid}": {
    parameters: {
      query?: never;
      header?: never;
      path?: never;
      cookie?: never;
    };
    get: operations["EventAirspace_Get"];
    put: operations["EventAirspace_Update"];
    post?: never;
    delete: operations["EventAirspace_Delete"];
    options?: never;
    head?: never;
    patch?: never;
    trace?: never;
  };
  "/api/events/{eid}/slots": {
    parameters: {
      query?: never;
      header?: never;
      path?: never;
      cookie?: never;
    };
    get: operations["EventSlot_List"];
    put?: never;
    post: operations["EventSlot_Create"];
    delete?: never;
    options?: never;
    head?: never;
    patch?: never;
    trace?: never;
  };
  "/api/events/{eid}/slots/{sid}": {
    parameters: {
      query?: never;
      header?: never;
      path?: never;
      cookie?: never;
    };
    get: operations["EventSlot_Get"];
    put: operations["EventSlot_Update"];
    post?: never;
    delete: operations["EventSlot_Delete"];
    options?: never;
    head?: never;
    patch?: never;
    trace?: never;
  };
  "/api/events/{eid}/slots/{sid}/booking": {
    parameters: {
      query?: never;
      header?: never;
      path?: never;
      cookie?: never;
    };
    get: operations["EventSlotBooking_Get"];
    put: operations["EventSlotBooking_Put"];
    post?: never;
    delete: operations["EventSlotBooking_Delete"];
    options?: never;
    head?: never;
    patch?: never;
    trace?: never;
  };
  "/api/session": {
    parameters: {
      query?: never;
      header?: never;
      path?: never;
      cookie?: never;
    };
    /** Get Current */
    get: operations["Session_Get"];
    put?: never;
    /**
     * Login
     * @description Login with username and password. This API does not comply with OAuth 2.1,
     *     and only supports first-party applications (the built-in web frontend).
     *     It is based on `grant_type` `password` (which has been drooped in OAuth 2.1)
     *     or `refresh_token`. It requires additional parameters for security control.
     *
     *     **Request with password**
     *
     *     It requires `username`, `password`, `captcha`.
     *
     *     ```text
     *     username=alice&password=foobar&captcha=foobar&grant_type=password
     *     ```
     *
     *     **Request with refresh token**
     *
     *     It requires `refresh_token`.
     *
     *     ```text
     *     grant_type=refresh_token&refresh_token=507f0155-577e-448d-870b-5abe98a41d3f
     *     ```
     */
    post: operations["Session_Login"];
    /** Logout */
    delete: operations["Session_Logout"];
    options?: never;
    head?: never;
    patch?: never;
    trace?: never;
  };
  "/api/users": {
    parameters: {
      query?: never;
      header?: never;
      path?: never;
      cookie?: never;
    };
    get: operations["User_List"];
    put?: never;
    post?: never;
    delete?: never;
    options?: never;
    head?: never;
    patch?: never;
    trace?: never;
  };
  "/api/users/{id}": {
    parameters: {
      query?: never;
      header?: never;
      path?: never;
      cookie?: never;
    };
    get: operations["User_Get"];
    put?: never;
    post?: never;
    delete?: never;
    options?: never;
    head?: never;
    patch?: never;
    trace?: never;
  };
}
export type webhooks = Record<string, never>;
export interface components {
  schemas: {
    CreateEventAirspaceDto: {
      name: string;
    };
    CreateEventDto: {
      title: string;
      /** Format: date-time */
      start_at: string;
      /** Format: date-time */
      end_at: string;
      /** Format: date-time */
      start_booking_at: string;
      /** Format: date-time */
      end_booking_at: string;
    };
    CreateEventSlotDto: {
      airspace_id: string;
      /** Format: date-time */
      enter_at: string;
    };
    ErrorProdResponse: {
      error_code: string;
      message: string;
      [key: string]: unknown;
    };
    EventAirspaceDto: {
      id: string;
      event_id: string;
      name: string;
      /** Format: date-time */
      created_at: string;
      /** Format: date-time */
      updated_at: string;
    };
    EventBookingDto: {
      id: string;
      user_id: string;
      /** Format: date-time */
      created_at: string;
      /** Format: date-time */
      updated_at: string;
    };
    EventDto: {
      id: string;
      /** Format: date-time */
      created_at: string;
      /** Format: date-time */
      updated_at: string;
      title: string;
      /** Format: date-time */
      start_at: string;
      /** Format: date-time */
      end_at: string;
      /** Format: date-time */
      start_booking_at: string;
      /** Format: date-time */
      end_booking_at: string;
      image_url?: string | null;
    };
    EventSlotDto: {
      id: string;
      event_id: string;
      airspace_id: string;
      airspace: components["schemas"]["EventAirspaceDto"];
      /** Format: date-time */
      enter_at: string;
      /** Format: date-time */
      created_at: string;
      /** Format: date-time */
      updated_at: string;
      booking?: components["schemas"]["EventBookingDto"];
    };
    LoginResDto: {
      access_token: string;
      /** Format: int32 */
      expires_in: number;
      refresh_token: string;
      scope: string;
      token_type: string;
      issued_token_type: string;
    };
    TokenDto: {
      user: components["schemas"]["UserDto"];
      /** Format: date-time */
      issued_at: string;
      /** Format: date-time */
      expires_at: string;
    };
    UpdateEventAirspaceDto: {
      name: string;
    };
    UpdateEventDto: {
      title: string;
      /** Format: date-time */
      start_at: string;
      /** Format: date-time */
      end_at: string;
      /** Format: date-time */
      start_booking_at: string;
      /** Format: date-time */
      end_booking_at: string;
    };
    UpdateEventSlotDto: {
      /** Format: date-time */
      enter_at: string;
    };
    UserDto: {
      id: string;
      cid: string;
      full_name: string;
      /** Format: date-time */
      created_at: string;
      /** Format: date-time */
      updated_at: string;
      roles: string[];
    };
  };
  responses: never;
  parameters: never;
  requestBodies: never;
  headers: never;
  pathItems: never;
}
export type $defs = Record<string, never>;
export interface operations {
  Auth_Authorize: {
    parameters: {
      query?: {
        response_type?: string;
        client_id?: string;
        redirect_uri?: string;
      };
      header?: never;
      path?: never;
      cookie?: never;
    };
    requestBody?: never;
    responses: {
      /** @description Success */
      200: {
        headers: {
          [name: string]: unknown;
        };
        content?: never;
      };
      /** @description INTERNAL_SERVER_ERROR */
      500: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
    };
  };
  Auth_Login: {
    parameters: {
      query?: never;
      header?: never;
      path?: never;
      cookie?: never;
    };
    requestBody?: never;
    responses: {
      /** @description Success */
      200: {
        headers: {
          [name: string]: unknown;
        };
        content?: never;
      };
      /** @description INTERNAL_SERVER_ERROR */
      500: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
    };
  };
  Auth_VatsimCallback: {
    parameters: {
      query?: {
        code?: string;
        state?: string;
      };
      header?: never;
      path?: never;
      cookie?: never;
    };
    requestBody?: never;
    responses: {
      /** @description Success */
      200: {
        headers: {
          [name: string]: unknown;
        };
        content?: never;
      };
      /** @description INTERNAL_SERVER_ERROR */
      500: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
    };
  };
  Event_List: {
    parameters: {
      query?: never;
      header?: never;
      path?: never;
      cookie?: never;
    };
    requestBody?: never;
    responses: {
      /** @description Success */
      200: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["EventDto"][];
        };
      };
      /** @description INTERNAL_SERVER_ERROR */
      500: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
    };
  };
  Event_Create: {
    parameters: {
      query?: never;
      header?: never;
      path?: never;
      cookie?: never;
    };
    requestBody?: {
      content: {
        "application/json": components["schemas"]["CreateEventDto"];
      };
    };
    responses: {
      /** @description Success */
      200: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["EventDto"];
        };
      };
      /** @description INVALID_TOKEN */
      401: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
      /** @description INTERNAL_SERVER_ERROR */
      500: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
    };
  };
  Event_Get: {
    parameters: {
      query?: never;
      header?: never;
      path: {
        eid: string;
      };
      cookie?: never;
    };
    requestBody?: never;
    responses: {
      /** @description Success */
      200: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["EventDto"];
        };
      };
      /** @description EVENT_NOT_FOUND */
      404: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
      /** @description INTERNAL_SERVER_ERROR */
      500: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
    };
  };
  Event_Update: {
    parameters: {
      query?: never;
      header?: never;
      path: {
        eid: string;
      };
      cookie?: never;
    };
    requestBody?: {
      content: {
        "application/json": components["schemas"]["UpdateEventDto"];
      };
    };
    responses: {
      /** @description Success */
      200: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["EventDto"];
        };
      };
      /** @description INVALID_TOKEN */
      401: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
      /** @description INTERNAL_SERVER_ERROR */
      500: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
    };
  };
  Event_Delete: {
    parameters: {
      query?: never;
      header?: never;
      path: {
        eid: string;
      };
      cookie?: never;
    };
    requestBody?: never;
    responses: {
      /** @description Success */
      200: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["EventDto"];
        };
      };
      /** @description INVALID_TOKEN */
      401: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
      /** @description INTERNAL_SERVER_ERROR */
      500: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
    };
  };
  EventAirspace_List: {
    parameters: {
      query?: never;
      header?: never;
      path: {
        eid: string;
      };
      cookie?: never;
    };
    requestBody?: never;
    responses: {
      /** @description Success */
      200: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["EventAirspaceDto"][];
        };
      };
      /** @description INTERNAL_SERVER_ERROR */
      500: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
    };
  };
  EventAirspace_Create: {
    parameters: {
      query?: never;
      header?: never;
      path: {
        eid: string;
      };
      cookie?: never;
    };
    requestBody?: {
      content: {
        "application/json": components["schemas"]["CreateEventAirspaceDto"];
      };
    };
    responses: {
      /** @description Success */
      200: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["EventAirspaceDto"];
        };
      };
      /** @description INVALID_TOKEN */
      401: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
      /** @description INTERNAL_SERVER_ERROR */
      500: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
    };
  };
  EventAirspace_Get: {
    parameters: {
      query?: never;
      header?: never;
      path: {
        eid: string;
        aid: string;
      };
      cookie?: never;
    };
    requestBody?: never;
    responses: {
      /** @description Success */
      200: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["EventAirspaceDto"];
        };
      };
      /** @description EVENT_NOT_FOUND */
      404: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
      /** @description INTERNAL_SERVER_ERROR */
      500: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
    };
  };
  EventAirspace_Update: {
    parameters: {
      query?: never;
      header?: never;
      path: {
        eid: string;
        aid: string;
      };
      cookie?: never;
    };
    requestBody?: {
      content: {
        "application/json": components["schemas"]["UpdateEventAirspaceDto"];
      };
    };
    responses: {
      /** @description Success */
      200: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["EventAirspaceDto"];
        };
      };
      /** @description INVALID_TOKEN */
      401: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
      /** @description INTERNAL_SERVER_ERROR */
      500: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
    };
  };
  EventAirspace_Delete: {
    parameters: {
      query?: never;
      header?: never;
      path: {
        eid: string;
        aid: string;
      };
      cookie?: never;
    };
    requestBody?: never;
    responses: {
      /** @description Success */
      200: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["EventAirspaceDto"];
        };
      };
      /** @description INVALID_TOKEN */
      401: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
      /** @description INTERNAL_SERVER_ERROR */
      500: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
    };
  };
  EventSlot_List: {
    parameters: {
      query?: never;
      header?: never;
      path: {
        eid: string;
      };
      cookie?: never;
    };
    requestBody?: never;
    responses: {
      /** @description Success */
      200: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["EventSlotDto"][];
        };
      };
      /** @description INTERNAL_SERVER_ERROR */
      500: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
    };
  };
  EventSlot_Create: {
    parameters: {
      query?: never;
      header?: never;
      path: {
        eid: string;
      };
      cookie?: never;
    };
    requestBody?: {
      content: {
        "application/json": components["schemas"]["CreateEventSlotDto"];
      };
    };
    responses: {
      /** @description Success */
      200: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["EventSlotDto"];
        };
      };
      /** @description INVALID_TOKEN */
      401: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
      /** @description INTERNAL_SERVER_ERROR */
      500: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
    };
  };
  EventSlot_Get: {
    parameters: {
      query?: never;
      header?: never;
      path: {
        eid: string;
        sid: string;
      };
      cookie?: never;
    };
    requestBody?: never;
    responses: {
      /** @description Success */
      200: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["EventSlotDto"];
        };
      };
      /** @description EVENT_NOT_FOUND */
      404: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
      /** @description INTERNAL_SERVER_ERROR */
      500: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
    };
  };
  EventSlot_Update: {
    parameters: {
      query?: never;
      header?: never;
      path: {
        eid: string;
        sid: string;
      };
      cookie?: never;
    };
    requestBody?: {
      content: {
        "application/json": components["schemas"]["UpdateEventSlotDto"];
      };
    };
    responses: {
      /** @description Success */
      200: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["EventSlotDto"];
        };
      };
      /** @description INVALID_TOKEN */
      401: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
      /** @description INTERNAL_SERVER_ERROR */
      500: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
    };
  };
  EventSlot_Delete: {
    parameters: {
      query?: never;
      header?: never;
      path: {
        eid: string;
        sid: string;
      };
      cookie?: never;
    };
    requestBody?: never;
    responses: {
      /** @description Success */
      200: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["EventSlotDto"];
        };
      };
      /** @description INVALID_TOKEN */
      401: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
      /** @description INTERNAL_SERVER_ERROR */
      500: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
    };
  };
  EventSlotBooking_Get: {
    parameters: {
      query?: never;
      header?: never;
      path: {
        eid: string;
        sid: string;
      };
      cookie?: never;
    };
    requestBody?: never;
    responses: {
      /** @description Success */
      200: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["EventBookingDto"];
        };
      };
      /** @description INTERNAL_SERVER_ERROR */
      500: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
    };
  };
  EventSlotBooking_Put: {
    parameters: {
      query?: never;
      header?: never;
      path: {
        eid: string;
        sid: string;
      };
      cookie?: never;
    };
    requestBody?: never;
    responses: {
      /** @description Success */
      200: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["EventBookingDto"];
        };
      };
      /** @description INVALID_TOKEN */
      401: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
      /** @description EVENT_NOT_FOUND */
      404: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
      /** @description INTERNAL_SERVER_ERROR */
      500: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
    };
  };
  EventSlotBooking_Delete: {
    parameters: {
      query?: never;
      header?: never;
      path: {
        eid: string;
        sid: string;
      };
      cookie?: never;
    };
    requestBody?: never;
    responses: {
      /** @description Success */
      200: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["EventBookingDto"];
        };
      };
      /** @description INVALID_TOKEN */
      401: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
      /** @description INTERNAL_SERVER_ERROR */
      500: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
    };
  };
  Session_Get: {
    parameters: {
      query?: never;
      header?: never;
      path?: never;
      cookie?: never;
    };
    requestBody?: never;
    responses: {
      /** @description Success */
      200: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["TokenDto"];
        };
      };
      /** @description INVALID_TOKEN */
      401: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
      /** @description INVALID_TOKEN_NOT_FIRST_PARTY */
      403: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
      /** @description INTERNAL_SERVER_ERROR */
      500: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
    };
  };
  Session_Login: {
    parameters: {
      query?: never;
      header?: never;
      path?: never;
      cookie?: never;
    };
    requestBody?: {
      content: {
        "application/x-www-form-urlencoded": {
          username?: string;
          password?: string;
          grant_type?: string;
          refresh_token?: string;
          client_id?: string;
          code?: string;
          redirect_uri?: string;
        };
      };
    };
    responses: {
      /** @description Success */
      200: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["LoginResDto"];
        };
      };
      /** @description INVALID_GRANT_TYPE */
      400: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
      /** @description INVALID_REFRESH_TOKEN */
      403: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
      /** @description INTERNAL_SERVER_ERROR */
      500: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
    };
  };
  Session_Logout: {
    parameters: {
      query?: never;
      header?: never;
      path?: never;
      cookie?: never;
    };
    requestBody?: never;
    responses: {
      /** @description No Content */
      204: {
        headers: {
          [name: string]: unknown;
        };
        content?: never;
      };
      /** @description INVALID_TOKEN */
      401: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
      /** @description INVALID_TOKEN_NOT_FIRST_PARTY */
      403: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
      /** @description INTERNAL_SERVER_ERROR */
      500: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
    };
  };
  User_List: {
    parameters: {
      query?: never;
      header?: never;
      path?: never;
      cookie?: never;
    };
    requestBody?: never;
    responses: {
      /** @description Success */
      200: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["UserDto"][];
        };
      };
      /** @description INVALID_TOKEN */
      401: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
      /** @description INTERNAL_SERVER_ERROR */
      500: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
    };
  };
  User_Get: {
    parameters: {
      query?: never;
      header?: never;
      path: {
        id: string;
      };
      cookie?: never;
    };
    requestBody?: never;
    responses: {
      /** @description Success */
      200: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["UserDto"];
        };
      };
      /** @description INVALID_TOKEN */
      401: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
      /** @description USER_NOT_FOUND */
      404: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
      /** @description INTERNAL_SERVER_ERROR */
      500: {
        headers: {
          [name: string]: unknown;
        };
        content: {
          "application/json": components["schemas"]["ErrorProdResponse"];
        };
      };
    };
  };
}
