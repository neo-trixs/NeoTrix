use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialEntry {
    pub id: String,
    pub domain: String,
    pub username: String,
    pub password: String,
    pub notes: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialManager {
    entries: Vec<CredentialEntry>,
    by_domain: HashMap<String, Vec<usize>>,
    next_id: u64,
}

impl CredentialManager {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            by_domain: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn store(&mut self, domain: &str, username: &str, password: &str, notes: &str) -> CredentialEntry {
        let id = format!("cred-{}", self.next_id);
        self.next_id += 1;
        let entry = CredentialEntry {
            id,
            domain: domain.to_lowercase(),
            username: username.to_string(),
            password: password.to_string(),
            notes: notes.to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        let idx = self.entries.len();
        self.by_domain.entry(entry.domain.clone()).or_default().push(idx);
        self.entries.push(entry.clone());
        entry
    }

    pub fn find(&self, domain: &str) -> Vec<&CredentialEntry> {
        let key = domain.to_lowercase();
        self.by_domain
            .get(&key)
            .map(|indices| indices.iter().filter_map(|&i| self.entries.get(i)).collect())
            .unwrap_or_default()
    }

    pub fn all(&self) -> &[CredentialEntry] {
        &self.entries
    }

    pub fn remove(&mut self, id: &str) -> bool {
        let pos = match self.entries.iter().position(|e| e.id == id) {
            Some(p) => p,
            None => return false,
        };
        let _domain = self.entries[pos].domain.clone();
        self.entries.remove(pos);

        // 重建 by_domain 索引
        let mut new_by_domain: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, entry) in self.entries.iter().enumerate() {
            new_by_domain.entry(entry.domain.clone()).or_default().push(i);
        }
        self.by_domain = new_by_domain;
        true
    }

    pub fn auto_fill_script(&self, domain: &str) -> Option<String> {
        let entries = self.find(domain);
        if entries.is_empty() {
            return None;
        }
        let entry = entries[0];
        let escaped_user = entry.username.replace('\\', "\\\\").replace('\'', "\\'");
        let escaped_pass = entry.password.replace('\\', "\\\\").replace('\'', "\\'");
        let script = format!(
            r#"
(function() {{
    var flds = document.querySelectorAll('input[type="email"], input[type="text"][name*="user"], input[type="text"][name*="email"], input[name="login"], input[name="username"]');
    var pass = document.querySelectorAll('input[type="password"]');
    if (flds.length > 0) {{
        flds[0].value = '{}';
        flds[0].dispatchEvent(new Event('input', {{ bubbles: true }}));
    }}
    if (pass.length > 0) {{
        pass[0].value = '{}';
        pass[0].dispatchEvent(new Event('input', {{ bubbles: true }}));
    }}
}})();
"#,
            escaped_user,
            escaped_pass,
        );
        Some(script)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_find() {
        let mut cm = CredentialManager::new();
        cm.store("whatsapp.com", "user@test.com", "secret123", "my whatsapp");
        let found = cm.find("whatsapp.com");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].username, "user@test.com");
    }

    #[test]
    fn test_case_insensitive_domain() {
        let mut cm = CredentialManager::new();
        cm.store("WhatsApp.COM", "user", "pass", "");
        assert_eq!(cm.find("whatsapp.com").len(), 1);
        assert_eq!(cm.find("WHATSAPP.COM").len(), 1);
    }

    #[test]
    fn test_remove_rebuilds_index() {
        let mut cm = CredentialManager::new();
        let e1 = cm.store("a.com", "u1", "p1", "");
        let _e2 = cm.store("a.com", "u2", "p2", "");
        let _e3 = cm.store("b.com", "u3", "p3", "");
        assert!(cm.remove(&e1.id));
        // After removal, remaining entries should still be findable
        let a_entries = cm.find("a.com");
        assert_eq!(a_entries.len(), 1);
        assert_eq!(a_entries[0].username, "u2");
        let b_entries = cm.find("b.com");
        assert_eq!(b_entries.len(), 1);
        assert_eq!(b_entries[0].username, "u3");
    }

    #[test]
    fn test_id_monotonic() {
        let mut cm = CredentialManager::new();
        let e1 = cm.store("x.com", "u1", "p", "");
        cm.remove(&e1.id);
        let e2 = cm.store("x.com", "u2", "p", "");
        assert_ne!(e1.id, e2.id);
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut cm = CredentialManager::new();
        assert!(!cm.remove("nonexistent"));
    }

    #[test]
    fn test_remove_clean_state() {
        let mut cm = CredentialManager::new();
        let e = cm.store("x.com", "u", "p", "");
        cm.remove(&e.id);
        assert!(cm.find("x.com").is_empty());
        assert!(cm.all().is_empty());
    }

    #[test]
    fn test_auto_fill_script_generates() {
        let mut cm = CredentialManager::new();
        cm.store("test.com", "user@test.com", "mypass", "");
        let script = cm.auto_fill_script("test.com");
        assert!(script.is_some());
        assert!(script.as_ref().unwrap().contains("user@test.com"));
        assert!(script.as_ref().unwrap().contains("mypass"));
    }

    #[test]
    fn test_auto_fill_script_escapes_backslash() {
        let mut cm = CredentialManager::new();
        cm.store("esc.com", "user", "pass\\word", "");
        let script = cm.auto_fill_script("esc.com").unwrap();
        assert!(script.contains("pass\\\\word"), "backslash should be escaped: {}", script);
    }

    #[test]
    fn test_auto_fill_no_submit() {
        let mut cm = CredentialManager::new();
        cm.store("x.com", "u", "p", "");
        let script = cm.auto_fill_script("x.com").unwrap();
        assert!(!script.contains("submit"), "should not auto-submit");
    }

    #[test]
    fn test_auto_fill_script_no_match() {
        let cm = CredentialManager::new();
        assert!(cm.auto_fill_script("unknown.com").is_none());
    }
}
