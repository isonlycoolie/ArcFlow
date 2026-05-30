#!/usr/bin/env python3
"""ReAct-style graph: think -> act -> observe loop (bounded iterations)."""

from arcflow import Agent, Workflow


def main() -> None:
    think = Agent(name="think", role="reasoner", instructions="Plan next action")
    act = Agent(name="act", role="actor", instructions="Execute tool call")
    observe = Agent(name="observe", role="observer", instructions="Interpret result")

    wf = (
        Workflow("react_agent", graph=True)
        .max_iterations(5)
        .node("think", think)
        .node("act", act)
        .node("observe", observe)
        .set_entry("think")
        .add_edge("think", "act")
        .add_edge("act", "observe")
        .add_edge("observe", "think")
    )

    result = wf.run("question=What is the capital of France?")
    print(f"run_id={result.run_id} steps={result.step_count} output={result.output[:120]}")


if __name__ == "__main__":
    main()
