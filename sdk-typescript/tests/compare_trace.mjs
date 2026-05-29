import { Agent, Workflow } from "../index.js";

const wf = new Workflow({ name: "equiv-ts" });
wf.step(
  new Agent({
    name: "a",
    role: "writer",
    instructions: "Reply in one sentence.",
  }),
);
const result = await wf.run("hello from typescript");
const trace = wf.trace();
const payload = {
  stepCount: result.stepCount,
  status: trace.status,
  workflowName: trace.workflowName,
  stepFields: trace.steps[0] ? Object.keys(trace.steps[0]).sort() : [],
};
process.stdout.write(JSON.stringify(payload));
