/** Vitest/Jest helpers for deterministic workflow tests (Phase 2.3). */

export interface TestExecConfigOptions {
  stubResponses?: Record<string, unknown>;
  stream?: boolean;
}

/** Builds execution config JSON with recovery disabled for tests. */
export function buildTestExecConfig(
  options: TestExecConfigOptions = {},
): string {
  const payload: Record<string, unknown> = {
    recovery_enabled: false,
    test: { stub_responses: options.stubResponses ?? {} },
  };
  if (options.stream) {
    payload.stream = { enabled: true };
  }
  return JSON.stringify(payload);
}

/** No-op hook documenting stub-mode tests; workflow.test() injects stubs via exec config. */
export function enableStubMode(): void {
  // Stub mode is selected per run via buildTestExecConfig / workflow.test().
}
