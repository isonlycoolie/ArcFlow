# 04 Common tools bundle


## Before you start

Complete [02 Attach tools to agents](02-attach-tools-to-agents.md). You should know how to pass `tools=(...)` on `Agent`.

## Concept

ArcFlow ships optional **common tools** for demos and prototypes: web search, HTTP fetch, and document read. Instead of redefining schemas and execute handlers, import the bundle and attach it to an agent.

Preferred import (PascalCase facade):

```python
from arcflow.tools import CommonTools

tools = CommonTools.bundle()
agent = Agent(..., tools=CommonTools.bundle())
```

`CommonTools.bundle()` returns a tuple of three `Tool` instances:

| Tool name | Purpose |
|-----------|---------|
| `web_search` | Search via Tavily or Serper when API keys are set |
| `http_fetch` | Fetch a URL when the host is on `ARCFLOW_HTTP_ALLOWLIST` |
| `read_document` | Read a local file or allowlisted URL |

Without API keys or allowlist entries, these tools return JSON error or stub payloads instead of calling the network. Set `ARCFLOW_EMBEDDING_LOCAL_ONLY=1` to force local-only stub behavior for search.

Environment variables (when you want live behavior):

| Variable | Used by |
|----------|---------|
| `TAVILY_API_KEY` or `SERPER_API_KEY` | `web_search` |
| `ARCFLOW_HTTP_ALLOWLIST` | `http_fetch`, `read_document` for URLs |

### Branch and naming note

The `from arcflow.tools import CommonTools` import path ships on the **`feat/sdk-pascalcase-facades`** branch. On current **`main`**, use the legacy module instead:

```python
from arcflow_tools import common_tools

agent = Agent(..., tools=common_tools())
```

The legacy `common_tools()` and `register_common_tools()` functions return the same three tools. After the facade branch merges, prefer `CommonTools.bundle()`.

The same branch introduces PascalCase facades for other helpers. For example, external outcome reporting moves from `report_outcome(...)` to `ExternalOutcome.report(...)` on that branch, with the snake_case function kept as a deprecated alias. On **main** today, import `report_outcome` from `arcflow` directly.

## Example

Save as `common_tools_demo.py`:

```python
from arcflow import Agent, Workflow
from arcflow.tools import CommonTools

researcher = Agent(
    name="researcher",
    role="Researcher",
    instructions="Use web_search or read_document when helpful.",
    tools=CommonTools.bundle(),
)

workflow = Workflow("common-tools-demo")
workflow.step(researcher)

result = workflow.run("Summarize public docs about ArcFlow")
print(result.output[:200], "..." if len(result.output) > 200 else "")
print(f"status={result.status}")
```

If you are on **main** without the facade branch, replace the import block with:

```python
from arcflow_tools import common_tools

researcher = Agent(
    name="researcher",
    role="Researcher",
    instructions="Use web_search or read_document when helpful.",
    tools=common_tools(),
)
```

Run without API keys (stub or error JSON from tools):

```bash
python common_tools_demo.py
```

## Verify

| Check | Expected |
|-------|----------|
| Agent construction | Three tools attached, no duplicate name error |
| No network keys | Run completes; tool results are stub or JSON error strings |
| Local-only mode | With `ARCFLOW_EMBEDDING_LOCAL_ONLY=1`, search returns stub JSON |

Inspect tool names on the agent:

```python
from arcflow.tools import CommonTools

names = [t.name for t in CommonTools.bundle()]
print(names)
```

Expected: `['web_search', 'http_fetch', 'read_document']`.

## Next

| Goal | Document |
|------|----------|
| Full tool loop reference | [Tool execution loop](../../guides/agents-and-tools/tool-execution-loop.md) |
| Provider-backed tool selection | [Python quickstart](../quickstart-python.md) |
| LangChain tool import | `arcflow.langchain.from_langchain_tool` |
