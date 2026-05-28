"""LLM provider configuration (Sprint 6). Credentials from environment only."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Union

from arcflow.constants import (
    PROVIDER_DEFAULT_MAX_TOKENS,
    PROVIDER_DEFAULT_TEMPERATURE,
)
from arcflow.exceptions import ProviderConfigurationError


@dataclass(frozen=True)
class OpenAI:
    """OpenAI provider; reads ``OPENAI_API_KEY`` from the environment."""

    model: str
    max_tokens: int = PROVIDER_DEFAULT_MAX_TOKENS
    temperature: float = PROVIDER_DEFAULT_TEMPERATURE

    def __post_init__(self) -> None:
        if not self.model.strip():
            raise ProviderConfigurationError(
                "[ArcFlow] OpenAI model must be a non-empty string. "
                "Example: OpenAI(model='gpt-4o')."
            )
        if not 0.0 <= self.temperature <= 1.0:
            raise ProviderConfigurationError(
                f"[ArcFlow] OpenAI temperature must be between 0.0 and 1.0. "
                f"Got {self.temperature}."
            )
        if self.max_tokens < 1:
            raise ProviderConfigurationError(
                f"[ArcFlow] OpenAI max_tokens must be at least 1. Got {self.max_tokens}."
            )

    def binding_tuple(self) -> tuple[str, str, int, float]:
        return ("openai", self.model, self.max_tokens, self.temperature)


@dataclass(frozen=True)
class Anthropic:
    """Anthropic provider; reads ``ANTHROPIC_API_KEY`` from the environment."""

    model: str
    max_tokens: int = PROVIDER_DEFAULT_MAX_TOKENS
    temperature: float = PROVIDER_DEFAULT_TEMPERATURE

    def __post_init__(self) -> None:
        if not self.model.strip():
            raise ProviderConfigurationError(
                "[ArcFlow] Anthropic model must be a non-empty string. "
                "Example: Anthropic(model='claude-3-5-sonnet-20241022')."
            )
        if not 0.0 <= self.temperature <= 1.0:
            raise ProviderConfigurationError(
                f"[ArcFlow] Anthropic temperature must be between 0.0 and 1.0. "
                f"Got {self.temperature}."
            )
        if self.max_tokens < 1:
            raise ProviderConfigurationError(
                f"[ArcFlow] Anthropic max_tokens must be at least 1. Got {self.max_tokens}."
            )

    def binding_tuple(self) -> tuple[str, str, int, float]:
        return ("anthropic", self.model, self.max_tokens, self.temperature)


@dataclass(frozen=True)
class Gemini:
    """Gemini provider; reads ``GEMINI_API_KEY`` from the environment."""

    model: str
    max_tokens: int = PROVIDER_DEFAULT_MAX_TOKENS
    temperature: float = PROVIDER_DEFAULT_TEMPERATURE

    def __post_init__(self) -> None:
        if not self.model.strip():
            raise ProviderConfigurationError(
                "[ArcFlow] Gemini model must be a non-empty string. "
                "Example: Gemini(model='gemini-1.5-pro')."
            )
        if not 0.0 <= self.temperature <= 1.0:
            raise ProviderConfigurationError(
                f"[ArcFlow] Gemini temperature must be between 0.0 and 1.0. "
                f"Got {self.temperature}."
            )
        if self.max_tokens < 1:
            raise ProviderConfigurationError(
                f"[ArcFlow] Gemini max_tokens must be at least 1. Got {self.max_tokens}."
            )

    def binding_tuple(self) -> tuple[str, str, int, float]:
        return ("gemini", self.model, self.max_tokens, self.temperature)


ProviderConfig = Union[OpenAI, Anthropic, Gemini]
