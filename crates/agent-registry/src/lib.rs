use std::collections::HashMap;

use agent_core::card::AgentCard;
use agent_core::registry::TransportInfo;
use agent_core::registry::AgentRegistration;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct RegisteredAgent {
    pub card: AgentCard,
    pub transport: TransportInfo,
    pub last_heartbeat: DateTime<Utc>,
}

impl RegisteredAgent {
    pub fn is_alive(&self, deadline: DateTime<Utc>) -> bool {
        self.last_heartbeat > deadline
    }
}

pub struct RegistryState {
    pub agents: RwLock<HashMap<String, RegisteredAgent>>,
    pub heartbeat_ttl_secs: i64,
}

impl RegistryState {
    pub fn new() -> Self {
        Self {
            agents: RwLock::new(HashMap::new()),
            heartbeat_ttl_secs: 30,
        }
    }

    pub async fn register(&self, registration: AgentRegistration) {
        let mut agents = self.agents.write().await;
        agents.insert(
            registration.card.name.clone(),
            RegisteredAgent {
                card: registration.card,
                transport: registration.transport,
                last_heartbeat: Utc::now(),
            },
        );
    }

    pub async fn unregister(&self, name: &str) -> bool {
        let mut agents = self.agents.write().await;
        agents.remove(name).is_some()
    }

    pub async fn search_by_skill(&self, skill_id: &str) -> Vec<AgentCard> {
        let agents = self.agents.read().await;
        let deadline = Utc::now() - chrono::Duration::seconds(self.heartbeat_ttl_secs);
        agents
            .values()
            .filter(|a| {
                a.is_alive(deadline) && a.card.skills.iter().any(|s| s.id == skill_id)
            })
            .map(|a| a.card.clone())
            .collect()
    }

    pub async fn search_by_tag(&self, tag: &str) -> Vec<AgentCard> {
        let agents = self.agents.read().await;
        let deadline = Utc::now() - chrono::Duration::seconds(self.heartbeat_ttl_secs);
        agents
            .values()
            .filter(|a| {
                a.is_alive(deadline)
                    && a.card.skills.iter().any(|s| s.tags.contains(&tag.to_string()))
            })
            .map(|a| a.card.clone())
            .collect()
    }

    pub async fn search_by_text(&self, text: &str) -> Vec<AgentCard> {
        let agents = self.agents.read().await;
        let deadline = Utc::now() - chrono::Duration::seconds(self.heartbeat_ttl_secs);
        let lower = text.to_lowercase();
        agents
            .values()
            .filter(|a| {
                if !a.is_alive(deadline) {
                    return false;
                }
                let card = &a.card;
                card.name.to_lowercase().contains(&lower)
                    || card.description.to_lowercase().contains(&lower)
                    || card.skills.iter().any(|s| {
                        s.name.to_lowercase().contains(&lower)
                            || s.description.to_lowercase().contains(&lower)
                    })
            })
            .map(|a| a.card.clone())
            .collect()
    }

    pub async fn list_all(&self) -> Vec<AgentCard> {
        let agents = self.agents.read().await;
        let deadline = Utc::now() - chrono::Duration::seconds(self.heartbeat_ttl_secs);
        agents
            .values()
            .filter(|a| a.is_alive(deadline))
            .map(|a| a.card.clone())
            .collect()
    }

    pub async fn heartbeat(&self, name: &str) -> bool {
        let mut agents = self.agents.write().await;
        if let Some(agent) = agents.get_mut(name) {
            agent.last_heartbeat = Utc::now();
            true
        } else {
            false
        }
    }

    pub async fn cleanup_dead(&self) -> usize {
        let mut agents = self.agents.write().await;
        let deadline = Utc::now() - chrono::Duration::seconds(self.heartbeat_ttl_secs * 2);
        let before = agents.len();
        agents.retain(|_, a| a.is_alive(deadline));
        before - agents.len()
    }

    pub async fn stats(&self) -> RegistryStats {
        let agents = self.agents.read().await;
        let deadline = Utc::now() - chrono::Duration::seconds(self.heartbeat_ttl_secs);
        let total = agents.len();
        let alive = agents.values().filter(|a| a.is_alive(deadline)).count();
        let skills: Vec<String> = agents
            .values()
            .flat_map(|a| a.card.skills.iter().map(|s| s.id.clone()))
            .collect();
        RegistryStats {
            total_agents: total,
            alive_agents: alive,
            unique_skills: {
                let mut s = skills;
                s.sort();
                s.dedup();
                s.len()
            },
            agents: agents
                .iter()
                .map(|(name, a)| AgentBrief {
                    name: name.clone(),
                    alive: a.is_alive(deadline),
                    last_heartbeat: a.last_heartbeat.to_rfc3339(),
                    version: a.card.version.clone(),
                    skills: a.card.skill_ids().into_iter().map(String::from).collect(),
                    skills_count: a.card.skills.len(),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RegistryStats {
    pub total_agents: usize,
    pub alive_agents: usize,
    pub unique_skills: usize,
    pub agents: Vec<AgentBrief>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentBrief {
    pub name: String,
    pub alive: bool,
    pub last_heartbeat: String,
    pub version: String,
    pub skills: Vec<String>,
    pub skills_count: usize,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub skill: Option<String>,
    pub tag: Option<String>,
    pub text: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_core::card::{AgentCapabilities, SkillDecl};

    fn make_registration(name: &str, skill_id: &str, tags: Vec<&str>) -> AgentRegistration {
        AgentRegistration {
            card: AgentCard {
                name: name.to_string(),
                description: format!("description for {}", name),
                url: format!("http://{}.local:0", name),
                version: "1.0.0".to_string(),
                capabilities: AgentCapabilities {
                    streaming: true,
                    push_notifications: false,
                },
                skills: vec![SkillDecl {
                    id: skill_id.to_string(),
                    name: format!("skill-{}", skill_id),
                    description: format!("does {}", skill_id),
                    tags: tags.into_iter().map(String::from).collect(),
                    examples: vec![],
                }],
                registry_url: None,
            },
            transport: TransportInfo {
                host: "localhost".to_string(),
                port: 9999,
                protocol: "http".to_string(),
            },
        }
    }

    fn make_registration_with(
        name: &str,
        desc: &str,
        skills: Vec<SkillDecl>,
    ) -> AgentRegistration {
        AgentRegistration {
            card: AgentCard {
                name: name.to_string(),
                description: desc.to_string(),
                url: "http://localhost:0".to_string(),
                version: "1.0.0".to_string(),
                capabilities: AgentCapabilities {
                    streaming: true,
                    push_notifications: false,
                },
                skills,
                registry_url: None,
            },
            transport: TransportInfo {
                host: "localhost".to_string(),
                port: 9999,
                protocol: "http".to_string(),
            },
        }
    }

    #[tokio::test]
    async fn test_new_state_is_empty() {
        let state = RegistryState::new();
        let stats = state.stats().await;
        assert_eq!(stats.total_agents, 0);
        assert_eq!(stats.alive_agents, 0);
        assert_eq!(state.heartbeat_ttl_secs, 30);
    }

    #[tokio::test]
    async fn test_register_and_list() {
        let state = RegistryState::new();
        let reg = make_registration("alice", "code-review", vec!["dev"]);
        state.register(reg).await;

        let agents = state.list_all().await;
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].name, "alice");
    }

    #[tokio::test]
    async fn test_register_and_unregister() {
        let state = RegistryState::new();
        let reg = make_registration("bob", "deploy", vec!["ops"]);
        state.register(reg).await;

        assert!(state.unregister("bob").await);
        let agents = state.list_all().await;
        assert!(agents.is_empty());
    }

    #[tokio::test]
    async fn test_unregister_nonexistent() {
        let state = RegistryState::new();
        assert!(!state.unregister("ghost").await);
    }

    #[tokio::test]
    async fn test_heartbeat() {
        let state = RegistryState::new();
        let reg = make_registration("carol", "test", vec!["qa"]);
        state.register(reg).await;

        // Verify initially alive
        let agents = state.list_all().await;
        assert_eq!(agents.len(), 1);

        // Successful heartbeat
        assert!(state.heartbeat("carol").await);
    }

    #[tokio::test]
    async fn test_heartbeat_nonexistent() {
        let state = RegistryState::new();
        assert!(!state.heartbeat("ghost").await);
    }

    #[tokio::test]
    async fn test_search_by_skill() {
        let state = RegistryState::new();
        state
            .register(make_registration("dave", "docker", vec!["devops"]))
            .await;
        state
            .register(make_registration("eve", "k8s", vec!["devops"]))
            .await;
        state
            .register(make_registration("frank", "docker", vec!["ci"]))
            .await;

        let docker_agents = state.search_by_skill("docker").await;
        assert_eq!(docker_agents.len(), 2);
        assert!(docker_agents.iter().any(|c| c.name == "dave"));
        assert!(docker_agents.iter().any(|c| c.name == "frank"));

        let k8s_agents = state.search_by_skill("k8s").await;
        assert_eq!(k8s_agents.len(), 1);
        assert_eq!(k8s_agents[0].name, "eve");
    }

    #[tokio::test]
    async fn test_search_by_tag() {
        let state = RegistryState::new();
        state
            .register(make_registration("grace", "monitor", vec!["observability"]))
            .await;
        state
            .register(make_registration("heidi", "alert", vec!["observability"]))
            .await;

        let obs = state.search_by_tag("observability").await;
        assert_eq!(obs.len(), 2);
    }

    #[tokio::test]
    async fn test_search_by_tag_no_match() {
        let state = RegistryState::new();
        state
            .register(make_registration("ivan", "build", vec!["ci"]))
            .await;

        let matches = state.search_by_tag("security").await;
        assert!(matches.is_empty());
    }

    #[tokio::test]
    async fn test_search_by_text_name() {
        let state = RegistryState::new();
        state
            .register(make_registration("alice-ml", "machine learning", vec!["ai"]))
            .await;

        let matches = state.search_by_text("alice").await;
        assert_eq!(matches.len(), 1);
    }

    #[tokio::test]
    async fn test_search_by_text_description() {
        let state = RegistryState::new();
        state
            .register(make_registration("bob-db", "database optimizer", vec!["data"]))
            .await;

        let matches = state.search_by_text("optimizer").await;
        assert_eq!(matches.len(), 1);
    }

    #[tokio::test]
    async fn test_search_by_text_case_insensitive() {
        let state = RegistryState::new();
        state
            .register(make_registration("MIXED", "Case Test", vec!["test"]))
            .await;

        let matches = state.search_by_text("mixed").await;
        assert_eq!(matches.len(), 1);
    }

    #[tokio::test]
    async fn test_search_by_text_skill_name() {
        let state = RegistryState::new();
        let s = SkillDecl {
            id: "skill-cache".into(),
            name: "cache-invalidation".into(),
            description: "invalidates caches".into(),
            tags: vec!["perf".into()],
            examples: vec![],
        };
        state
            .register(make_registration_with("mallory", "cache service", vec![s]))
            .await;

        let matches = state.search_by_text("invalidation").await;
        assert_eq!(matches.len(), 1);
    }

    #[tokio::test]
    async fn test_search_by_text_no_match() {
        let state = RegistryState::new();
        state
            .register(make_registration("nobody", "nothing", vec!["void"]))
            .await;

        let matches = state.search_by_text("zzzzz").await;
        assert!(matches.is_empty());
    }

    #[tokio::test]
    async fn test_search_skill_no_match() {
        let state = RegistryState::new();
        state
            .register(make_registration("oscar", "logging", vec!["infra"]))
            .await;

        let matches = state.search_by_skill("nonexistent-skill").await;
        assert!(matches.is_empty());
    }

    #[tokio::test]
    async fn test_stats() {
        let state = RegistryState::new();
        state
            .register(make_registration("p1", "skill-a", vec!["tag1"]))
            .await;
        state
            .register(make_registration("p2", "skill-b", vec!["tag2"]))
            .await;

        let stats = state.stats().await;
        assert_eq!(stats.total_agents, 2);
        assert_eq!(stats.alive_agents, 2);
        assert_eq!(stats.unique_skills, 2);
        assert_eq!(stats.agents.len(), 2);
    }

    #[tokio::test]
    async fn test_stats_dedup_skills() {
        let state = RegistryState::new();
        state
            .register(make_registration("q1", "shared-skill", vec!["t"]))
            .await;
        state
            .register(make_registration("q2", "shared-skill", vec!["t"]))
            .await;

        let stats = state.stats().await;
        assert_eq!(stats.total_agents, 2);
        assert_eq!(stats.unique_skills, 1);
    }

    #[tokio::test]
    async fn test_cleanup_dead() {
        let state = RegistryState::new();
        state
            .register(make_registration("r1", "ephemeral", vec!["tmp"]))
            .await;

        // Set heartbeat far in the past
        {
            let mut agents = state.agents.write().await;
            if let Some(agent) = agents.get_mut("r1") {
                agent.last_heartbeat =
                    Utc::now() - chrono::Duration::seconds(state.heartbeat_ttl_secs * 3);
            }
        }

        let cleaned = state.cleanup_dead().await;
        assert_eq!(cleaned, 1);

        let stats = state.stats().await;
        assert_eq!(stats.total_agents, 0);
    }

    #[tokio::test]
    async fn test_cleanup_dead_noop() {
        let state = RegistryState::new();
        state
            .register(make_registration("s1", "stable", vec!["live"]))
            .await;

        let cleaned = state.cleanup_dead().await;
        assert_eq!(cleaned, 0);
    }

    #[tokio::test]
    async fn test_multiple_agents_independent_heartbeats() {
        let state = RegistryState::new();
        state
            .register(make_registration("t1", "skill-x", vec!["a"]))
            .await;
        state
            .register(make_registration("t2", "skill-y", vec!["b"]))
            .await;

        // Only t1 heartbeat times out
        {
            let mut agents = state.agents.write().await;
            if let Some(agent) = agents.get_mut("t1") {
                agent.last_heartbeat =
                    Utc::now() - chrono::Duration::seconds(state.heartbeat_ttl_secs * 3);
            }
        }

        let alive = state.list_all().await;
        assert_eq!(alive.len(), 1);
        assert_eq!(alive[0].name, "t2");
    }

    #[tokio::test]
    async fn test_search_only_live_agents() {
        let state = RegistryState::new();
        state
            .register(make_registration("u1", "search-test", vec!["live"]))
            .await;

        // Make u1 dead
        {
            let mut agents = state.agents.write().await;
            if let Some(agent) = agents.get_mut("u1") {
                agent.last_heartbeat =
                    Utc::now() - chrono::Duration::seconds(state.heartbeat_ttl_secs * 3);
            }
        }

        let matches = state.search_by_skill("search-test").await;
        assert!(matches.is_empty(), "dead agent should not appear in search");
    }

    #[test]
    fn test_is_alive_recent() {
        let agent = RegisteredAgent {
            card: AgentCard {
                name: "test".into(),
                description: "".into(),
                url: "".into(),
                version: "1".into(),
                capabilities: AgentCapabilities {
                    streaming: false,
                    push_notifications: false,
                },
                skills: vec![],
                registry_url: None,
            },
            transport: TransportInfo {
                host: "localhost".into(),
                port: 0,
                protocol: "http".into(),
            },
            last_heartbeat: Utc::now(),
        };
        let deadline = Utc::now() - chrono::Duration::seconds(10);
        assert!(agent.is_alive(deadline));
    }

    #[test]
    fn test_is_alive_expired() {
        let agent = RegisteredAgent {
            card: AgentCard {
                name: "test".into(),
                description: "".into(),
                url: "".into(),
                version: "1".into(),
                capabilities: AgentCapabilities {
                    streaming: false,
                    push_notifications: false,
                },
                skills: vec![],
                registry_url: None,
            },
            transport: TransportInfo {
                host: "localhost".into(),
                port: 0,
                protocol: "http".into(),
            },
            last_heartbeat: Utc::now() - chrono::Duration::seconds(60),
        };
        let deadline = Utc::now() - chrono::Duration::seconds(10);
        assert!(!agent.is_alive(deadline));
    }
}
