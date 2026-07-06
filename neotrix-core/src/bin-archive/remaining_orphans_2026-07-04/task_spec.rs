use serde::{Serialize, Deserialize};

/// A semantic task specification, inspired by CrewAI's Task class.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSpec {
    pub id: String,
    pub description: String,
    pub expected_output: Option<String>,
    pub agent_role: Option<String>,
    pub allowed_tools: Vec<String>,
    pub context_refs: Vec<String>,
    pub output_format: Option<OutputFormat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    Raw,
    Json,
    Markdown,
    Code(String),
}

impl TaskSpec {
    pub fn new(description: &str) -> Self {
        Self {
            id: format!("task-{}", uuid::Uuid::new_v4()),
            description: description.to_string(),
            expected_output: None,
            agent_role: None,
            allowed_tools: Vec::new(),
            context_refs: Vec::new(),
            output_format: None,
        }
    }

    pub fn with_expected_output(mut self, output: &str) -> Self {
        self.expected_output = Some(output.to_string());
        self
    }

    pub fn assigned_to(mut self, role: &str) -> Self {
        self.agent_role = Some(role.to_string());
        self
    }

    pub fn with_tools(mut self, tools: Vec<String>) -> Self {
        self.allowed_tools = tools;
        self
    }

    pub fn depends_on(mut self, task_ids: Vec<String>) -> Self {
        self.context_refs = task_ids;
        self
    }

    pub fn with_format(mut self, format: OutputFormat) -> Self {
        self.output_format = Some(format);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_spec_creation() {
        let task = TaskSpec::new("Write a binary search tree in Rust")
            .with_expected_output("Working BST implementation with tests")
            .assigned_to("coder")
            .with_tools(vec!["rustc".into(), "cargo".into()]);

        assert!(task.description.contains("binary search tree"));
        assert_eq!(task.agent_role, Some("coder".into()));
        assert_eq!(task.allowed_tools.len(), 2);
    }

    #[test]
    fn test_task_with_dependencies() {
        let task1 = TaskSpec::new("Design API");
        let task2 = TaskSpec::new("Implement API")
            .depends_on(vec![task1.id.clone()]);
        assert_eq!(task2.context_refs.len(), 1);
    }

    #[test]
    fn test_output_format() {
        let task = TaskSpec::new("Generate report")
            .with_format(OutputFormat::Markdown);
        assert!(matches!(task.output_format, Some(OutputFormat::Markdown)));
    }

    #[test]
    fn test_default_values() {
        let task = TaskSpec::new("Simple task");
        assert!(task.expected_output.is_none());
        assert!(task.allowed_tools.is_empty());
        assert!(task.context_refs.is_empty());
    }

    #[test]
    fn test_unique_ids() {
        let a = TaskSpec::new("A");
        let b = TaskSpec::new("B");
        assert_ne!(a.id, b.id);
    }
}
