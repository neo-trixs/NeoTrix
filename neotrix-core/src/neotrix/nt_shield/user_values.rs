use serde::{Deserialize, Serialize};

/// User-configurable value preference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserValue {
    pub name: String,
    pub description: String,
    pub importance: f64,
    pub category: ValueCategory,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ValueCategory {
    Quality,
    Safety,
    Creativity,
    Efficiency,
    Ethics,
}

/// User's personal value profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserValueProfile {
    pub user_id: String,
    pub values: Vec<UserValue>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl UserValueProfile {
    pub fn new(user_id: &str) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        Self {
            user_id: user_id.to_string(),
            values: vec![
                UserValue {
                    name: "accuracy".into(),
                    description: "prefer correct over fast".into(),
                    importance: 0.9,
                    category: ValueCategory::Quality,
                    enabled: true,
                },
                UserValue {
                    name: "safety".into(),
                    description: "avoid harmful operations".into(),
                    importance: 0.9,
                    category: ValueCategory::Safety,
                    enabled: true,
                },
                UserValue {
                    name: "clarity".into(),
                    description: "clear explanations".into(),
                    importance: 0.7,
                    category: ValueCategory::Quality,
                    enabled: true,
                },
                UserValue {
                    name: "efficiency".into(),
                    description: "optimize for speed".into(),
                    importance: 0.5,
                    category: ValueCategory::Efficiency,
                    enabled: true,
                },
                UserValue {
                    name: "creativity".into(),
                    description: "explore novel solutions".into(),
                    importance: 0.5,
                    category: ValueCategory::Creativity,
                    enabled: true,
                },
            ],
            created_at: now,
            updated_at: now,
        }
    }

    pub fn set_importance(&mut self, name: &str, importance: f64) -> bool {
        if let Some(v) = self.values.iter_mut().find(|v| v.name == name) {
            v.importance = importance.clamp(0.0, 1.0);
            self.updated_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            true
        } else {
            false
        }
    }

    pub fn toggle(&mut self, name: &str) -> bool {
        if let Some(v) = self.values.iter_mut().find(|v| v.name == name) {
            v.enabled = !v.enabled;
            true
        } else {
            false
        }
    }

    pub fn add_value(&mut self, value: UserValue) {
        self.values.push(value);
    }

    pub fn get(&self, name: &str) -> Option<&UserValue> {
        self.values.iter().find(|v| v.name == name)
    }

    pub fn category_weight(&self, category: ValueCategory) -> f64 {
        let total: f64 = self
            .values
            .iter()
            .filter(|v| v.category == category && v.enabled)
            .map(|v| v.importance)
            .sum();
        let count = self
            .values
            .iter()
            .filter(|v| v.category == category && v.enabled)
            .count();
        if count == 0 {
            0.5
        } else {
            total / count as f64
        }
    }

    pub fn save(&self) {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let path = std::path::PathBuf::from(home)
            .join(".neotrix")
            .join("user_values.json");
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(&path, json);
        }
    }

    pub fn load(user_id: &str) -> Option<Self> {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let path = std::path::PathBuf::from(home)
            .join(".neotrix")
            .join("user_values.json");
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<UserValueProfile>(&s).ok())
            .or_else(|| {
                let profile = Self::new(user_id);
                profile.save();
                Some(profile)
            })
    }
}

impl Default for UserValueProfile {
    fn default() -> Self {
        Self::new("default")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_profile() {
        let p = UserValueProfile::new("test_user");
        assert_eq!(p.user_id, "test_user");
        assert_eq!(p.values.len(), 5);
    }

    #[test]
    fn test_set_importance() {
        let mut p = UserValueProfile::new("test");
        assert!(p.set_importance("accuracy", 0.5));
        assert!((p.get("accuracy").unwrap().importance - 0.5).abs() < 1e-6);
        assert!(!p.set_importance("nonexistent", 0.5));
    }

    #[test]
    fn test_toggle() {
        let mut p = UserValueProfile::new("test");
        let was = p.get("safety").unwrap().enabled;
        assert!(p.toggle("safety"));
        assert_eq!(p.get("safety").unwrap().enabled, !was);
    }

    #[test]
    fn test_category_weight() {
        let p = UserValueProfile::new("test");
        let w = p.category_weight(ValueCategory::Quality);
        assert!(w > 0.0);
    }

    #[test]
    fn test_add_value() {
        let mut p = UserValueProfile::new("test");
        p.add_value(UserValue {
            name: "test_value".into(),
            description: "test".into(),
            importance: 1.0,
            category: ValueCategory::Ethics,
            enabled: true,
        });
        assert_eq!(p.values.len(), 6);
    }

    #[test]
    fn test_save_load_roundtrip() {
        let mut p = UserValueProfile::new("roundtrip");
        p.set_importance("accuracy", 1.0);
        let json = serde_json::to_string(&p).unwrap();
        let loaded: UserValueProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.user_id, "roundtrip");
        assert!((loaded.get("accuracy").unwrap().importance - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_importance_clamp() {
        let mut p = UserValueProfile::new("clamp");
        p.set_importance("accuracy", 1.5);
        assert!((p.get("accuracy").unwrap().importance - 1.0).abs() < 1e-6);
        p.set_importance("accuracy", -0.5);
        assert!((p.get("accuracy").unwrap().importance - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_category_weight_empty_disabled() {
        let mut p = UserValueProfile::new("empty");
        for v in p.values.iter_mut() {
            v.enabled = false;
        }
        let w = p.category_weight(ValueCategory::Quality);
        assert!((w - 0.5).abs() < 1e-6);
    }
}
