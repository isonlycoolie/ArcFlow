import fs from "node:fs";
import path from "node:path";

import { Agent, Workflow } from "../index.js";

const fixturePath = path.join(__dirname, "../../tests/fixtures/linear_graph.rcs.json");

describe("graph parity", () => {
  it("linear graph matches fixture step count", async () => {
    const fixture = JSON.parse(fs.readFileSync(fixturePath, "utf8")) as {
      steps: unknown[];
    };
    const wf = new Workflow({ name: "linear_graph_parity", graph: true });
    wf.node(
      "first",
      new Agent({ name: "first", role: "first", instructions: "Run first." }),
    );
    wf.node(
      "second",
      new Agent({ name: "second", role: "second", instructions: "Run second." }),
    );
    wf.setEntry("first");
    wf.addEdge("first", "second");
    wf.addEdge("second");
    const result = await wf.run("parity-input");
    expect(result.stepCount).toBe(fixture.steps.length);
  });
});
