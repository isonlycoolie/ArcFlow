import http from "node:http";
import { Agent, Anthropic, Gemini, OpenAI, Workflow } from "../index.js";

function agent() {
  return new Agent({
    name: "writer",
    role: "author",
    instructions: "Write a short reply.",
  });
}

async function withMockServer(
  handler: (req: http.IncomingMessage, res: http.ServerResponse) => void,
  fn: (baseUrl: string) => Promise<void>,
): Promise<void> {
  const server = http.createServer(handler);
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", () => resolve()));
  const addr = server.address();
  if (!addr || typeof addr === "string") {
    server.close();
    throw new Error("failed to bind mock server");
  }
  const baseUrl = `http://127.0.0.1:${addr.port}`;
  try {
    await fn(baseUrl);
  } finally {
    await new Promise<void>((resolve, reject) =>
      server.close((err) => (err ? reject(err) : resolve())),
    );
  }
}

describe("provider swap", () => {
  it("openai matches stub step count", async () => {
    await withMockServer((_req, res) => {
      res.writeHead(200, { "Content-Type": "application/json" });
      res.end(
        JSON.stringify({
          model: "gpt-4o",
          choices: [{ message: { content: "ok" }, finish_reason: "stop" }],
          usage: { prompt_tokens: 3, completion_tokens: 2, total_tokens: 5 },
        }),
      );
    }, async (baseUrl) => {
      process.env.OPENAI_API_KEY = "test-key";
      process.env.ARCFLOW_OPENAI_API_ENDPOINT = `${baseUrl}/v1/chat/completions`;
      const stub = await new Workflow({ name: "stub" })
        .step(agent())
        .run("hello");
      const provider = await new Workflow({ name: "openai" })
        .step(agent())
        .run("hello", { provider: new OpenAI({ model: "gpt-4o" }) });
      expect(provider.stepCount).toBe(stub.stepCount);
      expect(provider.stepCount).toBe(1);
    });
  });

  it("anthropic matches stub step count", async () => {
    await withMockServer((_req, res) => {
      res.writeHead(200, { "Content-Type": "application/json" });
      res.end(
        JSON.stringify({
          model: "claude-3-5-sonnet-20241022",
          content: [{ text: "ok" }],
          stop_reason: "end_turn",
          usage: { input_tokens: 4, output_tokens: 2 },
        }),
      );
    }, async (baseUrl) => {
      process.env.ANTHROPIC_API_KEY = "test-key";
      process.env.ARCFLOW_ANTHROPIC_API_ENDPOINT = `${baseUrl}/v1/messages`;
      const stub = await new Workflow({ name: "stub" })
        .step(agent())
        .run("hello");
      const provider = await new Workflow({ name: "anthropic" })
        .step(agent())
        .run("hello", {
          provider: new Anthropic({ model: "claude-3-5-sonnet-20241022" }),
        });
      expect(provider.stepCount).toBe(stub.stepCount);
    });
  });

  it("gemini matches stub step count", async () => {
    await withMockServer((_req, res) => {
      res.writeHead(200, { "Content-Type": "application/json" });
      res.end(
        JSON.stringify({
          candidates: [
            {
              content: { role: "model", parts: [{ text: "ok" }] },
              finishReason: "STOP",
            },
          ],
          usageMetadata: {
            promptTokenCount: 3,
            candidatesTokenCount: 2,
            totalTokenCount: 5,
          },
        }),
      );
    }, async (baseUrl) => {
      process.env.GEMINI_API_KEY = "test-key";
      process.env.ARCFLOW_GEMINI_API_ENDPOINT = `${baseUrl}/v1beta/models`;
      const stub = await new Workflow({ name: "stub" })
        .step(agent())
        .run("hello");
      const provider = await new Workflow({ name: "gemini" })
        .step(agent())
        .run("hello", { provider: new Gemini({ model: "gemini-1.5-pro" }) });
      expect(provider.stepCount).toBe(stub.stepCount);
    });
  });
});
