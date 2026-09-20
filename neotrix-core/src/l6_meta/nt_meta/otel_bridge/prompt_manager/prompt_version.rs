#![deny(clippy::unwrap_used)]

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PromptVersion {
    pub id: String,
    pub name: String,
    pub version: u32,
    pub template: String,
    pub variables: Vec<String>,
    pub created_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderError {
    MissingVariable(String),
}

impl PromptVersion {
    pub fn new(
        id: String,
        name: String,
        version: u32,
        template: String,
        variables: Vec<String>,
        created_at: u64,
    ) -> Self {
        Self {
            id,
            name,
            version,
            template,
            variables,
            created_at,
        }
    }

    pub fn render(&self, args: &HashMap<String, String>) -> Result<String, RenderError> {
        let mut result = self.template.clone();
        for var in &self.variables {
            match args.get(var) {
                Some(value) => {
                    let placeholder = format!("{{{}}}", var);
                    result = result.replace(&placeholder, value);
                }
                None => return Err(RenderError::MissingVariable(var.clone())),
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_basic() {
        let prompt = PromptVersion::new(
            "p1".into(),
            "greeting".into(),
            1,
            "Hello {name}, welcome to {place}!".into(),
            vec!["name".into(), "place".into()],
            1000,
        );
        let mut args = HashMap::new();
        args.insert("name".into(), "Alice".into());
        args.insert("place".into(), "NeoTrix".into());
        assert_eq!(
            prompt.render(&args).unwrap(),
            "Hello Alice, welcome to NeoTrix!"
        );
    }

    #[test]
    fn test_render_missing_variable() {
        let prompt = PromptVersion::new(
            "p2".into(),
            "greeting".into(),
            1,
            "Hello {name}!".into(),
            vec!["name".into()],
            1000,
        );
        let args = HashMap::new();
        assert!(matches!(
            prompt.render(&args),
            Err(RenderError::MissingVariable(n)) if n == "name"
        ));
    }

    #[test]
    fn test_render_no_variables() {
        let prompt = PromptVersion::new(
            "p3".into(),
            "static".into(),
            1,
            "No placeholders here.".into(),
            vec![],
            1000,
        );
        let args = HashMap::new();
        assert_eq!(prompt.render(&args).unwrap(), "No placeholders here.");
    }

    #[test]
    fn test_render_duplicate_variable_in_template() {
        let prompt = PromptVersion::new(
            "p4".into(),
            "repeat".into(),
            1,
            "{x} and {x} again".into(),
            vec!["x".into()],
            1000,
        );
        let mut args = HashMap::new();
        args.insert("x".into(), "hello".into());
        assert_eq!(prompt.render(&args).unwrap(), "hello and hello again");
    }

    #[test]
    fn test_render_empty_variable_value() {
        let prompt = PromptVersion::new(
            "p5".into(),
            "empty_val".into(),
            1,
            "Hi {name}!".into(),
            vec!["name".into()],
            1000,
        );
        let mut args = HashMap::new();
        args.insert("name".into(), String::new());
        assert_eq!(prompt.render(&args).unwrap(), "Hi !");
    }

    #[test]
    fn test_render_multiple_missing_variables() {
        let prompt = PromptVersion::new(
            "p6".into(),
            "multi_missing".into(),
            1,
            "{a} {b} {c}".into(),
            vec!["a".into(), "b".into(), "c".into()],
            1000,
        );
        let mut args = HashMap::new();
        args.insert("a".into(), "1".into());
        // b and c are missing — should fail on first missing
        match prompt.render(&args) {
            Err(RenderError::MissingVariable(var)) => {
                assert!(var == "b" || var == "c");
            }
            _ => panic!("expected MissingVariable error"),
        }
    }

    #[test]
    fn test_prompt_version_clone() {
        let prompt = PromptVersion::new(
            "c1".into(),
            "clone_test".into(),
            2,
            "template {x}".into(),
            vec!["x".into()],
            5000,
        );
        let cloned = prompt.clone();
        assert_eq!(cloned.id, prompt.id);
        assert_eq!(cloned.name, prompt.name);
        assert_eq!(cloned.version, prompt.version);
        assert_eq!(cloned.template, prompt.template);
        assert_eq!(cloned.variables, prompt.variables);
    }

    #[test]
    fn test_render_debug_format() {
        let prompt = PromptVersion::new(
            "d1".into(),
            "debug".into(),
            1,
            "test".into(),
            vec![],
            1000,
        );
        let debug_str = format!("{:?}", prompt);
        assert!(debug_str.contains("PromptVersion"));
        assert!(debug_str.contains("debug"));
    }
}
