**Audience:** `[developer]`

# Edge WASM (alpha)

`runtime/arcflow-wasm/` builds a WebAssembly module for edge hosts (Cloudflare Workers, experimental CDN workers). Status: **alpha**. Not recommended for production workloads.

## What works today

| Capability | Status |
|------------|--------|
| Linear workflow stub | Alpha |
| `executeWorkflow` JSON export | Alpha |
| Graph workflows | Not supported |
| RAG / vector memory | Not supported |
| Recovery / Postgres | Not supported |
| HITL / external callbacks | Not supported |
| Tools / live LLM providers | Stub echo only |

Full `arcflow-core` linkage is deferred until native dependencies are wasm-gated.

## Artifacts

After build:

```text
runtime/arcflow-wasm/pkg/
  arcflow_wasm.js
  arcflow_wasm_bg.wasm
  arcflow_wasm.d.ts
```

Build from repo (requires `wasm-pack`):

```bash
cd runtime/arcflow-wasm
wasm-pack build --target web
```

## JavaScript API

```javascript
import init, { executeWorkflow } from "./pkg/arcflow_wasm.js";

await init();
const workflowJson = JSON.stringify({
  workflow: {
    id: "...",
    name: "echo",
    execution_mode: "Linear",
    steps: [{ id: "...", agent_id: "...", order: 0 }],
  },
  agents: [{
    id: "...",
    name: "echo",
    role: "assistant",
    instructions: "Echo input.",
  }],
});
const resultJson = executeWorkflow(workflowJson, JSON.stringify("hello"));
```

Rust host entry: `execute_workflow_json` in `runtime/arcflow-wasm/src/lib.rs`.

Errors return JSON strings with `code` and `message` (e.g. `unsupported_mode`, `empty_workflow`).

## Cloudflare Workers pattern (sketch)

```javascript
import wasm from "./arcflow_wasm_bg.wasm";
import { executeWorkflow } from "./arcflow_wasm.js";

export default {
  async fetch(request) {
    const { workflow, input } = await request.json();
    const out = executeWorkflow(JSON.stringify(workflow), JSON.stringify(input));
    return new Response(out, { headers: { "Content-Type": "application/json" } });
  },
};
```

Workers WASM packaging details change with platform versions; treat this as illustrative only.

## When to use WASM vs server/SDK

| Scenario | Recommendation |
|----------|----------------|
| Production chat widget | [static-product/overview.md](../static-product/overview.md) + Relay |
| Backend batch jobs | Python/TS SDK or [server/overview.md](../server/overview.md) |
| Edge latency experiment | WASM alpha with stub only |
| Graph or RAG at edge | Wait for wasm-core parity (not scheduled GA) |

## Production paths

- **arcflow-server** for durable runs, registry, admin
- **Embedded SDK** for in-process LLM and RAG
- **Relay + static SDK** for browser production

WASM alpha does not replace any of the above today.

## Testing

```bash
cargo test -p arcflow-wasm
```

Includes `execute_workflow_json_round_trip` in `lib.rs` tests.

## Related gaps

| ID | Note |
|----|------|
| FP-2 | Server SSE not related to WASM; browser uses trace poll |
| FP-1.01 | Graph recovery partial on server; WASM has no graph |

Maturity: [maturity-and-known-gaps.md](../concepts/maturity-and-known-gaps.md).

**Source:** capabilities reference §20; `runtime/arcflow-wasm/src/lib.rs`, `runtime/arcflow-wasm/Cargo.toml`; Appendix I (WASM column).
