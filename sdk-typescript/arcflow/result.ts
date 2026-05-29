export interface WorkflowResult {
  readonly output: string;
  readonly runId: string;
  readonly stepCount: number;
  readonly traceEvents: ReadonlyArray<Record<string, unknown>>;
}

export function toWorkflowResult(native: {
  output: string;
  runId: string;
  stepCount: number;
  traceEventsJson: string;
}): WorkflowResult {
  let traceEvents: Record<string, unknown>[] = [];
  if (native.traceEventsJson && native.traceEventsJson !== "[]") {
    const parsed: unknown = JSON.parse(native.traceEventsJson);
    if (Array.isArray(parsed)) {
      traceEvents = parsed.filter(
        (item): item is Record<string, unknown> =>
          typeof item === "object" && item !== null,
      );
    }
  }
  return {
    output: native.output,
    runId: native.runId,
    stepCount: native.stepCount,
    traceEvents,
  };
}
