import { ArcFlowClient } from "../src/client.js";

describe("relay mode paths", () => {
  it("uses site-scoped runs path when siteId is set", () => {
    const client = new ArcFlowClient({
      baseUrl: "https://relay.example.com",
      apiKey: "st_live_test",
      mode: "relay",
      siteId: "s_abc123",
      useArcFlowHeader: false,
    });
    expect(client.mode).toBe("relay");
    expect((client as unknown as { runsBasePath(): string }).runsBasePath()).toBe(
      "/v1/sites/s_abc123/runs",
    );
  });

  it("parses site id from baseUrl", () => {
    const client = new ArcFlowClient({
      baseUrl: "https://relay.example.com/v1/sites/s_xyz",
      apiKey: "st_live_test",
      mode: "relay",
    });
    expect((client as unknown as { runsBasePath(): string }).runsBasePath()).toBe(
      "/v1/sites/s_xyz/runs",
    );
  });

  it("bff mode is alias for relay", () => {
    const client = new ArcFlowClient({
      baseUrl: "https://relay.example.com/v1/sites/s_xyz",
      apiKey: "token",
      mode: "bff",
    });
    expect(client.mode).toBe("relay");
  });
});

describe("runPublished payload", () => {
  it("builds workflow_ref without inline workflow", async () => {
    const calls: Array<{ method: string; path: string; body?: unknown }> = [];
    const original = global.fetch;
    global.fetch = (async (input: RequestInfo | URL, init?: RequestInit) => {
      calls.push({
        method: init?.method ?? "GET",
        path: String(input),
        body: init?.body ? JSON.parse(String(init.body)) : undefined,
      });
      return new Response(
        JSON.stringify({ run_id: "r1", trace_id: "t1", status: "completed", result: { output: "ok", step_count: 1 } }),
        { status: 200, headers: { "Content-Type": "application/json" } },
      );
    }) as typeof fetch;

    const client = new ArcFlowClient({
      baseUrl: "http://localhost:8090/v1/sites/s_test",
      apiKey: "st_live_test",
      mode: "relay",
    });
    await client.runPublished("chat", "^1.0.0", "Hello");

    expect(calls[0]?.path).toContain("/v1/sites/s_test/runs");
    expect(calls[0]?.body).toMatchObject({
      workflow_ref: { name: "chat", version: "^1.0.0" },
      input: "Hello",
    });
    global.fetch = original;
  });
});
