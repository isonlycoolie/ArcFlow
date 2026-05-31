import { Agent, Workflow, buildTestExecConfig } from "../index.js";

describe("agent tool loop smoke", () => {
  it("chains step context in stub mode", async () => {
    const first = new Agent({ name: "researcher", role: "researcher", instructions: "Research." });
    const second = new Agent({ name: "analyst", role: "analyst", instructions: "Analyze." });
    const wf = new Workflow({ name: "chain" }).step(first).step(second);
    const result = await wf.run("AAPL");
    expect(result.output).toContain("analyst");
  });

  it("accepts initialState on run options", async () => {
    const agent = new Agent({ name: "observer", role: "observer", instructions: "Observe." });
    const wf = new Workflow({ name: "state", graph: true })
      .node("observe", agent)
      .addEdge("observe");
    const result = await wf.run("task", { initialState: { seed: "context" } });
    expect(result.output).toContain("observer");
    expect(result.output).toContain("seed=context");
  });

  it("stub test config includes step outputs for tool-loop trace kinds", () => {
    const execJson = buildTestExecConfig({
      stubResponses: {
        step_1: { output: "tool result", tool_calls: [{ name: "web_search" }] },
      },
    });
    expect(execJson).toContain("stub_responses");
    expect(execJson).toContain("web_search");
  });
});
