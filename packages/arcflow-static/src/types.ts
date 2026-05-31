export type ExternalBindingKind = "browser_automation" | "schedule_trigger" | "custom";

export type ExternalBindingMode = "sync_tool" | "async_callback";

export interface ExternalBinding {
  id: string;
  kind: ExternalBindingKind;
  attach_to_step_id: string;
  mode: ExternalBindingMode;
  outcome_schema: Record<string, unknown>;
}

export interface ExternalOutcomeReport {
  binding_id: string;
  status: "success" | "failed" | "needs_input";
  error_code?: string;
  fields?: Record<string, unknown>;
}
