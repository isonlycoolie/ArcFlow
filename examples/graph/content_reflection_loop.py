#!/usr/bin/env python3
# Draft–critique–revise graph loop with conditional re-entry for content quality.

from arcflow import Agent, Workflow


def main() -> None:
    draft = Agent(
        name="draft",
        role="writer",
        instructions="Write the first draft for the topic. Target practitioner audience.",
    )
    critique = Agent(
        name="critique",
        role="senior_editor",
        instructions="List structural weaknesses. End with NEEDS_MORE or APPROVED.",
    )
    revise = Agent(
        name="revise",
        role="editor",
        instructions="Apply critique. Output route=needs_more or route=done in the last line.",
    )

    wf = (
        Workflow("content_reflection_loop", graph=True)
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

    result = wf.run("topic=Benefits of local-first agent runtimes for regulated teams")
    print(f"run_id={result.run_id} steps={result.step_count}")


if __name__ == "__main__":
    main()
