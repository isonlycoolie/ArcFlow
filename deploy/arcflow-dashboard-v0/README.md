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

Admin routes (source of truth in OSS):

- Meta-repo path: `arcflow/dashboard/spec/03-admin-api-contract.md`
- OSS link: [dashboard/spec/03-admin-api-contract.md](../../dashboard/spec/03-admin-api-contract.md)

Do not use `contracts/normative/runtime/server-api-v1.md` for admin routes — that document is stale for runtime/admin surfaces.

## Meta-repo

See [deploy/meta-repo-template/](../meta-repo-template/) and [contracts/guides/deployment/meta-repo.md](../../contracts/guides/deployment/meta-repo.md).
