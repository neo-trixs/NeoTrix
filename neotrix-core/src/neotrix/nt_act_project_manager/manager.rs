use std::collections::VecDeque;
use std::fs;
use std::path::Path;

use super::types::{ProjectInfo, ProjectManager};

impl ProjectManager {
    pub fn new() -> Self {
        ProjectManager {
            projects: Vec::new(),
            recent_projects: VecDeque::with_capacity(10),
            active_id: None,
        }
    }

    pub fn open(&mut self, path: &Path) -> Result<&ProjectInfo, String> {
        let canonical = path
            .canonicalize()
            .map_err(|e| format!("cannot resolve path: {}", e))?;
        let name = canonical
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("untitled")
            .to_string();
        let id = canonical.to_string_lossy().to_string();

        let tech_stack = Self::detect_tech_stack(&canonical);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        if let Some(existing) = self.projects.iter_mut().find(|p| p.id == id) {
            existing.last_opened = now;
            existing.tech_stack = tech_stack;
            self.active_id = Some(id.clone());
            self.push_recent(&id);
            return self
                .projects
                .iter()
                .find(|p| p.id == id)
                .ok_or_else(|| "project vanished after update".to_string());
        }

        let project = ProjectInfo {
            id: id.clone(),
            name,
            path: canonical,
            tech_stack,
            last_opened: now,
            provider: None,
            model: None,
            system_prompt: None,
        };
        self.projects.push(project);
        self.active_id = Some(id.clone());
        self.push_recent(&id);
        self.projects
            .last()
            .ok_or_else(|| "empty projects".to_string())
    }

    fn push_recent(&mut self, id: &str) {
        if let Some(pos) = self.recent_projects.iter().position(|x| x == id) {
            self.recent_projects.remove(pos);
        }
        self.recent_projects.push_front(id.to_string());
        while self.recent_projects.len() > 10 {
            self.recent_projects.pop_back();
        }
    }

    pub fn detect_tech_stack(path: &Path) -> Vec<String> {
        let mut stack = Vec::new();
        if path.join("Cargo.toml").exists() {
            stack.push("rust".to_string());
        }
        if path.join("package.json").exists() {
            stack.push("node".to_string());
        }
        if path.join("pyproject.toml").exists()
            || path.join("setup.py").exists()
            || path.join("requirements.txt").exists()
        {
            stack.push("python".to_string());
        }
        if path.join("go.mod").exists() {
            stack.push("go".to_string());
        }
        if path.join("Gemfile").exists() {
            stack.push("ruby".to_string());
        }
        if path.join("Cargo.toml").exists() {
            if let Ok(content) = fs::read_to_string(path.join("Cargo.toml")) {
                if content.contains("tauri") {
                    stack.push("tauri".to_string());
                }
            }
        }
        if path.join("package.json").exists() {
            if let Ok(content) = fs::read_to_string(path.join("package.json")) {
                if content.contains("\"next\"") || content.contains("next/dist") {
                    stack.push("nextjs".to_string());
                }
            }
        }
        if path.join(".git").exists() {
            stack.push("git".to_string());
        }
        if path.join("Dockerfile").exists() || path.join("docker-compose.yml").exists() {
            stack.push("docker".to_string());
        }
        stack
    }

    pub fn close(&mut self) {
        self.active_id = None;
    }

    pub fn switch(&mut self, id: &str) -> Result<&ProjectInfo, String> {
        if !self.projects.iter().any(|p| p.id == id) {
            return Err(format!("project not found: {}", id));
        }
        self.active_id = Some(id.to_string());
        self.push_recent(id);
        self.projects
            .iter()
            .find(|p| p.id == id)
            .ok_or_else(|| format!("project not found: {}", id))
    }

    pub fn recent(&self) -> Vec<&ProjectInfo> {
        self.recent_projects
            .iter()
            .filter_map(|id| self.projects.iter().find(|p| p.id == id.as_str()))
            .collect()
    }

    pub fn all(&self) -> &[ProjectInfo] {
        &self.projects
    }

    pub fn active(&self) -> Option<&ProjectInfo> {
        self.active_id
            .as_ref()
            .and_then(|id| self.projects.iter().find(|p| p.id == id.as_str()))
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("cannot create parent dir: {}", e))?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("serialization error: {}", e))?;
        fs::write(path, json).map_err(|e| format!("write error: {}", e))
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        let json = fs::read_to_string(path).map_err(|e| format!("read error: {}", e))?;
        serde_json::from_str(&json).map_err(|e| format!("deserialization error: {}", e))
    }

    pub fn set_config(&mut self, id: &str, provider: &str, model: &str, prompt: &str) {
        if let Some(project) = self.projects.iter_mut().find(|p| p.id == id) {
            project.provider = Some(provider.to_string());
            project.model = Some(model.to_string());
            project.system_prompt = Some(prompt.to_string());
        }
    }

    pub fn get_config(&self, id: &str) -> Option<(&str, &str, &str)> {
        self.projects.iter().find(|p| p.id == id).and_then(|p| {
            Some((
                p.provider.as_ref()?.as_str(),
                p.model.as_ref()?.as_str(),
                p.system_prompt.as_ref()?.as_str(),
            ))
        })
    }
}

impl Default for ProjectManager {
    fn default() -> Self {
        Self::new()
    }
}
