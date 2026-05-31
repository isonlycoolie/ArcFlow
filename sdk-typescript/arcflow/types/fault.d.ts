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
    stream?: {
        enabled: boolean;
    };
    test?: {
        stub_responses: Record<string, unknown>;
    };
    initial_state?: Record<string, unknown>;
}
export declare function buildExecConfigJson(options: {
    retry?: RetryOptions;
    workflowTimeoutSeconds?: number;
    stepTimeoutSeconds?: number;
    recoveryEnabled?: boolean;
    stream?: boolean;
    test?: {
        stub_responses: Record<string, unknown>;
    };
    initialState?: Record<string, unknown>;
}): string | undefined;
//# sourceMappingURL=fault.d.ts.map