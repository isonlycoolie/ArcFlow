# ArcFlow Platform (meta-repo template)

Private meta-repo: submodules OSS runtime + operator dashboard.

## Submodule URLs

| Submodule | Path | Repository |
|-----------|------|------------|
| ArcFlow OSS | `arcflow/` | https://github.com/isonlycoolie/ArcFlow.git |
| ArcFlow Dashboard | `dashboard/` | https://github.com/isonlycoolie/ArcFlow-Dashboard.git |

Copy [`.gitmodules`](.gitmodules) to your meta-repo root and run `git submodule update --init --recursive`.

## Setup

```bash
git submodule update --init --recursive
cd arcflow
docker compose -f docker/docker-compose.server.yml up -d
cd ../dashboard
cp .env.example .env
npm install && npm run dev
```

Open http://localhost:5174 and set admin URL `http://localhost:8080` with `ARCFLOW_ADMIN_API_KEY` from compose.

Admin API contract: [documentation/operator/dashboard-spec.md](https://github.com/isonlycoolie/ArcFlow/blob/master/documentation/operator/dashboard-spec.md) (OSS); detailed spec files live in the ArcFlow-Dashboard repo.

## Release tags

Pin submodule SHAs on each release tag so server and dashboard stay aligned:

```bash
cd arcflow && git checkout <oss-sha> && cd ../dashboard && git checkout <dashboard-sha> && cd ..
git add arcflow dashboard
git commit -m "chore: pin arcflow and dashboard submodules for release"
git tag v1.0.0
```
