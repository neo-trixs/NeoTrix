use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Credential {
    pub key: String,
    pub value: String,
    pub label: String,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct KeyVault {
    credentials: HashMap<String, Credential>,
}

impl KeyVault {
    pub fn new() -> Self {
        Self {
            credentials: HashMap::new(),
        }
    }

    pub fn store(&mut self, key: &str, value: &str, label: &str) {
        self.credentials.insert(
            key.to_string(),
            Credential {
                key: key.to_string(),
                value: value.to_string(),
                label: label.to_string(),
                is_active: true,
            },
        );
    }

    pub fn get(&self, key: &str) -> Option<&Credential> {
        self.credentials.get(key)
    }

    pub fn has_key(&self, key: &str) -> bool {
        self.credentials.contains_key(key)
    }

    pub fn deactivate(&mut self, key: &str) {
        if let Some(cred) = self.credentials.get_mut(key) {
            cred.is_active = false;
        }
    }

    pub fn active_keys(&self) -> Vec<&Credential> {
        self.credentials.values().filter(|c| c.is_active).collect()
    }

    pub fn remove(&mut self, key: &str) {
        self.credentials.remove(key);
    }

    pub fn key_count(&self) -> usize {
        self.credentials.len()
    }

    pub fn clear(&mut self) {
        self.credentials.clear();
    }

    pub fn all_keys(&self) -> Vec<String> {
        self.credentials.keys().cloned().collect()
    }
}

impl Default for KeyVault {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_retrieve() {
        let mut vault = KeyVault::new();
        vault.store("exchange_api", "sk-test-key", "Test Exchange");
        let cred = vault.get("exchange_api");
        assert!(cred.is_some());
        assert_eq!(cred.unwrap().value, "sk-test-key");
    }

    #[test]
    fn test_has_key() {
        let mut vault = KeyVault::new();
        assert!(!vault.has_key("missing"));
        vault.store("test", "val", "test");
        assert!(vault.has_key("test"));
    }

    #[test]
    fn test_deactivate() {
        let mut vault = KeyVault::new();
        vault.store("api", "key", "test");
        vault.deactivate("api");
        assert!(!vault.get("api").unwrap().is_active);
    }

    #[test]
    fn test_remove() {
        let mut vault = KeyVault::new();
        vault.store("tmp", "val", "tmp");
        assert_eq!(vault.key_count(), 1);
        vault.remove("tmp");
        assert_eq!(vault.key_count(), 0);
    }

    #[test]
    fn test_empty_vault() {
        let vault = KeyVault::new();
        assert_eq!(vault.key_count(), 0);
        assert!(vault.active_keys().is_empty());
    }
}
