#!/usr/bin/env python3
# Expense reimbursement workflow with manager approval gate (HITL + recovery).

import os
import sys

from arcflow import Agent, HitlConfig, Workflow, WorkflowInterruptedError

RUNTIME = os.environ.get("ARCFLOW_RUNTIME", "http://localhost:8080")
APPROVAL_KEY = "manager_approval"


def main() -> None:
    submit = Agent(name="submit", role="employee", instructions="Submit expense request")
    manager = Agent(name="manager", role="reviewer", instructions="Manager review gate")
    accounting = Agent(name="accounting", role="finance", instructions="Post to accounting")

    wf = (
        Workflow("expense_reimbursement", runtime=RUNTIME)
        .enable_recovery()
        .step(submit)
        .step(manager, hitl=HitlConfig(approval_key=APPROVAL_KEY, timeout_seconds=3600))
        .step(accounting)
    )

    try:
        result = wf.run("amount=250.00;desc=client lunch")
        print(f"Completed run_id={result.run_id} output={result.output[:80]}")
    except WorkflowInterruptedError as exc:
        print(f"Interrupted run_id={exc.run_id} approval_key={exc.approval_key}")
        print(f"Approve with: examples/hitl/approve_cli.sh {exc.run_id}")
        sys.exit(0)


if __name__ == "__main__":
    main()
