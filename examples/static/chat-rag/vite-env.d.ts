/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_ARCFLOW_RELAY_URL: string;
  readonly VITE_ARCFLOW_SITE_TOKEN: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
