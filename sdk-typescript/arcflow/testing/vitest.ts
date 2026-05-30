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

/** Marks the process as stub-mode for lower-level native tests. */
export function enableStubMode(): void {
  process.env.ARCFLOW_STUB_MODE = "true";
}

export interface WorkflowTestCase {
  name?: string;
  input?: string;
  expected_output?: string;
  stub_responses?: Record<string, unknown>;
  mock_step_failure?: string;
  mock_fail_count?: number;
  assert_retries?: number;
}

/** Maps spec aliases to stub_responses entries. */
export function normalizeTestCase(
  case: WorkflowTestCase,
): WorkflowTestCase & { stub_responses?: Record<string, unknown> } {
  const normalized = { ...case };
  const mockStep = normalized.mock_step_failure;
  const mockFailCount = normalized.mock_fail_count;
  if (mockStep !== undefined) {
    const key = String(mockStep);
    const failN = mockFailCount ?? 1;
    const stub = { ...(normalized.stub_responses ?? {}) };
    const entry: Record<string, unknown> = { fail_times: failN };
    if (normalized.expected_output !== undefined) {
      entry.then_output = normalized.expected_output;
    }
    stub[key] = { ...(stub[key] as Record<string, unknown> | undefined), ...entry };
    normalized.stub_responses = stub;
  }
  return normalized;
}

export function attemptsFromStub(
  stubResponses: Record<string, unknown> | undefined,
  mockStep: string | undefined,
  passed: boolean,
): number | undefined {
  if (!stubResponses || !mockStep) {
    return undefined;
  }
  const entry = stubResponses[mockStep] as { fail_times?: number } | undefined;
  if (entry?.fail_times === undefined) {
    return undefined;
  }
  return passed ? entry.fail_times + 1 : entry.fail_times;
}
