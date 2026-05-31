#!/usr/bin/env python3
# Graph fan-out/join — merge web and internal research into a launch competitive brief.

from arcflow import Agent, Workflow


def main() -> None:
    router = Agent(
        name="router",
        role="research_planner",
        instructions="Split the launch query into web and internal doc search tasks.",
    )
    search_web = Agent(
        name="search_web",
        role="web_researcher",
        instructions="Summarize public news and competitor pages for the product category.",
    )
    search_docs = Agent(
        name="search_docs",
        role="internal_researcher",
        instructions="Summarize internal strategy docs and prior launch retros.",
    )
    synthesize = Agent(
        name="synthesize",
        role="pm_analyst",
        instructions="Merge both branches into a one-page competitive brief with risks and talking points.",
    )

    wf = (
        Workflow("launch_competitive_brief", graph=True)
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
        .join_node("synthesize", ["search_web", "search_docs"])
        .add_edge("synthesize", None)
    )

    result = wf.run(
        "query=ArcFlow graph execution — prepare competitive brief for Q3 agent platform launch"
    )
    print(f"run_id={result.run_id} steps={result.step_count}")


if __name__ == "__main__":
    main()
