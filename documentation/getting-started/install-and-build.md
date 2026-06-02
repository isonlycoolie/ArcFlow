# Install and build


ArcFlow ships two in-process SDKs (Python and TypeScript) that bind to the same Rust engine (`arcflow-core`). Both packages aim for zero runtime dependencies beyond what the native extension needs. Python builds the extension with maturin from source in this repository. TypeScript can be installed from npm with a prebuilt native binary, or built locally with `cargo` and `npm`.

## Prerequisites

| Requirement | Python SDK | TypeScript SDK |
|-------------|--------------|----------------|
| Python 3.9+ | Required | Not used |
| Node.js 18+ | Not used | Required |
| Rust toolchain (`rustup`) | Required for source build | Required for local native build |
| Docker (optional) | Memory backends, server quickstart | Same |

Install Rust from [https://rustup.rs](https://rustup.rs). On Windows, use the MSVC toolchain and ensure Visual Studio Build Tools are present so native crates compile.

## Python SDK (maturin / pip)

From the repository root, build and install the extension in editable mode:

```bash
cd sdk-python
pip install maturin
maturin develop
pip install -e ".[dev]"
```

### macOS and Linux

The commands above work unchanged on macOS and Linux. Use a virtual environment if your system Python is managed (recommended):

```bash
python3 -m venv.venv
source.venv/bin/activate
cd sdk-python
pip install maturin
maturin develop
pip install -e ".[dev]"
```

### Windows (PowerShell)

```powershell
python -m venv.venv
.\.venv\Scripts\Activate.ps1
cd sdk-python
pip install maturin
maturin develop
pip install -e ".[dev]"
```

If `maturin develop` fails with a linker error, confirm the Rust MSVC target is installed (`rustup default stable-msvc` on Windows).

### Verify Python installation

This command imports the package and runs a one-step workflow without any API keys:

```bash
python -c "
from arcflow import Agent, Workflow
wf = Workflow('install-check')
wf.step(Agent(name='writer', role='author', instructions='Reply briefly.'))
result = wf.run('hello')
assert result.step_count == 1
assert len(result.output) > 0
print('arcflow python ok', result.run_id)
"
```

Expected output includes a line like `arcflow python ok` followed by a UUID. If import fails, re-run `maturin develop` from `sdk-python/` with the same virtual environment activated.

## TypeScript SDK (npm)

### Published package

For application projects outside this monorepo:

```bash
npm install arcflow
```

The published tarball includes a prebuilt `.node` binary for common platforms. No separate `cargo` step is required.

### Local development (this repository)

```bash
cd sdk-typescript
npm install
npm run build
```

`npm run build` compiles the native binding (`cargo build --release -p arcflow-node`) and TypeScript sources (`tsc`).

### macOS and Linux

After `npm run build`, run tests to confirm the native module loads:

```bash
npm test
```

### Windows

Use the same `npm install` and `npm run build` sequence. If the native build fails, verify Rust MSVC and that no antivirus is locking `target/release/` artifacts.

### Verify TypeScript installation

From `sdk-typescript/` after a local build:

```bash
node --input-type=module -e "
import { Agent, Workflow } from './index.js';
const wf = new Workflow({ name: 'install-check' });
wf.step(new Agent({ name: 'writer', role: 'author', instructions: 'Reply briefly.' }));
const result = await wf.run('hello');
if (result.stepCount !== 1 || !result.output) throw new Error('unexpected result');
console.log('arcflow typescript ok', result.runId);
"
```

In a consumer project that installed from npm, replace the import path with `'arcflow'`.

## What you have after install

Both SDKs delegate execution to the Rust runtime. Python and TypeScript only declare workflow structure (agents, steps, optional tools and memory). The default agent implementation requires no LLM API key for the verification commands above or for [First workflow in five minutes](first-workflow-in-five-minutes.md).

Optional backends (PostgreSQL for persistent memory and recovery, Qdrant for vector memory) start with Docker:

```bash
docker compose -f docker/docker-compose.dev.yml up -d
export ARCFLOW_POSTGRESQL_URL=postgresql://arcflow:arcflow@localhost:5432/arcflow
export ARCFLOW_QDRANT_URL=http://localhost:6333
```

Embedded SDK runs do not require Postgres unless you enable recovery or registry features.

## Next steps

| Goal | Document |
|------|----------|
| Full curriculum (recommended) | [Getting started README](README.md) |
| Fastest first run (Python, no API key) | [First workflow in five minutes](first-workflow-in-five-minutes.md) |
| Python with optional OpenAI | [Python quickstart](quickstart-python.md) |
| TypeScript with optional OpenAI | [TypeScript quickstart](quickstart-typescript.md) |
| HTTP integration | [Server API quickstart](quickstart-server-api.md) |
| Public website chat widget | [Static site chatbot](paths/static-site-chatbot.md) |
