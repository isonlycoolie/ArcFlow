//! ArcFlow CLI — reads traces from the in-process Rust store (no Python).

use clap::Parser;

#[derive(Parser)]
#[command(name = "arcflow", about = "ArcFlow command-line tools")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Inspect a workflow execution trace (stub)
    Trace,
}

fn main() {
    let _cli = Cli::parse();
}
