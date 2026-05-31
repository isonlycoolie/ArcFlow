**Audience:** `[platform]` `[operator]`

# Workflow registry

The workflow registry stores versioned workflow definitions on the server so clients resolve semver ranges instead of sending full JSON on every run. Platform teams publish definitions with `PUT /v1/workflows/{name}/versions/{version}`. Callers use `workflow_ref` in `POST /v1/runs` or the static SDK `runPublished("chat", "^1.0.0", message)`.

Registry requires Postgres and a running [arcflow-server](../../getting-started/quickstart-server-api.md). Embedded SDK runs without the server do not use the registry unless you call registry HTTP routes yourself.

## Operations

| Operation | Route | Auth |
|-----------|-------|------|
| Publish version | `PUT /v1/workflows/{name}/versions/{version}` | Runtime API key |
| Get version | `GET /v1/workflows/{name}/versions/{version}` | Runtime API key |
| Resolve semver | `GET /v1/workflows/{name}/resolve?range=^1.0.0` | Runtime API key |
| Set alias | `POST /v1/workflows/{name}/aliases/{alias}` | Runtime API key |

Admin static product also publishes chat workflows via `POST /v1/admin/sites/{id}/workflows/chat/publish` (see [Knowledge ingestion](../memory-and-rag/knowledge-ingestion.md) for the full static operator path).

## Publish a workflow version

```http
PUT /v1/workflows/chat/versions/1.0.0
Authorization: Bearer <ARCFLOW_SERVER_API_KEY>
Content-Type: application/json
```

Body:

```json
{
  "definition": {
    "id": "00000000-0000-4000-8000-000000000001",
    "name": "chat",
    "execution_mode": "linear",
    "steps": [
      {
        "id": "00000000-0000-4000-8000-000000000010",
        "agent_id": "00000000-0000-4000-8000-000000000020",
        "order": 1
      }
    ]
  },
  "agents": [
    {
      "id": "00000000-0000-4000-8000-000000000020",
      "name": "assistant",
      "role": "Support",
      "instructions": "Answer from ingested knowledge only. Say when unsure.",
      "memory_config": {
        "memory_type": "vector",
        "scope": "workflow",
        "namespace": "acme-support-kb",
        "embedding": "openai/text-embedding-3-small",
        "retrieval": { "mode": "hybrid", "top_k": 5 }
      },
      "provider": {
        "provider_id": "openai",
        "model": "gpt-4o-mini",
        "api_key_env": "OPENAI_API_KEY"
      }
    }
  ]
}
```

Exact publish body shape may include agents inline or reference stored agents depending on server version; align with `arcflow_workflows` migration schema (migration `20240531000006`).

## Resolve semver range

```http
GET /v1/workflows/chat/resolve?range=^1.0.0
Authorization: Bearer <ARCFLOW_SERVER_API_KEY>
```

Response:

```json
{
  "name": "chat",
  "version": "1.0.2",
  "definition": {
    "id": "00000000-0000-4000-8000-000000000001",
    "name": "chat",
    "execution_mode": "linear",
    "steps": []
  }
}
```

The resolver picks the highest matching version per semver rules. Pin exact versions in production when you need reproducibility; use ranges for consumer apps that should pick up patch releases.

## Set alias

Point `latest` (or any alias name) at a concrete version:
