/**
 * Production entry — landing-page support chat via ArcFlow Relay.
 *
 * Setup (dashboard): create site → upload knowledge → publish chat workflow.
 * Frontend: two env vars + runPublished(). No inline workflow definitions here.
 */
import { ArcFlowClient, StepForm } from "@arcflow/static";

const relayUrl = import.meta.env.VITE_ARCFLOW_RELAY_URL;
const siteToken = import.meta.env.VITE_ARCFLOW_SITE_TOKEN;

if (!relayUrl || !siteToken) {
  throw new Error(
    "Set VITE_ARCFLOW_RELAY_URL and VITE_ARCFLOW_SITE_TOKEN from ArcFlow Dashboard → Sites.",
  );
}

const client = new ArcFlowClient({
  baseUrl: relayUrl,
  apiKey: siteToken,
  mode: "relay",
});
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
    const result = await client.runPublished("chat", "^1.0.0", message, {
      initialState: form.toInitialState(),
    });
    form.addTurn("assistant", result.output);
    output.textContent = result.output;
  } catch (err) {
    output.textContent = err instanceof Error ? err.message : String(err);
  }
});
