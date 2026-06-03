//! Linear stub workflow runner for WASM edge alpha.

use std::collections::HashMap;

use crate::types::{AgentDefinition, ExecutionMode, RunResult, WasmRunError, WorkflowBundle};

pub fn run_linear_stub(bundle: &WorkflowBundle, input: &str) -> Result<RunResult, WasmRunError> {
    if bundle.workflow.execution_mode != ExecutionMode::Linear {
        return Err(WasmRunError::UnsupportedMode(format!(
            "{:?}",
            bundle.workflow.execution_mode
        )));
    }
    if bundle.workflow.steps.is_empty() {
        return Err(WasmRunError::EmptyWorkflow);
    }

    let agents: HashMap<_, &AgentDefinition> = bundle.agents.iter().map(|a| (a.id, a)).collect();

    let mut steps = bundle.workflow.steps.clone();
    steps.sort_by_key(|s| s.order);

    let mut content = input.to_string();
    for step in &steps {
        let agent = agents
            .get(&step.agent_id)
            .ok_or(WasmRunError::MissingAgent {
                step_id: step.id,
                agent_id: step.agent_id,
            })?;
        content = format!(
            "[edge-stub:{}] role={} input={}",
            agent.name, agent.role, content
        );
    }

    Ok(RunResult {
        output: content,
        step_count: steps.len() as u32,
        status: "completed".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::parse_bundle;
    use uuid::Uuid;

    fn sample_bundle_json() -> String {
        let wf_id = Uuid::new_v4();
        let step_id = Uuid::new_v4();
        let agent_id = Uuid::new_v4();
        serde_json::json!({
            "workflow": {
                "id": wf_id,
                "name": "echo",
                "execution_mode": "Linear",
                "steps": [{ "id": step_id, "agent_id": agent_id, "order": 0 }]
            },
            "agents": [{
                "id": agent_id,
                "name": "echo",
                "role": "assistant",
                "instructions": "Echo input."
            }]
        })
        .to_string()
    }

    #[test]
    fn stub_runs_linear_workflow() {
        let bundle = parse_bundle(&sample_bundle_json()).unwrap();
        let result = run_linear_stub(&bundle, "hello").unwrap();
        assert_eq!(result.step_count, 1);
        assert!(result.output.contains("hello"));
        assert_eq!(result.status, "completed");
    }
}
