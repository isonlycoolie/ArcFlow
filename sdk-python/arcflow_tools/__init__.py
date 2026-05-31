"""Deprecated: use ``arcflow.tools`` (CommonTools)."""

from __future__ import annotations

import warnings

warnings.warn(
    "arcflow_tools is deprecated; use from arcflow.tools import CommonTools",
    DeprecationWarning,
    stacklevel=2,
)

from arcflow.tools import CommonTools

common_tools = CommonTools.bundle
register_common_tools = CommonTools.bundle

__all__ = [
    "CommonTools",
    "common_tools",
    "register_common_tools",
]
