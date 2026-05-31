export interface RunResult {
  runId: string;
  traceId: string;
  status: string;
  output: string;
  stepCount: number;
  error?: { code: string; message: string; stepId?: string };
  interrupt?: { approvalKey: string; expiresAt: string; stepIndex?: number };
}

export function parseRunStatus(body: Record<string, unknown>): RunResult {
  const result = body.result as { output?: string; step_count?: number } | undefined;
  const error = body.error as { code?: string; message?: string; step_id?: string } | undefined;
  const interrupt = body.interrupt as
    | { approval_key?: string; expires_at?: string; step_index?: number }
    | undefined;
  return {
    runId: String(body.run_id ?? ""),
    traceId: String(body.trace_id ?? ""),
    status: String(body.status ?? ""),
    output: String(result?.output ?? ""),
    stepCount: Number(result?.step_count ?? 0),
    error: error
      ? {
          code: String(error.code ?? ""),
          message: String(error.message ?? ""),
          stepId: error.step_id ? String(error.step_id) : undefined,
        }
      : undefined,
    interrupt: interrupt
      ? {
          approvalKey: String(interrupt.approval_key ?? ""),
          expiresAt: String(interrupt.expires_at ?? ""),
          stepIndex: interrupt.step_index,
        }
      : undefined,
  };
}

export function parseCreateRun(body: Record<string, unknown>): { runId: string; status: string } {
  return { runId: String(body.run_id ?? ""), status: String(body.status ?? "") };
}
