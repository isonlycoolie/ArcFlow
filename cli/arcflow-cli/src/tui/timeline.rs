//! Timeline list of workflow steps.

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};

use arcflow_core::tracing::types::{ExecutionTrace, StepExecutionStatus};

pub struct TimelineState {
    pub list_state: ListState,
}

impl TimelineState {
    pub fn new() -> Self {
        Self {
            list_state: ListState::default().with_selected(Some(0)),
        }
    }

    pub fn selected(&self) -> usize {
        self.list_state.selected().unwrap_or(0)
    }

    pub fn select_next(&mut self, len: usize) {
        if len == 0 {
            return;
        }
        let i = self.selected().saturating_add(1).min(len - 1);
        self.list_state.select(Some(i));
    }

    pub fn select_prev(&mut self) {
        let i = self.selected().saturating_sub(1);
        self.list_state.select(Some(i));
    }
}

pub fn render_timeline(frame: &mut Frame, area: Rect, trace: &ExecutionTrace, state: &mut TimelineState) {
    let block = Block::default().title(" Timeline ").borders(Borders::ALL);
    let items: Vec<ListItem> = trace
        .steps
        .iter()
        .map(|s| {
            let mark = match s.status {
                StepExecutionStatus::Completed => "ok",
                StepExecutionStatus::Failed => "FAIL",
                StepExecutionStatus::InProgress => "..",
            };
            let dur = s
                .duration_ms
                .map(|ms| format!("{ms}ms"))
                .unwrap_or_else(|| "-".into());
            ListItem::new(format!(
                "step_{} {}  {}  [{mark}]",
                s.step_index, s.agent_name, dur
            ))
        })
        .collect();
    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    frame.render_stateful_widget(list, area, &mut state.list_state);
}
