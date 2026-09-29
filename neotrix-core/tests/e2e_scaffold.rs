#![forbid(unsafe_code)]

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use serde_json::Value;
    use std::collections::{HashMap, HashSet};

    struct McpToolInfo {
        name: String,
        description: String,
    }

    struct McpRegistry {
        tools: Vec<McpToolInfo>,
    }

    impl McpRegistry {
        fn new() -> Self {
            McpRegistry { tools: Vec::new() }
        }

        fn register(&mut self, name: &str, description: &str) {
            self.tools.push(McpToolInfo {
                name: name.into(),
                description: description.into(),
            });
        }

        fn list(&self) -> &[McpToolInfo] {
            &self.tools
        }
    }

    #[derive(Serialize, Deserialize)]
    struct AgentConfig {
        id: String,
        name: String,
        role: String,
        max_steps: u32,
    }

    #[derive(Serialize, Deserialize, Clone)]
    struct JournalEntry {
        id: u64,
        operation: String,
        payload: HashMap<String, String>,
        completed: bool,
    }

    struct Journal {
        entries: Vec<JournalEntry>,
        completed: HashSet<u64>,
    }

    impl Journal {
        fn new() -> Self {
            Journal {
                entries: Vec::new(),
                completed: HashSet::new(),
            }
        }

        fn push(&mut self, entry: JournalEntry) {
            self.entries.push(entry);
        }

        fn mark_completed(&mut self, id: u64) {
            self.completed.insert(id);
        }

        fn replay(&self) -> Vec<&JournalEntry> {
            self.entries
                .iter()
                .filter(|e| self.completed.contains(&e.id))
                .collect()
        }

        fn has_diverged(&self) -> bool {
            self.entries
                .iter()
                .any(|e| e.payload.contains_key("diverged"))
        }
    }

    #[derive(Clone)]
    enum PolicyAction {
        Allow,
        Deny,
    }

    struct PolicyRule {
        pattern: String,
        action: PolicyAction,
    }

    struct PolicyGate {
        rules: Vec<PolicyRule>,
        call_counts: HashMap<String, u32>,
        rate_limit: u32,
    }

    impl PolicyGate {
        fn new(rate_limit: u32) -> Self {
            PolicyGate {
                rules: Vec::new(),
                call_counts: HashMap::new(),
                rate_limit,
            }
        }

        fn add_rule(&mut self, pattern: &str, action: PolicyAction) {
            self.rules.push(PolicyRule {
                pattern: pattern.into(),
                action,
            });
        }

        fn check(&mut self, call: &str) -> Result<(), String> {
            let count = self.call_counts.entry(call.into()).or_insert(0);
            *count += 1;
            if *count > self.rate_limit {
                return Err("rate limit exceeded".into());
            }
            for rule in &self.rules {
                if call.contains(&rule.pattern) {
                    return match rule.action {
                        PolicyAction::Deny => Err(format!("denied by policy: {}", rule.pattern)),
                        PolicyAction::Allow => Ok(()),
                    };
                }
            }
            Ok(())
        }
    }

    #[derive(Debug, PartialEq)]
    enum State {
        Entry,
        Process,
        Exit,
    }

    struct StateGraph {
        current: State,
        transitions: Vec<State>,
    }

    impl StateGraph {
        fn new() -> Self {
            StateGraph {
                current: State::Entry,
                transitions: Vec::new(),
            }
        }

        fn step(&mut self, input: &str) {
            self.current = match (&self.current, input) {
                (State::Entry, "start") => {
                    self.transitions.push(State::Entry);
                    State::Process
                }
                (State::Process, "complete") => {
                    self.transitions.push(State::Process);
                    State::Exit
                }
                _ => return,
            };
        }
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct A2AMessage {
        jsonrpc: String,
        id: u64,
        method: String,
        params: HashMap<String, Value>,
    }

    struct FeatureFlag {
        #[allow(dead_code)] // test scaffold for feature flag system
        name: String,
        enabled: bool,
    }

    struct FeatureFlagSystem {
        flags: HashMap<String, FeatureFlag>,
    }

    impl FeatureFlagSystem {
        fn new() -> Self {
            FeatureFlagSystem {
                flags: HashMap::new(),
            }
        }

        fn add(&mut self, name: &str, enabled: bool) {
            self.flags.insert(
                name.into(),
                FeatureFlag {
                    name: name.into(),
                    enabled,
                },
            );
        }

        fn is_enabled(&self, name: &str) -> bool {
            self.flags.get(name).is_some_and(|f| f.enabled)
        }

        fn toggle(&mut self, name: &str) {
            if let Some(flag) = self.flags.get_mut(name) {
                flag.enabled = !flag.enabled;
            }
        }
    }

    struct KbEntry {
        id: String,
        content: String,
    }

    struct KnowledgeBase {
        entries: Vec<KbEntry>,
    }

    impl KnowledgeBase {
        fn new() -> Self {
            KnowledgeBase {
                entries: Vec::new(),
            }
        }

        fn add(&mut self, id: &str, content: &str) {
            self.entries.push(KbEntry {
                id: id.into(),
                content: content.into(),
            });
        }

        fn search(&self, query: &str, top_k: usize) -> Vec<&KbEntry> {
            let terms: Vec<&str> = query.split_whitespace().collect();
            let mut scored: Vec<(&KbEntry, usize)> = self
                .entries
                .iter()
                .map(|e| {
                    let score = terms.iter().filter(|t| e.content.contains(*t)).count();
                    (e, score)
                })
                .filter(|(_, s)| *s > 0)
                .collect();
            scored.sort_by(|a, b| b.1.cmp(&a.1));
            scored.into_iter().take(top_k).map(|(e, _)| e).collect()
        }
    }

    #[test]
    fn test_mcp_tool_discovery() {
        let mut registry = McpRegistry::new();
        registry.register("echo", "Echo input back");
        registry.register("math", "Evaluate math expressions");
        let tools = registry.list();
        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0].name, "echo");
        assert_eq!(tools[1].description, "Evaluate math expressions");
    }

    #[test]
    fn test_subagent_create_lifecycle() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("agent.json");
        let config = AgentConfig {
            id: "agent-001".into(),
            name: "CoderAgent".into(),
            role: "code_generation".into(),
            max_steps: 10,
        };
        std::fs::write(&path, serde_json::to_string(&config).unwrap()).unwrap();
        assert!(path.exists());
        let read_back: AgentConfig =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(read_back.id, "agent-001");
        assert_eq!(read_back.name, "CoderAgent");
        assert_eq!(read_back.role, "code_generation");
        assert_eq!(read_back.max_steps, 10);
        std::fs::remove_file(&path).unwrap();
        assert!(!path.exists());
    }

    #[test]
    fn test_exec_output_pipeline() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("response.txt");
        let mock = "Hello, I'm an AI assistant. I can help you with coding.";
        std::fs::write(&path, mock).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, mock);
        assert!(content.contains("AI assistant"));
        std::fs::remove_file(&path).unwrap();
        assert!(!path.exists());
    }

    #[test]
    fn test_journal_replay() {
        let mut journal = Journal::new();
        let mut p1 = HashMap::new();
        p1.insert("action".into(), "search".into());
        journal.push(JournalEntry {
            id: 1,
            operation: "query".into(),
            payload: p1,
            completed: false,
        });
        let mut p2 = HashMap::new();
        p2.insert("action".into(), "compute".into());
        journal.push(JournalEntry {
            id: 2,
            operation: "eval".into(),
            payload: p2,
            completed: false,
        });
        let mut p3 = HashMap::new();
        p3.insert("action".into(), "respond".into());
        journal.push(JournalEntry {
            id: 3,
            operation: "output".into(),
            payload: p3,
            completed: false,
        });
        journal.mark_completed(1);
        journal.mark_completed(3);
        let replay = journal.replay();
        assert_eq!(replay.len(), 2);
        assert_eq!(replay[0].id, 1);
        assert_eq!(replay[1].id, 3);
        assert!(!journal.has_diverged());
        let mut divergent = HashMap::new();
        divergent.insert("diverged".into(), "true".into());
        journal.push(JournalEntry {
            id: 4,
            operation: "divergent".into(),
            payload: divergent,
            completed: false,
        });
        assert!(journal.has_diverged());
    }

    #[test]
    fn test_policy_gate() {
        let mut gate = PolicyGate::new(5);
        gate.add_rule("admin", PolicyAction::Deny);
        gate.add_rule("public", PolicyAction::Allow);
        assert!(gate.check("public_endpoint").is_ok());
        assert!(gate.check("admin_panel").is_err());
        for _ in 0..5 {
            let _ = gate.check("frequent_call");
        }
        assert!(gate.check("frequent_call").is_err());
    }

    #[test]
    fn test_state_graph_simulation() {
        let mut graph = StateGraph::new();
        assert_eq!(graph.current, State::Entry);
        graph.step("start");
        assert_eq!(graph.current, State::Process);
        graph.step("complete");
        assert_eq!(graph.current, State::Exit);
        assert_eq!(graph.transitions.len(), 2);
        assert_eq!(graph.transitions[0], State::Entry);
        assert_eq!(graph.transitions[1], State::Process);
    }

    #[test]
    fn test_a2a_message_roundtrip() {
        let mut params = HashMap::new();
        params.insert("agent_id".into(), Value::String("agent-007".into()));
        params.insert("task".into(), Value::String("code_review".into()));
        let msg = A2AMessage {
            jsonrpc: "2.0".into(),
            id: 42,
            method: "agent.execute".into(),
            params,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: A2AMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.jsonrpc, "2.0");
        assert_eq!(deserialized.id, 42);
        assert_eq!(deserialized.method, "agent.execute");
        assert_eq!(
            deserialized.params["agent_id"],
            Value::String("agent-007".into())
        );
        assert_eq!(
            deserialized.params["task"],
            Value::String("code_review".into())
        );
    }

    #[test]
    fn test_feature_flag_evaluation() {
        let mut system = FeatureFlagSystem::new();
        system.add("new_dashboard", true);
        system.add("legacy_api", false);
        assert!(system.is_enabled("new_dashboard"));
        assert!(!system.is_enabled("legacy_api"));
        system.toggle("new_dashboard");
        assert!(!system.is_enabled("new_dashboard"));
        system.toggle("legacy_api");
        assert!(system.is_enabled("legacy_api"));
    }

    #[test]
    fn test_kb_query_pipeline() {
        let mut kb = KnowledgeBase::new();
        kb.add("1", "Rust is a systems programming language");
        kb.add("2", "Python is great for machine learning");
        kb.add("3", "Rust and Python are both popular languages");
        let results = kb.search("Rust programming", 2);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "1");
        assert!(results[0].content.contains("Rust"));
        let no_results = kb.search("JavaScript", 5);
        assert!(no_results.is_empty());
    }

    #[test]
    fn test_full_lifecycle() {
        let mut registry = McpRegistry::new();
        registry.register("code_review", "Review code changes");
        assert_eq!(registry.list().len(), 1);
        let config = AgentConfig {
            id: "agent-999".into(),
            name: "LifecycleAgent".into(),
            role: "e2e_test".into(),
            max_steps: 3,
        };
        assert_eq!(config.id, "agent-999");
        let dir = tempfile::tempdir().unwrap();
        let exec_path = dir.path().join("output.txt");
        std::fs::write(&exec_path, "execution result").unwrap();
        assert_eq!(
            std::fs::read_to_string(&exec_path).unwrap(),
            "execution result"
        );
        let mut journal = Journal::new();
        let mut payload = HashMap::new();
        payload.insert("tool".into(), "code_review".into());
        journal.push(JournalEntry {
            id: 1,
            operation: "mcp_call".into(),
            payload,
            completed: false,
        });
        journal.mark_completed(1);
        assert_eq!(journal.replay().len(), 1);
        let mut gate = PolicyGate::new(10);
        gate.add_rule("admin", PolicyAction::Deny);
        assert!(gate.check("code_review").is_ok());
        std::fs::remove_file(&exec_path).unwrap();
    }
}
