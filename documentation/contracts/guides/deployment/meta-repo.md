# ArcFlow meta-repo layout

Some teams keep the open-source ArcFlow tree as a git submodule inside a private platform repository. That pattern pins a known OSS commit next to your application code, Docker Compose overrides, and environment-specific config.

## Layout

```text
Your-Platform/               # your private repo
  .gitmodules
  arcflow/                   # submodule → public ArcFlow OSS
  your-app/                  # services that call arcflow-server or SDKs
  docker-compose.yml         # optional: wires compose from arcflow/docker/
```

## Clone

```bash
git clone --recurse-submodules git@github.com:YOUR_ORG/Your-Platform.git
cd Your-Platform/arcflow
```

## Local ports (convention)

| Service | Port |
|---------|------|
| arcflow-server | 8080 |
| arcflow-relay | 8090 |

## Bootstrap template

Copy [deploy/meta-repo-template/](../../../../deploy/meta-repo-template/) into your private repo and set the submodule URL to the public ArcFlow repository.

## Related pages

- [Self-hosted deployment](self-hosted.md)
- [Deployment overview](../../../deployment/overview.md)
- [Docker Compose production](../../../deployment/docker-compose-production.md)
