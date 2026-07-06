use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: String,
    pub language: String,
    pub last_opened: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechStack {
    pub languages: Vec<String>,
    pub frameworks: Vec<String>,
    pub has_docker: bool,
    pub has_ci: bool,
    pub package_manager: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectConfig {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub system_prompt: Option<String>,
}

pub struct ProjectManager {
    projects: Vec<Project>,
    active: Option<String>,
    recent: VecDeque<String>,
}

impl ProjectManager {
    pub fn new() -> Self {
        Self {
            projects: Vec::new(),
            active: None,
            recent: VecDeque::with_capacity(11),
        }
    }

    pub fn open(&mut self, path: &str) -> Result<Project, String> {
        let p = Path::new(path);
        if !p.exists() {
            return Err(format!("Path does not exist: {}", path));
        }
        let name = p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        let language = scan_language(p);
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();

        let project = Project {
            id,
            name,
            path: path.to_string(),
            language,
            last_opened: now,
        };

        if let Some(pos) = self.projects.iter().position(|pr| pr.path == path) {
            let mut p = self.projects.remove(pos);
            p.last_opened = now;
            self.projects.insert(0, p.clone());
            self.active = Some(p.id.clone());
            self.push_recent(p.id.clone());
            return Ok(p);
        }

        self.push_recent(project.id.clone());
        self.active = Some(project.id.clone());
        self.projects.insert(0, project.clone());
        Ok(project)
    }

    pub fn switch(&mut self, id: &str) -> Result<(), String> {
        if !self.projects.iter().any(|p| p.id == id) {
            return Err(format!("Project not found: {}", id));
        }
        self.active = Some(id.to_string());
        self.push_recent(id.to_string());
        Ok(())
    }

    pub fn list(&self) -> Vec<Project> {
        self.projects.clone()
    }

    pub fn recent(&self) -> Vec<Project> {
        let mut result = Vec::new();
        for id in &self.recent {
            if let Some(p) = self.projects.iter().find(|pr| pr.id == *id) {
                result.push(p.clone());
            }
        }
        result
    }

    fn push_recent(&mut self, id: String) {
        if let Some(pos) = self.recent.iter().position(|x| *x == id) {
            self.recent.remove(pos);
        }
        self.recent.push_front(id);
        while self.recent.len() > 10 {
            self.recent.pop_back();
        }
    }
}

fn scan_language(path: &Path) -> String {
    if path.join("Cargo.toml").exists() {
        "Rust".into()
    } else if path.join("package.json").exists() {
        "JavaScript/TypeScript".into()
    } else if path.join("pyproject.toml").exists() || path.join("setup.py").exists() {
        "Python".into()
    } else if path.join("go.mod").exists() {
        "Go".into()
    } else if path.join("build.gradle").exists() || path.join("build.gradle.kts").exists() {
        "Java/Kotlin".into()
    } else {
        "Unknown".into()
    }
}

pub fn scan_tech_stack(path: &str) -> TechStack {
    let dir = Path::new(path);
    let mut languages = Vec::new();
    let mut frameworks = Vec::new();
    let mut package_manager = None;

    if dir.join("Cargo.toml").exists() {
        languages.push("Rust".into());
        if let Ok(content) = std::fs::read_to_string(dir.join("Cargo.toml")) {
            if content.contains("tauri") {
                frameworks.push("Tauri".into());
            }
            if content.contains("axum") {
                frameworks.push("Axum".into());
            }
        }
    }
    if dir.join("package.json").exists() {
        languages.push("JavaScript/TypeScript".into());
        if dir.join("yarn.lock").exists() {
            package_manager = Some("yarn".into());
        } else if dir.join("pnpm-lock.yaml").exists() {
            package_manager = Some("pnpm".into());
        } else if dir.join("bun.lockb").exists() {
            package_manager = Some("bun".into());
        } else {
            package_manager = Some("npm".into());
        }
        if let Ok(content) = std::fs::read_to_string(dir.join("package.json")) {
            if content.contains("react") || content.contains("next") {
                frameworks.push("React/Next.js".into());
            }
            if content.contains("vue") {
                frameworks.push("Vue".into());
            }
            if content.contains("svelte") {
                frameworks.push("Svelte".into());
            }
        }
    }
    if dir.join("pyproject.toml").exists() || dir.join("setup.py").exists() {
        languages.push("Python".into());
        if let Ok(content) = std::fs::read_to_string(dir.join("pyproject.toml")) {
            if content.contains("django") {
                frameworks.push("Django".into());
            }
            if content.contains("fastapi") {
                frameworks.push("FastAPI".into());
            }
            if content.contains("flask") {
                frameworks.push("Flask".into());
            }
        }
    }
    if dir.join("go.mod").exists() {
        languages.push("Go".into());
    }
    let has_docker = dir.join("Dockerfile").exists() || dir.join("docker-compose.yml").exists();
    let has_ci = dir.join(".github/workflows").exists() || dir.join(".gitlab-ci.yml").exists();

    TechStack {
        languages,
        frameworks,
        has_docker,
        has_ci,
        package_manager,
    }
}

fn config_dir() -> Result<std::path::PathBuf, String> {
    let base = dirs::home_dir()
        .ok_or_else(|| "Cannot find home directory".to_string())?
        .join(".neotrix")
        .join("projects");
    std::fs::create_dir_all(&base).map_err(|e| e.to_string())?;
    Ok(base)
}

pub fn load_config(project_id: &str) -> Result<ProjectConfig, String> {
    let path = config_dir()?.join(format!("{}.json", project_id));
    if !path.exists() {
        return Ok(ProjectConfig::default());
    }
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

pub fn save_config(project_id: &str, config: &ProjectConfig) -> Result<(), String> {
    let path = config_dir()?.join(format!("{}.json", project_id));
    let content = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_project_open_and_list() {
        let dir = std::env::temp_dir().join("neotrix_test_proj_open");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("failed to create test directory");
        fs::write(dir.join("Cargo.toml"), "").expect("failed to write Cargo.toml");

        let mut mgr = ProjectManager::new();
        let project = mgr
            .open(dir.to_str().expect("temp path is not valid UTF-8"))
            .expect("failed to open project");
        assert_eq!(
            project.name,
            dir.file_name()
                .expect("dir has no file name")
                .to_str()
                .expect("file name is not valid UTF-8")
        );
        assert_eq!(project.language, "Rust");
        assert!(!mgr.list().is_empty());
        assert_eq!(mgr.recent().len(), 1);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_project_switch() {
        let mut mgr = ProjectManager::new();
        let dir1 = std::env::temp_dir().join("neotrix_test_switch_a");
        let dir2 = std::env::temp_dir().join("neotrix_test_switch_b");
        let _ = fs::remove_dir_all(&dir1);
        let _ = fs::remove_dir_all(&dir2);
        fs::create_dir_all(&dir1).expect("failed to create dir1");
        fs::create_dir_all(&dir2).expect("failed to create dir2");
        fs::write(dir1.join("Cargo.toml"), "").expect("failed to write Cargo.toml");
        fs::write(dir2.join("package.json"), "{}").expect("failed to write package.json");

        let p1 = mgr
            .open(dir1.to_str().expect("dir1 path is not valid UTF-8"))
            .expect("failed to open dir1");
        let _p2 = mgr
            .open(dir2.to_str().expect("dir2 path is not valid UTF-8"))
            .expect("failed to open dir2");
        mgr.switch(&p1.id).expect("failed to switch project");
        assert_eq!(
            mgr.recent()
                .first()
                .expect("recent list is empty")
                .id,
            p1.id
        );

        let _ = fs::remove_dir_all(&dir1);
        let _ = fs::remove_dir_all(&dir2);
    }

    #[test]
    fn test_tech_stack_scan() {
        let dir = std::env::temp_dir().join("neotrix_test_techstack");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("failed to create test directory for tech stack");
        fs::write(
            dir.join("package.json"),
            r#"{"dependencies": {"react": "^18"}}"#,
        )
        .expect("failed to write package.json");
        fs::write(dir.join("Dockerfile"), "").expect("failed to write Dockerfile");
        fs::create_dir_all(dir.join(".github/workflows"))
            .expect("failed to create workflows directory");
        fs::write(dir.join(".github/workflows/ci.yml"), "").expect("failed to write ci.yml");

        let ts = scan_tech_stack(dir.to_str().expect("dir path is not valid UTF-8"));
        assert!(ts.languages.contains(&"JavaScript/TypeScript".to_string()));
        assert!(ts.frameworks.contains(&"React/Next.js".to_string()));
        assert!(ts.has_docker);
        assert!(ts.has_ci);
        assert_eq!(
            ts.package_manager.expect("expected package manager to be Some"),
            "npm"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_config_save_load() {
        let id = "test-config-001";
        let cfg = ProjectConfig {
            provider: Some("openai".into()),
            model: Some("gpt-4".into()),
            system_prompt: Some("You are a helpful assistant.".into()),
        };
        save_config(id, &cfg).expect("failed to save config");
        let loaded = load_config(id).expect("failed to load config");
        assert_eq!(loaded.provider, Some("openai".into()));
        assert_eq!(loaded.model, Some("gpt-4".into()));

        let path = config_dir()
            .expect("failed to get config directory")
            .join(format!("{}.json", id));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_project_open_nonexistent() {
        let mut mgr = ProjectManager::new();
        let result = mgr.open("/nonexistent/path");
        assert!(result.is_err());
    }
}
