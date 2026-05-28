//! ArcFlow CLI — reads traces from the in-process Rust store (no Python).

use std::process::ExitCode;

use arcflow_core::get_execution_trace;
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "arcflow", about = "ArcFlow command-line tools")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Inspect a workflow execution trace
    Trace {
        /// Run id from workflow.run()
        run_id: String,
        #[arg(long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum OutputFormat {
    Human,
    Json,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Commands::Trace { run_id, format } => match get_execution_trace(&run_id) {
            Some(trace) => match format {
                OutputFormat::Json => {
                    if let Ok(json) = serde_json::to_string_pretty(&trace) {
                        println!("{json}");
                    } else {
                        eprintln!("failed to serialize trace");
                        return ExitCode::from(1);
                    }
                }
                OutputFormat::Human => {
                    println!("run_id: {}", trace.run_id);
                    println!("workflow: {}", trace.workflow_name);
                    println!("status: {:?}", trace.status);
                    println!("steps: {}", trace.steps.len());
                    if trace.events_dropped > 0 {
                        println!("warning: {} events dropped", trace.events_dropped);
                    }
                }
            },
            None => {
                eprintln!("trace not found for run_id={run_id}");
                return ExitCode::from(1);
            }
        },
    }
    ExitCode::SUCCESS
}
