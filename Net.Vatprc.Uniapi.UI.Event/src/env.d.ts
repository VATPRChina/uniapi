/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_API_ENDPOINT: string;
  readonly VITE_API_CLIENT_ID: string;
  readonly VITE_API_REDIRECT_URI: string;
  // readonly VITE_TURNSTILE_SITE_KEY: string;
  // more env variables...
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
