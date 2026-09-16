use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ExternalConfig {
    values: HashMap<String, String>,
    env_prefix: String,
}

impl ExternalConfig {
    pub fn new(env_prefix: &str) -> Self {
        Self {
            values: HashMap::new(),
            env_prefix: env_prefix.to_string(),
        }
    }

    pub fn from_env(&mut self) {
        for (key, value) in std::env::vars() {
            if key.starts_with(&self.env_prefix) {
                let config_key = key
                    .strip_prefix(&self.env_prefix)
                    .unwrap_or(&key)
                    .to_lowercase();
                self.values.insert(config_key, value);
            }
        }
    }

    pub fn from_file(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        for line in content.lines() {
            if let Some((key, value)) = line.split_once('=') {
                self.values
                    .insert(key.trim().to_string(), value.trim().to_string());
            }
        }
        Ok(())
    }

    pub fn get<'a>(&'a self, key: &str) -> Option<&'a String> {
        self.values.get(key)
    }

    pub fn get_or(&self, key: &str, default: &str) -> String {
        match self.values.get(key) {
            Some(v) => v.clone(),
            None => default.to_string(),
        }
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }

    pub fn keys<'a>(&'a self) -> Vec<&'a str> {
        self.values.keys().map(|s| s.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_get() {
        let mut c = ExternalConfig::new("APP_");
        c.set("debug", "true");
        assert_eq!(c.get("debug"), Some(&"true".to_string()));
    }

    #[test]
    fn test_get_or() {
        let c = ExternalConfig::new("APP_");
        assert_eq!(c.get_or("missing", "default"), "default");
    }
}
