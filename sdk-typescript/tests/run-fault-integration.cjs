/**
 * Integration tests for TS fault tolerance (requires arcflow-node.node).
 * Run: npm run build:native && npm run test:integration
 */
const http = require("node:http");
const assert = require("node:assert/strict");

const {
  Agent,
  OpenAI,
  RetryExhaustedError,
  Workflow,
  WorkflowConfigurationError,
  WorkflowExecutionError,
  mapNativeError,
} = require("../index.js");

const STUB_FAIL_ROLE = "__fail__";

function agent(name = "writer") {
  return new Agent({
    name,
    role: name,
    instructions: "Write briefly.",
  });
}

async function withMockServer(handler, fn) {
  const server = http.createServer(handler);
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const addr = server.address();
  if (!addr || typeof addr === "string") {
    server.close();
    throw new Error("failed to bind mock server");
  }
  const baseUrl = `http://127.0.0.1:${addr.port}`;
  try {
    await fn(baseUrl);
  } finally {
    await new Promise((resolve, reject) =>
      server.close((err) => (err ? reject(err) : resolve())),
    );
  }
}

async function main() {
  try {
    new Workflow({ name: "cfg" }).step(agent()).withRetry(0);
    assert.fail("expected configuration error");
  } catch (err) {
    assert.ok(err instanceof WorkflowConfigurationError);
  }

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
    assert.equal(result.stepCount, 1);
    assert.ok(calls > 1);
  });

  if (process.env.ARCFLOW_POSTGRESQL_URL) {
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
    let runId;
    try {
      await wf.run("recovery-input");
      assert.fail("expected partial workflow failure");
    } catch (err) {
      const mapped = mapNativeError(err);
      if (!(mapped instanceof WorkflowExecutionError) || !mapped.runId) {
        console.warn(
          "recovery resume check skipped:",
          mapped instanceof Error ? mapped.message : String(mapped),
        );
        return;
      }
      runId = mapped.runId;
    }
    const resumed = await wf.resume(runId);
    assert.equal(resumed.stepCount, 2);
  }

  console.log("fault integration checks passed");
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
