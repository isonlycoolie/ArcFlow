import fs from "node:fs";
import path from "node:path";

import { Agent, Workflow } from "../index.js";

const linearFixturePath = path.join(__dirname, "../../tests/fixtures/linear_graph.rcs.json");
const reactFixturePath = path.join(__dirname, "../../tests/fixtures/react_graph.rcs.json");

function buildReactWorkflow(maxIterations: number): Workflow {
  const wf = new Workflow({ name: "react_graph_parity", graph: true });
  wf.withMaxIterations(maxIterations);
  wf.node("think", new Agent({ name: "think", role: "think", instructions: "Think." }));
  wf.node("act", new Agent({ name: "act", role: "act", instructions: "Act." }));
  wf.node("observe", new Agent({ name: "observe", role: "observe", instructions: "Observe." }));
  wf.setEntry("think");
  wf.addEdge("think", "act");
  wf.addEdge("act", "observe");
  wf.addEdge("observe", "think");
  return wf;
}

describe("graph parity", () => {
  it("linear graph matches fixture step count", async () => {
    const fixture = JSON.parse(fs.readFileSync(linearFixturePath, "utf8")) as {
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

  it("react graph one iteration matches fixture node count", async () => {
    const fixture = JSON.parse(fs.readFileSync(reactFixturePath, "utf8")) as {
      steps: unknown[];
    };
    const result = await buildReactWorkflow(1).run("react-input");
    expect(result.stepCount).toBe(fixture.steps.length);
  });
});
