import { components, paths } from "../api";
import client, { queryClient, useApi } from "@/client";
import { atom, getDefaultStore } from "jotai";
import { atomWithStorage, createJSONStorage } from "jotai/utils";
import createClient, { type Middleware } from "openapi-fetch";

const authClient = createClient<paths>({ baseUrl: import.meta.env.VITE_API_ENDPOINT });

interface IAccessToken {
  access_token: string;
  expires_at: string;
}

let refreshTokenObservers: ((value: unknown) => unknown)[] = [];

const sessionStore = getDefaultStore();
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const defaultStorage = createJSONStorage<any>(() => localStorage);
export const accessTokenAtom = atomWithStorage<IAccessToken | null>("access_token", null, defaultStorage, {
  getOnInit: true,
});
export const refreshTokenAtom = atomWithStorage<string | null>("refresh_token", null, defaultStorage, {
  getOnInit: true,
});
export const isRefreshingAtom = atom(false);
export const hasAuthenticatedAtom = atom(
  (get) => !!get(accessTokenAtom) && Date.parse(get(accessTokenAtom)?.expires_at ?? "0") >= Date.now(),
);

export const getAccessToken = () => sessionStore.get(accessTokenAtom)?.access_token;

const handleSessionLoginResponse = (
  result: Pick<components["schemas"]["LoginResDto"], "access_token" | "expires_in" | "refresh_token">,
) => {
  const expires_at = new Date(Date.now() + result.expires_in * 1000).toJSON();
  sessionStore.set(accessTokenAtom, { access_token: result.access_token, expires_at });
  sessionStore.set(refreshTokenAtom, result.refresh_token);
  queryClient.invalidateQueries().catch(console.error); // eslint-disable-line no-console
};

export const devLogin = async (username: string, password: string) => {
  const data = await authClient.POST("/api/session", {
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
    body: { grant_type: "password", username, password },
    bodySerializer(body) {
      return new URLSearchParams(body).toString();
    },
  });
  if (data.error) throw data.error;
  handleSessionLoginResponse(data.data);
};
export const login = async (code: string) => {
  const data = await authClient.POST("/api/session", {
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
    body: {
      grant_type: "authorization_code",
      code,
      redirect_uri: import.meta.env.VITE_API_REDIRECT_URI,
      client_id: import.meta.env.VITE_API_CLIENT_ID,
    },
    bodySerializer(body) {
      return new URLSearchParams(body).toString();
    },
  });
  if (data.error) throw data.error;
  handleSessionLoginResponse(data.data);
};
export const refresh = async () => {
  if (sessionStore.get(isRefreshingAtom)) {
    await new Promise((resolve, _) => refreshTokenObservers.push(resolve));
    return;
  }
  sessionStore.set(isRefreshingAtom, true);
  const result = await authClient.POST("/api/session", {
    headers: {
      "Content-Type": "application/x-www-form-urlencoded",
    },
    body: {
      grant_type: "refresh_token",
      refresh_token: sessionStore.get(refreshTokenAtom) ?? "",
    },
    bodySerializer(body) {
      return new URLSearchParams(body).toString();
    },
  });
  if (result.error?.error_code === "INVALID_REFRESH_TOKEN") {
    forceLogout();
    return;
  } else if (!result.data) {
    throw result.error;
  }
  handleSessionLoginResponse(result.data);
  const observers = refreshTokenObservers;
  refreshTokenObservers = [];
  observers.forEach((observer) => observer(null));
  sessionStore.set(isRefreshingAtom, false);
};
export const logout = async () => {
  try {
    await client.DELETE("/api/session");
  } catch (e) {
    // eslint-disable-next-line no-console
    console.error("lougout failed = ", e);
  }
  forceLogout();
  queryClient.invalidateQueries().catch(console.error); // eslint-disable-line no-console
};
export const forceLogout = () => {
  sessionStore.set(accessTokenAtom, null);
  sessionStore.set(refreshTokenAtom, null);
};

/**
 * api security handler
 *
 * 1. check if access token exists and not expired
 * 2. if not, refresh access token
 *     1. check if refresh token exists
 *     2. if not, force logout
 * @returns request parameters with authentication
 */
export const authMiddleware: Middleware = {
  async onRequest({ request }) {
    let accessToken = sessionStore.get(accessTokenAtom);
    const refreshToken = sessionStore.get(refreshTokenAtom);
    if (!accessToken || Date.parse(accessToken.expires_at) < Date.now()) {
      if (refreshToken) await refresh();
    }
    accessToken = sessionStore.get(accessTokenAtom);
    if (accessToken) {
      request.headers.set("Authorization", `Bearer ${getAccessToken()}`);
    }
    return request;
  },
};

export const useUser = () => {
  const { data } = useApi("/api/session", {});
  return data?.user;
};
