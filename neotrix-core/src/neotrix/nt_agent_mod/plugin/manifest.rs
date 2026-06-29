use serde::{Deserialize, Serialize};

/// Plugin manifest parsed from plugin.json
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: Option<String>,
    pub hooks: Vec<String>,    // Hook events this plugin listens to
    pub tools: Vec<String>,    // Tool names this plugin provides
    pub commands: Vec<String>, // CLI commands this plugin provides
    pub min_core_version: Option<String>,
}

impl PluginManifest {
    /// Parse a plugin manifest from JSON string.
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| format!("Invalid plugin manifest: {}", e))
    }

    /// Validate the manifest contents.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("Plugin name cannot be empty".to_string());
        }
        if self.version.is_empty() {
            errors.push("Plugin version cannot be empty".to_string());
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_from_valid_json() {
        let json = r#"{
            "name": "test-plugin",
            "version": "1.0.0",
            "description": "A test plugin",
            "hooks": ["PreToolUse", "PostToolUse"],
            "tools": ["custom_tool"],
            "commands": ["/custom"]
        }"#;
        let manifest = PluginManifest::from_json(json).expect("value should be ok in test");
        assert_eq!(manifest.name, "test-plugin");
        assert_eq!(manifest.hooks.len(), 2);
        assert_eq!(manifest.tools.len(), 1);
    }

    #[test]
    fn test_manifest_validation() {
        let manifest = PluginManifest {
            name: "".to_string(),
            version: "".to_string(),
            description: "bad".to_string(),
            author: None,
            hooks: vec![],
            tools: vec![],
            commands: vec![],
            min_core_version: None,
        };
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_manifest_invalid_json() {
        let result = PluginManifest::from_json("not json");
        assert!(result.is_err());
    }

    #[test]
    fn test_manifest_all_fields() {
        let json = r#"{
            "name": "full-plugin",
            "version": "2.1.0",
            "description": "Full featured",
            "author": "NeoTrix",
            "hooks": ["PreToolUse", "PostToolUse", "PreCommand"],
            "tools": ["tool_a", "tool_b"],
            "commands": ["/cmd1", "/cmd2"],
            "min_core_version": "0.18.0"
        }"#;
        let manifest = PluginManifest::from_json(json).expect("value should be ok in test");
        assert_eq!(manifest.author, Some("NeoTrix".to_string()));
        assert_eq!(manifest.commands.len(), 2);
        assert_eq!(manifest.min_core_version, Some("0.18.0".to_string()));
    }
}
