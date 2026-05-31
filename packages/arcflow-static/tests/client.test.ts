import { ArcFlowClient, StepForm } from "../src/index.js";

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

describe("ArcFlowClient", () => {
  it("constructs with base URL", () => {
    const client = new ArcFlowClient({ baseUrl: "http://localhost:8080", apiKey: "k" });
    expect(client).toBeDefined();
  });
});
