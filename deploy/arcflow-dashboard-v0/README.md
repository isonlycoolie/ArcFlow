# ArcFlow Dashboard v0

Private operator UI for sites, knowledge ingest, and chat publish. This tree is the OSS starter — push or sync to [ArcFlow-Dashboard](https://github.com/isonlycoolie/ArcFlow-Dashboard.git) or use as the `dashboard/` submodule in your meta-repo.

## Dev

```bash
cp .env.example .env
npm install
npm run dev
```

Open http://localhost:5174. Requires `arcflow-server` with `ARCFLOW_ADMIN_API_KEY` matching `.env`.

## API contract

Admin routes (source of truth in OSS documentation):

- [Dashboard spec](../../documentation/operator/dashboard-spec.md)
- [Admin API reference](../../documentation/operator/admin-api-reference.md)
- [HTTP API reference](../../documentation/server/http-api-reference.md)

## Meta-repo

See [deploy/meta-repo-template/](../meta-repo-template/) and [documentation/deployment/overview.md](../../documentation/deployment/overview.md#meta-repo-layout).
