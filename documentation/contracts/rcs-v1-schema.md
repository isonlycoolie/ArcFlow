
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
          "description": "JSON Schema describing tool inputs. Sprint 4.",
          "type": "object"
        },
        "permissions": {
          "type": "array",
          "items": { "type": "string" },
          "description": "Permission strings required to invoke the tool. Sprint 4."
        }
      },
      "required": ["name", "input_schema"],
      "additionalProperties": false
    },
    "ProviderConfig": {
      "type": "object",
      "description": "LLM provider settings for a run. Sprint 6.",
      "properties": {
        "provider_id": { "$ref": "#/$defs/ProviderId" },
        "model": {
          "type": "string",
          "description": "Model id passed to the provider API. Sprint 6."
        },
        "api_key_env": {
          "type": "string",
          "description": "Environment variable holding the API key. Sprint 6."
        },
        "params": {
          "description": "Optional provider-specific parameters. Sprint 6.",
          "type": "object"
        }
      },
      "required": ["provider_id", "model", "api_key_env"],
      "additionalProperties": false
    },
    "AgentDefinition": {
      "type": "object",
      "description": "Agent role and capabilities referenced by workflow steps. Sprint 2â€“6.",
      "properties": {
        "id": { "$ref": "#/$defs/Uuid" },
        "name": {
          "type": "string",
          "description": "Human-readable agent name. Sprint 2â€“3 SDK binding."
        },
        "role": {
          "type": "string",
          "description": "Role label used in prompts. Sprint 2â€“3."
        },
        "instructions": {
          "type": "string",
          "description": "System or task instructions. Sprint 2â€“3, Sprint 6 provider calls."
        },
        "tools": {
          "type": "array",
          "items": { "$ref": "#/$defs/ToolDefinition" },
          "description": "Tools available to the agent. Sprint 4."
        },
        "memory_config": {
          "$ref": "#/$defs/MemoryConfig",
          "description": "Optional memory configuration. Sprint 4."
        },
        "context": {
          "type": "object",
          "description": "Context assembly policy (Phase 2-Pro / RCS v0.6).",
          "properties": {
            "include_prior_steps": {
              "type": "string",
              "enum": ["all", "last", "none"],
              "default": "all"
            },
            "include_run_input": { "type": "boolean", "default": true },
            "max_prior_step_chars": { "type": "integer", "default": 4096 }
          },
          "additionalProperties": false
        },
        "tool_execution": {
          "type": "object",
          "description": "Tool loop configuration (Phase 2-Pro / RCS v0.6).",
          "properties": {
            "mode": {
              "type": "string",
              "enum": ["legacy_eager", "llm_select"],
              "default": "llm_select"
            },
            "max_iterations": { "type": "integer", "default": 5, "minimum": 1, "maximum": 20 }
          },
          "additionalProperties": false
        }
      },
      "required": ["id", "name", "role", "instructions"],
      "additionalProperties": false
    },
    "StepDefinition": {
      "type": "object",
      "description": "Ordered step within a workflow. Sprint 2 execution.",
      "properties": {
        "id": { "$ref": "#/$defs/Uuid" },
        "agent_id": {
          "$ref": "#/$defs/Uuid",
          "description": "Agent that executes this step. Sprint 2â€“3."
        },
        "order": {
          "type": "integer",
          "minimum": 0,
          "description": "Execution order among steps. Sprint 2."
        },
        "fallback_step_id": {
          "$ref": "#/$defs/Uuid",
          "description": "Optional fallback step on failure. Sprint 7."
        },
        "hitl": {
          "$ref": "#/$defs/HitlConfig",
          "description": "Optional human approval gate (Phase 1.4)."
        }
      },
      "required": ["id", "agent_id", "order"],
      "additionalProperties": false
    },
    "HitlConfig": {
      "type": "object",
      "description": "Human-in-the-loop gate on a step (Phase 1.4).",
      "properties": {
        "approval_key": {
          "type": "string",
          "minLength": 1,
          "description": "Stable key scoped to the run for approval requests."
        },
        "timeout_seconds": {
          "type": "integer",
          "minimum": 1,
          "description": "Wall-clock seconds before the approval expires."
        },
        "interrupt": {
          "type": "boolean",
          "default": true,
          "description": "When true, checkpoint and return Interrupted instead of blocking."
        }
      },
      "required": ["approval_key", "timeout_seconds"],
      "additionalProperties": false
    },
    "ExecutionMode": {
      "type": "string",
      "enum": ["linear", "graph"],
      "default": "linear",
      "description": "Workflow execution strategy. linear = ordered steps; graph = DAG with conditional edges. Phase 1.1."
    },
    "GraphNode": {
      "type": "object",
      "description": "Node in a graph workflow referencing a StepDefinition. Phase 1.1.",
      "properties": {
        "id": {
          "type": "string",
          "description": "Graph-local node identifier."
        },
        "step_ref": {
          "$ref": "#/$defs/Uuid",
          "description": "References StepDefinition.id."
        },
        "inputs": {
          "type": "array",
          "items": { "type": "string" },
          "description": "Optional input state keys for this node."
        },
        "outputs": {
          "type": "array",
          "items": { "type": "string" },
          "description": "Optional output state keys for this node."
        }
      },
      "required": ["id", "step_ref"],
      "additionalProperties": false
    },
    "GraphEdge": {
      "type": "object",
      "description": "Directed edge between graph nodes. Phase 1.1.",
      "properties": {
        "from": {
          "type": "string",
          "description": "Source node id."
        },
        "to": {
          "type": "string",
          "description": "Target node id; omit for terminal."
        },
        "condition": {
          "type": "string",
          "description": "Edge key returned by source node; omit for unconditional."
        }
      },
      "required": ["from"],
      "additionalProperties": false
