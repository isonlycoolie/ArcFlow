/** External binding types for workflow publish payloads (RCS v0.7). */

export type ExternalBindingKind = "browser_automation" | "schedule_trigger" | "custom";

export type ExternalBindingMode = "sync_tool" | "async_callback";

export type ExternalNeedsInputAction = "agent_reask" | "fail_run";

export type ExternalFatalAction = "hitl_escalate" | "fail_run";

export interface ExternalRecoveryPolicy {
  max_retries?: number;
  on_needs_input?: ExternalNeedsInputAction;
  on_fatal?: ExternalFatalAction;
}

export interface ExternalBinding {
  id: string;
  kind: ExternalBindingKind;
  attach_to_step_id: string;
  mode: ExternalBindingMode;
  outcome_schema: Record<string, unknown>;
  recovery?: ExternalRecoveryPolicy;
}

export interface ExternalOutcomeReport {
  binding_id: string;
  status: "success" | "failed" | "needs_input";
  error_code?: string;
  fields?: Record<string, unknown>;
  artifact_refs?: string[];
}

export function externalBinding(
  id: string,
  attachToStepId: string,
  outcomeSchema: Record<string, unknown>,
  options?: {
    kind?: ExternalBindingKind;
    mode?: ExternalBindingMode;
    recovery?: ExternalRecoveryPolicy;
  },
): ExternalBinding {
  return {
    id,
    kind: options?.kind ?? "browser_automation",
    attach_to_step_id: attachToStepId,
    mode: options?.mode ?? "async_callback",
    outcome_schema: outcomeSchema,
    recovery: options?.recovery,
  };
}
