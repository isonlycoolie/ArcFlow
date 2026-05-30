#!/usr/bin/env python3
"""Reflection loop: generate draft, critique, then revise."""

from arcflow import Agent, Workflow


def main() -> None:
    draft = Agent(name="draft", role="writer", instructions="Write first draft")
    critique = Agent(name="critique", role="reviewer", instructions="Find weaknesses")
    revise = Agent(name="revise", role="editor", instructions="Apply improvements")

    wf = (
        Workflow("reflection_loop", graph=True)
        .max_iterations(3)
        .node("draft", draft)
        .node("critique", critique)
        .node("revise", revise)
        .set_entry("draft")
        .add_edge("draft", "critique")
        .add_edge("critique", "revise")
        .add_edge("revise", "draft", condition="needs_more")
        .add_edge("revise", None)
    )

    result = wf.run("topic=Benefits of local-first agent runtimes")
    print(f"run_id={result.run_id} steps={result.step_count}")


if __name__ == "__main__":
    main()
