
# RCS v1 JSON Schema

Machine-readable JSON Schema for the Runtime Contract Specification (RCS) v0.1. Narrative field reference: [RCS schema](rcs-schema.md). Conceptual intro: [The RCS contract](../concepts/the-rcs-contract.md).

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "https://arcflow.dev/contracts/normative/rcs/v1.schema.json",
  "title": "ArcFlow Runtime Contract Specification v0.1 (APPROVED 2026-05-26)",
  "description": "RCS v0.1 â€” machine contract between language SDKs and arcflow-core. Matches arcflow_sprint01.md Â§8. Week 3 gate: Chief Architecture Agent marked this schema APPROVED on 2026-05-26. Breaking changes require a MAJOR version bump; additive changes use MINOR per CONTRIBUTING RCS policy.",
  "$defs": {
    "Uuid": {
      "type": "string",
      "format": "uuid",
      "description": "RFC 4122 identifier. Used on all entity ids and trace correlation."
    },
    "DateTimeUtc": {
      "type": "string",
      "format": "date-time",
      "description": "ISO-8601 UTC timestamp on envelopes and trace events."
    },
    "MessageType": {
      "type": "string",
      "enum": [
        "RegisterWorkflow",
        "RunWorkflow",
        "WorkflowResult",
        "TraceEvent",
        "Error"
      ],
      "description": "Envelope dispatch label. Sprint 2+ all SDK traffic."
    },
    "ExecutionStatus": {
      "type": "string",
      "enum": [
        "Pending",
        "Running",
        "Completed",
        "Failed",
        "Retrying",
        "Cancelled",
        "Interrupted"
      ],
      "description": "Workflow or step lifecycle state. Sprint 2 execution, Sprint 5 traces."
    },
    "ErrorCode": {
      "type": "string",
      "enum": [
        "WorkflowNotFound",
        "InvalidWorkflowDefinition",
        "StepExecutionFailed",
        "ProviderError",
        "ToolExecutionFailed",
        "MemoryError",
        "Timeout",
        "RateLimited",
        "InternalError",
        "UnsupportedRcsVersion",
        "HumanTimeout",
        "HumanRejected",
        "ApprovalNotFound",
        "AlreadyApproved"
      ],
      "description": "Stable machine-readable error code in ErrorPayload. Sprint 7+ error handling."
    },
    "MemoryType": {
      "type": "string",
      "enum": ["Session", "Shared", "Persistent", "Vector"],
      "description": "Memory backend kind on AgentDefinition. Sprint 4 memory subsystem."
    },
    "MemoryScope": {
      "type": "string",
      "enum": ["Agent", "Workflow", "Global"],
      "description": "Boundary for memory reads/writes. Sprint 4."
    },
    "TraceEventKind": {
      "type": "string",
      "enum": [
        "WorkflowStarted",
        "StepStarted",
        "AgentInvoked",
        "MemoryRead",
        "MemoryWrite",
        "ToolExecuted",
        "StepCompleted",
        "WorkflowCompleted",
        "StepFailed",
        "WorkflowFailed",
        "RetryAttempted",
        "GraphNodeStarted",
        "GraphNodeCompleted",
        "GraphIterationLimitReached"
      ],
      "description": "Observability event classification. Sprint 5 tracing engine."
    },
    "ProviderId": {
      "type": "string",
      "enum": ["OpenAI", "Anthropic", "Gemini", "Custom"],
      "description": "LLM provider identifier on ProviderConfig. Sprint 6 providers."
    },
    "RetryPolicy": {
      "type": "object",
      "description": "Retry limits for workflow or step failures. Sprint 7 retry engine.",
      "properties": {
        "max_attempts": {
          "type": "integer",
          "minimum": 1,
          "description": "Total attempts including the first run. Sprint 7."
        },
        "backoff_ms": {
          "type": "integer",
          "minimum": 0,
          "description": "Initial backoff delay in milliseconds. Sprint 7."
        },
        "max_backoff_ms": {
          "type": "integer",
          "minimum": 0,
          "description": "Upper bound on backoff delay in milliseconds. Sprint 7."
        }
      },
      "required": ["max_attempts", "backoff_ms", "max_backoff_ms"],
      "additionalProperties": false
    },
    "MemoryConfig": {
      "type": "object",
      "description": "Agent memory access configuration. Sprint 4, extended Phase 2.5 (RCS v0.5).",
      "properties": {
        "memory_type": { "$ref": "#/$defs/MemoryType" },
        "scope": { "$ref": "#/$defs/MemoryScope" },
        "namespace": {
          "type": "string",
          "description": "Required for persistent and vector backends. Sprint 4."
        },
        "ttl_seconds": {
          "type": "integer",
          "minimum": 1,
          "description": "Optional TTL for memory entries in seconds. Sprint 4."
        },
        "embedding": {
          "type": "string",
          "description": "Embedding provider spec, e.g. openai/text-embedding-3-small. Phase 2.5."
        },
        "retrieval": { "$ref": "#/$defs/MemoryRetrievalConfig" },
        "chunking": { "$ref": "#/$defs/MemoryChunkingConfig" }
      },
      "required": ["memory_type", "scope"],
      "additionalProperties": false
    },
    "RetrievalModeSpec": {
      "type": "string",
      "enum": ["dense", "hybrid"],
      "description": "Vector retrieval mode. Phase 2.5."
    },
    "RerankProviderSpec": {
      "type": "string",
      "enum": ["cohere", "local"],
      "description": "Optional rerank provider. Phase 2.5."
    },
    "MemoryRetrievalConfig": {
      "type": "object",
      "description": "Hybrid retrieval and rerank settings. Phase 2.5.",
      "properties": {
        "mode": { "$ref": "#/$defs/RetrievalModeSpec" },
        "dense_weight": { "type": "number", "minimum": 0, "maximum": 1 },
        "sparse_weight": { "type": "number", "minimum": 0, "maximum": 1 },
        "rerank": { "$ref": "#/$defs/RerankProviderSpec" },
        "top_k": { "type": "integer", "minimum": 1, "maximum": 100 }
      },
      "additionalProperties": false
    },
    "MemoryChunkingConfig": {
      "type": "object",
      "description": "Document chunking for vector ingest. Phase 2.5.",
      "properties": {
        "strategy": { "type": "string" },
        "chunk_size": { "type": "integer", "minimum": 64 },
        "overlap": { "type": "integer", "minimum": 0 }
      },
      "additionalProperties": false
    },
    "ToolDefinition": {
      "type": "object",
      "description": "External tool exposed to an agent. Sprint 4 tool runtime.",
      "properties": {
        "name": {
          "type": "string",
          "description": "Tool name used for dispatch. Sprint 4."
        },
        "input_schema": {
