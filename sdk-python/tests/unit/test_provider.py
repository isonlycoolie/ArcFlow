"""Provider configuration validation tests."""

from __future__ import annotations

import pytest

from arcflow.exceptions import ProviderConfigurationError
from arcflow.provider import Anthropic, Gemini, OpenAI


def test_openai_rejects_empty_model() -> None:
    with pytest.raises(ProviderConfigurationError, match=r"\[ArcFlow\]"):
        OpenAI(model="")


def test_anthropic_rejects_invalid_temperature() -> None:
    with pytest.raises(ProviderConfigurationError, match=r"\[ArcFlow\]"):
        Anthropic(model="claude-3-5-sonnet-20241022", temperature=2.0)


def test_gemini_binding_tuple_shape() -> None:
    provider = Gemini(model="gemini-1.5-pro", max_tokens=100, temperature=0.2)
    assert provider.binding_tuple() == ("gemini", "gemini-1.5-pro", 100, 0.2)
