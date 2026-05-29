"""Cross-language trace structural equivalence (Python vs TypeScript)."""

from __future__ import annotations

import json
import shutil
import subprocess
from pathlib import Path

import pytest

from arcflow import Agent, Workflow

ROOT = Path(__file__).resolve().parents[3]
TS_COMPARE = ROOT / "sdk-typescript" / "tests" / "compare_trace.mjs"


def _python_trace_shape() -> dict[str, object]:
    wf = Workflow("equiv-python")
    wf.step(Agent(name="a", role="writer", instructions="Reply in one sentence."))
    result = wf.run("hello from python")
    trace = wf.trace()
    return {
        "stepCount": result.step_count,
        "status": trace.status,
        "workflowName": trace.workflow_name,
        "stepFields": sorted(trace.steps[0].__dataclass_fields__.keys())
        if trace.steps
        else [],
    }


@pytest.mark.skipif(shutil.which("node") is None, reason="node not installed")
def test_python_and_typescript_trace_structure_match() -> None:
    if not TS_COMPARE.exists():
        pytest.skip("TypeScript compare script missing")
    py_shape = _python_trace_shape()
    proc = subprocess.run(
        ["node", str(TS_COMPARE)],
        cwd=ROOT / "sdk-typescript",
        capture_output=True,
        text=True,
        check=False,
    )
    assert proc.returncode == 0, proc.stderr or proc.stdout
    ts_shape = json.loads(proc.stdout)
    assert ts_shape["stepCount"] == py_shape["stepCount"] == 1
    assert ts_shape["workflowName"] == "equiv-ts"
    assert py_shape["workflowName"] == "equiv-python"
    assert ts_shape["status"] == py_shape["status"]
    assert "stepIndex" in ts_shape["stepFields"]
    assert "step_index" in py_shape["stepFields"]
