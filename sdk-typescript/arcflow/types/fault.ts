/** Retry and timeout options for fault-tolerant workflow runs. */

export interface RetryOptions {
  maxAttempts: number;
  backoff?: {
    kind: "exponential" | "linear" | "constant";
    baseMs?: number;
    multiplier?: number;
    incrementMs?: number;
    maxMs?: number;
    jitterMs?: number;
    delayMs?: number;
  };
}

export interface ExecConfigPayload {
  retry?: {
    max_attempts: number;
    backoff: Record<string, unknown>;
  };
  workflow_timeout_secs?: number;
  step_timeout_secs?: number;
  recovery_enabled?: boolean;
}

export function buildExecConfigJson(options: {
  retry?: RetryOptions;
  workflowTimeoutSeconds?: number;
  stepTimeoutSeconds?: number;
  recoveryEnabled?: boolean;
}): string | undefined {
  const payload: ExecConfigPayload = {};
  if (options.retry) {
    const backoff = options.retry.backoff ?? {
      kind: "exponential" as const,
      baseMs: 100,
      multiplier: 2,
      maxMs: 30_000,
      jitterMs: 0,
    };
    payload.retry = {
      max_attempts: options.retry.maxAttempts,
      backoff:
        backoff.kind === "constant"
          ? {
              kind: "constant",
              delay_ms: backoff.delayMs ?? 100,
              jitter_ms: backoff.jitterMs ?? 0,
            }
          : backoff.kind === "linear"
            ? {
                kind: "linear",
                base_ms: backoff.baseMs ?? 100,
                increment_ms: backoff.incrementMs ?? 100,
                max_ms: backoff.maxMs ?? 30_000,
                jitter_ms: backoff.jitterMs ?? 0,
              }
            : {
                kind: "exponential",
                base_ms: backoff.baseMs ?? 100,
                multiplier: backoff.multiplier ?? 2,
                max_ms: backoff.maxMs ?? 30_000,
                jitter_ms: backoff.jitterMs ?? 0,
              },
    };
  }
  if (options.workflowTimeoutSeconds !== undefined) {
    payload.workflow_timeout_secs = options.workflowTimeoutSeconds;
  }
  if (options.stepTimeoutSeconds !== undefined) {
    payload.step_timeout_secs = options.stepTimeoutSeconds;
  }
  if (options.recoveryEnabled) {
    payload.recovery_enabled = true;
  }
  if (Object.keys(payload).length === 0) {
    return undefined;
  }
  return JSON.stringify(payload);
}
