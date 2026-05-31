# Edge deployment on Cloudflare Workers

ArcFlow’s edge runtime is an alpha WASM build for low-latency stub workflows at the CDN edge. It is not a drop-in replacement for the self-hosted server — the server runs the full engine with Postgres recovery, vector memory, and real LLM providers.

## Why edge exists

Teams that already serve traffic from Cloudflare Workers need agent-style workflows without round-tripping to a central Rust server. LangGraph and similar stacks cannot run natively on Workers; a `wasm32` ArcFlow bundle can.

## Build

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli   # once
bash scripts/build-wasm.sh
```

Output lands in `runtime/arcflow-wasm/pkg/` for the Worker adapter.

## Deploy (skeleton)

```bash
cd adapters/cloudflare
npx wrangler deploy
```

Configure secrets with `wrangler secret put` — never bake API keys into the wasm artifact.

## Limitations matrix

| Feature | Self-hosted server | Edge WASM (alpha) |
|---------|-------------------|-------------------|
| Linear workflows | Full | Stub agent responses |
| Graph workflows | Full | Not supported |
| Postgres recovery | Full | No |
| Vector / Qdrant memory | Full | No |
| Real LLM providers | Full | No (fetch provider planned) |
| Tool execution | Full | No arbitrary TCP |
| OpenTelemetry export | Opt-in OTLP | Not on edge alpha |
| HITL | Postgres-backed | Limited / external only |

Worker platform limits still apply (memory ~128MB, CPU time per request).

## When NOT to use edge runtime

- Workflows need graph execution, recovery, or vector RAG.
- You require audit-grade OTel in the same pipeline as the server.
- Steps call internal tools over private networks.
- You need semver registry resolution from Postgres (`workflow_ref`).

Use [`arcflow-server`](../server/arcflow-server/) or Relay for those cases.

## Example curl (local wrangler dev)

```bash
curl -X POST http://localhost:8787/run \
  -H "content-type: application/json" \
  -d '{"workflow":{"id":"00000000-0000-0000-0000-000000000001","name":"echo","execution_mode":"Linear","steps":[{"id":"00000000-0000-0000-0000-000000000002","agent_id":"00000000-0000-0000-0000-000000000003","order":0}]},"agents":[{"id":"00000000-0000-0000-0000-000000000003","name":"echo","role":"assistant","instructions":"Echo."}],"input":"hello"}'
```

Expected: JSON with `output`, `step_count`, and `status: "completed"`.

## Verification

| Command | Expect |
|---------|--------|
| `bash scripts/build-wasm.sh` | Creates `runtime/arcflow-wasm/pkg/` |
| `cargo test -p arcflow-wasm` | Host API + wasm export tests pass |

## Next steps (out of scope for alpha)

- Gate `arcflow-core` native deps behind a `wasm` feature and link the real stub engine.
- Fetch-based LLM provider using Worker `fetch`.
- D1-backed recovery (best-effort).
- Deno Deploy adapter.
