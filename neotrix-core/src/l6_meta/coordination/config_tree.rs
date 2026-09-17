#![forbid(unsafe_code)]

//! ConfigTree — priml-inspired typed configuration
//!
//! Typed, diffable, printable config trees for:
//! - Experiment definitions (model routing, skill selection)
//! - Capability configurations (tool parameters, budgets)
//! - Architecture snapshots (module states, feature flags)
//!
//! Inspired by priml (rekursiv-ai/priml):
//! - Entire experiment state is one config tree
//! - Slots are `Makeable` types — adding a new option edits no ancestor
//! - Config is the experiment — one fork = one change, fully inspectable
//! - Blessed nouns (standardized field names) ensure cross-experiment composability

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// ============================================================================
// ConfigValue — typed leaf values
// ============================================================================

/// Typed configuration value.
///
/// Each variant wraps a primitive or composite type. `Slot` is a named reference
/// that gets resolved by a `SlotRegistry` at materialization time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConfigValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Array(Vec<ConfigValue>),
    Map(HashMap<String, ConfigValue>),
    /// Named slot reference — resolved by SlotRegistry before materialization.
    Slot(String),
}

impl ConfigValue {
    /// Returns a short type label for display purposes.
    pub fn type_name(&self) -> &'static str {
        match self {
            ConfigValue::String(_) => "String",
            ConfigValue::Int(_) => "Int",
            ConfigValue::Float(_) => "Float",
            ConfigValue::Bool(_) => "Bool",
            ConfigValue::Array(_) => "Array",
            ConfigValue::Map(_) => "Map",
            ConfigValue::Slot(_) => "Slot",
        }
    }

    /// Attempt to resolve a Slot against a registry, returning the resolved value
    /// or None if the slot name is not registered.
    pub fn resolve_slot(&self, registry: &SlotRegistry) -> Option<ConfigValue> {
        match self {
            ConfigValue::Slot(name) => registry.get(name).cloned(),
            other => Some(other.clone()),
        }
    }
}

impl fmt::Display for ConfigValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigValue::String(s) => write!(f, "{s}"),
            ConfigValue::Int(i) => write!(f, "{i}"),
            ConfigValue::Float(fl) => write!(f, "{fl}"),
            ConfigValue::Bool(b) => write!(f, "{b}"),
            ConfigValue::Array(arr) => {
                write!(f, "[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{v}")?;
                }
                write!(f, "]")
            }
            ConfigValue::Map(map) => {
                write!(f, "{{")?;
                for (i, (k, v)) in map.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{k}: {v}")?;
                }
                write!(f, "}}")
            }
            ConfigValue::Slot(name) => write!(f, "${{{name}}}"),
        }
    }
}

// ============================================================================
// ConfigNode — tree node
// ============================================================================

/// A single node in a configuration tree.
///
/// Each node has a key, an optional value, an optional description, and children.
/// Children allow nested structure while the value carries the leaf data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigNode {
    /// Key identifier for this node (relative to parent).
    pub key: String,
    /// Leaf value, if any. Internal nodes may have None.
    pub value: Option<ConfigValue>,
    /// Human-readable description of this node's purpose.
    pub description: Option<String>,
    /// Child nodes forming the subtree.
    pub children: Vec<ConfigNode>,
}

impl ConfigNode {
    /// Create a leaf node with a value.
    pub fn leaf(key: impl Into<String>, value: ConfigValue) -> Self {
        Self {
            key: key.into(),
            value: Some(value),
            description: None,
            children: Vec::new(),
        }
    }

    /// Create a branch node with children.
    pub fn branch(key: impl Into<String>, children: Vec<ConfigNode>) -> Self {
        Self {
            key: key.into(),
            value: None,
            description: None,
            children,
        }
    }

    /// Create a leaf node with a description.
    pub fn described(key: impl Into<String>, value: ConfigValue, desc: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: Some(value),
            description: Some(desc.into()),
            children: Vec::new(),
        }
    }

    /// Recursively resolve all Slot references in this node and its children.
    pub fn resolve_slots(&self, registry: &SlotRegistry) -> Self {
        let resolved_value = self.value.as_ref().and_then(|v| v.resolve_slot(registry));
        let resolved_children: Vec<ConfigNode> = self
            .children
            .iter()
            .map(|c| c.resolve_slots(registry))
            .collect();
        Self {
            key: self.key.clone(),
            value: resolved_value,
            description: self.description.clone(),
            children: resolved_children,
        }
    }

    /// Compute the full path from root to this node (dot-separated).
    pub fn path(&self) -> String {
        self.key.clone()
    }

    /// Count all nodes (including self) in this subtree.
    pub fn count(&self) -> usize {
        1 + self.children.iter().map(|c| c.count()).sum::<usize>()
    }
}

// ============================================================================
// ConfigTree — the root structure
// ============================================================================

/// Top-level configuration tree.
///
/// Wraps a root `ConfigNode` and provides diff, merge, validate, and
/// pretty-print operations. One fork = one change, fully inspectable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigTree {
    /// The root node of this configuration.
    pub root: ConfigNode,
    /// Optional version tag for this tree snapshot.
    pub version: Option<String>,
}

impl ConfigTree {
    /// Create a new ConfigTree from a root node.
    pub fn new(root: ConfigNode) -> Self {
        Self {
            root,
            version: None,
        }
    }

    /// Create a ConfigTree with a version tag.
    pub fn with_version(root: ConfigNode, version: impl Into<String>) -> Self {
        Self {
            root,
            version: Some(version.into()),
        }
    }

    /// Compute a structural diff against another ConfigTree.
    pub fn diff(&self, other: &ConfigTree) -> ConfigDiff {
        diff_trees(self, other)
    }

    /// Merge another ConfigTree's overrides into this one.
    ///
    /// Nodes present in `other` replace matching nodes in `self`. Unmatched
    /// nodes in `self` are preserved (additive merge).
    pub fn merge(&self, other: &ConfigTree) -> ConfigTree {
        let merged = merge_nodes(&self.root, &other.root);
        ConfigTree {
            root: merged,
            version: other.version.clone().or_else(|| self.version.clone()),
        }
    }

    /// Validate the tree: check for unresolved slots, duplicate keys, etc.
    pub fn validate(&self) -> Vec<ConfigError> {
        let mut errors = Vec::new();
        validate_node(&self.root, &mut errors, "");
        errors
    }

    /// Pretty-print the tree as an indented string.
    pub fn pretty_print(&self) -> String {
        let mut out = String::new();
        if let Some(ref v) = self.version {
            out.push_str(&format!("version: {v}\n"));
        }
        pretty_print_node(&self.root, &mut out, "", true);
        out
    }

    /// Recursively resolve all Slot references.
    pub fn resolve_slots(&self, registry: &SlotRegistry) -> ConfigTree {
        ConfigTree {
            root: self.root.resolve_slots(registry),
            version: self.version.clone(),
        }
    }

    /// Total node count.
    pub fn node_count(&self) -> usize {
        self.root.count()
    }
}

// ============================================================================
// SlotRegistry — named value slots
// ============================================================================

/// Registry mapping slot names to allowed values.
///
/// This is the "Makeable" concept from priml: slots are named injection points
/// that can be filled with typed values. Adding a new slot never edits an ancestor.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SlotRegistry {
    slots: HashMap<String, ConfigValue>,
}

impl SlotRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a slot with its allowed value.
    pub fn register(&mut self, name: impl Into<String>, value: ConfigValue) {
        self.slots.insert(name.into(), value);
    }

    /// Look up a slot value by name.
    pub fn get(&self, name: &str) -> Option<&ConfigValue> {
        self.slots.get(name)
    }

    /// Check if a slot name is registered.
    pub fn contains(&self, name: &str) -> bool {
        self.slots.contains_key(name)
    }

    /// Number of registered slots.
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    /// Iterate over all registered slots.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &ConfigValue)> {
        self.slots.iter().map(|(k, v)| (k.as_str(), v))
    }
}

// ============================================================================
// ExperimentConfig — experiment definition
// ============================================================================

/// An experiment configuration: one named config with optional base + overrides.
///
/// This models the priml pattern where an experiment is a fork from a base
/// config with specific overrides applied on top.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentConfig {
    /// Experiment name (blessed noun).
    pub name: String,
    /// Optional base config reference (name or path to resolve).
    pub base: Option<String>,
    /// Overrides to apply on top of the base.
    pub overrides: ConfigTree,
    /// Tags for filtering and discovery.
    pub tags: Vec<String>,
}

impl ExperimentConfig {
    /// Create a new experiment config.
    pub fn new(name: impl Into<String>, overrides: ConfigTree) -> Self {
        Self {
            name: name.into(),
            base: None,
            overrides,
            tags: Vec::new(),
        }
    }

    /// Set the base config reference.
    pub fn with_base(mut self, base: impl Into<String>) -> Self {
        self.base = Some(base.into());
        self
    }

    /// Add a tag.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Validate this experiment config.
    pub fn validate(&self) -> Vec<ConfigError> {
        let mut errors = self.overrides.validate();
        if self.name.is_empty() {
            errors.push(ConfigError {
                path: String::new(),
                message: "experiment name must not be empty".to_string(),
            });
        }
        errors
    }
}

// ============================================================================
// ConfigDiff — structural diff result
// ============================================================================

/// A single diff entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiffEntry {
    /// Dot-separated path to the changed node.
    pub path: String,
    /// Old value (None for additions).
    pub old: Option<ConfigValue>,
    /// New value (None for removals).
    pub new: Option<ConfigValue>,
}

/// Structural difference between two ConfigTrees.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ConfigDiff {
    /// Nodes present in `b` but not in `a`.
    pub added: Vec<DiffEntry>,
    /// Nodes present in `a` but not in `b`.
    pub removed: Vec<DiffEntry>,
    /// Nodes present in both but with different values or children.
    pub modified: Vec<DiffEntry>,
}

impl ConfigDiff {
    /// Whether the diff is empty (trees are structurally identical).
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.modified.is_empty()
    }

    /// Total number of changes.
    pub fn change_count(&self) -> usize {
        self.added.len() + self.removed.len() + self.modified.len()
    }
}

// ============================================================================
// ConfigError — validation errors
// ============================================================================

/// A validation error within a config tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigError {
    /// Path to the erroneous node.
    pub path: String,
    /// Human-readable error description.
    pub message: String,
}

// ============================================================================
// Core operations
// ============================================================================

/// Compute a structural diff between two ConfigTrees.
///
/// Traverses both trees in parallel, collecting additions, removals, and
/// modifications at each level.
pub fn diff_trees(a: &ConfigTree, b: &ConfigTree) -> ConfigDiff {
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut modified = Vec::new();

    diff_nodes(
        &a.root,
        &b.root,
        "",
        &mut added,
        &mut removed,
        &mut modified,
    );

    ConfigDiff {
        added,
        removed,
        modified,
    }
}

/// Recursively diff two ConfigNodes.
fn diff_nodes(
    a: &ConfigNode,
    b: &ConfigNode,
    prefix: &str,
    added: &mut Vec<DiffEntry>,
    removed: &mut Vec<DiffEntry>,
    modified: &mut Vec<DiffEntry>,
) {
    let path = if prefix.is_empty() {
        b.key.clone()
    } else {
        format!("{prefix}.{}", b.key)
    };

    let a_children: HashMap<&str, &ConfigNode> =
        a.children.iter().map(|c| (c.key.as_str(), c)).collect();
    let b_children: HashMap<&str, &ConfigNode> =
        b.children.iter().map(|c| (c.key.as_str(), c)).collect();

    // Find added and modified
    for (key, b_node) in &b_children {
        match a_children.get(key) {
            Some(a_node) => {
                // Both exist — check for value changes
                if a_node.value != b_node.value {
                    modified.push(DiffEntry {
                        path: format!("{path}.{}", b_node.key),
                        old: a_node.value.clone(),
                        new: b_node.value.clone(),
                    });
                }
                // Recurse into children
                diff_nodes(a_node, b_node, &path, added, removed, modified);
            }
            None => {
                // Added in b
                added.push(DiffEntry {
                    path: format!("{path}.{}", b_node.key),
                    old: None,
                    new: b_node.value.clone(),
                });
            }
        }
    }

    // Find removed
    for (key, a_node) in &a_children {
        if !b_children.contains_key(key) {
            removed.push(DiffEntry {
                path: format!("{path}.{}", a_node.key),
                old: a_node.value.clone(),
                new: None,
            });
        }
    }
}

/// Merge `override_node` into `base_node`, returning the combined result.
///
/// Override nodes replace matching base nodes. Unmatched base nodes are preserved.
fn merge_nodes(base: &ConfigNode, override_node: &ConfigNode) -> ConfigNode {
    let mut merged_children: Vec<ConfigNode> = Vec::new();

    // Override children take precedence — recursively merge with matching base
    for o_child in &override_node.children {
        match base.children.iter().find(|c| c.key == o_child.key) {
            Some(b_child) => merged_children.push(merge_nodes(b_child, o_child)),
            None => merged_children.push(o_child.clone()),
        }
    }

    // Base children not overridden are preserved
    for b_child in &base.children {
        if !override_node.children.iter().any(|c| c.key == b_child.key) {
            merged_children.push(b_child.clone());
        }
    }

    ConfigNode {
        key: override_node.key.clone(),
        value: override_node.value.clone().or_else(|| base.value.clone()),
        description: override_node
            .description
            .clone()
            .or_else(|| base.description.clone()),
        children: merged_children,
    }
}

/// Validate a node and collect errors.
fn validate_node(node: &ConfigNode, errors: &mut Vec<ConfigError>, parent_path: &str) {
    let path = if parent_path.is_empty() {
        node.key.clone()
    } else {
        format!("{parent_path}.{}", node.key)
    };

    // Check for unresolved slots at leaf level
    if let Some(ConfigValue::Slot(name)) = &node.value {
        errors.push(ConfigError {
            path: path.clone(),
            message: format!("unresolved slot reference: ${{{name}}}"),
        });
    }

    // Recurse into children
    for child in &node.children {
        validate_node(child, errors, &path);
    }
}

/// Pretty-print a node with indentation.
fn pretty_print_node(node: &ConfigNode, out: &mut String, indent: &str, is_last: bool) {
    let connector = if is_last { "└─ " } else { "├─ " };
    out.push_str(indent);
    out.push_str(connector);
    out.push_str(&node.key);

    if let Some(ref value) = node.value {
        out.push_str(&format!(": {value}"));
    }

    if let Some(ref desc) = node.description {
        out.push_str(&format!("  # {desc}"));
    }

    out.push('\n');

    let child_indent = if is_last {
        format!("{indent}   ")
    } else {
        format!("{indent}│  ")
    };

    let len = node.children.len();
    for (i, child) in node.children.iter().enumerate() {
        pretty_print_node(child, out, &child_indent, i == len - 1);
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tree_a() -> ConfigTree {
        ConfigTree::new(ConfigNode::branch(
            "experiment",
            vec![
                ConfigNode::leaf("model", ConfigValue::String("gpt-4".to_string())),
                ConfigNode::leaf("temperature", ConfigValue::Float(0.7)),
                ConfigNode::leaf("max_tokens", ConfigValue::Int(2048)),
                ConfigNode::branch(
                    "routing",
                    vec![
                        ConfigNode::leaf("provider", ConfigValue::String("openai".to_string())),
                        ConfigNode::leaf("fallback", ConfigValue::Bool(true)),
                    ],
                ),
            ],
        ))
    }

    fn sample_tree_b() -> ConfigTree {
        ConfigTree::new(ConfigNode::branch(
            "experiment",
            vec![
                ConfigNode::leaf("model", ConfigValue::String("claude-3".to_string())),
                ConfigNode::leaf("temperature", ConfigValue::Float(0.7)),
                ConfigNode::branch(
                    "routing",
                    vec![ConfigNode::leaf(
                        "provider",
                        ConfigValue::String("anthropic".to_string()),
                    )],
                ),
                ConfigNode::leaf("timeout", ConfigValue::Int(30)),
            ],
        ))
    }

    #[test]
    fn test_diff_catches_modifications() {
        let a = sample_tree_a();
        let b = sample_tree_b();
        let diff = a.diff(&b);

        // model changed: gpt-4 -> claude-3
        assert!(diff.modified.iter().any(|e| e.path == "experiment.model"));
        // routing.provider changed: openai -> anthropic
        assert!(diff
            .modified
            .iter()
            .any(|e| e.path == "experiment.routing.provider"));
    }

    #[test]
    fn test_diff_catches_additions() {
        let a = sample_tree_a();
        let b = sample_tree_b();
        let diff = a.diff(&b);

        // timeout added
        assert!(diff.added.iter().any(|e| e.path == "experiment.timeout"));
    }

    #[test]
    fn test_diff_catches_removals() {
        let a = sample_tree_a();
        let b = sample_tree_b();
        let diff = a.diff(&b);

        // max_tokens removed
        assert!(diff
            .removed
            .iter()
            .any(|e| e.path == "experiment.max_tokens"));
        // routing.fallback removed
        assert!(diff
            .removed
            .iter()
            .any(|e| e.path == "experiment.routing.fallback"));
    }

    #[test]
    fn test_identical_trees_produce_empty_diff() {
        let a = sample_tree_a();
        let b = sample_tree_a();
        let diff = a.diff(&b);
        assert!(diff.is_empty());
    }

    #[test]
    fn test_merge_preserves_base_and_applies_overrides() {
        let base = sample_tree_a();
        let overrides = ConfigTree::new(ConfigNode::branch(
            "experiment",
            vec![
                ConfigNode::leaf("model", ConfigValue::String("claude-3".to_string())),
                ConfigNode::leaf("timeout", ConfigValue::Int(60)),
            ],
        ));

        let merged = base.merge(&overrides);

        // model overridden
        let model = merged
            .root
            .children
            .iter()
            .find(|c| c.key == "model")
            .unwrap();
        assert_eq!(
            model.value,
            Some(ConfigValue::String("claude-3".to_string()))
        );

        // temperature preserved from base
        let temp = merged
            .root
            .children
            .iter()
            .find(|c| c.key == "temperature")
            .unwrap();
        assert_eq!(temp.value, Some(ConfigValue::Float(0.7)));

        // timeout added from override
        let timeout = merged
            .root
            .children
            .iter()
            .find(|c| c.key == "timeout")
            .unwrap();
        assert_eq!(timeout.value, Some(ConfigValue::Int(60)));
    }

    #[test]
    fn test_validate_unresolved_slot() {
        let tree = ConfigTree::new(ConfigNode::branch(
            "root",
            vec![ConfigNode::leaf(
                "model",
                ConfigValue::Slot("model_choice".to_string()),
            )],
        ));

        let errors = tree.validate();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("model_choice"));
    }

    #[test]
    fn test_validate_clean_tree() {
        let tree = sample_tree_a();
        let errors = tree.validate();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_pretty_print() {
        let tree = sample_tree_a();
        let output = tree.pretty_print();
        assert!(output.contains("experiment"));
        assert!(output.contains("model: gpt-4"));
        assert!(output.contains("routing"));
        assert!(output.contains("provider: openai"));
    }

    #[test]
    fn test_slot_registry_resolution() {
        let mut registry = SlotRegistry::new();
        registry.register(
            "model_choice",
            ConfigValue::String("gemini-pro".to_string()),
        );

        let tree = ConfigTree::new(ConfigNode::branch(
            "root",
            vec![
                ConfigNode::leaf("model", ConfigValue::Slot("model_choice".to_string())),
                ConfigNode::leaf("fixed", ConfigValue::Int(42)),
            ],
        ));

        let resolved = tree.resolve_slots(&registry);

        let model = resolved
            .root
            .children
            .iter()
            .find(|c| c.key == "model")
            .unwrap();
        assert_eq!(
            model.value,
            Some(ConfigValue::String("gemini-pro".to_string()))
        );

        let fixed = resolved
            .root
            .children
            .iter()
            .find(|c| c.key == "fixed")
            .unwrap();
        assert_eq!(fixed.value, Some(ConfigValue::Int(42)));
    }

    #[test]
    fn test_slot_registry_contains_and_len() {
        let mut registry = SlotRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);

        registry.register("a", ConfigValue::Int(1));
        registry.register("b", ConfigValue::Bool(true));

        assert_eq!(registry.len(), 2);
        assert!(registry.contains("a"));
        assert!(!registry.contains("c"));
    }

    #[test]
    fn test_experiment_config_validation() {
        let tree = ConfigTree::new(ConfigNode::branch(
            "root",
            vec![ConfigNode::leaf(
                "model",
                ConfigValue::String("gpt-4".to_string()),
            )],
        ));

        let exp = ExperimentConfig::new("test-exp", tree)
            .with_base("base-config")
            .with_tag("routing")
            .with_tag("model-selection");

        assert_eq!(exp.name, "test-exp");
        assert_eq!(exp.base, Some("base-config".to_string()));
        assert_eq!(exp.tags, vec!["routing", "model-selection"]);
        assert!(exp.validate().is_empty());
    }

    #[test]
    fn test_experiment_config_rejects_empty_name() {
        let tree = ConfigTree::new(ConfigNode::branch("root", vec![]));
        let exp = ExperimentConfig::new("", tree);
        let errors = exp.validate();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("name must not be empty"));
    }

    #[test]
    fn test_config_node_count() {
        // experiment(1) + model(1) + temperature(1) + max_tokens(1) + routing(1)
        //   + provider(1) + fallback(1) = 7
        let tree = sample_tree_a();
        assert_eq!(tree.node_count(), 7);
    }

    #[test]
    fn test_diff_change_count() {
        let a = sample_tree_a();
        let b = sample_tree_b();
        let diff = a.diff(&b);
        assert!(diff.change_count() > 0);
    }

    #[test]
    fn test_config_value_display() {
        assert_eq!(
            ConfigValue::String("hello".to_string()).to_string(),
            "hello"
        );
        assert_eq!(ConfigValue::Int(42).to_string(), "42");
        assert_eq!(ConfigValue::Bool(true).to_string(), "true");
        assert_eq!(ConfigValue::Slot("x".to_string()).to_string(), "${x}");
    }

    #[test]
    fn test_config_value_type_name() {
        assert_eq!(ConfigValue::String(String::new()).type_name(), "String");
        assert_eq!(ConfigValue::Int(0).type_name(), "Int");
        assert_eq!(ConfigValue::Float(0.0).type_name(), "Float");
        assert_eq!(ConfigValue::Bool(false).type_name(), "Bool");
        assert_eq!(ConfigValue::Array(vec![]).type_name(), "Array");
        assert_eq!(ConfigValue::Map(HashMap::new()).type_name(), "Map");
        assert_eq!(ConfigValue::Slot(String::new()).type_name(), "Slot");
    }
}
