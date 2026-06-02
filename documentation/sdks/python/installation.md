# Python SDK installation

The Python SDK ships as a native extension built with maturin against `arcflow-core`. Published wheels are on PyPI as **`arcflow`** (Linux, macOS, Windows; Python 3.9–3.12).

## Install from PyPI

```bash
pip install arcflow
```

Pin a release:

```bash
pip install "arcflow==0.3.0"
```

Wheels are built in CI when a maintainer pushes a tag `sdk-python/vX.Y.Z`. See [releasing](releasing.md) for the maintainer process.

## Install from source (monorepo)

Use this when developing ArcFlow itself or before a wheel exists for your platform.

## Requirements

| Requirement | Version / notes |
|-------------|-----------------|
| Python | 3.9 or newer |
| Rust toolchain | `rustup` with a working linker (MSVC on Windows) |
| pip | Current pip for editable install |
| maturin | Builds and installs the PyO3 extension |

Optional for memory backends and server integration:

| Backend | Environment variable |
|---------|---------------------|
| PostgreSQL (persistent memory, recovery) | `ARCFLOW_POSTGRESQL_URL` |
| Qdrant (vector memory) | `ARCFLOW_QDRANT_URL` |
| OTLP trace export | `ARCFLOW_OTLP_ENDPOINT` |

## Install from repository root

```bash
cd sdk-python
pip install maturin
maturin develop
pip install -e ".[dev]"
```

Verify:

```bash
python -c "from arcflow import Agent, Workflow; print('import ok')"
```

## Virtual environment (recommended)

### macOS and Linux

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

On Windows, install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the C++ workload so Rust native crates link correctly.

## LangChain adapter (optional)

LangChain conversion lives in the `arcflow_langchain` package inside `sdk-python/`. It is not installed by default unless your editable install includes it:

```bash
pip install -e ".[dev,langchain]"
```

Import path:

```python
from arcflow.langchain import from_langchain_tool, langgraph_to_arcflow
```

If the extra is not installed, `import arcflow.langchain` fails with a missing dependency error.

## Local memory stack (Docker)

For persistent or vector memory during development:

```bash
docker compose -f docker/docker-compose.dev.yml up -d
export ARCFLOW_POSTGRESQL_URL=postgresql://arcflow:arcflow@localhost:5432/arcflow
export ARCFLOW_QDRANT_URL=http://localhost:6333
```

Session and shared memory types work without Docker.

## Provider API keys

Set keys in the environment before calling `run(..., provider=...)`:

| Provider | Environment variable |
|----------|---------------------|
| OpenAI | `OPENAI_API_KEY` |
| Anthropic | `ANTHROPIC_API_KEY` |
| Gemini | `GEMINI_API_KEY` |

Credentials are never passed through workflow JSON. The runtime reads them at execution time.

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| `ImportError: arcflow._arcflow_binding` | Extension not built | Run `maturin develop` from `sdk-python/` |
| Linker errors on Windows | Missing MSVC | Install VS Build Tools, restart shell |
| `InfrastructureUnavailableError` on vector run | Qdrant URL unset or down | Start Docker stack, export `ARCFLOW_QDRANT_URL` |
| `InfrastructureUnavailableError` on recovery | Postgres URL unset | Export `ARCFLOW_POSTGRESQL_URL`, run migrations |

For shared install guidance across Python and TypeScript, see [install and build](../../getting-started/install-and-build.md).
