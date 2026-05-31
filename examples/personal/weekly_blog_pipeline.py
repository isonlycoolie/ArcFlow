# Weekly blog pipeline — research, draft, and SEO in three linear steps.

from __future__ import annotations

import sys

from arcflow import Agent, Workflow

DEFAULT_TOPIC = "Context assembly policies in multi-step agent workflows"


def main() -> None:
    topic = sys.argv[1] if len(sys.argv) > 1 else DEFAULT_TOPIC

    researcher = Agent(
        name="researcher",
        role="research_assistant",
        instructions=(
            "Collect 5 bullet facts and 3 reputable reference angles for the blog topic. "
            "Focus on practitioner value, not hype."
        ),
    )
    writer = Agent(
        name="writer",
        role="technical_writer",
        instructions=(
            "Write a ~800 word blog draft in markdown using prior research. "
            "Use clear headings, one code-free example, and a short conclusion."
        ),
    )
    seo = Agent(
        name="seo_editor",
        role="seo_specialist",
        instructions=(
            "Propose: (1) title under 60 chars, (2) meta description under 155 chars, "
            "(3) three slug keywords. Do not rewrite the full draft."
        ),
    )
    wf = Workflow("weekly_blog_pipeline").step(researcher).step(writer).step(seo)
    result = wf.run(f"Topic: {topic}")
    print(result.output)
    print(f"\nrun_id={result.run_id} steps={result.step_count}")


if __name__ == "__main__":
    main()
