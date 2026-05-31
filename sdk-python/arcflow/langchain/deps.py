"""Optional langchain-core dependency gate."""

from __future__ import annotations

_INSTALL_HINT = (
    "arcflow.langchain requires langchain-core. "
    "Install with: pip install 'arcflow[langchain]'"
)


def require_langchain_core() -> None:
    try:
        import langchain_core  # noqa: F401
    except ImportError as exc:
        raise ImportError(_INSTALL_HINT) from exc
