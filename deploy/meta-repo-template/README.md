# ArcFlow Platform (meta-repo template)

Private meta-repo: submodules OSS runtime + operator dashboard.

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

## Release tags

Pin submodule SHAs on each release tag so server and dashboard stay aligned.
