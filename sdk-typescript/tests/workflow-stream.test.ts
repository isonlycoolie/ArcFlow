import { Agent, Workflow } from "../index.js";

describe("workflow.runStream()", () => {
  it("yields multiple live events from the native iterator", async () => {
    const wf = new Workflow({ name: "stream-test" });
    wf.step(
      new Agent({
        name: "writer",
        role: "author",
        instructions: "Write.",
      }),
    );

    const events: string[] = [];
    for await (const event of wf.runStream("hello")) {
      events.push(event.type);
    }
    expect(events).toEqual(["step_start", "step_complete"]);
  });
});
