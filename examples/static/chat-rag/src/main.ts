import {
  Agent,
  ArcFlowClient,
  MemoryConfig,
  StepForm,
  Workflow,
} from "@arcflow/static";

const baseUrl = import.meta.env.VITE_ARCFLOW_URL ?? "http://localhost:8080";
const apiKey = import.meta.env.VITE_ARCFLOW_KEY ?? "dev-secret";

const bot = new Agent({
  name: "support",
  role: "support",
  instructions: "Answer using the knowledge base. Be concise.",
  memory: new MemoryConfig({
    type: "Vector",
    namespace: "support-kb",
    embedding: "stub/384",
  }),
});

const workflow = new Workflow({ name: "static_chat", runtime: baseUrl }).step(bot);
const client = new ArcFlowClient({ baseUrl, apiKey, mode: "direct" });
const form = new StepForm();

const input = document.getElementById("input") as HTMLTextAreaElement;
const output = document.getElementById("output") as HTMLPreElement;
const send = document.getElementById("send") as HTMLButtonElement;

send.addEventListener("click", async () => {
  const message = input.value.trim();
  if (!message) return;
  output.textContent = "Running...";
  form.addTurn("user", message);
  try {
    const result = await client.runWorkflow(workflow, message, {
      initialState: form.toInitialState(),
    });
    form.addTurn("assistant", result.output);
    output.textContent = result.output;
  } catch (err) {
    output.textContent = err instanceof Error ? err.message : String(err);
  }
});
