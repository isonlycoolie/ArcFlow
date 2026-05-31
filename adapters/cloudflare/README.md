# ArcFlow Cloudflare Worker adapter (alpha)

Skeleton worker that loads `arcflow-wasm` and exposes `POST /run`.

## Prerequisites

- Rust `wasm32-unknown-unknown` target
- [wasm-bindgen-cli](https://rustwasm.github.io/wasm-bindgen/): `cargo install wasm-bindgen-cli`
- [Wrangler](https://developers.cloudflare.com/workers/wrangler/)

## Build WASM

From repo root:

```bash
bash scripts/build-wasm.sh
```

## Local dev

```bash
cd adapters/cloudflare
npx wrangler dev
```

## Example request

```bash
curl -X POST http://localhost:8787/run \
  -H "content-type: application/json" \
  -d '{"workflow":{"id":"00000000-0000-0000-0000-000000000001","name":"echo","execution_mode":"Linear","steps":[{"id":"00000000-0000-0000-0000-000000000002","agent_id":"00000000-0000-0000-0000-000000000003","order":0}]},"agents":[{"id":"00000000-0000-0000-0000-000000000003","name":"echo","role":"assistant","instructions":"Echo."}],"input":"hello"}'
```

See [docker/edge-deployment-cloudflare.md](../../docker/edge-deployment-cloudflare.md) for limitations.
