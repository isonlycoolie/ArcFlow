"""Validate production chat-rag example structure."""

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parent


def test_chat_rag_files_exist():
    assert (ROOT / "src" / "main.ts").is_file()
    assert (ROOT / "index.html").is_file()


def test_main_is_relay_production_entry():
    text = (ROOT / "src" / "main.ts").read_text(encoding="utf-8")
    assert "@arcflow/static" in text
    assert "ArcFlowClient" in text
    assert "runPublished" in text
    assert "mode: \"relay\"" in text or "mode: 'relay'" in text
    assert "MemoryConfig" not in text
    assert "new Agent" not in text


def test_dev_direct_entry_exists_separately():
    dev = (ROOT / "src" / "main-dev-direct.ts").read_text(encoding="utf-8")
    assert "MemoryConfig" in dev
    assert "mode: \"direct\"" in dev or "mode: 'direct'" in dev
