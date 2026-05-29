//! Graph step-flow view from assembled trace.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem};

use arcflow_core::tracing::types::ExecutionTrace;

pub fn render_graph(frame: &mut Frame, area: Rect, trace: &ExecutionTrace) {
    let block = Block::default().title(" Graph ").borders(Borders::ALL);
    let items: Vec<ListItem> = if trace.steps.is_empty() {
        vec![ListItem::new("(no steps recorded)")]
    } else {
        trace
            .steps
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let arrow = if i + 1 < trace.steps.len() { " ->" } else { "" };
                ListItem::new(format!("{} ({}){}", s.agent_name, s.step_id, arrow))
            })
            .collect()
    };
    frame.render_widget(List::new(items).block(block), area);
}
