"use strict";
/** Vitest/Jest helpers for deterministic workflow tests (Phase 2.3). */
Object.defineProperty(exports, "__esModule", { value: true });
exports.buildTestExecConfig = buildTestExecConfig;
exports.enableStubMode = enableStubMode;
/** Builds execution config JSON with recovery disabled for tests. */
function buildTestExecConfig(options = {}) {
    const payload = {
        recovery_enabled: false,
        test: { stub_responses: options.stubResponses ?? {} },
    };
    if (options.stream) {
        payload.stream = { enabled: true };
    }
    return JSON.stringify(payload);
}
/** No-op hook documenting stub-mode tests; workflow.test() injects stubs via exec config. */
function enableStubMode() {
    // Stub mode is selected per run via buildTestExecConfig / workflow.test().
}
