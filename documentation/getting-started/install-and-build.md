# Install and build

**Audience:** `[developer]`

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
python3 -m venv .venv
source .venv/bin/activate
cd sdk-python
pip install maturin
maturin develop
pip install -e ".[dev]"
```

### Windows (PowerShell)

```powershell
python -m venv .venv
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
