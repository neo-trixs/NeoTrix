#[derive(Debug, Clone)]
pub struct ScaffoldTemplate {
    pub name: String,
    pub language: String,
    pub files: Vec<String>,
    pub patterns: Vec<String>,
}

pub struct MgmScaffold {
    templates: Vec<ScaffoldTemplate>,
}

impl MgmScaffold {
    pub fn new() -> Self {
        Self {
            templates: vec![
                ScaffoldTemplate {
                    name: "rust-cli".into(),
                    language: "rust".into(),
                    files: vec!["main.rs".into(), "lib.rs".into(), "Cargo.toml".into()],
                    patterns: vec!["error-handling".into(), "cli-args".into()],
                },
                ScaffoldTemplate {
                    name: "python-module".into(),
                    language: "python".into(),
                    files: vec!["__init__.py".into(), "core.py".into(), "setup.py".into()],
                    patterns: vec!["docstrings".into(), "tests".into()],
                },
            ],
        }
    }

    pub fn get_template(&self, name: &str) -> Option<&ScaffoldTemplate> {
        self.templates.iter().find(|t| t.name == name)
    }

    pub fn list_templates(&self) -> Vec<&str> {
        self.templates.iter().map(|t| t.name.as_str()).collect()
    }

    pub fn add_template(&mut self, t: ScaffoldTemplate) {
        self.templates.push(t);
    }
}

impl Default for MgmScaffold {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_template() {
        let s = MgmScaffold::new();
        assert!(s.get_template("rust-cli").is_some());
        assert!(s.get_template("missing").is_none());
    }

    #[test]
    fn test_list() {
        let s = MgmScaffold::new();
        assert_eq!(s.list_templates().len(), 2);
    }
}
