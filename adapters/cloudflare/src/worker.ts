/// <reference types="@cloudflare/workers-types" />

import wasmUrl from "../../runtime/arcflow-wasm/pkg/arcflow_wasm_bg.wasm";

export interface Env {
  // Bind secrets via wrangler — never embed in the wasm bundle.
  ARCFLOW_ENV?: string;
}

interface ExecuteBody {
  workflow: unknown;
  agents: unknown[];
  input: string;
}

let initPromise: Promise<{ executeWorkflow: (wf: string, input: string) => string }> | null =
  null;

async function loadWasm(): Promise<{ executeWorkflow: (wf: string, input: string) => string }> {
  if (!initPromise) {
    initPromise = (async () => {
      const response = await fetch(wasmUrl);
      const bytes = await response.arrayBuffer();
      const { instantiate } = await import("../../runtime/arcflow-wasm/pkg/arcflow_wasm.js");
      const instance = await instantiate(bytes, {});
      return instance as { executeWorkflow: (wf: string, input: string) => string };
    })();
  }
  return initPromise;
}

export default {
  async fetch(request: Request, _env: Env): Promise<Response> {
    if (request.method !== "POST") {
      return new Response(JSON.stringify({ error: "POST /run with workflow bundle" }), {
        status: 405,
        headers: { "content-type": "application/json" },
      });
    }

    let body: ExecuteBody;
    try {
      body = (await request.json()) as ExecuteBody;
    } catch {
      return new Response(JSON.stringify({ error: "invalid JSON body" }), {
        status: 400,
        headers: { "content-type": "application/json" },
      });
    }

    const bundle = JSON.stringify({ workflow: body.workflow, agents: body.agents });
    const input = JSON.stringify(body.input ?? "");

    try {
      const wasm = await loadWasm();
      const result = wasm.executeWorkflow(bundle, input);
      return new Response(result, { headers: { "content-type": "application/json" } });
    } catch (err) {
      const message = err instanceof Error ? err.message : "wasm execution failed";
      return new Response(JSON.stringify({ error: message }), {
        status: 500,
        headers: { "content-type": "application/json" },
      });
    }
  },
};
