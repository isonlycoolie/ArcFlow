#!/usr/bin/env python3
# Bounded ReAct graph — think, act, observe loop for support troubleshooting.

from arcflow import Agent, Workflow


def main() -> None:
    think = Agent(
        name="think",
        role="reasoner",
        instructions="Plan the next action. Stop when you have a final answer.",
    )
    act = Agent(
        name="act",
        role="actor",
        instructions="Execute the planned tool or lookup step.",
    )
    observe = Agent(
        name="observe",
        role="observer",
        instructions="Interpret the action result and decide whether to continue or finish.",
    )

    wf = (
        Workflow("support_react_loop", graph=True)
        .max_iterations(5)
        .node("think", think)
        .node("act", act)
        .node("observe", observe)
        .set_entry("think")
        .add_edge("think", "act")
        .add_edge("act", "observe")
        .add_edge("observe", "think")
    )

    result = wf.run(
        "question=Customer asks: Why did SSO break after certificate rotation? Use knowledge if available."
    )
    preview = (result.output or "")[:200]
    print(f"run_id={result.run_id} steps={result.step_count} output={preview!r}")


if __name__ == "__main__":
    main()
