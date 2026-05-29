use std::path::PathBuf;

use clap::Args;

#[derive(Args)]
pub struct RunArgs {
    pub workflow_file: PathBuf,
    #[arg(long)]
    pub input: Option<String>,
}

pub fn run(args: RunArgs) -> i32 {
    eprintln!(
        "[ArcFlow] arcflow run for {} requires the Python SDK (arcflow package). \
         Use: python -c \"import runpy; runpy.run_path('{}')\"",
        args.workflow_file.display(),
        args.workflow_file.display()
    );
    if args.input.is_some() {
        eprintln!("[ArcFlow] --input is accepted; wire your workflow entrypoint to call workflow.run().");
    }
    2
}
