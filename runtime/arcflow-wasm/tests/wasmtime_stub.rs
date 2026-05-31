//! wasmtime integration test — builds wasm32 artifact and verifies export surface.

use std::path::{Path, PathBuf};
use std::process::Command;

use wasmtime::{Engine, Module};

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf()
}

fn wasm_artifact_path() -> PathBuf {
    workspace_root().join("target/wasm32-unknown-unknown/release/arcflow_wasm.wasm")
}

fn build_wasm_release() {
    let status = Command::new("cargo")
        .args([
            "build",
            "-p",
            "arcflow-wasm",
            "--release",
            "--target",
            "wasm32-unknown-unknown",
        ])
        .current_dir(workspace_root())
        .status()
        .expect("spawn cargo build for wasm32");
    assert!(status.success(), "wasm32 build failed");
}

fn sample_workflow_json() -> String {
    let wf_id = uuid::Uuid::new_v4();
    let step_id = uuid::Uuid::new_v4();
    let agent_id = uuid::Uuid::new_v4();
    serde_json::json!({
        "workflow": {
            "id": wf_id,
            "name": "echo",
            "execution_mode": "Linear",
            "steps": [{ "id": step_id, "agent_id": agent_id, "order": 0 }]
        },
        "agents": [{
            "id": agent_id,
            "name": "echo",
            "role": "assistant",
            "instructions": "Echo input."
        }]
    })
    .to_string()
}

#[test]
fn execute_workflow_json_host_api() {
    let out = arcflow_wasm::execute_workflow_json(&sample_workflow_json(), "\"edge\"")
        .expect("host API should run stub workflow");
    assert!(out.contains("edge"));
    assert!(out.contains("step_count"));
}

#[test]
fn wasm_module_exports_execute_workflow() {
    build_wasm_release();
    let path = wasm_artifact_path();
    assert!(
        Path::new(&path).exists(),
        "missing wasm artifact at {}",
        path.display()
    );

    let engine = Engine::default();
    let module = Module::from_file(&engine, &path).expect("load wasm module");
    let has_export = module
        .exports()
        .any(|export| export.name() == "execute_workflow");
    assert!(has_export, "execute_workflow export not found");
}
