use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct PromptEntry {
    pub version: u64,
    pub name: String,
    pub content: String,
    pub metric: f64,
    pub guard_floor_checked: bool,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct PromptRegistry {
    prompts: HashMap<String, Vec<PromptEntry>>,
    max_versions: usize,
    global_version: u64,
}

impl PromptRegistry {
    pub fn new(max_versions: usize) -> Self {
        Self {
            prompts: HashMap::new(),
            max_versions,
            global_version: 0,
        }
    }

    pub fn register(&mut self, name: &str, content: &str, metric: f64) -> &PromptEntry {
        self.global_version += 1;
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let entry = PromptEntry {
            version: self.global_version,
            name: name.to_string(),
            content: content.to_string(),
            metric,
            guard_floor_checked: false,
            timestamp: ts,
        };
        let versions = self.prompts.entry(name.to_string()).or_default();
        versions.push(entry);
        if versions.len() > self.max_versions {
            versions.remove(0);
        }
        versions.last().expect("result")
    }

    pub fn get(&self, name: &str) -> Option<&PromptEntry> {
        self.prompts.get(name).and_then(|v| v.last())
    }

    pub fn get_version(&self, name: &str, version: u64) -> Option<&PromptEntry> {
        self.prompts
            .get(name)
            .and_then(|v| v.iter().find(|e| e.version == version))
    }

    pub fn history(&self, name: &str) -> Vec<&PromptEntry> {
        self.prompts.get(name).map(|v| v.iter().collect()).unwrap_or_default()
    }

    pub fn best(&self, name: &str) -> Option<&PromptEntry> {
        self.prompts
            .get(name)
            .and_then(|v| v.iter().max_by(|a, b| a.metric.partial_cmp(&b.metric).unwrap_or(std::cmp::Ordering::Equal)))
    }

    pub fn mark_guard_checked(&mut self, name: &str) {
        if let Some(entry) = self.prompts.get_mut(name).and_then(|v| v.last_mut()) {
            entry.guard_floor_checked = true;
        }
    }

    pub fn register_with_guard(&mut self, name: &str, content: &str, metric: f64, guard_config: &super::guard::GuardConfig) -> Result<&PromptEntry, String> {
        let initial_metric = self.get(name).map(|e| e.metric).unwrap_or(metric);
        let result = super::guard::check_guard_violation(guard_config, metric, initial_metric);
        match result {
            super::guard::GuardResult::Pass => {
                let entry = self.register(name, content, metric);
                Ok(entry)
            }
            super::guard::GuardResult::Violation { current, floor, delta } => {
                Err(format!("Guard floor violation: metric {:.4} < floor {:.4} (delta {:.4})", current, floor, delta))
            }
        }
    }
}
