//! Shared helpers for stub domain plugins.

use crate::domain::{serde_json, ActionSpec, DomainError};

pub(crate) fn stub_action(name: &str) -> ActionSpec {
    ActionSpec {
        name: name.into(),
        description: String::new(),
        params: vec![],
        returns: "Value".into(),
    }
}

pub(crate) fn stub_call(
    action: &str,
    actions: &[&str],
) -> Result<serde_json::Value, DomainError> {
    if actions.contains(&action) {
        Ok(serde_json::json!({ "ok": true, "stub": true }))
    } else {
        Err(DomainError {
            code: "UNKNOWN_ACTION".into(),
            message: format!("Unknown action: {}", action),
            recoverable: true,
        })
    }
}

pub(crate) fn bump_version(version: &str) -> String {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return "0.1.0".into();
    }
    let major = parts[0].parse::<u32>().unwrap_or(0);
    let minor = parts[1].parse::<u32>().unwrap_or(0);
    let patch = parts[2].parse::<u32>().unwrap_or(0);
    format!("{}.{}.{}", major, minor, patch + 1)
}
