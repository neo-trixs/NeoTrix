use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub messages: Vec<(String, String)>,
    pub created_at: u64,
}

pub struct SessionStore {
    sessions: HashMap<String, Session>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    pub fn create(&mut self, id: &str) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        self.sessions.insert(
            id.to_string(),
            Session {
                id: id.to_string(),
                messages: Vec::new(),
                created_at: now,
            },
        );
    }

    pub fn add_message(&mut self, id: &str, role: &str, content: &str) -> bool {
        if let Some(s) = self.sessions.get_mut(id) {
            s.messages.push((role.to_string(), content.to_string()));
            true
        } else {
            false
        }
    }

    pub fn get(&self, id: &str) -> Option<&Session> {
        self.sessions.get(id)
    }

    pub fn delete(&mut self, id: &str) -> bool {
        self.sessions.remove(id).is_some()
    }

    pub fn count(&self) -> usize {
        self.sessions.len()
    }

    pub fn message_count(&self, id: &str) -> usize {
        self.sessions.get(id).map_or(0, |s| s.messages.len())
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_add() {
        let mut s = SessionStore::new();
        s.create("s1");
        assert!(s.add_message("s1", "user", "hello"));
        assert_eq!(s.message_count("s1"), 1);
    }

    #[test]
    fn test_delete() {
        let mut s = SessionStore::new();
        s.create("s1");
        assert!(s.delete("s1"));
        assert_eq!(s.count(), 0);
    }

    #[test]
    fn test_not_found() {
        let mut s = SessionStore::new();
        assert!(!s.add_message("missing", "user", "hi"));
    }
}
