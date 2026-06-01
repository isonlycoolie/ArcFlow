# ArcFlow meta-repo (private)

The open-source [ArcFlow](https://github.com/isonlycoolie/ArcFlow) tree stays public. The operator webapp and pinned releases live in a **private meta-repo** that submodules both projects.

## Layout

```text
ArcFlow-Platform/          # private meta-repo (your name may differ)
  .gitmodules
  arcflow/                 # submodule → ArcFlow OSS
  webapp/                  # submodule → ArcFlow-WebApp
  README.md
  docker-compose.yml       # optional: wires compose from arcflow/docker/
```

## Clone

```bash
git clone --recurse-submodules git@github.com:YOUR_ORG/ArcFlow-Platform.git
cd ArcFlow-Platform
```

## Local ports (convention)

| Service | Port |
|---------|------|
| arcflow-server | 8080 |
| arcflow-relay | 8090 |
| webapp dev (Next.js) | 5174 |
| operator-api | 8091 |

See the `webapp/` submodule README. It calls admin APIs on `arcflow-server` via BFF; it does not embed workflow secrets in the browser build for production.
