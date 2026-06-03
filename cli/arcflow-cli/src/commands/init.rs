use std::fs;
use std::path::PathBuf;

use clap::Args;

#[derive(Args)]
pub struct InitArgs {
    #[arg(default_value = "my-arcflow-project")]
    pub output_dir: PathBuf,
    #[arg(long, default_value = "python")]
    pub lang: String,
    #[arg(long)]
    pub force: bool,
}

pub fn run(args: InitArgs) -> i32 {
    if args.output_dir.exists() && !args.force {
        if fs::read_dir(&args.output_dir)
            .map(|d| d.count())
            .unwrap_or(0)
            > 0
        {
            eprintln!(
                "[ArcFlow] Directory '{}' is not empty. Use --force to overwrite.",
                args.output_dir.display()
            );
            return 1;
        }
    }
    let dirs = ["workflows", "agents", "tools"];
    for d in dirs {
        let p = args.output_dir.join(d);
        if let Err(e) = fs::create_dir_all(&p) {
            eprintln!("[ArcFlow] Failed to create {}: {e}", p.display());
            return 3;
        }
    }
    let ext = if args.lang == "typescript" {
        "ts"
    } else {
        "py"
    };
    let workflow =
        format!("# Example workflow — run with: arcflow run workflows/example_workflow.{ext}\n");
    let wf_path = args
        .output_dir
        .join(format!("workflows/example_workflow.{ext}"));
    if let Err(e) = fs::write(&wf_path, workflow) {
        eprintln!("[ArcFlow] Failed to write workflow: {e}");
        return 3;
    }
    let config = "runtime_mode: embedded\n";
    if let Err(e) = fs::write(args.output_dir.join("arcflow.config.yaml"), config) {
        eprintln!("[ArcFlow] Failed to write config: {e}");
        return 3;
    }
    println!(
        "[ArcFlow] Created project at {}. Next: cd {} && arcflow run workflows/example_workflow.{ext}",
        args.output_dir.display(),
        args.output_dir.display(),
    );
    0
}
