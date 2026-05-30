use std::path::PathBuf;

use arcflow_core::tracing::builder::ExecutionTraceBuilder;
use arcflow_core::tracing::registry::try_get_execution_trace;
use arcflow_core::tracing::registry::TraceLookupError;
use arcflow_core::tracing::types::{ExecutionTrace, TraceEvent};
use clap::Args;

#[derive(Args)]
pub struct TraceArgs {
    pub run_id: String,
    #[arg(long, default_value = "human")]
    pub format: String,
    #[arg(long)]
    pub verbose: bool,
    #[arg(long)]
    pub tui: bool,
    #[arg(long)]
    pub file: Option<PathBuf>,
    #[arg(long)]
    pub server: Option<String>,
}

pub fn run(args: TraceArgs) -> i32 {
    let trace = match load_trace(&args) {
        Ok(t) => t,
        Err(code) => return code,
    };

    if args.tui {
        if let Err(err) = crate::tui::run_tui(trace) {
            eprintln!("[ArcFlow] TUI error: {err}");
            return 1;
        }
        return 0;
    }

    if args.format == "json" {
        if let Ok(json) = serde_json::to_string_pretty(&trace) {
            println!("{json}");
        } else {
            return 1;
        }
    } else {
        println!("run_id: {}", trace.run_id);
        println!("workflow: {}", trace.workflow_name);
        println!("steps: {}", trace.steps.len());
    }
    if args.verbose {
        if let Ok(events) = serde_json::to_string_pretty(&trace.events) {
            println!("--- events ---\n{events}");
        }
    }
    0
}

fn load_trace(args: &TraceArgs) -> Result<ExecutionTrace, i32> {
    if let Some(path) = &args.file {
        return load_trace_file(path);
    }
    if let Some(server) = &args.server {
        return load_trace_server(server, &args.run_id);
    }
    match try_get_execution_trace(&args.run_id) {
        Err(TraceLookupError::StoreLockFailed) => {
            eprintln!("trace store lock failed");
            Err(2)
        }
        Ok(None) => {
            eprintln!("[ArcFlow] Trace not found for run_id={}.", args.run_id);
            Err(1)
        }
        Ok(Some(trace)) => Ok(trace),
    }
}

fn load_trace_file(path: &PathBuf) -> Result<ExecutionTrace, i32> {
    let raw = std::fs::read_to_string(path).map_err(|e| {
        eprintln!("[ArcFlow] Failed to read trace file: {e}");
        1
    })?;
    if let Ok(trace) = serde_json::from_str::<ExecutionTrace>(&raw) {
        return Ok(trace);
    }
    let events: Vec<TraceEvent> = serde_json::from_str(&raw).map_err(|e| {
        eprintln!("[ArcFlow] Invalid trace JSON: {e}");
        1
    })?;
    Ok(ExecutionTraceBuilder::build(
        "file-import",
        &events,
        0,
    ))
}

