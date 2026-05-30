"""
Phase 2.1 streaming demo — step events on stub path (no API key required).

Usage: python examples/streaming/chat_stream.py
"""

from __future__ import annotations

import asyncio

from arcflow import Agent, Workflow


async def main() -> None:
    wf = Workflow("chat_stream_demo")
    wf.step(Agent(name="assistant", role="helper", instructions="Reply briefly."))

    print("Streaming events:")
    event_count = 0
    async for event in wf.run_stream("Hello from ArcFlow"):
        event_count += 1
        if event.type == "token":
            print(event.text, end="", flush=True)
        elif event.type == "step_start":
            print(f"\n[step start: {event.step_id}]")
        elif event.type == "step_complete":
            print(f"[step complete: {event.step_id} in {event.duration_ms}ms]")
        else:
            print(f"[{event.type}]")
    if event_count == 0:
        raise RuntimeError("Expected at least one stream event.")
    print("\nDone.")


if __name__ == "__main__":
    asyncio.run(main())
