/**
 * Phase 2.1 streaming demo — step events on stub path (no API key required).
 *
 * Usage: npm run build && npx tsx examples/streaming/chat_stream.ts
 */
import { Agent, Workflow } from "../../sdk-typescript/index.js";

async function main(): Promise<void> {
  const wf = new Workflow({ name: "chat_stream_demo" });
  wf.step(
    new Agent({
      name: "assistant",
      role: "helper",
      instructions: "Reply briefly.",
    }),
  );

  process.stdout.write("Streaming events:\n");
  let eventCount = 0;
  for await (const event of wf.runStream("Hello from ArcFlow")) {
    eventCount += 1;
    switch (event.type) {
      case "token":
        process.stdout.write(event.text);
        break;
      case "step_start":
        process.stdout.write(`\n[step start: ${event.step_id}]\n`);
        break;
      case "step_complete":
        process.stdout.write(
          `[step complete: ${event.step_id} in ${event.duration_ms}ms]\n`,
        );
        break;
      default:
        process.stdout.write(`[${event.type}]\n`);
    }
  }
  if (eventCount === 0) {
    throw new Error("Expected at least one stream event.");
  }
  process.stdout.write("\nDone.\n");
}

main().catch((err: unknown) => {
  console.error(err);
  process.exit(1);
});
