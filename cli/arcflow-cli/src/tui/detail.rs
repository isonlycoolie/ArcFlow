//! Step detail pane rendering.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use arcflow_core::tracing::types::ExecutionTrace;

pub fn render_detail(frame: &mut Frame, area: Rect, trace: &ExecutionTrace, selected: usize) {
    let block = Block::default().title(" Detail ").borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(step) = trace.steps.get(selected) else {
        let p = Paragraph::new("No step selected.").wrap(Wrap { trim: true });
        frame.render_widget(p, inner);
        return;
    };

    let mut lines = vec![
        format!("Step: {} ({})", step.agent_name, step.step_id),
        format!("Role: {}", step.agent_role),
        format!("Status: {:?}", step.status),
    ];
    if let Some(ms) = step.duration_ms {
        lines.push(format!("Duration: {ms}ms"));
    }
    lines.push(format!(
        "Tokens: {} prompt / {} completion",
        step.tokens.prompt_tokens, step.tokens.completion_tokens
    ));
    if let Some(err) = &step.error {
        lines.push(format!("Error: {} ({})", err.message, err.error_code));
    }
    if !step.tool_calls.is_empty() {
        lines.push(format!("Tool calls: {}", step.tool_calls.len()));
    }
    let text = lines.join("\n");
    frame.render_widget(Paragraph::new(text).wrap(Wrap { trim: true }), inner);
}
