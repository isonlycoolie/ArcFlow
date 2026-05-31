# TypeScript SDK installation

**Audience:** `[developer]`

The TypeScript SDK publishes to npm as `arcflow`. The package bundles a prebuilt native N-API module and compiles TypeScript to plain JavaScript with zero production npm dependencies.

## Requirements

| Requirement | Version / notes |
|-------------|-----------------|
| Node.js | 18 or newer |
| npm or compatible package manager | For install and scripts |
| Rust toolchain | Required only for local native rebuild from monorepo |

## Install from npm (consumer projects)

```bash
npm install arcflow
```

Verify:

```bash
node --input-type=module -e "import { Agent, Workflow } from 'arcflow'; console.log('import ok')"
```

Set provider keys before live LLM runs:

| Provider | Environment variable |
|----------|---------------------|
| OpenAI | `OPENAI_API_KEY` |
| Anthropic | `ANTHROPIC_API_KEY` |
| Gemini | `GEMINI_API_KEY` |

## Install from monorepo (local development)

From repository root:

```bash
cd sdk-typescript
npm install
npm run build
```

Verify against local build:

```bash
node --input-type=module -e "import { Agent, Workflow } from './index.js'; console.log('import ok')"
```

The build compiles TypeScript and links `index.native.js` against `arcflow-core`.

## Platform notes

| Platform | Notes |
|----------|-------|
| Linux x64 | Prebuilt binary in published package |
| macOS arm64 / x64 | Prebuilt binary in published package |
| Windows x64 | Prebuilt binary; ensure MSVC runtime installed for local rebuild |

If the prebuilt binary does not match your platform, build from source in `sdk-typescript/` with Rust installed.

## Optional backends

Same environment variables as Python for Postgres recovery and Qdrant vector memory when running workflows that need them:

```bash
export ARCFLOW_POSTGRESQL_URL=postgresql://arcflow:arcflow@localhost:5432/arcflow
export ARCFLOW_QDRANT_URL=http://localhost:6333
```

Start local services:

```bash
docker compose -f docker/docker-compose.dev.yml up -d
```

Run migrations before server or recovery-backed runs:

```bash
cargo run -p arcflow-cli -- migrate up
```

## Testing helpers

Vitest-oriented stubs ship in the package:

```typescript
import { buildTestExecConfig, enableStubMode } from "arcflow";
```

Use in unit tests to avoid live LLM calls. See `sdk-typescript/tests/` for patterns.

## Troubleshooting

