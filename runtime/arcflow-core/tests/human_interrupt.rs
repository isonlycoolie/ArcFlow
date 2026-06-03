//! HITL interrupt and resume integration tests.

use std::collections::HashMap;

use uuid::Uuid;

use arcflow_core::human::{resume_workflow_with_approval, ApprovalResult, HitlConfig};
use arcflow_core::rcs::types::{
    AgentDefinition, ExecutionMode, ExecutionStatus, StepDefinition, WorkflowDefinition,
};
use arcflow_core::workflow::{ExecutionConfig, WorkflowEngine, WorkflowRunError};

fn agent(id: Uuid, name: &str) -> AgentDefinition {
    AgentDefinition {
        id,
        name: name.into(),
        role: "r".into(),
        instructions: "i".into(),
        tools: None,
        memory_config: None,
        context: None,
        tool_execution: None,
    }
}

fn expense_workflow(
    a1: Uuid,
    a2: Uuid,
    a3: Uuid,
    s1: Uuid,
    s2: Uuid,
    s3: Uuid,
) -> WorkflowDefinition {
    WorkflowDefinition {
        id: Uuid::new_v4(),
        name: "expense".into(),
        steps: vec![
            StepDefinition {
                id: s1,
                agent_id: a1,
                order: 1,
                fallback_step_id: None,
                hitl: None,
            },
            StepDefinition {
                id: s2,
                agent_id: a2,
                order: 2,
                fallback_step_id: None,
                hitl: Some(HitlConfig {
                    approval_key: "manager_approval".into(),
                    timeout_seconds: 3600,
                    interrupt: true,
                }),
            },
            StepDefinition {
                id: s3,
                agent_id: a3,
                order: 3,
                fallback_step_id: None,
                hitl: None,
            },
        ],
        retry_policy: None,
        execution_mode: ExecutionMode::Linear,
        graph: None,
        external_bindings: None,
    }
}

#[test]
fn hitl_without_recovery_returns_invalid_definition() {
    let a1 = Uuid::new_v4();
    let a2 = Uuid::new_v4();
    let a3 = Uuid::new_v4();
    let mut agents = HashMap::new();
    agents.insert(a1, agent(a1, "submit"));
    agents.insert(a2, agent(a2, "manager"));
    agents.insert(a3, agent(a3, "accounting"));
    let wf = expense_workflow(a1, a2, a3, Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    let err = WorkflowEngine::new()
        .execute_with_config(
            &wf,
            &agents,
            "request",
            None,
            None,
            None,
            1024,
            0.7,
            &ExecutionConfig::default(),
            None,
        )
        .unwrap_err();
    assert!(matches!(
        err,
        WorkflowRunError::Aborted(
            arcflow_core::error::RuntimeError::InvalidWorkflowDefinition { .. }
        ) | WorkflowRunError::Aborted(
            arcflow_core::error::RuntimeError::RecoveryStorageError { .. }
        )
    ));
}

#[test]
#[ignore = "requires ARCFLOW_POSTGRESQL_URL"]
fn hitl_interrupt_and_resume_completes() {
    let url = std::env::var("ARCFLOW_POSTGRESQL_URL").expect("set for ignored test");
    std::env::set_var("ARCFLOW_POSTGRESQL_URL", url);

    let a1 = Uuid::new_v4();
    let a2 = Uuid::new_v4();
    let a3 = Uuid::new_v4();
    let s1 = Uuid::new_v4();
    let s2 = Uuid::new_v4();
    let s3 = Uuid::new_v4();
    let mut agents = HashMap::new();
    agents.insert(a1, agent(a1, "submit"));
    agents.insert(a2, agent(a2, "manager"));
    agents.insert(a3, agent(a3, "accounting"));
    let wf = expense_workflow(a1, a2, a3, s1, s2, s3);

    let exec_config = ExecutionConfig {
        recovery_enabled: true,
        ..ExecutionConfig::default()
    };
    let engine = WorkflowEngine::new();
    let interrupted = engine
        .execute_with_config(
            &wf,
            &agents,
            "expense:100",
            None,
            None,
            None,
            1024,
            0.7,
            &exec_config,
            None,
        )
        .unwrap_err();
    let (partial, approval_key) = match interrupted {
        WorkflowRunError::Interrupted {
            approval_key,
            partial,
            ..
        } => (partial, approval_key),
        other => panic!("expected Interrupted, got {other:?}"),
    };
    assert_eq!(approval_key, "manager_approval");
    assert_eq!(partial.step_outputs.len(), 1);

    let original_run_id = partial.run_id.to_string();
    let approval = ApprovalResult {
        approved: true,
        data: serde_json::json!({"manager_id": "mgr-42"}),
    };
    let record = resume_workflow_with_approval(
        &arcflow_core::agent::AgentRuntime::new(),
        &wf,
        &agents,
        &original_run_id,
        "manager_approval",
        approval,
        None,
        None,
        None,
        1024,
        0.7,
        &exec_config,
        true,
    )
    .expect("resume after approval");
    assert_eq!(record.step_outputs.len(), 3);
    assert!(record
        .step_outputs
        .iter()
        .all(|s| s.status == ExecutionStatus::Completed));
}
