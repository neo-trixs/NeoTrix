use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use serde_json::Value;

const READ_ONLY_TOOLS: &[&str] = &["Read", "Glob", "Grep", "WebFetch", "WebSearch"];

pub struct ReadOnlyDedup {
    seen: HashMap<(String, u64), String>,
}

impl ReadOnlyDedup {
    pub fn new() -> Self {
        Self { seen: HashMap::new() }
    }

    pub fn check(&mut self, name: &str, args: &Value) -> Option<String> {
        if !READ_ONLY_TOOLS.contains(&name) {
            return None;
        }
        let hash = hash_args(args);
        let key = (name.to_string(), hash);
        if self.seen.contains_key(&key) {
//            Some("[dedup] same call already executed this turn — using previous result".into())
        } else {
            self.seen.insert(key, String::new());
            None
        }
    }

    pub fn clear(&mut self) {
        self.seen.clear();
    }
}

fn hash_args(args: &Value) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    args.to_string().hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dedup_new_is_empty() {
        let d = ReadOnlyDedup::new();
        assert!(d.seen.is_empty());
    }

    #[test]
    fn test_dedup_read_only_tool_returns_none_first_call() {
        let mut d = ReadOnlyDedup::new();
        let args = serde_json::json!({"path": "src/main.rs"});
        let result = d.check("Read", &args);
        assert!(result.is_none());
    }

    #[test]
    fn test_dedup_duplicate_call_returns_some() {
        let mut d = ReadOnlyDedup::new();
        let args = serde_json::json!({"path": "src/main.rs"});
        d.check("Read", &args);
        let result = d.check("Read", &args);
        assert!(result.is_some());
    }

    #[test]
    fn test_dedup_non_readonly_returns_none() {
        let mut d = ReadOnlyDedup::new();
        let args = serde_json::json!({"cmd": "write"});
        let result = d.check("Write", &args);
        assert!(result.is_none());
    }

    #[test]
    fn test_dedup_diff_args_not_deduped() {
        let mut d = ReadOnlyDedup::new();
        let args1 = serde_json::json!({"path": "a.rs"});
        let args2 = serde_json::json!({"path": "b.rs"});
        assert!(d.check("Read", &args1).is_none());
        assert!(d.check("Read", &args2).is_none());
    }

    #[test]
    fn test_dedup_clear_resets() {
        let mut d = ReadOnlyDedup::new();
        let args = serde_json::json!({"path": "x.rs"});
        d.check("Read", &args);
        d.clear();
        assert!(d.seen.is_empty());
    }

    #[test]
    fn test_hash_args_deterministic() {
        let args = serde_json::json!({"a": 1, "b": 2});
        let h1 = hash_args(&args);
        let h2 = hash_args(&args);
        assert_eq!(h1, h2);
    }
}
