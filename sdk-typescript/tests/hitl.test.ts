import { Agent, HitlConfig, Workflow } from "../index.js";

describe("HITL", () => {
  it("serializes hitl on step into run payload", () => {
    const submitter = new Agent({ name: "submitter", role: "employee", instructions: "submit" });
    const manager = new Agent({ name: "manager", role: "manager", instructions: "review" });
    const wf = new Workflow({ name: "expense" })
      .step(submitter)
      .step(manager, {
        hitl: new HitlConfig({
          approvalKey: "manager_approval",
          timeoutSeconds: 3600,
        }),
      });

    const payload = wf.buildRunPayload("expense:100");
    const steps = (payload.workflow as { steps: Array<Record<string, unknown>> }).steps;
    expect(steps[1]?.hitl).toEqual({
      approval_key: "manager_approval",
      timeout_seconds: 3600,
      interrupt: true,
    });
  });

  it("round-trips HitlConfig JSON", () => {
    const cfg = new HitlConfig({ approvalKey: "key", timeoutSeconds: 120 });
    expect(JSON.parse(cfg.toJson())).toEqual({
      approval_key: "key",
      timeout_seconds: 120,
      interrupt: true,
    });
  });
});
