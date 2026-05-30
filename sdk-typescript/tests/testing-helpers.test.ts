import {
  buildTestExecConfig,
  enableStubMode,
} from "../arcflow/testing/vitest.js";

describe("arcflow testing helpers", () => {
  it("builds test exec config with recovery disabled", () => {
    enableStubMode();
    const json = buildTestExecConfig({
      stubResponses: { step_1: { output: "fixed" } },
    });
    const parsed = JSON.parse(json) as {
      recovery_enabled: boolean;
      test: { stub_responses: Record<string, unknown> };
    };
    expect(parsed.recovery_enabled).toBe(false);
    expect(parsed.test.stub_responses.step_1).toEqual({ output: "fixed" });
  });
});
