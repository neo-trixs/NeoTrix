use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::agent::tool::lifecycle::{ToolError, ToolFs, ToolStorage};

/// Manages sandboxed data directories for all tools.
pub struct SandboxManager {
    root: PathBuf,
    stores: Mutex<HashMap<String, ToolSandbox>>,
}

impl SandboxManager {
    /// Create a sandbox manager rooted at `~/.neotrix/tool-data/`.
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            root: home.join(".neotrix").join("tool-data"),
            stores: Mutex::new(HashMap::new()),
        }
    }

    pub fn with_root(root: PathBuf) -> Self {
        Self {
            root,
            stores: Mutex::new(HashMap::new()),
        }
    }

    /// Get (or create) the sandbox for a given tool.
    pub fn for_tool(&self, tool_id: &str) -> ToolSandbox {
        let mut stores = self.stores.lock().expect("result");
        if let Some(existing) = stores.get(tool_id) {
            return existing.clone();
        }
        let sandbox = ToolSandbox::new(&self.root, tool_id);
        stores.insert(tool_id.to_string(), sandbox.clone());
        sandbox
    }
}

impl Default for SandboxManager {
    fn default() -> Self {
        Self::new()
    }
}

/// A single tool's sandboxed directory + KV storage.
#[derive(Clone, Debug)]
pub struct ToolSandbox {
    pub data_dir: PathBuf,
    storage: std::sync::Arc<Mutex<HashMap<String, String>>>,
}

impl ToolSandbox {
    pub fn new(root: &Path, tool_id: &str) -> Self {
        let data_dir = root.join(tool_id);
        let _ = std::fs::create_dir_all(&data_dir);
        Self {
            data_dir,
            storage: std::sync::Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn storage(&self) -> SandboxedStorage {
        SandboxedStorage {
            store: self.storage.clone(),
        }
    }

    pub fn fs(&self) -> SandboxedFs {
        SandboxedFs {
            root: self.data_dir.clone(),
        }
    }
}

// === ToolStorage implementation ===

#[derive(Clone)]
pub struct SandboxedStorage {
    store: std::sync::Arc<Mutex<HashMap<String, String>>>,
}

impl ToolStorage for SandboxedStorage {
    fn get(&self, key: &str) -> Option<String> {
        self.store.lock().ok().and_then(|s| s.get(key).cloned())
    }

    fn set(&self, key: &str, value: &str) {
        if let Ok(mut store) = self.store.lock() {
            store.insert(key.to_string(), value.to_string());
        }
    }

    fn delete(&self, key: &str) {
        if let Ok(mut store) = self.store.lock() {
            store.remove(key);
        }
    }
}

// === ToolFs implementation ===

#[derive(Clone)]
pub struct SandboxedFs {
    root: PathBuf,
}

impl SandboxedFs {
    /// Resolve a relative path against the sandbox root, ensuring it doesn't escape.
    /// Works for both existing and non-existing paths (unlike canonicalize).
    fn resolve(&self, rel_path: &str) -> Result<PathBuf, ToolError> {
        let root_canon = self
            .root
            .canonicalize()
            .map_err(|e| ToolError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, e)))?;
        // Resolve rel_path relative to the canonicalized root (handles /tmp → /private/tmp
        // symlinks on macOS correctly).
        let candidate = root_canon.join(rel_path);
        let normalized = normalize_path(&candidate);
        if !normalized.starts_with(&root_canon) {
            return Err(ToolError::Runtime {
                id: "sandbox".into(),
                message: format!("Path '{}' escapes sandbox", rel_path),
            });
        }
        Ok(normalized)
    }
}

/// Normalize a path by resolving `.` and `..` components without requiring the
/// path to exist on disk (unlike `canonicalize`).
fn normalize_path(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                result.pop();
            }
            std::path::Component::CurDir => {}
            other => result.push(other.as_os_str()),
        }
    }
    result
}

impl ToolFs for SandboxedFs {
    fn read(&self, path: &str) -> Result<String, ToolError> {
        let resolved = self.resolve(path)?;
        Ok(std::fs::read_to_string(&resolved)?)
    }

    fn write(&self, path: &str, contents: &str) -> Result<(), ToolError> {
        let resolved = self.resolve(path)?;
        if let Some(parent) = resolved.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(std::fs::write(&resolved, contents)?)
    }

    fn exists(&self, path: &str) -> bool {
        self.resolve(path).map(|p| p.exists()).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sandbox_creates_dir() {
        let tmp = std::env::temp_dir().join("neotrix-test-sandbox");
        let _ = std::fs::remove_dir_all(&tmp);
        let manager = SandboxManager::with_root(tmp.clone());
        let sandbox = manager.for_tool("test-tool");
        assert!(sandbox.data_dir().exists());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_storage_get_set() {
        let manager = SandboxManager::with_root(std::env::temp_dir().join("neotrix-test-storage"));
        let store = manager.for_tool("test-tool").storage();
        store.set("key1", "value1");
        assert_eq!(store.get("key1"), Some("value1".into()));
        store.delete("key1");
        assert_eq!(store.get("key1"), None);
    }

    #[test]
    fn test_fs_perimeter() {
        let tmp = std::env::temp_dir().join("neotrix-test-fs");
        let _ = std::fs::remove_dir_all(&tmp);
        let manager = SandboxManager::with_root(tmp.clone());
        let sandbox = manager.for_tool("test-fs");
        let fs = sandbox.fs();
        fs.write("hello.txt", "world").expect("value should be ok in test");
        assert_eq!(fs.read("hello.txt").expect("value should be ok in test"), "world");
        assert!(fs.exists("hello.txt"));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_path_traversal_blocked() {
        let manager =
            SandboxManager::with_root(std::env::temp_dir().join("neotrix-test-traversal"));
        let sandbox = manager.for_tool("test-tool");
        let fs = sandbox.fs();
        let result = fs.read("../../../etc/passwd");
        assert!(result.is_err());
    }
}
