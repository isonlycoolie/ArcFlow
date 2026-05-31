"""Schedule manifest validation (Phase 2-Pro v2)."""

from __future__ import annotations

from pathlib import Path
from typing import Any

import yaml


class ScheduleManifest:
    """Loads and validates arcflow.schedule.yaml (structure only)."""

    def __init__(self, schedules: list[dict[str, Any]]) -> None:
        self.schedules = schedules

    @classmethod
    def load(cls, path: str | Path) -> ScheduleManifest:
        raw = Path(path).read_text(encoding="utf-8")
        doc = yaml.safe_load(raw)
        if not isinstance(doc, dict):
            raise ValueError("[ArcFlow] schedule manifest root must be a mapping")
        schedules = doc.get("schedules")
        if not isinstance(schedules, list) or not schedules:
            raise ValueError("[ArcFlow] schedules must be a non-empty array")
        return cls(schedules)

    def validate(self) -> None:
        for i, entry in enumerate(self.schedules):
            if not isinstance(entry, dict):
                raise ValueError(f"[ArcFlow] schedules[{i}] must be an object")
            if not str(entry.get("id", "")).strip():
                raise ValueError(f"[ArcFlow] schedules[{i}] missing id")
            if not str(entry.get("cron", "")).strip():
                raise ValueError(f"[ArcFlow] schedules[{i}] missing cron")
            wf = entry.get("workflow")
            if not isinstance(wf, dict) or not str(wf.get("name", "")).strip():
                raise ValueError(f"[ArcFlow] schedules[{i}] missing workflow.name")
