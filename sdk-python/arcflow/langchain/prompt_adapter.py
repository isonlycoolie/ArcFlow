"""LangChain prompt templates → ArcFlow Agent steps."""

from __future__ import annotations

from typing import Any

from arcflow.agent import Agent

from arcflow.langchain.deps import require_langchain_core


def _extract_template_string(prompt: Any) -> str:
    if isinstance(prompt, str):
        return prompt.strip()
    template = getattr(prompt, "template", None)
    if isinstance(template, str) and template.strip():
        return template.strip()
    messages = getattr(prompt, "messages", None)
    if messages:
        parts: list[str] = []
        for msg in messages:
            if isinstance(msg, tuple) and len(msg) >= 2:
                parts.append(str(msg[1]))
            elif hasattr(msg, "prompt") and hasattr(msg.prompt, "template"):
                parts.append(str(msg.prompt.template))
            else:
                parts.append(str(msg))
        joined = "\n".join(p for p in parts if p.strip())
        if joined.strip():
            return joined.strip()
    if hasattr(prompt, "format") and callable(prompt.format):
        try:
            keys = getattr(prompt, "input_variables", []) or []
            sample = {k: f"{{{k}}}" for k in keys}
            return str(prompt.format(**sample)).strip()
        except (TypeError, ValueError, KeyError):
            pass
    text = str(prompt).strip()
    if text:
        return text
    raise ValueError(
        "[ArcFlow] Could not extract prompt text from LangChain template."
    )


def to_arcflow_step(
    prompt_template: Any,
    *,
    name: str = "step",
    role: str = "assistant",
    model: str = "default",
) -> Agent:
    """Map a LangChain ``PromptTemplate`` (or compatible) to an ArcFlow ``Agent``."""
    require_langchain_core()
    instructions = _extract_template_string(prompt_template)
    return Agent(
        name=name,
        role=role,
        instructions=instructions,
        model=model,
    )
