#[derive(Debug, Clone)]
pub struct SessionStorage {
    sessions: Vec<String>,
}

impl Default for SessionStorage {
    fn default() -> Self { Self::new() }
}

impl SessionStorage {
    pub fn new() -> Self { Self { sessions: Vec::new() } }

    pub fn list_sessions(&self) -> Vec<String> {
        self.sessions.clone()
    }

    pub fn add_session(&mut self, _name: &str) {
        self.sessions.push(_name.to_string());
    }

    pub fn remove_session(&mut self, _name: &str) {
        self.sessions.retain(|s| s != _name);
    }

    pub fn contains(&self, _name: &str) -> bool {
        self.sessions.contains(&_name.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct SessionData {
    pub id: String,
    pub name: String,
    pub messages: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

pub struct SessionStore {
    sessions: Vec<SessionData>,
}

impl Default for SessionStore {
    fn default() -> Self { Self::new() }
}

impl SessionStore {
    pub fn new() -> Self { Self { sessions: Vec::new() } }

    pub fn list_sessions(&self) -> Vec<SessionData> {
        self.sessions.clone()
    }

    pub fn save_session(&mut self, _name: &str, _data: &SessionData) -> Result<(), String> {
        Ok(())
    }

    pub fn load_session(&self, _name: &str) -> Result<SessionData, String> {
        Err("not implemented".into())
    }

    pub fn delete_session(&mut self, _name: &str) -> Result<(), String> {
        Ok(())
    }

    pub fn fork(&mut self, _name: &str) -> Result<String, String> {
        Ok(format!("{}-fork", _name))
    }

    pub fn export_to_file(&self, _name: &str, _path: &str) -> Result<(), String> {
        Ok(())
    }

    pub fn export_to_json(&self, _name: &str) -> Result<String, String> {
        Ok("{}".into())
    }

    pub fn import_from_file(&mut self, _path: &str) -> Result<String, String> {
        Ok("imported".into())
    }

    pub fn get_last_session(&self) -> Option<String> {
        None
    }
}
