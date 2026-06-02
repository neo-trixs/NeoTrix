use std::path::PathBuf;

use crate::agent::tool::builtin_adapter;
use crate::agent::tool::lifecycle::AgentTool;

/// Severity of a detected tool update
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateSeverity {
    /// Bugfix — safe to apply
    Patch,
    /// New feature — review recommended
    Minor,
    /// Breaking change — requires testing
    Major,
    /// Security fix — high priority
    Security,
}

/// A detected tool update (advisory — NOT auto-applied)
#[derive(Debug, Clone)]
pub struct ToolUpdate {
    pub tool_id: String,
    pub current_version: String,
    pub available_version: String,
    pub changelog: Vec<String>,
    pub severity: UpdateSeverity,
}

/// Result of an update check cycle
pub struct UpdateCheckResult {
    pub updates: Vec<ToolUpdate>,
    pub checked_at: i64,
}

/// Checks registered tools for available updates.
/// All findings are advisory — no updates are ever applied.
pub struct ToolUpdateChecker;

impl ToolUpdateChecker {
    /// Check all registered tools for available updates.
    /// Returns advisory updates — does NOT apply them.
    pub fn check_all() -> UpdateCheckResult {
        let updates = builtin_adapter::registered_tools()
            .iter()
            .flat_map(|tool| Self::check_tool(tool.as_ref()))
            .collect::<Vec<_>>();
        let checked_at = chrono::Utc::now().timestamp();
        UpdateCheckResult { updates, checked_at }
    }

    /// Check a single tool by scanning its manifest for version hints.
    /// Reads `~/.neotrix/tool-updates/{tool_id}/version` if present.
    pub fn check_tool(tool: &dyn AgentTool) -> Vec<ToolUpdate> {
        let manifest = tool.manifest();
        let current = &manifest.version;
        let tool_id = manifest.id.clone();

        let updates_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".neotrix")
            .join("tool-updates")
            .join(&tool_id);

        let version_file = updates_dir.join("version");
        if !version_file.exists() {
            return vec![];
        }

        let available = match std::fs::read_to_string(&version_file) {
            Ok(v) => v.trim().to_string(),
            Err(_) => return vec![],
        };

        if available == *current {
            return vec![];
        }

        let changelog = Self::read_changelog(&updates_dir);
        let severity = Self::compare_versions(current, &available);

        vec![ToolUpdate {
            tool_id,
            current_version: current.clone(),
            available_version: available,
            changelog,
            severity,
        }]
    }

    /// Read optional CHANGELOG file from update directory
    fn read_changelog(dir: &PathBuf) -> Vec<String> {
        let changelog_path = dir.join("CHANGELOG");
        match std::fs::read_to_string(&changelog_path) {
            Ok(content) => content.lines().map(|l| l.to_string()).collect(),
            Err(_) => vec![],
        }
    }

    /// Simple semver comparison to determine severity.
    fn compare_versions(current: &str, available: &str) -> UpdateSeverity {
        let parse = |v: &str| -> (u64, u64, u64) {
            let parts: Vec<&str> = v.splitn(3, '.').collect();
            let major = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
            let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
            (major, minor, patch)
        };
        let (cur_maj, cur_min, cur_pat) = parse(current);
        let (avail_maj, avail_min, avail_pat) = parse(available);

        if avail_maj > cur_maj {
            UpdateSeverity::Major
        } else if avail_min > cur_min {
            UpdateSeverity::Minor
        } else if avail_pat > cur_pat {
            UpdateSeverity::Patch
        } else {
            UpdateSeverity::Patch
        }
    }

    /// Display pending updates (console log for now).
    pub fn report_updates(result: &UpdateCheckResult) -> String {
        if result.updates.is_empty() {
            return "No tool updates available.".to_string();
        }
        let mut report = format!(
            "Tool Update Check (checked at {})\n",
            chrono::DateTime::from_timestamp(result.checked_at, 0)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_default()
        );
        report.push_str(&format!("{} update(s) available:\n\n", result.updates.len()));
        for (i, update) in result.updates.iter().enumerate() {
            report.push_str(&format!(
                "{}. {}: {} → {} [{:?}]\n",
                i + 1,
                update.tool_id,
                update.current_version,
                update.available_version,
                update.severity
            ));
            if !update.changelog.is_empty() {
                report.push_str("   Changes:\n");
                for line in &update.changelog {
                    report.push_str(&format!("     • {}\n", line));
                }
            }
            report.push('\n');
        }
//        report.push_str("⚠  Advisory only — no updates were applied.");
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::tool::lifecycle::ToolManifest;

    struct TestTool {
        manifest: ToolManifest,
    }

    impl AgentTool for TestTool {
        fn id(&self) -> &str {
            &self.manifest.id
        }
        fn manifest(&self) -> &ToolManifest {
            &self.manifest
        }
        fn start(&mut self, _api: crate::agent::tool::lifecycle::ToolApi) -> Result<(), crate::agent::tool::lifecycle::ToolError> {
            Ok(())
        }
        fn execute(&self, _ctx: crate::agent::tool::lifecycle::ToolContext) -> Result<crate::agent::tool::lifecycle::ToolOutput, crate::agent::tool::lifecycle::ToolError> {
            Ok(crate::agent::tool::lifecycle::ToolOutput {
                result: String::new(),
                metadata: std::collections::HashMap::new(),
            })
        }
        fn stop(&mut self) -> Result<(), crate::agent::tool::lifecycle::ToolError> {
            Ok(())
        }
    }

    fn test_tool(id: &str, version: &str) -> TestTool {
        TestTool {
            manifest: ToolManifest {
                id: id.into(),
                name: id.into(),
                version: version.into(),
                permissions: vec![],
                mcp: None,
                min_runtime: "0.1.0".into(),
                description: "test".into(),
                author: None,
            },
        }
    }

    #[test]
    fn test_no_update_file_returns_empty() {
        let tool = test_tool("test-no-update", "0.1.0");
        let result = ToolUpdateChecker::check_tool(&tool);
        assert!(result.is_empty());
    }

    #[test]
    fn test_update_with_different_version() {
        let tool_id = "test-version-diff";
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let updates_dir = home.join(".neotrix").join("tool-updates").join(tool_id);
        let _ = std::fs::create_dir_all(&updates_dir);
        std::fs::write(updates_dir.join("version"), "0.2.0").expect("failed to write version file");

        let tool = test_tool(tool_id, "0.1.0");
        let result = ToolUpdateChecker::check_tool(&tool);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].current_version, "0.1.0");
        assert_eq!(result[0].available_version, "0.2.0");
        assert_eq!(result[0].severity, UpdateSeverity::Minor);

        let _ = std::fs::remove_dir_all(&home.join(".neotrix").join("tool-updates").join(tool_id));
    }

    #[test]
    fn test_same_version_no_update() {
        let tool_id = "test-same-version";
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let updates_dir = home.join(".neotrix").join("tool-updates").join(tool_id);
        let _ = std::fs::create_dir_all(&updates_dir);
        std::fs::write(updates_dir.join("version"), "0.1.0").expect("failed to write version file");

        let tool = test_tool(tool_id, "0.1.0");
        let result = ToolUpdateChecker::check_tool(&tool);
        assert!(result.is_empty());

        let _ = std::fs::remove_dir_all(&home.join(".neotrix").join("tool-updates").join(tool_id));
    }

    #[test]
    fn test_major_version_severity() {
        let tool_id = "test-major";
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let updates_dir = home.join(".neotrix").join("tool-updates").join(tool_id);
        let _ = std::fs::create_dir_all(&updates_dir);
        std::fs::write(updates_dir.join("version"), "2.0.0").expect("failed to write version file");

        let tool = test_tool(tool_id, "1.0.0");
        let result = ToolUpdateChecker::check_tool(&tool);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].severity, UpdateSeverity::Major);

        let _ = std::fs::remove_dir_all(&home.join(".neotrix").join("tool-updates").join(tool_id));
    }

    #[test]
    fn test_patch_version_severity() {
        let tool_id = "test-patch";
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let updates_dir = home.join(".neotrix").join("tool-updates").join(tool_id);
        let _ = std::fs::create_dir_all(&updates_dir);
        std::fs::write(updates_dir.join("version"), "0.1.1").expect("failed to write version file");

        let tool = test_tool(tool_id, "0.1.0");
        let result = ToolUpdateChecker::check_tool(&tool);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].severity, UpdateSeverity::Patch);

        let _ = std::fs::remove_dir_all(&home.join(".neotrix").join("tool-updates").join(tool_id));
    }

    #[test]
    fn test_report_updates_empty() {
        let result = UpdateCheckResult {
            updates: vec![],
            checked_at: 0,
        };
        let report = ToolUpdateChecker::report_updates(&result);
        assert!(report.contains("No tool updates"));
    }

    #[test]
    fn test_report_updates_with_items() {
        let result = UpdateCheckResult {
            updates: vec![ToolUpdate {
                tool_id: "web_scrape".into(),
                current_version: "0.1.0".into(),
                available_version: "0.2.0".into(),
                changelog: vec!["Added stealth rotation".into()],
                severity: UpdateSeverity::Minor,
            }],
            checked_at: 946684800,
        };
        let report = ToolUpdateChecker::report_updates(&result);
        assert!(report.contains("web_scrape"));
        assert!(report.contains("0.1.0 → 0.2.0"));
        assert!(report.contains("stealth rotation"));
        assert!(report.contains("Advisory only"));
    }

    #[test]
    fn test_report_updates_no_changelog() {
        let result = UpdateCheckResult {
            updates: vec![ToolUpdate {
                tool_id: "simple_tool".into(),
                current_version: "1.0.0".into(),
                available_version: "1.0.1".into(),
                changelog: vec![],
                severity: UpdateSeverity::Patch,
            }],
            checked_at: 0,
        };
        let report = ToolUpdateChecker::report_updates(&result);
        assert!(report.contains("simple_tool"));
        assert!(report.contains("[Patch]"));
        assert!(!report.contains("Changes:"));
    }

    #[test]
    fn test_check_all_no_registry() {
        let result = ToolUpdateChecker::check_all();
        assert!(result.updates.is_empty());
    }

    #[test]
    fn test_changelog_read() {
        let tool_id = "test-changelog";
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let updates_dir = home.join(".neotrix").join("tool-updates").join(tool_id);
        let _ = std::fs::create_dir_all(&updates_dir);
        std::fs::write(updates_dir.join("version"), "0.2.0").expect("failed to write version file");
        std::fs::write(
            updates_dir.join("CHANGELOG"),
            "Fixed timeout bug\nImproved error handling\n",
        )
        .expect("failed to write changelog file");

        let tool = test_tool(tool_id, "0.1.0");
        let result = ToolUpdateChecker::check_tool(&tool);
        assert_eq!(result[0].changelog.len(), 2);
        assert_eq!(result[0].changelog[0], "Fixed timeout bug");

        let _ = std::fs::remove_dir_all(&home.join(".neotrix").join("tool-updates").join(tool_id));
    }
}
