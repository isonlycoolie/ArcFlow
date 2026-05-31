/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_ARCFLOW_ADMIN_URL: string;
  readonly VITE_ARCFLOW_ADMIN_KEY: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
