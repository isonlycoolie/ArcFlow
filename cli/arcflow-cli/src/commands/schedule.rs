//! `arcflow schedule validate` — validate schedule manifest YAML (structure only).

use std::fs;
use std::path::PathBuf;

use clap::Args;

#[derive(Args)]
pub struct ScheduleArgs {
    #[command(subcommand)]
    pub command: ScheduleCommand,
}

#[derive(clap::Subcommand)]
pub enum ScheduleCommand {
    /// Validate an arcflow.schedule.yaml manifest.
    Validate(ScheduleValidateArgs),
}

#[derive(Args)]
pub struct ScheduleValidateArgs {
    /// Path to arcflow.schedule.yaml
    pub path: PathBuf,
}

pub fn run(args: ScheduleArgs) -> i32 {
    match args.command {
        ScheduleCommand::Validate(v) => validate_manifest(&v.path),
    }
}

fn validate_manifest(path: &PathBuf) -> i32 {
    let raw = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[ArcFlow] cannot read {}: {e}", path.display());
            return 1;
        }
    };
    let doc: serde_json::Value = match serde_yaml::from_str(&raw) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[ArcFlow] invalid YAML: {e}");
            return 1;
        }
    };
    let schedules = match doc.get("schedules").and_then(|s| s.as_array()) {
        Some(arr) if !arr.is_empty() => arr,
        _ => {
            eprintln!("[ArcFlow] schedules must be a non-empty array");
            return 1;
        }
    };
    for (i, entry) in schedules.iter().enumerate() {
        if entry
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .is_empty()
        {
            eprintln!("[ArcFlow] schedules[{i}] missing id");
            return 1;
        }
        if entry
            .get("cron")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .is_empty()
        {
            eprintln!("[ArcFlow] schedules[{i}] missing cron");
            return 1;
        }
        let wf_name = entry
            .get("workflow")
            .and_then(|w| w.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("");
        if wf_name.is_empty() {
            eprintln!("[ArcFlow] schedules[{i}] missing workflow.name");
            return 1;
        }
    }
    println!(
        "[ArcFlow] schedule manifest valid ({} schedule(s))",
        schedules.len()
    );
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn valid_manifest_passes() {
        let mut f = NamedTempFile::new().unwrap();
        write!(
            f,
            r#"
schedules:
  - id: daily
    cron: "0 9 * * *"
    workflow:
      name: my_workflow
"#
        )
        .unwrap();
        assert_eq!(validate_manifest(&f.path().to_path_buf()), 0);
    }

    #[test]
    fn empty_schedules_fails() {
        let mut f = NamedTempFile::new().unwrap();
        write!(f, "schedules: []").unwrap();
        assert_eq!(validate_manifest(&f.path().to_path_buf()), 1);
    }
}
