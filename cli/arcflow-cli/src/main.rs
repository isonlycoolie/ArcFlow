//! ArcFlow CLI (Sprint 8).

mod commands;

use std::process::ExitCode;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "arcflow", about = "ArcFlow command-line interface")]
struct Cli {
    #[arg(long, global = true)]
    no_color: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init(commands::init::InitArgs),
    Run(commands::run::RunArgs),
    Trace(commands::trace::TraceArgs),
    Validate(commands::validate::ValidateArgs),
}

fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(c) => c,
        Err(e) => {
            e.print().expect("clap");
            return ExitCode::from(3);
        }
    };
    if cli.no_color {
        std::env::set_var("NO_COLOR", "1");
    }
    let code = match cli.command {
        Commands::Init(a) => commands::init::run(a),
        Commands::Run(a) => commands::run::run(a),
        Commands::Trace(a) => commands::trace::run(a),
        Commands::Validate(a) => commands::validate::run(a),
    };
    ExitCode::from(code as u8)
}
