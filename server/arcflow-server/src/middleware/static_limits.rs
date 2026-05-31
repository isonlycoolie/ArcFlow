//! Payload limits for inline static workflow definitions.

use arcflow_core::rcs::types::{AgentDefinition, WorkflowDefinition};

pub const MAX_INLINE_AGENTS: usize = 20;
pub const MAX_INLINE_STEPS: usize = 50;
pub const MAX_INSTRUCTION_CHARS: usize = 32_768;

pub fn validate_static_payload(
    workflow: &WorkflowDefinition,
    agents: &[AgentDefinition],
) -> Result<(), String> {
    if agents.len() > MAX_INLINE_AGENTS {
        return Err(format!(
            "inline workflow exceeds max agents ({MAX_INLINE_AGENTS})"
        ));
    }
    if workflow.steps.len() > MAX_INLINE_STEPS {
        return Err(format!(
            "inline workflow exceeds max steps ({MAX_INLINE_STEPS})"
        ));
    }
    for agent in agents {
        if agent.instructions.len() > MAX_INSTRUCTION_CHARS {
            return Err(format!(
                "agent '{}' instructions exceed {MAX_INSTRUCTION_CHARS} characters",
                agent.name
            ));
        }
    }
    Ok(())
}
