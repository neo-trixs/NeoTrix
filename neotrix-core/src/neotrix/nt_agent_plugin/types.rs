//! Plugin types — PluginState, PluginInfo, PluginManifest

/// Current lifecycle state of a loaded plugin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginState {
    Loaded,
    Unloaded,
    Error(String),
}

/// Snapshot of a registered plugin visible to CLI / introspection.
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub state: PluginState,
    pub source: String,
}

/// Declarative description carried by a plugin artifact (embedded or on-disk).
#[derive(Debug, Clone)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: Option<String>,
    pub entry: String,
}

impl PluginManifest {
    pub fn new(name: &str, version: &str, description: &str, entry: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            description: description.to_string(),
            author: None,
            entry: entry.to_string(),
        }
    }
}
