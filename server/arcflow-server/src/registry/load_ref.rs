//! Resolve a registry reference to a workflow bundle.

use arcflow_core::rcs::types::{AgentDefinition, WorkflowDefinition};
use semver::{Version, VersionReq};

use crate::dto::registry::WorkflowBundle;
use crate::registry::resolve;
use crate::store::workflow_registry::WorkflowRegistryStore;

pub struct LoadedWorkflow {
    pub workflow: WorkflowDefinition,
    pub agents: Vec<AgentDefinition>,
    pub version: String,
}

pub async fn load(
    store: &WorkflowRegistryStore,
    name: &str,
    version_or_range: &str,
) -> Result<LoadedWorkflow, String> {
    let version = resolve_version(store, name, version_or_range).await?;
    let published = store
        .get(name, &version)
        .await
        .map_err(|e| format!("database error: {e}"))?
        .ok_or_else(|| format!("workflow '{name}' version '{version}' not found"))?;
    let bundle: WorkflowBundle = serde_json::from_value(published.definition_json)
        .map_err(|e| format!("invalid stored definition: {e}"))?;
    Ok(LoadedWorkflow {
        workflow: bundle.workflow,
        agents: bundle.agents,
        version,
    })
}

async fn resolve_version(
    store: &WorkflowRegistryStore,
    name: &str,
    version_or_range: &str,
) -> Result<String, String> {
    if let Some(alias_version) = store
        .get_alias_version(name, version_or_range)
        .await
        .map_err(|e| format!("database error: {e}"))?
    {
        return Ok(alias_version);
    }
    if Version::parse(version_or_range).is_ok() {
        return Ok(version_or_range.to_string());
    }
    if VersionReq::parse(version_or_range).is_ok() {
        let versions = store
            .list_versions(name)
            .await
            .map_err(|e| format!("database error: {e}"))?;
        return resolve::pick_matching_version(&versions, version_or_range).ok_or_else(|| {
            format!("no workflow '{name}' version matching '{version_or_range}'")
        });
    }
    Err(format!(
        "invalid workflow version or alias '{version_or_range}'"
    ))
}

#[cfg(test)]
mod tests {
    #[test]
    fn caret_is_not_valid_exact_semver() {
        assert!(semver::Version::parse("^1.2.0").is_err());
        assert!(semver::VersionReq::parse("^1.2.0").is_ok());
    }
}
