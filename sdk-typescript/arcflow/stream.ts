/** User-facing stream events from workflow.runStream() (Phase 2.1). */

export type StreamEvent =
  | { type: "token"; text: string; step_id: string }
  | { type: "step_start"; step_id: string; node_id?: string }
  | { type: "step_complete"; step_id: string; duration_ms: number }
  | { type: "tool_call"; tool_name: string; args_keys: string[] }
  | { type: "error"; code: string; message: string; step_id: string };

export interface StreamRunResult {
  output: string;
  runId: string;
  stepCount: number;
  traceEventsJson: string;
}
