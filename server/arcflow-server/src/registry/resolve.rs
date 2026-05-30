//! Semver resolution for registry lookups.
use semver::{Version, VersionReq};

/// Picks the highest semver matching `range` from published versions.
pub fn pick_matching_version(versions: &[String], range: &str) -> Option<String> {
    let req = VersionReq::parse(range).ok()?;
    let mut matches: Vec<Version> = versions
        .iter()
        .filter_map(|v| Version::parse(v).ok())
        .filter(|v| req.matches(v))
        .collect();
    matches.sort();
    matches.pop().map(|v| v.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caret_range_picks_highest_match() {
        let versions = vec!["1.0.0".into(), "1.2.0".into(), "1.3.1".into(), "2.0.0".into()];
        assert_eq!(
            pick_matching_version(&versions, "^1.2.0").as_deref(),
            Some("1.3.1")
        );
    }
}
