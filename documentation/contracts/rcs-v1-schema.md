
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
