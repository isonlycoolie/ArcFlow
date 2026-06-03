# ArcFlow Examples

Production-shaped samples that start from a **real problem**, show which ArcFlow capabilities solve it, and include a README with prerequisites and verification steps.

## How to use these examples

| Step | Action |
|------|--------|
| 1 | Read the category README to understand the business problem before the code |
| 2 | Set prerequisites (SDK, env vars, optional Docker stack) |
| 3 | Run the sample |
| 4 | Confirm output / trace matches the **Verify** section |

Most Python samples use `stub` embedding and run without live LLM keys. Swap `embedding` and `provider` for production (OpenAI, Qdrant, Postgres recovery).

## Catalog

| Directory | Real-world problem | ArcFlow capabilities |
|-----------|-------------------|----------------------|
| [education/](education/) | University course assistant overwhelmed by repeat syllabus questions | Vector RAG, namespace isolation |
| [healthcare/](healthcare/) | Clinic staff need protocol lookup with human safety gate | RAG + HITL disclaimer review |
| [support/](support/) | Tier-1 support drowning in repeat SSO/password tickets | Hybrid RAG, persistent namespace |
| [trading/](trading/) | Analyst wants research, thesis, paper trade with approval | Multi-step workflow, tools, HITL |
| [personal/](personal/) | Solo creator publishing weekly technical blog posts | Linear multi-agent pipeline |
| [rag/](rag/) | Docs team Q&A over long internal guides | Chunking, hybrid retrieval, rerank |
| [graph/](graph/) | Research, routing, and quality loops that are not linear | Graph DAG, joins, conditional edges |
| [hitl/](hitl/) | Finance team requires manager sign-off on expenses | Interrupt, approve, recovery |
| [external/](external/) | Government portal callback after form pre-fill | External binding webhook |
| [streaming/](streaming/) | Chat UI needs token-by-token display | SDK `run_stream()` |
| [langchain/](langchain/) | Team migrating LangChain/LangGraph prototypes | Interop adapters |
| [static/](static/) | Marketing site chat without secrets in the browser | Relay, static SDK, dashboard publish |
| [relay/](relay/) | Self-hosted Relay for BYO infrastructure | Docker, site token proxy |

## Shared prerequisites

```bash
# From repo root, Python SDK (developer / editable install)
cd sdk-python && pip install -e .

# Or install the published SDK from PyPI for normal use:
pip install arcflow-sdk

# Optional: full server stack for HITL, external callbacks, static admin
docker compose -f docker/docker-compose.server.yml up -d
```

## Related docs

- [Full capabilities reference](../docs/_draft/ARCFLOW-FULL-CAPABILITIES-REFERENCE.md)
- [Static product vision](../ArcFlow_Improvement_Plans/arcflow-static-product-vision/) (local plan)
