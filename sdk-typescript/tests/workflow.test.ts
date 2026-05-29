import { Agent, Workflow } from "../index.js";

describe("Workflow", () => {
  it("runs a single-step workflow without provider (stub path)", async () => {
    const wf = new Workflow({ name: "ts-stub" });
    wf.step(
      new Agent({
        name: "writer",
        role: "author",
        instructions: "Reply briefly.",
      }),
    );
    const result = await wf.run("hello");
    expect(result.stepCount).toBe(1);
    expect(result.output.length).toBeGreaterThan(0);
    expect(result.runId).toMatch(
      /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i,
    );
  });

  it("returns trace after run", async () => {
    const wf = new Workflow({ name: "ts-trace" });
    wf.step(
      new Agent({
        name: "writer",
        role: "author",
        instructions: "Reply briefly.",
      }),
    );
    await wf.run("hello");
    const trace = wf.trace();
    expect(trace.steps.length).toBe(1);
    expect(trace.workflowName).toBe("ts-trace");
  });
});
