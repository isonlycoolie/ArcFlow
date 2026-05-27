"""Sprint 4 exception types."""

from __future__ import annotations

import pytest

from arcflow.exceptions import (
    ArcFlowError,
    InfrastructureUnavailableError,
    MemoryConfigurationError,
    MemoryOperationError,
    ToolConfigurationError,
    ToolExecutionError,
)


def test_sprint4_exceptions_inherit_arcflow_error() -> None:
    for cls in (
        ToolConfigurationError,
        ToolExecutionError,
        MemoryConfigurationError,
        MemoryOperationError,
        InfrastructureUnavailableError,
    ):
        assert issubclass(cls, ArcFlowError)


def test_tool_execution_error_carries_context() -> None:
    err = ToolExecutionError(
        "[ArcFlow] Tool 'search' failed.",
        tool_name="search",
        run_id="run-1",
        failed_step="step-2",
    )
    assert err.tool_name == "search"
    assert err.run_id == "run-1"
    assert err.failed_step == "step-2"
    assert "[ArcFlow]" in str(err)


def test_infrastructure_unavailable_error_carries_backend_hint() -> None:
    err = InfrastructureUnavailableError(
        "[ArcFlow] PostgreSQL is not available.",
        backend="postgresql",
        suggestion="Set ARCFLOW_POSTGRESQL_URL",
    )
    assert err.backend == "postgresql"
    assert err.suggestion == "Set ARCFLOW_POSTGRESQL_URL"


def test_memory_configuration_error_message() -> None:
    with pytest.raises(MemoryConfigurationError, match="\\[ArcFlow\\]"):
        raise MemoryConfigurationError("[ArcFlow] Invalid memory scope.")
