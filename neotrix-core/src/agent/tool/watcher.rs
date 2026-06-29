/// Tool hot-reload via filesystem watching.
/// Monitors ~/.neotrix/tool-tweaks/ for JSON manifest + JS wasm files.
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

/// Policy for handling tool updates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdatePolicy {
    /// Warn user and ask for approval before updating
    Advisory,
    /// Automatically apply updates without asking
    Auto,
    /// Ignore all updates
    Disabled,
}

impl Default for UpdatePolicy {
    fn default() -> Self {
        Self::Advisory
    }
}

/// Information about a pending update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub tool_id: String,
    pub current_version: String,
    pub new_version: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweakManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub main: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
}

/// Watcher for tool tweak directories
pub struct ToolWatcher {
    tweaks_dir: PathBuf,
    loaded: Mutex<HashMap<String, TweakManifest>>,
}

impl ToolWatcher {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            tweaks_dir: home.join(".neotrix").join("tool-tweaks"),
            loaded: Mutex::new(HashMap::new()),
        }
    }

    pub fn with_dir(dir: PathBuf) -> Self {
        Self {
            tweaks_dir: dir,
            loaded: Mutex::new(HashMap::new()),
        }
    }

    /// Scan the tweaks directory and (re)load any changed manifests.
    /// Returns list of newly loaded/changed tool IDs.
    pub fn scan_and_reload(&self) -> Vec<String> {
        let mut changed = Vec::new();
        if !self.tweaks_dir.exists() {
            let _ = std::fs::create_dir_all(&self.tweaks_dir);
            return changed;
        }
        let entries = match std::fs::read_dir(&self.tweaks_dir) {
            Ok(e) => e,
            Err(_) => return changed,
        };
        let mut loaded = self.loaded.lock().unwrap_or_else(|e| e.into_inner());
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let manifest_path = path.join("manifest.json");
            if !manifest_path.exists() {
                continue;
            }
            let content = match std::fs::read_to_string(&manifest_path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let manifest: TweakManifest = match serde_json::from_str(&content) {
                Ok(m) => m,
                Err(_) => continue,
            };
            let inserted = loaded
                .get(&manifest.id)
                .map(|existing| existing.version != manifest.version)
                .unwrap_or(true);
            if inserted {
                loaded.insert(manifest.id.clone(), manifest.clone());
                changed.push(manifest.id.clone());
            }
        }
        changed
    }

    /// Get all currently loaded tweak manifests
    pub fn loaded_tweaks(&self) -> Vec<TweakManifest> {
        self.loaded
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .values()
            .cloned()
            .collect()
    }

    /// Check if a specific tweak has been updated (version change)
    pub fn has_update(&self, tool_id: &str, current_version: &str) -> bool {
        self.loaded
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(tool_id)
            .map(|m| m.version != current_version)
            .unwrap_or(false)
    }

    /// Check if a specific tool has an update and return info about it
    pub fn check_for_update(&self, tool_id: &str, current_version: &str) -> Option<UpdateInfo> {
        let loaded = self.loaded.lock().unwrap_or_else(|e| e.into_inner());
        loaded.get(tool_id).and_then(|manifest| {
            if manifest.version != current_version {
                Some(UpdateInfo {
                    tool_id: tool_id.to_string(),
                    current_version: current_version.to_string(),
                    new_version: manifest.version.clone(),
                    name: manifest.name.clone(),
                })
            } else {
                None
            }
        })
    }

    /// Check all tools in the given version map for updates
    pub fn check_all_updates(&self, current_versions: &HashMap<String, String>) -> Vec<UpdateInfo> {
        current_versions
            .iter()
            .filter_map(|(id, ver)| self.check_for_update(id, ver))
            .collect()
    }
}

impl Default for ToolWatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watcher_creates_dir() {
        let tmp = std::env::temp_dir().join("neotrix-test-watcher");
        let _ = std::fs::remove_dir_all(&tmp);
        let watcher = ToolWatcher::with_dir(tmp.clone());
        assert!(watcher.scan_and_reload().is_empty());
        assert!(tmp.exists());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_scans_manifests() {
        let tmp = std::env::temp_dir().join("neotrix-test-scan");
        let _ = std::fs::remove_dir_all(&tmp);
        let tool_dir = tmp.join("my-tweak");
        std::fs::create_dir_all(&tool_dir).expect("create test tool dir");
        let manifest = r#"{
            "id": "my-tweak",
            "name": "My Tweak",
            "version": "0.1.0"
        }"#;
        std::fs::write(tool_dir.join("manifest.json"), manifest).expect("write test manifest");
        let watcher = ToolWatcher::with_dir(tmp.clone());
        let changed = watcher.scan_and_reload();
        assert!(changed.contains(&"my-tweak".into()));
        assert_eq!(watcher.loaded_tweaks().len(), 1);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_detects_version_change() {
        let tmp = std::env::temp_dir().join("neotrix-test-version");
        let _ = std::fs::remove_dir_all(&tmp);
        let tool_dir = tmp.join("my-tool");
        std::fs::create_dir_all(&tool_dir).expect("create test tool dir");
        let manifest = r#"{"id":"my-tool","name":"My Tool","version":"0.2.0"}"#;
        std::fs::write(tool_dir.join("manifest.json"), manifest).expect("write test manifest");
        let watcher = ToolWatcher::with_dir(tmp.clone());
        watcher.scan_and_reload();
        assert!(watcher.has_update("my-tool", "0.1.0"));
        assert!(!watcher.has_update("my-tool", "0.2.0"));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_check_for_update_matches() {
        let tmp = std::env::temp_dir().join("neotrix-test-update-info");
        let _ = std::fs::remove_dir_all(&tmp);
        let tool_dir = tmp.join("my-update-tool");
        std::fs::create_dir_all(&tool_dir).expect("create test tool dir");
        let manifest = r#"{"id":"my-update-tool","name":"Update Tool","version":"0.3.0"}"#;
        std::fs::write(tool_dir.join("manifest.json"), manifest).expect("write test manifest");

        let watcher = ToolWatcher::with_dir(tmp.clone());
        watcher.scan_and_reload();

        let info = watcher.check_for_update("my-update-tool", "0.2.0");
        assert!(info.is_some());
        assert_eq!(
            info.as_ref()
                .expect("update info should be Some")
                .new_version,
            "0.3.0"
        );
        assert_eq!(
            info.expect("update info should be Some").name,
            "Update Tool"
        );

        let same = watcher.check_for_update("my-update-tool", "0.3.0");
        assert!(same.is_none());

        let missing = watcher.check_for_update("nonexistent", "0.1.0");
        assert!(missing.is_none());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_skips_dirs_without_manifest() {
        let tmp = std::env::temp_dir().join("neotrix-test-empty-dir");
        let _ = std::fs::remove_dir_all(&tmp);
        let tool_dir = tmp.join("no-manifest");
        std::fs::create_dir_all(&tool_dir).expect("create test tool dir");
        let watcher = ToolWatcher::with_dir(tmp.clone());
        assert!(watcher.scan_and_reload().is_empty());
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
