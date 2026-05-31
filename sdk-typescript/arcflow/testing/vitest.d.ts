/** Vitest/Jest helpers for deterministic workflow tests (Phase 2.3). */
export interface TestExecConfigOptions {
    stubResponses?: Record<string, unknown>;
    stream?: boolean;
}
/** Builds execution config JSON with recovery disabled for tests. */
export declare function buildTestExecConfig(options?: TestExecConfigOptions): string;
/** Marks the process as stub-mode for lower-level native tests. */
export declare function enableStubMode(): void;
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
export declare function normalizeTestCase(testCase: WorkflowTestCase): WorkflowTestCase & {
    stub_responses?: Record<string, unknown>;
};
export declare function attemptsFromStub(stubResponses: Record<string, unknown> | undefined, mockStep: string | undefined, passed: boolean): number | undefined;
//# sourceMappingURL=vitest.d.ts.map