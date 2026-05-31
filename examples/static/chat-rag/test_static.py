"""Validate @arcflow/static chat-rag example structure."""

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parent


def test_chat_rag_files_exist():
    assert (ROOT / "src" / "main.ts").is_file()
    assert (ROOT / "index.html").is_file()


def test_main_imports_static_sdk():
    text = (ROOT / "src" / "main.ts").read_text(encoding="utf-8")
    assert "@arcflow/static" in text
    assert "ArcFlowClient" in text
    assert "MemoryConfig" in text
