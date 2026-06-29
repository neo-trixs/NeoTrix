use std::collections::HashMap;

/// A step yielded by an agent workflow
#[derive(Debug, Clone)]
pub enum YieldedStep {
    ToolCall {
        tool: String,
        args: HashMap<String, String>,
    },
    Thought {
        content: String,
    },
    SubAgent {
        agent_id: String,
        prompt: String,
        capabilities: Vec<String>,
    },
    Return {
        value: String,
    },
    Branch {
        condition: String,
        if_true: Vec<YieldedStep>,
        if_false: Vec<YieldedStep>,
    },
    Loop {
        steps: Vec<YieldedStep>,
        max_iterations: usize,
    },
    Parallel {
        steps: Vec<Vec<YieldedStep>>,
    },
}

/// Generator for producing agent steps in sequence
pub struct StepGenerator {
    steps: Vec<YieldedStep>,
    position: usize,
    context: HashMap<String, String>,
    max_steps: usize,
}

impl StepGenerator {
    pub fn new(max_steps: usize) -> Self {
        Self {
            steps: Vec::new(),
            position: 0,
            context: HashMap::new(),
            max_steps,
        }
    }

    pub fn add_steps(&mut self, steps: Vec<YieldedStep>) {
        self.steps.extend(steps);
    }

    pub fn add_step(&mut self, step: YieldedStep) {
        self.steps.push(step);
    }

    pub fn next_step(&mut self) -> Option<YieldedStep> {
        if self.position >= self.steps.len() || self.position >= self.max_steps {
            return None;
        }
        let step = self.steps[self.position].clone();
        self.position += 1;
        Some(step)
    }

    pub fn peek(&self) -> Option<&YieldedStep> {
        self.steps.get(self.position)
    }

    pub fn set_context(&mut self, key: &str, value: &str) {
        self.context.insert(key.to_string(), value.to_string());
    }

    pub fn get_context(&self, key: &str) -> Option<&String> {
        self.context.get(key)
    }

    pub fn reset(&mut self) {
        self.position = 0;
    }

    pub fn remaining(&self) -> usize {
        self.steps.len().saturating_sub(self.position)
    }

    pub fn len(&self) -> usize {
        self.steps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    pub fn is_done(&self) -> bool {
        self.position >= self.steps.len()
    }

    pub fn file_picker(pattern: &str) -> Self {
        let mut g = Self::new(10);
        g.add_steps(vec![
            YieldedStep::Thought {
                content: format!("Finding files matching {}", pattern),
            },
            YieldedStep::ToolCall {
                tool: "glob".into(),
                args: HashMap::from([("pattern".into(), pattern.into())]),
            },
            YieldedStep::Return {
                value: "files".into(),
            },
        ]);
        g
    }

    pub fn planner(task: &str) -> Self {
        let mut g = Self::new(20);
        g.add_steps(vec![
            YieldedStep::Thought {
                content: format!("Planning steps for: {}", task),
            },
            YieldedStep::ToolCall {
                tool: "analyze".into(),
                args: HashMap::from([("task".into(), task.into())]),
            },
            YieldedStep::SubAgent {
                agent_id: "file-picker".into(),
                prompt: "find relevant files".into(),
                capabilities: vec!["code_search".into()],
            },
            YieldedStep::Return {
                value: "plan".into(),
            },
        ]);
        g
    }

    pub fn editor(file_path: &str, content: &str) -> Self {
        let mut g = Self::new(10);
        g.add_steps(vec![
            YieldedStep::Thought {
                content: format!("Editing {}", file_path),
            },
            YieldedStep::ToolCall {
                tool: "read_file".into(),
                args: HashMap::from([("path".into(), file_path.into())]),
            },
            YieldedStep::ToolCall {
                tool: "edit_file".into(),
                args: HashMap::from([
                    ("path".into(), file_path.into()),
                    ("content".into(), content.into()),
                ]),
            },
            YieldedStep::Return {
                value: file_path.into(),
            },
        ]);
        g
    }

    pub fn reviewer(file_path: &str) -> Self {
        let mut g = Self::new(15);
        g.add_steps(vec![
            YieldedStep::Thought {
                content: format!("Reviewing {}", file_path),
            },
            YieldedStep::ToolCall {
                tool: "read_file".into(),
                args: HashMap::from([("path".into(), file_path.into())]),
            },
            YieldedStep::ToolCall {
                tool: "code_review".into(),
                args: HashMap::from([("path".into(), file_path.into())]),
            },
            YieldedStep::Branch {
                condition: "has_issues".into(),
                if_true: vec![YieldedStep::ToolCall {
                    tool: "report".into(),
                    args: HashMap::from([("severity".into(), "warning".into())]),
                }],
                if_false: vec![YieldedStep::Thought {
                    content: "No issues found".into(),
                }],
            },
            YieldedStep::Return {
                value: "review_done".into(),
            },
        ]);
        g
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_step_generation() {
        let mut gen = StepGenerator::new(10);
        gen.add_step(YieldedStep::Thought {
            content: "step1".into(),
        });
        gen.add_step(YieldedStep::Thought {
            content: "step2".into(),
        });
        gen.add_step(YieldedStep::Return {
            value: "done".into(),
        });
        assert_eq!(gen.len(), 3);
        assert_eq!(gen.remaining(), 3);
        let s1 = gen.next_step();
        assert!(matches!(s1, Some(YieldedStep::Thought { .. })));
        let s2 = gen.next_step();
        assert!(matches!(s2, Some(YieldedStep::Thought { .. })));
        let s3 = gen.next_step();
        assert!(matches!(s3, Some(YieldedStep::Return { .. })));
        assert!(gen.next_step().is_none());
    }

    #[test]
    fn test_peek_next_remaining_done() {
        let mut gen = StepGenerator::new(10);
        gen.add_step(YieldedStep::Thought {
            content: "peek me".into(),
        });
        assert!(!gen.is_done());
        assert_eq!(gen.remaining(), 1);
        let peeked = gen.peek();
        assert!(peeked.is_some());
        assert_eq!(gen.remaining(), 1);
        let _consumed = gen.next_step();
        assert!(gen.is_done());
        assert_eq!(gen.remaining(), 0);
    }

    #[test]
    fn test_context_storage_and_retrieval() {
        let mut gen = StepGenerator::new(10);
        gen.set_context("key1", "value1");
        gen.set_context("key2", "value2");
        assert_eq!(gen.get_context("key1"), Some(&"value1".to_string()));
        assert_eq!(gen.get_context("key2"), Some(&"value2".to_string()));
        assert_eq!(gen.get_context("nonexistent"), None);
    }

    #[test]
    fn test_file_picker_workflow() -> Result<(), String> {
        let mut gen = StepGenerator::file_picker("*.rs");
        assert_eq!(gen.len(), 3);
        let t = gen.next_step();
        assert!(matches!(t, Some(YieldedStep::Thought { .. })));
        let tc = gen.next_step();
        let (tool, args) = match tc {
            Some(YieldedStep::ToolCall { tool, args }) => (tool, args),
            other => return Err(format!("expected ToolCall, got {:?}", other)),
        };
        assert_eq!(tool, "glob");
        assert_eq!(args.get("pattern"), Some(&"*.rs".to_string()));
        let r = gen.next_step();
        assert!(matches!(r, Some(YieldedStep::Return { .. })));
        Ok(())
    }

    #[test]
    fn test_branch_step_structure() -> Result<(), String> {
        let mut gen = StepGenerator::reviewer("test.rs");
        // consume thought and tool calls to get to branch
        gen.next_step(); // thought
        gen.next_step(); // read_file
        gen.next_step(); // code_review
        let branch = gen.peek();
        let (condition, if_true, if_false) = match branch {
            Some(YieldedStep::Branch {
                condition,
                if_true,
                if_false,
            }) => (condition, if_true, if_false),
            other => return Err(format!("expected Branch step, got {:?}", other)),
        };
        assert_eq!(condition, "has_issues");
        assert_eq!(if_true.len(), 1);
        assert_eq!(if_false.len(), 1);
        Ok(())
    }
}
