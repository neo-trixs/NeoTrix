//! Rule Store - 规则存储
//!
//! 管理清理规则配置
//! 域: NT-MEMORY (知识守护者)
//! 层: L1 Action

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupRule {
    pub id: String,
    pub name: String,
    pub category: String,
    pub path_pattern: String,
    pub min_age_days: u32,
    pub min_size_bytes: u64,
    pub enabled: bool,
}

pub struct RuleStore {
    rules: HashMap<String, CleanupRule>,
    config_path: PathBuf,
}

impl RuleStore {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        let config_path = home.join(".config/neotrix/cleanup_rules.json");
        let mut store = Self { rules: HashMap::new(), config_path };
        store.init_defaults();
        store
    }

    fn init_defaults(&mut self) {
        let defaults = vec![
            CleanupRule { id: "system_cache".into(), name: "系统缓存".into(), category: "system".into(), path_pattern: "~/Library/Caches/*".into(), min_age_days: 7, min_size_bytes: 1024 * 1024, enabled: true },
            CleanupRule { id: "npm_cache".into(), name: "NPM 缓存".into(), category: "developer".into(), path_pattern: "~/.npm/*".into(), min_age_days: 30, min_size_bytes: 100 * 1024 * 1024, enabled: true },
            CleanupRule { id: "pip_cache".into(), name: "PIP 缓存".into(), category: "developer".into(), path_pattern: "~/Library/Caches/pip/*".into(), min_age_days: 30, min_size_bytes: 100 * 1024 * 1024, enabled: true },
        ];
        for rule in defaults { self.rules.insert(rule.id.clone(), rule); }
    }

    pub fn add_rule(&mut self, rule: CleanupRule) { self.rules.insert(rule.id.clone(), rule); }
    pub fn get_rule(&self, id: &str) -> Option<&CleanupRule> { self.rules.get(id) }
    pub fn get_enabled_rules(&self) -> Vec<&CleanupRule> { self.rules.values().filter(|r| r.enabled).collect() }
    pub fn enable_rule(&mut self, id: &str) { if let Some(r) = self.rules.get_mut(id) { r.enabled = true; } }
    pub fn disable_rule(&mut self, id: &str) { if let Some(r) = self.rules.get_mut(id) { r.enabled = false; } }

    pub fn save(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.rules).map_err(|e| e.to_string())?;
        if let Some(parent) = self.config_path.parent() { std::fs::create_dir_all(parent).ok(); }
        std::fs::write(&self.config_path, json).map_err(|e| e.to_string())
    }

    pub fn load(&mut self) -> Result<(), String> {
        if self.config_path.exists() {
            let json = std::fs::read_to_string(&self.config_path).map_err(|e| e.to_string())?;
            self.rules = serde_json::from_str(&json).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

impl Default for RuleStore { fn default() -> Self { Self::new() } }
