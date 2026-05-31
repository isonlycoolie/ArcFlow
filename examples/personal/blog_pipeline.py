"""Personal blog pipeline: research → write → SEO check (Phase 2-Pro)."""

from __future__ import annotations

from arcflow import Agent, Workflow


def main() -> None:
    researcher = Agent(name="researcher", role="researcher", instructions="Collect topic facts.")
    writer = Agent(name="writer", role="writer", instructions="Draft a blog post.")
    seo = Agent(name="seo", role="seo", instructions="Suggest title and meta description.")
    wf = (
        Workflow("blog_pipeline")
        .step(researcher)
        .step(writer)
        .step(seo)
    )
    print(wf.run("Write about context assembly in agent workflows").output)


if __name__ == "__main__":
    main()
