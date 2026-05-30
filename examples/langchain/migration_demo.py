#!/usr/bin/env python3
"""LangChain → ArcFlow migration demo (Phase 2.2 MVP).

Requires optional deps: pip install 'arcflow[langchain]' (from sdk-python/).
"""

from __future__ import annotations


def main() -> None:
    try:
        from arcflow import Agent, Workflow
        from arcflow_langchain import (
            from_langchain_tool,
            langgraph_to_arcflow,
            to_arcflow_step,
        )
    except ImportError as exc:
        print(
            "Skip: install langchain extras — pip install 'arcflow[langchain]' "
            f"({exc})"
        )
        return

    # --- Prompt adapter (LangChain PromptTemplate or mock) ---
    try:
        from langchain_core.prompts import PromptTemplate

        prompt = PromptTemplate.from_template("Answer concisely: {question}")
    except ImportError:
        prompt = type(
            "PromptTemplate",
            (),
            {"template": "Answer concisely: {question}", "input_variables": ["question"]},
        )()

    answer_agent = to_arcflow_step(prompt, name="answer")

    # --- Tool adapter (duck-typed if langchain-community not installed) ---
    class _EchoTool:
        name = "echo"
        description = "Echo input"
        args_schema = None

        def _run(self, text: str) -> str:
            return text

    echo = from_langchain_tool(_EchoTool())
    worker = Agent(
        name="worker",
        role="assistant",
        instructions="Use tools when helpful.",
        tools=(echo,),
    )

    linear = Workflow("langchain_linear")
    linear.step(answer_agent)
    linear.step(worker)
    print(f"linear workflow: {linear._name}, steps={len(linear._steps)}")

    # --- LangGraph heuristic (mock 3-node graph) ---
    mock_graph = type(
        "CompiledGraph",
        (),
        {
            "entry_point": "plan",
            "nodes": {"plan": {}, "act": {}, "summarize": {}},
            "edges": [("plan", "act"), ("act", "summarize"), ("summarize", "__end__")],
        },
    )()

    graph_wf = langgraph_to_arcflow(mock_graph, workflow_name="langgraph_demo")
    print(
        f"graph workflow: nodes={len(graph_wf._graph_nodes)}, "
        f"entry={graph_wf._entry_node}"
    )


if __name__ == "__main__":
    main()
