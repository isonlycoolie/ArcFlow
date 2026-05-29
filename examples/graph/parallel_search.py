#!/usr/bin/env python3
"""Parallel search graph: fan-out to two sources, converge at synthesize."""

from arcflow import Agent, Workflow


def main() -> None:
    router = Agent(name="router", role="planner", instructions="Split query into searches")
    search_web = Agent(name="search_web", role="researcher", instructions="Web search branch")
    search_docs = Agent(name="search_docs", role="researcher", instructions="Docs search branch")
    synthesize = Agent(name="synthesize", role="writer", instructions="Merge findings")

    wf = (
        Workflow("parallel_search", graph=True)
        .max_iterations(10)
        .node("router", router)
        .node("search_web", search_web)
        .node("search_docs", search_docs)
        .node("synthesize", synthesize)
        .set_entry("router")
        .add_edge("router", "search_web")
        .add_edge("router", "search_docs")
        .add_edge("search_web", "synthesize")
        .add_edge("search_docs", "synthesize")
        .add_edge("synthesize", None)
    )

    result = wf.run("query=ArcFlow graph execution parallel fan-out")
    print(f"run_id={result.run_id} steps={result.step_count}")


if __name__ == "__main__":
    main()
