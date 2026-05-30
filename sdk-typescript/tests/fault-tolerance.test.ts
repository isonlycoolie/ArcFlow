import http from "node:http";

import {
  Agent,
  OpenAI,
  RetryExhaustedError,
  Workflow,
  WorkflowConfigurationError,
  WorkflowExecutionError,
  WorkflowTimeoutError,
} from "../index.js";

const STUB_FAIL_ROLE = "__fail__";

function agent(name = "writer") {
  return new Agent({
    name,
    role: name,
    instructions: "Write briefly.",
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

describe("fault tolerance", () => {
  it("rejects zero retry attempts", () => {
    const wf = new Workflow({ name: "cfg" }).step(agent());
    expect(() => wf.withRetry(0)).toThrow(WorkflowConfigurationError);
  });

  it("survives mock 429 with retry", async () => {
    let calls = 0;
    await withMockServer((_req, res) => {
      calls += 1;
      if (calls === 1) {
        res.writeHead(429, { "Content-Type": "application/json" });
        res.end(JSON.stringify({ error: { code: "rate_limit_exceeded" } }));
        return;
      }
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
      const wf = new Workflow({ name: "retry_ok" })
        .step(agent())
        .withRetry(3, { backoff: { kind: "constant", delayMs: 1 } });
      const result = await wf.run("input", {
        provider: new OpenAI({ model: "gpt-4o" }),
      });
      expect(result.stepCount).toBe(1);
      expect(calls).toBeGreaterThan(1);
    });
  });

  it("raises RetryExhaustedError when retries fail", async () => {
    await withMockServer((_req, res) => {
      res.writeHead(429, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ error: { code: "rate_limit_exceeded" } }));
    }, async (baseUrl) => {
      process.env.OPENAI_API_KEY = "test-key";
      process.env.ARCFLOW_OPENAI_API_ENDPOINT = `${baseUrl}/v1/chat/completions`;
      const wf = new Workflow({ name: "retry_fail" })
        .step(agent())
        .withRetry(3, { backoff: { kind: "constant", delayMs: 1 } });
      await expect(
        wf.run("input", { provider: new OpenAI({ model: "gpt-4o" }) }),
      ).rejects.toBeInstanceOf(RetryExhaustedError);
    });
  });

  it("enforces step timeout", async () => {
    await withMockServer((_req, res) => {
      setTimeout(() => {
        res.writeHead(200, { "Content-Type": "application/json" });
        res.end(
          JSON.stringify({
            model: "gpt-4o",
            choices: [{ message: { content: "ok" }, finish_reason: "stop" }],
          }),
        );
      }, 2000);
    }, async (baseUrl) => {
      process.env.OPENAI_API_KEY = "test-key";
      process.env.ARCFLOW_OPENAI_API_ENDPOINT = `${baseUrl}/v1/chat/completions`;
      const wf = new Workflow({ name: "step_timeout" })
        .step(agent())
        .withStepTimeout(1);
      await expect(
        wf.run("input", { provider: new OpenAI({ model: "gpt-4o" }) }),
      ).rejects.toBeInstanceOf(WorkflowTimeoutError);
    });
  }, 10_000);

  const postgresUrl = process.env.ARCFLOW_POSTGRESQL_URL;
  (postgresUrl ? it : it.skip)("resumes after partial failure", async () => {
    const ok = agent("writer");
    const fail = new Agent({
      name: "mid",
      role: STUB_FAIL_ROLE,
      instructions: "fail mid",
    });
    const wf = new Workflow({ name: "recovery_flow" })
      .step(ok)
      .step(fail)
      .step(ok)
      .enableRecovery();
    let runId: string | undefined;
    try {
      await wf.run("recovery-input");
    } catch (err) {
      expect(err).toBeInstanceOf(WorkflowExecutionError);
      runId = (err as WorkflowExecutionError).runId;
    }
    expect(runId).toBeTruthy();
    const resumed = await wf.resume(runId!);
    expect(resumed.stepCount).toBe(2);
  });
});
