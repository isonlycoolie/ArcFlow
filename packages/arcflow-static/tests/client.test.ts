import { Agent } from "../src/agent.js";
import { MemoryConfig } from "../src/memory.js";
import { StepForm } from "../src/step-form.js";
import { Workflow } from "../src/workflow.js";

describe("StepForm", () => {
  it("builds conversation_turns initial state", () => {
    const form = new StepForm()
      .addTurn("user", "Apply for license")
      .addTurn("assistant", "Legal name?");
    expect(form.toInitialState()).toEqual({
      conversation_turns: [
        { role: "user", content: "Apply for license" },
        { role: "assistant", content: "Legal name?" },
      ],
    });
  });
});

describe("Workflow.buildRunPayload", () => {
  it("builds inline RCS payload with exec_config.initial_state", () => {
    const agent = new Agent({
      name: "bot",
      role: "support",
      instructions: "Help the user.",
      memory: new MemoryConfig({ type: "Vector", namespace: "kb", embedding: "stub/384" }),
    });
    const wf = new Workflow({ name: "chat" }).step(agent);
    const state = new StepForm().addTurn("user", "Hi").toInitialState();
    const payload = wf.buildRunPayload("Hello", state);

    expect(payload.input).toBe("Hello");
    expect(payload.workflow_ref).toBeUndefined();
    const workflow = payload.workflow as Record<string, unknown>;
    expect(workflow.name).toBe("chat");
    expect(workflow.execution_mode).toBe("linear");
    const agents = payload.agents as Array<Record<string, unknown>>;
    expect(agents[0].memory_config).toBeDefined();
    const exec = payload.exec_config as Record<string, unknown>;
    expect(exec.initial_state).toEqual(state);
  });

  it("builds workflow_ref payload when resolved", () => {
    const wf = new Workflow({ name: "chat", runtime: "http://localhost:8080" });
    wf.registryRef = { name: "chat", version: "1.0.0" };
    const payload = wf.buildRunPayload("Hi");
    expect(payload.workflow_ref).toEqual({ name: "chat", version: "1.0.0" });
    expect(payload.workflow).toBeUndefined();
  });
});

describe("Agent.bindingRow", () => {
  it("includes memory_config and tools", () => {
    const agent = new Agent({
      name: "a",
      role: "r",
      instructions: "do",
      memory: new MemoryConfig({ type: "Session" }),
    });
    const row = agent.bindingRow();
    expect(row.memory_config).toMatchObject({ memory_type: "Session" });
  });
});
