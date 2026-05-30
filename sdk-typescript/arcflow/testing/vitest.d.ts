/** Vitest/Jest helpers for deterministic workflow tests (Phase 2.3). */
export interface TestExecConfigOptions {
    stubResponses?: Record<string, unknown>;
    stream?: boolean;
}
/** Builds execution config JSON with recovery disabled for tests. */
export declare function buildTestExecConfig(options?: TestExecConfigOptions): string;
/** No-op hook documenting stub-mode tests; workflow.test() injects stubs via exec config. */
export declare function enableStubMode(): void;
//# sourceMappingURL=vitest.d.ts.map