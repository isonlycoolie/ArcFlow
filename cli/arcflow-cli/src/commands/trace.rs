use arcflow_core::tracing::registry::{try_get_execution_trace, TraceLookupError};
use clap::Args;

#[derive(Args)]
pub struct TraceArgs {
    pub run_id: String,
    #[arg(long, default_value = "human")]
    pub format: String,
    #[arg(long)]
    pub verbose: bool,
}

pub fn run(args: TraceArgs) -> i32 {
    match try_get_execution_trace(&args.run_id) {
        Err(TraceLookupError::StoreLockFailed) => {
            eprintln!("trace store lock failed");
            2
        }
        Ok(None) => {
            eprintln!("[ArcFlow] Trace not found for run_id={}.", args.run_id);
            1
        }
        Ok(Some(trace)) => {
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
    }
}
