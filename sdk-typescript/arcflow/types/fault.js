"use strict";
/** Retry and timeout options for fault-tolerant workflow runs. */
Object.defineProperty(exports, "__esModule", { value: true });
exports.buildExecConfigJson = buildExecConfigJson;
function buildExecConfigJson(options) {
    const payload = {};
    if (options.retry) {
        const backoff = options.retry.backoff ?? {
            kind: "exponential",
            baseMs: 100,
            multiplier: 2,
            maxMs: 30_000,
            jitterMs: 0,
        };
        payload.retry = {
            max_attempts: options.retry.maxAttempts,
            backoff: backoff.kind === "constant"
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
    if (options.stream) {
        payload.stream = { enabled: true };
    }
    if (options.test) {
        payload.test = options.test;
    }
    if (options.initialState) {
        payload.initial_state = options.initialState;
    }
    if (Object.keys(payload).length === 0) {
        return undefined;
    }
    return JSON.stringify(payload);
}
