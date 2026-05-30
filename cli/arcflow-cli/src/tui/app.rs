//! TUI event loop for trace inspection.

use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use arcflow_core::tracing::types::ExecutionTrace;

use super::detail;
use super::graph;
use super::timeline::{self, TimelineState};

#[derive(Clone, Copy, PartialEq, Eq)]
enum ViewTab {
    Timeline,
    Graph,
}

pub fn run_tui(trace: ExecutionTrace) -> io::Result<()> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut timeline_state = TimelineState::new();
    let mut tab = ViewTab::Timeline;
    let result = loop {
        terminal.draw(|frame| {
            render(frame, &trace, &mut timeline_state, tab);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break Ok(()),
                    KeyCode::Char('j') | KeyCode::Down => timeline_state.select_next(trace.steps.len()),
                    KeyCode::Char('k') | KeyCode::Up => timeline_state.select_prev(),
                    KeyCode::Char('g') => tab = ViewTab::Graph,
                    KeyCode::Char('t') => tab = ViewTab::Timeline,
                    _ => {}
                }
            }
        }
    };

    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn render(frame: &mut Frame, trace: &ExecutionTrace, timeline_state: &mut TimelineState, tab: ViewTab) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(2),
        ])
        .split(frame.area());

    let status = format!(
        "Run: {}  Status: {:?}  Steps: {}  Duration: {}ms",
        trace.run_id,
        trace.status,
        trace.steps.len(),
        trace.duration_ms.unwrap_or(0)
    );
    frame.render_widget(
        Paragraph::new(status).block(Block::default().title(" ArcFlow Trace ").borders(Borders::ALL)),
        chunks[0],
    );

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    match tab {
        ViewTab::Timeline => {
            timeline::render_timeline(frame, body[0], trace, timeline_state);
            detail::render_detail(frame, body[1], trace, timeline_state.selected());
        }
        ViewTab::Graph => {
            graph::render_graph(frame, body[0], trace);
            detail::render_detail(frame, body[1], trace, timeline_state.selected());
        }
    }

    let help = "j/k: navigate  g: graph  t: timeline  q: quit";
    frame.render_widget(Paragraph::new(help), chunks[2]);
}
