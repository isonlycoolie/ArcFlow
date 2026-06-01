# ArcFlow Platform (meta-repo template)

Private meta-repo: submodules OSS runtime + operator webapp.

## Submodule URLs

| Submodule | Path | Repository |
|-----------|------|------------|
| ArcFlow OSS | `arcflow/` | https://github.com/isonlycoolie/ArcFlow.git |
| ArcFlow WebApp | `webapp/` | https://github.com/isonlycoolie/ArcFlow-WebApp.git |

Copy [`.gitmodules`](.gitmodules) to your meta-repo root and run `git submodule update --init --recursive`.

## Setup

```bash
git submodule update --init --recursive
cd arcflow
docker compose -f docker/docker-compose.server.yml up -d
cd ../webapp
cp .env.example .env.local
npm install && npm run dev
cd operator-api
docker compose -f docker-compose.dev.yml up -d
alembic upgrade head
uvicorn app.main:app --reload --port 8091
```

Open http://localhost:5174 and set admin URL `http://localhost:8080` with `ARCFLOW_ADMIN_API_KEY` from compose.

Operator dashboard spec archive: `webapp/docs/operator/` in the WebApp repo.

## Release tags

Pin submodule SHAs on each release tag so server and webapp stay aligned:

```bash
cd arcflow && git checkout <oss-sha> && cd ../webapp && git checkout <webapp-sha> && cd ..
git add arcflow webapp
git commit -m "chore: pin arcflow and webapp submodules for release"
git tag v1.0.0
```
