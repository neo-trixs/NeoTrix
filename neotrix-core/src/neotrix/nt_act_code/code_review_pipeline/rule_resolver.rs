use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

// RuleEntry is defined in this file

#[derive(Debug, Clone)]
pub struct RuleLayer {
    pub name: &'static str,
    pub rules: Vec<PathRule>,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct PathRule {
    pub path_pattern: String,
    pub rule_text: String,
}

#[derive(Debug, Clone)]
pub struct ResolvedRule {
    pub rule_text: String,
    pub layer: String,
    pub should_review: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleEntry {
    pub id: String,
    pub category: String,
    pub severity: String,
    pub pattern: String,
    pub description: String,
    pub recommendation: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReviewCmdConfig {
    pub rule_categories: Vec<String>,
}

pub struct LayeredRuleResolver {
    pub layers: Vec<RuleLayer>,
    pub config: ReviewCmdConfig,
}

/// Thread-safe rule cache — Mutex-guarded HashMap for safe concurrent access.
static RULE_CACHE: OnceLock<Mutex<HashMap<String, Vec<RuleEntry>>>> = OnceLock::new();

fn rule_files_dir() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("review_rules");
    dir
}

fn parse_rule_file(lang: &str) -> Vec<RuleEntry> {
    let dir = rule_files_dir();
    let path = dir.join(format!("{}.md", lang));
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut rules: Vec<RuleEntry> = Vec::new();
    let mut current_category = String::from("general");
    let mut current_severity = String::from("medium");

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") {
            current_category = trimmed.trim_start_matches("## ").trim().to_lowercase();
            current_severity = match current_category.as_str() {
                "safety" | "security" => "high".to_string(),
                "error handling" | "null safety" | "type safety" | "type hints" => {
                    "high".to_string()
                }
                "concurrency" | "thread safety" | "async" => "high".to_string(),
                "performance" => "medium".to_string(),
                _ => "medium".to_string(),
            };
        } else if trimmed.starts_with("- ") {
            let text = trimmed.trim_start_matches("- ").trim();
            let parts: Vec<&str> = text.splitn(2, " — ").collect();
            let description = parts[0].trim();
            let recommendation = parts.get(1).map(|s| s.trim()).unwrap_or("");

            let id = format!("{}-{}", lang, rules.len().wrapping_add(1));
            let pattern = String::new();
            rules.push(RuleEntry {
                id,
                category: current_category.clone(),
                severity: current_severity.clone(),
                pattern,
                description: description.to_string(),
                recommendation: recommendation.to_string(),
            });
        }
    }
    rules
}

pub fn load_rules_for_language(lang: &str) -> Vec<RuleEntry> {
    let cache = RULE_CACHE.get_or_init(|| {
        let mut map: HashMap<String, Vec<RuleEntry>> = HashMap::new();
        for lang_key in &["rust", "python", "go", "java", "typescript"] {
            map.insert(lang_key.to_string(), parse_rule_file(lang_key));
        }
        Mutex::new(map)
    });
    let guard = cache
        .lock()
        .expect("RULE_CACHE Mutex poisoned: another thread panicked while holding the lock");
    guard.get(lang).cloned().unwrap_or_default()
}

impl LayeredRuleResolver {
    pub fn new() -> Self {
        Self {
            layers: vec![Self::default_system_layer()],
            config: ReviewCmdConfig::default(),
        }
    }

    pub fn with_config(mut self, config: ReviewCmdConfig) -> Self {
        self.config = config;
        self
    }

    pub fn get_rules_for_language(&self, lang: &str) -> Vec<RuleEntry> {
        let mut rules = load_rules_for_language(lang);
        if !self.config.rule_categories.is_empty() {
            rules.retain(|r| self.config.rule_categories.contains(&r.category));
        }
        rules
    }

    pub fn with_cli_rules(mut self, rules: Vec<PathRule>) -> Self {
        self.layers.insert(
            0,
            RuleLayer {
                name: "--rule flag",
                rules,
                include: None,
                exclude: None,
            },
        );
        self
    }

    pub fn with_project_config(mut self, base: &Path) -> Self {
        let config_path = base.join(".opencodereview").join("rule.json");
        if let Some(layer) = self.load_from_file(config_path, "project config") {
            self.layers.push(layer);
        }
        self
    }

    pub fn with_global_config(mut self) -> Self {
        if let Some(home) = dirs::home_dir() {
            let config_path = home.join(".opencodereview").join("rule.json");
            if let Some(layer) = self.load_from_file(config_path, "global config") {
                self.layers.push(layer);
            }
        }
        self
    }

    pub fn resolve(&self, file_path: &str) -> ResolvedRule {
        for layer in &self.layers {
            if let Some(rule) = self.match_in_layer(file_path, layer) {
                return rule;
            }
        }
        ResolvedRule {
            rule_text: "Perform a general code quality review".into(),
            layer: "default".into(),
            should_review: true,
        }
    }

    pub fn should_include_file(&self, file_path: &str) -> bool {
        for layer in &self.layers {
            if let Some(exclude) = &layer.exclude {
                if exclude.iter().any(|p| glob_match(p, file_path)) {
                    return false;
                }
            }
            if let Some(include) = &layer.include {
                if include.iter().any(|p| glob_match(p, file_path)) {
                    return true;
                }
            }
        }
        true
    }

    fn match_in_layer(&self, file_path: &str, layer: &RuleLayer) -> Option<ResolvedRule> {
        for rule in &layer.rules {
            if glob_match(&rule.path_pattern, file_path) {
                return Some(ResolvedRule {
                    rule_text: rule.rule_text.clone(),
                    layer: layer.name.to_string(),
                    should_review: self.should_include_file(file_path),
                });
            }
        }
        None
    }

    fn load_from_file(&self, path: PathBuf, layer_name: &'static str) -> Option<RuleLayer> {
        if !path.exists() {
            return None;
        }
        let content = std::fs::read_to_string(path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;
        let rules = json.get("rules")?;
        let path_rules: Vec<PathRule> = rules
            .as_array()?
            .iter()
            .filter_map(|r| {
                let path = r.get("path")?.as_str()?;
                let rule = r.get("rule")?.as_str()?;
                Some(PathRule {
                    path_pattern: path.to_string(),
                    rule_text: rule.to_string(),
                })
            })
            .collect();
        let include = json.get("include").and_then(|v| v.as_array()).map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        });
        let exclude = json.get("exclude").and_then(|v| v.as_array()).map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        });
        Some(RuleLayer {
            name: layer_name,
            rules: path_rules,
            include,
            exclude,
        })
    }

    fn default_system_layer() -> RuleLayer {
        RuleLayer {
            name: "system default",
            rules: vec![
                PathRule { path_pattern: "**/*.rs".into(), rule_text: "Standard Rust review rules (safety, errors, performance)".into() },
                PathRule { path_pattern: "**/*.{js,ts,jsx,tsx}".into(), rule_text: "Check for type safety, null handling, async error handling, and React hook dependencies".into() },
                PathRule { path_pattern: "**/*.py".into(), rule_text: "Check for type annotations, exception handling, and import organization".into() },
                PathRule { path_pattern: "**/*.go".into(), rule_text: "Check for error handling, goroutine leaks, and interface compliance".into() },
                PathRule { path_pattern: "**/*.{yml,yaml}".into(), rule_text: "Check for indentation, anchor usage, and secret exposure".into() },
                PathRule { path_pattern: "**/*.json".into(), rule_text: "Check for trailing commas, duplicate keys, and schema compliance".into() },
                PathRule { path_pattern: "**/*.md".into(), rule_text: "Check for broken links and markdown syntax".into() },
            ],
            include: None,
            exclude: Some(vec![
                "**/*_test.rs".into(), "**/tests/**".into(), "**/test/**".into(),
                "**/vendor/**".into(), "**/node_modules/**".into(), "**/target/**".into(),
                "**/generated/**".into(), "**/*.pb.rs".into(),
            ]),
        }
    }
}

pub(crate) fn glob_match(pattern: &str, path: &str) -> bool {
    let normalized_path = path.replace('\\', "/");
    let expanded = expand_braces(pattern);
    if expanded.len() > 1 {
        return expanded
            .iter()
            .any(|p| glob_match_single(p, &normalized_path));
    }
    glob_match_single(pattern, &normalized_path)
}

fn expand_braces(pattern: &str) -> Vec<String> {
    if let Some(start) = pattern.find('{') {
        if let Some(end) = pattern.find('}') {
            let inner = &pattern[start + 1..end];
            let prefix = &pattern[..start];
            let suffix = &pattern[end + 1..];
            let mut results = Vec::new();
            for part in inner.split(',') {
                results.push(format!("{}{}{}", prefix, part, suffix));
            }
            return results;
        }
    }
    vec![pattern.to_string()]
}

fn glob_match_single(pattern: &str, path: &str) -> bool {
    let pattern_lower = pattern.to_lowercase();
    let path_lower = path.to_lowercase();

    if pattern_lower == path_lower {
        return true;
    }

    if pattern_lower.starts_with("**/") {
        let suffix = &pattern_lower[3..];
        if suffix.ends_with("/**") {
            let prefix = &suffix[..suffix.len() - 3];
            return path_lower.starts_with(prefix);
        }
        if suffix.contains("*.") {
            let ext = suffix.split("*.").last().unwrap_or("");
            return path_lower.ends_with(&format!(".{}", ext));
        }
        if suffix.contains('*') || suffix.contains('?') {
            let s = suffix
                .replace("**", ".+?")
                .replace("*", "[^/]*")
                .replace('?', ".");
            if let Ok(re) = regex::Regex::new(&format!("^(?:{})$", s)) {
                return re.is_match(&path_lower);
            }
            return path_lower.contains(&suffix.replace('*', "").replace('?', ""));
        }
        return path_lower.contains(suffix);
    }

    if pattern_lower.contains('*') || pattern_lower.contains('?') {
        let s = pattern_lower
            .replace("**", ".+?")
            .replace("*", "[^/]*")
            .replace('?', ".");
        if let Ok(re) = regex::Regex::new(&format!("^(?:{})$", s)) {
            return re.is_match(&path_lower);
        }
    }

    false
}
