
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

```http
POST /v1/workflows/chat/aliases/latest
Authorization: Bearer <ARCFLOW_SERVER_API_KEY>
Content-Type: application/json
```

```json
{
  "version": "1.0.2"
}
```

## Run via workflow_ref

Clients omit inline `workflow` and pass `workflow_ref` instead (not both):

```json
{
  "workflow_ref": {
    "name": "chat",
    "version": "^1.0.0"
  },
  "input": "What is your refund policy?",
  "exec_config": {
    "recovery_enabled": true
  }
}
```

POST `/v1/runs`. Response:

```json
{
  "run_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
  "trace_id": "trace-7c9e6679",
  "status": "Running"
}
```

Missing registry entries return `WorkflowNotFound` (HTTP 404). Invalid definitions return `InvalidWorkflowDefinition` (HTTP 400).

## Static product publish flow

Operators using the static chat product:

1. Create site via admin API (`POST /v1/admin/sites`).
2. Ingest knowledge to site `kb_namespace`.
3. Publish chat workflow:

```json
POST /v1/admin/sites/{site_id}/workflows/chat/publish

{
  "instructions": "Answer only from ingested knowledge. Say when unsure.",
  "version": "1.0.1"
}
```

4. Browser calls `runPublished("chat", "^1.0.0", userMessage)` through Relay.

Scoped runtime keys (`ARCFLOW_STATIC_RUNTIME_KEYS`) can limit which workflow names a Relay site key may invoke.

## Postgres tables

Registry data lives in `arcflow_workflows` and `arcflow_workflow_aliases`. Apply migrations with `arcflow migrate up` before first publish. Server `/ready` fails if migrations are pending.

## Validation before publish

Validate workflow JSON against [RCS schema](../../contracts/rcs-schema.md) in CI. Engine runs `validate_workflow` and `validate_graph` before execution. CLI `arcflow validate` is a stub (**FP-5.04**); see [Validation and testing](validation-and-testing.md).

## Idempotency on runs

Registry resolves workflow at run creation time. Re-posting the same run body with the same `Idempotency-Key` deduplicates within the server window without re-resolving if the original run is returned. New runs always resolve the current highest matching version for the range.

## Related pages

- [Linear workflows](linear-workflows.md) and [Graph workflows](graph-workflows.md) for definition authoring
- [The RCS contract](../../concepts/the-rcs-contract.md)
- [Server API quickstart](../../getting-started/quickstart-server-api.md)
- [Surfaces and when to use them](../../concepts/surfaces-and-when-to-use-them.md)
