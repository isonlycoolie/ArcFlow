"""Retry configuration — logic runs in the Rust runtime only."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Union

from arcflow.constants import (
    RETRY_DEFAULT_BASE_MS,
    RETRY_DEFAULT_MAX_MS,
    RETRY_MAX_ALLOWED_ATTEMPTS,
)
from arcflow.exceptions import WorkflowConfigurationError


@dataclass(frozen=True)
class ExponentialBackoff:
    base_ms: int = RETRY_DEFAULT_BASE_MS
    multiplier: float = 2.0
    max_ms: int = RETRY_DEFAULT_MAX_MS
    jitter_ms: int = 0

    def __post_init__(self) -> None:
        if self.base_ms < 1:
            raise WorkflowConfigurationError(
                f"[ArcFlow] ExponentialBackoff base_ms must be at least 1. Got {self.base_ms}."
            )
        if self.multiplier < 1.0:
            raise WorkflowConfigurationError(
                f"[ArcFlow] ExponentialBackoff multiplier must be at least 1.0. Got {self.multiplier}."
            )
        if self.max_ms <= self.base_ms:
            raise WorkflowConfigurationError(
                f"[ArcFlow] ExponentialBackoff max_ms must be greater than base_ms."
            )


@dataclass(frozen=True)
class LinearBackoff:
    base_ms: int = 1_000
    increment_ms: int = 1_000
    max_ms: int = RETRY_DEFAULT_MAX_MS
    jitter_ms: int = 0

    def __post_init__(self) -> None:
        if self.base_ms < 1:
            raise WorkflowConfigurationError(
                f"[ArcFlow] LinearBackoff base_ms must be at least 1. Got {self.base_ms}."
            )


@dataclass(frozen=True)
class ConstantBackoff:
    delay_ms: int = 1_000
    jitter_ms: int = 0

    def __post_init__(self) -> None:
        if self.delay_ms < 1:
            raise WorkflowConfigurationError(
                f"[ArcFlow] ConstantBackoff delay_ms must be at least 1. Got {self.delay_ms}."
            )


BackoffStrategy = Union[ExponentialBackoff, LinearBackoff, ConstantBackoff]

__all__ = [
    "BackoffStrategy",
    "ConstantBackoff",
    "ExponentialBackoff",
    "LinearBackoff",
    "RETRY_MAX_ALLOWED_ATTEMPTS",
]
