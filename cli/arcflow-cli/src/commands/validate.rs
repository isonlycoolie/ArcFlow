use std::fs;
use std::path::PathBuf;

use clap::Args;

#[derive(Args)]
pub struct ValidateArgs {
    pub workflow_file: PathBuf,
}

pub fn run(args: ValidateArgs) -> i32 {
    let content = match fs::read_to_string(&args.workflow_file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "[ArcFlow] Cannot read {}: {e}",
                args.workflow_file.display()
            );
            return 1;
        }
    };
    if content.trim().is_empty() {
        eprintln!("[ArcFlow] Workflow file is empty.");
        return 4;
    }
    println!("[ArcFlow] Workflow file is readable (syntax validation via Python SDK pending).");
    0
}
