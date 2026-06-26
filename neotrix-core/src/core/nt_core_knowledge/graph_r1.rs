use std::collections::HashMap;

/// Actions the GraphR1 agent can take
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphAction {
    Retrieve(String),
    WebSearch(String),
    AddEntity(String, String),
    AddEdge(String, String, String),
    MergeEntities(String, String),
    Answer(String),
}

impl GraphAction {
    fn action_key(&self) -> String {
        match self {
            GraphAction::Retrieve(s) => format!("retrieve({})", s),
            GraphAction::WebSearch(s) => format!("web_search({})", s),
            GraphAction::AddEntity(n, _) => format!("add_entity({})", n),
            GraphAction::AddEdge(s, r, t) => format!("add_edge({},{},{})", s, r, t),
            GraphAction::MergeEntities(s, t) => format!("merge({},{})", s, t),
            GraphAction::Answer(s) => format!("answer({})", s),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GraphState {
    pub entity_count: usize,
    pub edge_count: usize,
    pub subgraph_embedding: Vec<f64>,
}

pub struct GraphR1Config {
    pub max_steps: usize,
    pub explore_rate: f64,
    pub gamma: f64,
    pub learning_rate: f64,
}

impl Default for GraphR1Config {
    fn default() -> Self {
        Self {
            max_steps: 20,
            explore_rate: 0.2,
            gamma: 0.95,
            learning_rate: 0.01,
        }
    }
}

pub struct GraphR1Agent {
    pub config: GraphR1Config,
    policy_logits: HashMap<(String, String), f64>,
    episode_trajectory: Vec<(String, GraphAction, f64)>,
    step_count: usize,
}

impl GraphR1Agent {
    pub fn new(config: GraphR1Config) -> Self {
        Self {
            config,
            policy_logits: HashMap::new(),
            episode_trajectory: Vec::new(),
            step_count: 0,
        }
    }

    pub fn reset_episode(&mut self) {
        self.episode_trajectory.clear();
        self.step_count = 0;
    }

    fn state_key(state: &GraphState) -> String {
        let prefix: Vec<f64> = state.subgraph_embedding.iter().take(8).copied().collect();
        format!("{:?}", prefix)
    }

    pub fn select_action(
        &mut self,
        state: &GraphState,
        valid_actions: &[GraphAction],
    ) -> GraphAction {
        if valid_actions.is_empty() {
            return GraphAction::Answer("no actions available".to_string());
        }
        if self.step_count >= self.config.max_steps {
            return GraphAction::Answer("max_steps_reached".to_string());
        }
        let explore: f64 = rand::random();
        if explore < self.config.explore_rate {
            let idx = (rand::random::<f64>() * valid_actions.len() as f64).floor() as usize;
            return valid_actions[idx.min(valid_actions.len() - 1)].clone();
        }
        let sk = Self::state_key(state);
        let mut best_idx = 0;
        let mut best_logit = f64::NEG_INFINITY;
        for (i, action) in valid_actions.iter().enumerate() {
            let key = (sk.clone(), action.action_key());
            let logit = self.policy_logits.get(&key).copied().unwrap_or(0.0);
            if logit > best_logit {
                best_logit = logit;
                best_idx = i;
            }
        }
        valid_actions[best_idx].clone()
    }

    pub fn record_step(&mut self, state: &GraphState, action: &GraphAction, reward: f64) {
        let sk = Self::state_key(state);
        self.episode_trajectory.push((sk, action.clone(), reward));
        self.step_count += 1;
    }

    pub fn update_policy(&mut self) -> f64 {
        let n = self.episode_trajectory.len();
        if n == 0 {
            return 0.0;
        }
        let gamma = self.config.gamma;
        let lr = self.config.learning_rate;
        let mut returns = vec![0.0; n];
        let mut running = 0.0;
        for i in (0..n).rev() {
            running = running * gamma + self.episode_trajectory[i].2;
            returns[i] = running;
        }
        let mut total_change = 0.0;
        for (i, (ref state_key, ref action, _)) in self.episode_trajectory.iter().enumerate() {
            let key = (state_key.clone(), action.action_key());
            let entry = self.policy_logits.entry(key).or_insert(0.0);
            let delta = lr * returns[i];
            *entry += delta;
            total_change += delta.abs();
        }
        total_change
    }

    pub fn run_episode(&mut self, _initial_query: &str, env: &mut dyn GraphEnv) -> (String, f64) {
        self.reset_episode();
        let mut cumulative_reward = 0.0;
        loop {
            if self.step_count >= self.config.max_steps {
                let state = env.get_state();
                return (
                    format!(
                        "max_steps reached. entities={}, edges={}",
                        state.entity_count, state.edge_count
                    ),
                    cumulative_reward,
                );
            }
            let state = env.get_state();
            let valid = env.valid_actions();
            let action = self.select_action(&state, &valid);
            if matches!(action, GraphAction::Answer(_)) {
                let (obs, reward) = env.execute(&action);
                self.record_step(&state, &action, reward);
                cumulative_reward += reward;
                let _ = self.update_policy();
                return (obs, cumulative_reward);
            }
            let (_, reward) = env.execute(&action);
            self.record_step(&state, &action, reward);
            cumulative_reward += reward;
        }
    }
}

pub trait GraphEnv {
    fn get_state(&self) -> GraphState;
    fn execute(&mut self, action: &GraphAction) -> (String, f64);
    fn valid_actions(&self) -> Vec<GraphAction>;
}

pub fn info_gain_reward(before: &GraphState, after: &GraphState) -> f64 {
    let new_entities = (after.entity_count - before.entity_count) as f64;
    let new_edges = (after.edge_count - before.edge_count) as f64;
    new_entities * 2.0 + new_edges * 1.5
}

pub struct SyntheticGraphEnv {
    entities: HashMap<String, String>,
    edges: Vec<(String, String, String)>,
    query: String,
    steps: usize,
}

impl SyntheticGraphEnv {
    pub fn new(query: &str) -> Self {
        let mut entities = HashMap::new();
        entities.insert(
            "Einstein".to_string(),
            "Theoretical physicist, relativity".to_string(),
        );
        entities.insert(
            "Newton".to_string(),
            "Physicist, calculus, gravity".to_string(),
        );
        entities.insert(
            "Feynman".to_string(),
            "Physicist, QED, diagrams".to_string(),
        );
        let edges = vec![
            (
                "Einstein".to_string(),
                "influenced".to_string(),
                "Newton".to_string(),
            ),
            (
                "Newton".to_string(),
                "influenced".to_string(),
                "Feynman".to_string(),
            ),
        ];
        Self {
            entities,
            edges,
            query: query.to_string(),
            steps: 0,
        }
    }
}

impl GraphEnv for SyntheticGraphEnv {
    fn get_state(&self) -> GraphState {
        GraphState {
            entity_count: self.entities.len(),
            edge_count: self.edges.len(),
            subgraph_embedding: vec![0.0; 16],
        }
    }

    fn execute(&mut self, action: &GraphAction) -> (String, f64) {
        self.steps += 1;
        match action {
            GraphAction::Retrieve(name) => {
                if let Some(desc) = self.entities.get(name) {
                    (format!("{}: {}", name, desc), 1.0)
                } else {
                    (format!("Entity '{}' not found", name), -0.5)
                }
            }
            GraphAction::WebSearch(_query) => (format!("Web search results for '{}'", _query), 0.5),
            GraphAction::AddEntity(name, desc) => {
                if self.entities.contains_key(name) {
                    (format!("Entity '{}' already exists", name), -1.0)
                } else {
                    let before = self.get_state();
                    self.entities.insert(name.clone(), desc.clone());
                    let after = self.get_state();
                    let reward = info_gain_reward(&before, &after);
                    (format!("Added entity '{}'", name), reward)
                }
            }
            GraphAction::AddEdge(src, rel, tgt) => {
                if !self.entities.contains_key(src) || !self.entities.contains_key(tgt) {
                    return (format!("Source or target entity missing"), -2.0);
                }
                if src == "Einstein" && rel == "influenced" && tgt == "Newton" {
                    self.edges.push((src.clone(), rel.clone(), tgt.clone()));
                    (format!("Correct edge added!"), 10.0)
                } else if src == "Newton" && rel == "influenced" && tgt == "Feynman" {
                    self.edges.push((src.clone(), rel.clone(), tgt.clone()));
                    (format!("Correct edge added!"), 10.0)
                } else {
                    self.edges.push((src.clone(), rel.clone(), tgt.clone()));
                    (format!("Edge added: {} --{}--> {}", src, rel, tgt), 2.0)
                }
            }
            GraphAction::MergeEntities(src, tgt) => {
                if !self.entities.contains_key(src) || !self.entities.contains_key(tgt) {
                    return (format!("One or both entities missing"), -2.0);
                }
                if src == tgt {
                    return (format!("Cannot merge entity with itself"), -1.0);
                }
                let _ = self.entities.remove(src);
                (format!("Merged '{}' into '{}'", src, tgt), 3.0)
            }
            GraphAction::Answer(text) => {
                let lower = text.to_lowercase();
                let mut reward = 0.0;
                let mut obs = String::new();
                if self.query.to_lowercase().contains("einstein") && lower.contains("einstein") {
                    reward += 5.0;
                    obs.push_str("Correctly mentions Einstein. ");
                }
                if lower.contains("physics") || lower.contains("scientist") {
                    reward += 2.0;
                    obs.push_str("Relevant domain detected. ");
                }
                if reward == 0.0 {
                    reward = -1.0;
                    obs.push_str("Answer does not address query. ");
                }
                obs.push_str(&format!("Answer provided: {}", text));
                (obs, reward)
            }
        }
    }

    fn valid_actions(&self) -> Vec<GraphAction> {
        vec![
            GraphAction::Retrieve("Einstein".to_string()),
            GraphAction::Retrieve("Newton".to_string()),
            GraphAction::Retrieve("Feynman".to_string()),
            GraphAction::WebSearch(self.query.clone()),
            GraphAction::AddEntity("Maxwell".to_string(), "Physicist, EM theory".to_string()),
            GraphAction::AddEdge(
                "Einstein".to_string(),
                "influenced".to_string(),
                "Newton".to_string(),
            ),
            GraphAction::AddEdge(
                "Newton".to_string(),
                "influenced".to_string(),
                "Feynman".to_string(),
            ),
            GraphAction::MergeEntities("Einstein".to_string(), "Newton".to_string()),
            GraphAction::Answer("The key figures are Einstein, Newton, and Feynman.".to_string()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_creation_default_config() {
        let config = GraphR1Config::default();
        let agent = GraphR1Agent::new(config);
        assert_eq!(agent.config.max_steps, 20);
        assert!((agent.config.explore_rate - 0.2).abs() < 1e-9);
        assert!((agent.config.gamma - 0.95).abs() < 1e-9);
        assert!((agent.config.learning_rate - 0.01).abs() < 1e-9);
        assert!(agent.policy_logits.is_empty());
        assert!(agent.episode_trajectory.is_empty());
    }

    #[test]
    fn test_select_action_returns_valid_action() {
        let config = GraphR1Config {
            explore_rate: 0.0,
            ..Default::default()
        };
        let mut agent = GraphR1Agent::new(config);
        let state = GraphState {
            entity_count: 3,
            edge_count: 2,
            subgraph_embedding: vec![0.0; 16],
        };
        let actions = vec![
            GraphAction::Retrieve("Einstein".to_string()),
            GraphAction::Answer("done".to_string()),
        ];
        let selected = agent.select_action(&state, &actions);
        assert!(actions.contains(&selected));
    }

    #[test]
    fn test_record_and_retrieve_trajectory() {
        let config = GraphR1Config::default();
        let mut agent = GraphR1Agent::new(config);
        let state = GraphState {
            entity_count: 3,
            edge_count: 2,
            subgraph_embedding: vec![0.5; 16],
        };
        let action = GraphAction::Retrieve("Einstein".to_string());
        agent.record_step(&state, &action, 1.0);
        assert_eq!(agent.episode_trajectory.len(), 1);
        assert_eq!(agent.step_count, 1);
        let (sk, a, r) = &agent.episode_trajectory[0];
        assert_eq!(*a, action);
        assert!((*r - 1.0).abs() < 1e-9);
        assert!(sk.len() > 0);
    }

    #[test]
    fn test_update_policy_empty_trajectory() {
        let config = GraphR1Config::default();
        let mut agent = GraphR1Agent::new(config);
        let change = agent.update_policy();
        assert!((change - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_update_policy_single_step() {
        let config = GraphR1Config {
            gamma: 0.9,
            learning_rate: 0.1,
            ..Default::default()
        };
        let mut agent = GraphR1Agent::new(config);
        let state = GraphState {
            entity_count: 3,
            edge_count: 2,
            subgraph_embedding: vec![1.0; 16],
        };
        let action = GraphAction::WebSearch("physics".to_string());
        agent.record_step(&state, &action, 5.0);
        let change = agent.update_policy();
        assert!(change > 0.0);
        let sk = GraphR1Agent::state_key(&state);
        let key = (sk, action.action_key());
        let logit = agent.policy_logits.get(&key).copied().unwrap_or(0.0);
        assert!((logit - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_run_episode_synthetic_env() {
        let config = GraphR1Config {
            max_steps: 10,
            explore_rate: 0.0,
            ..Default::default()
        };
        let mut agent = GraphR1Agent::new(config);
        let mut env = SyntheticGraphEnv::new("Who is Einstein?");
        let (answer, reward) = agent.run_episode("Who is Einstein?", &mut env);
        assert!(!answer.is_empty());
        assert!(reward != 0.0 || agent.episode_trajectory.len() > 0);
    }

    #[test]
    fn test_synthetic_env_get_state() {
        let env = SyntheticGraphEnv::new("test");
        let state = env.get_state();
        assert_eq!(state.entity_count, 3);
        assert_eq!(state.edge_count, 2);
        assert_eq!(state.subgraph_embedding.len(), 16);
    }

    #[test]
    fn test_synthetic_env_execute_returns_observation() {
        let mut env = SyntheticGraphEnv::new("Einstein");
        let action = GraphAction::Retrieve("Einstein".to_string());
        let (obs, _) = env.execute(&action);
        assert!(obs.contains("Einstein"));
        assert!(obs.contains("relativity"));
    }

    #[test]
    fn test_multiple_episodes_increase_policy() {
        let config = GraphR1Config {
            max_steps: 5,
            explore_rate: 0.0,
            learning_rate: 0.05,
            ..Default::default()
        };
        let mut agent = GraphR1Agent::new(config);
        let logits_before = agent.policy_logits.len();
        let mut env = SyntheticGraphEnv::new("physics");
        let _ = agent.run_episode("physics", &mut env);
        let logits_after = agent.policy_logits.len();
        assert!(logits_after >= logits_before);
        let mut env2 = SyntheticGraphEnv::new("physics");
        let _ = agent.run_episode("physics", &mut env2);
        assert!(agent.policy_logits.len() >= logits_after);
    }

    #[test]
    fn test_explore_rate_affects_exploration() {
        let config = GraphR1Config {
            explore_rate: 1.0,
            max_steps: 5,
            ..Default::default()
        };
        let mut agent = GraphR1Agent::new(config);
        let state = GraphState {
            entity_count: 3,
            edge_count: 2,
            subgraph_embedding: vec![0.0; 16],
        };
        let actions = vec![
            GraphAction::Retrieve("Einstein".to_string()),
            GraphAction::Answer("x".to_string()),
        ];
        let mut saw_random = false;
        for _ in 0..50 {
            let a = agent.select_action(&state, &actions);
            if a != actions[0] {
                saw_random = true;
                break;
            }
        }
        assert!(
            saw_random,
            "With explore_rate=1.0, expected non-deterministic picks"
        );
    }

    #[test]
    fn test_reset_episode_clears_trajectory() {
        let config = GraphR1Config::default();
        let mut agent = GraphR1Agent::new(config);
        let state = GraphState {
            entity_count: 3,
            edge_count: 2,
            subgraph_embedding: vec![1.0; 16],
        };
        agent.record_step(&state, &GraphAction::Retrieve("x".to_string()), 1.0);
        assert_eq!(agent.episode_trajectory.len(), 1);
        agent.reset_episode();
        assert!(agent.episode_trajectory.is_empty());
        assert_eq!(agent.step_count, 0);
    }

    #[test]
    fn test_answer_action_ends_episode() {
        let config = GraphR1Config {
            max_steps: 100,
            explore_rate: 0.0,
            ..Default::default()
        };
        let mut agent = GraphR1Agent::new(config);
        let mut env = SyntheticGraphEnv::new("test");
        let (answer, _) = agent.run_episode("test", &mut env);
        assert!(!answer.is_empty());
        assert!(agent.step_count <= 100);
    }

    #[test]
    fn test_info_gain_reward() {
        let before = GraphState {
            entity_count: 3,
            edge_count: 2,
            subgraph_embedding: vec![],
        };
        let after = GraphState {
            entity_count: 5,
            edge_count: 4,
            subgraph_embedding: vec![],
        };
        let reward = info_gain_reward(&before, &after);
        assert!((reward - (2.0 * 2.0 + 2.0 * 1.5)).abs() < 1e-9);
    }

    #[test]
    fn test_add_entity_reward_in_synthetic_env() {
        let mut env = SyntheticGraphEnv::new("test");
        let before = env.get_state();
        let action = GraphAction::AddEntity("Maxwell".to_string(), "EM theory".to_string());
        let (obs, reward) = env.execute(&action);
        let after = env.get_state();
        assert!(obs.contains("Maxwell"));
        assert!((reward - info_gain_reward(&before, &after)).abs() < 1e-9);
        assert_eq!(after.entity_count, 4);
    }
}
