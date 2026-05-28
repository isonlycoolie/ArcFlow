//! ArcFlow CLI — reads traces from the in-process Rust store (no Python).

use std::process::ExitCode;

use arcflow_core::tracing::registry::{try_get_execution_trace, TraceLookupError};
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "arcflow", about = "ArcFlow command-line tools")]
struct Cli {
    /// Disable ANSI color codes in output
    #[arg(long, global = true)]
    no_color: bool,

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
        /// Print raw trace events below the summary
        #[arg(long)]
        verbose: bool,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum OutputFormat {
    Human,
    Json,
}

fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => {
            err.print().expect("clap error output");
            return ExitCode::from(3);
        }
    };
    if cli.no_color {
        std::env::set_var("NO_COLOR", "1");
    }
    match cli.command {
        Commands::Trace {
            run_id,
            format,
            verbose,
        } => print_trace(&run_id, format, verbose),
    }
}

fn print_trace(run_id: &str, format: OutputFormat, verbose: bool) -> ExitCode {
    match try_get_execution_trace(run_id) {
        Err(TraceLookupError::StoreLockFailed) => {
            eprintln!("trace store lock failed");
            ExitCode::from(2)
        }
        Ok(None) => {
            eprintln!("trace not found for run_id={run_id}");
            ExitCode::from(1)
        }
        Ok(Some(trace)) => {
            match format {
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
            }
            if verbose {
                if let Ok(events) = serde_json::to_string_pretty(&trace.events) {
                    println!("--- events ---");
                    println!("{events}");
                }
            }
            ExitCode::SUCCESS
        }
    }
}
