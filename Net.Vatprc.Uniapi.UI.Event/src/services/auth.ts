import api, { ErrorProdResponse, HttpResponse, LoginResDto, RequestParams } from "../api";
import { atom, getDefaultStore } from "jotai";
import { atomWithStorage, createJSONStorage } from "jotai/utils";
import { mutate } from "swr";

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

const clearSWRStore = () =>
  mutate(
    (key) => {
      if (key === "/api/session/config") return false;
      return true;
    },
    undefined,
    true,
  );
const handleSessionLoginResponse = async (
  result: Pick<LoginResDto, "access_token" | "expires_in" | "refresh_token">,
) => {
  const expires_at = new Date(Date.now() + result.expires_in * 1000).toJSON();
  sessionStore.set(accessTokenAtom, { access_token: result.access_token, expires_at });
  sessionStore.set(refreshTokenAtom, result.refresh_token);
  await clearSWRStore();
};

export const login = async (username: string, password: string, captcha: string) => {
  const result = await api.sessionLogin({ grant_type: "password", username, password, captcha });
  await handleSessionLoginResponse(result.data);
};
export const refresh = async () => {
  try {
    if (sessionStore.get(isRefreshingAtom)) {
      await new Promise((resolve, _) => refreshTokenObservers.push(resolve));
      return;
    }
    sessionStore.set(isRefreshingAtom, true);
    const result = await api.sessionLogin({
      grant_type: "refresh_token",
      refresh_token: sessionStore.get(refreshTokenAtom) ?? "",
    });
    await handleSessionLoginResponse(result.data);
    const observers = refreshTokenObservers;
    refreshTokenObservers = [];
    observers.forEach((observer) => observer(null));
  } catch (err) {
    if ((err as HttpResponse<void, ErrorProdResponse>)?.error?.error_code === "INVALID_REFRESH_TOKEN") {
      await forceLogout();
    }
  }
  sessionStore.set(isRefreshingAtom, false);
};
export const logout = async () => {
  try {
    await api.sessionLogout();
  } catch (e) {
    console.error("lougout failed = ", e);
  }
  await forceLogout();
};
export const forceLogout = async () => {
  sessionStore.set(accessTokenAtom, null);
  sessionStore.set(refreshTokenAtom, null);
  await clearSWRStore();
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
export const apiSecurityWorker = async (): Promise<RequestParams> => {
  const accessToken = sessionStore.get(accessTokenAtom);
  const refreshToken = sessionStore.get(refreshTokenAtom);
  if (!accessToken || Date.parse(accessToken.expires_at) < Date.now()) {
    if (refreshToken) await refresh();
  }
  return {
    headers: {
      Authorization: `Bearer ${getAccessToken()}`,
    },
  };
};
