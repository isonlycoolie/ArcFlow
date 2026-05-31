"use strict";
/** Vitest/Jest helpers for deterministic workflow tests (Phase 2.3). */
Object.defineProperty(exports, "__esModule", { value: true });
exports.buildTestExecConfig = buildTestExecConfig;
exports.enableStubMode = enableStubMode;
exports.normalizeTestCase = normalizeTestCase;
exports.attemptsFromStub = attemptsFromStub;
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
/** Marks the process as stub-mode for lower-level native tests. */
function enableStubMode() {
    process.env.ARCFLOW_STUB_MODE = "true";
}
/** Maps spec aliases to stub_responses entries. */
function normalizeTestCase(testCase) {
    const normalized = { ...testCase };
    const mockStep = normalized.mock_step_failure;
    const mockFailCount = normalized.mock_fail_count;
    if (mockStep !== undefined) {
        const key = String(mockStep);
        const failN = mockFailCount ?? 1;
        const stub = { ...(normalized.stub_responses ?? {}) };
        const entry = { fail_times: failN };
        if (normalized.expected_output !== undefined) {
            entry.then_output = normalized.expected_output;
        }
        stub[key] = { ...stub[key], ...entry };
        normalized.stub_responses = stub;
    }
    return normalized;
}
function attemptsFromStub(stubResponses, mockStep, passed) {
    if (!stubResponses || !mockStep) {
        return undefined;
    }
    const entry = stubResponses[mockStep];
    if (entry?.fail_times === undefined) {
        return undefined;
    }
    return passed ? entry.fail_times + 1 : entry.fail_times;
}
