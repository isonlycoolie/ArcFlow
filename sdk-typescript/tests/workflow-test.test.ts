import { Agent, Workflow } from "../index.js";

describe("workflow.test()", () => {
  it("runs deterministic stub cases", async () => {
    const wf = new Workflow({ name: "test_wf" });
    wf.step(
      new Agent({
        name: "writer",
        role: "author",
        instructions: "Write.",
      }),
    );
    const results = await wf.test([
      {
        name: "happy path",
        input: "hello",
        expected_output: "fixed",
        stub_responses: { step_1: { output: "fixed" } },
      },
    ]);
    expect(results).toHaveLength(1);
    expect(results[0]?.passed).toBe(true);
    expect(results[0]?.output).toBe("fixed");
  });
});
