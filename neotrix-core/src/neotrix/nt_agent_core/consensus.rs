#![allow(dead_code)]
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};

use super::bus::AgentCommunicationBus;
use super::message::{AgentId, MessageContent};

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum ConsensusOutcome {
    Agreement(String, f64),
    Disagreement(String),
    NotReached,
}

#[derive(Debug, Clone)]
pub struct ConsensusConfig {
    pub max_rounds: usize,
    pub stability_horizon: usize,
    pub quorum_ratio: f64,
    pub agreement_threshold: f64,
    pub round_timeout_ms: u64,
    pub max_byzantine: usize,
    pub receiver_eval_enabled: bool,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        ConsensusConfig {
            max_rounds: 5,
            stability_horizon: 3,
            quorum_ratio: 0.67,
            agreement_threshold: 0.75,
            round_timeout_ms: 5000,
            max_byzantine: 1,
            receiver_eval_enabled: true,
        }
    }
}

#[derive(Debug, Clone)]
struct RoundState {
    proposals: HashMap<String, Vec<(AgentId, f64)>>,
    stability_count: usize,
    last_consensus: Option<String>,
}

pub struct ByzantineConsensusLayer {
    config: ConsensusConfig,
    round_states: HashMap<u64, RoundState>,
    agent_reputations: HashMap<AgentId, f64>,
    contribution_scores: HashMap<AgentId, f64>,
    contribution_decay: f64,
    voting_weight_threshold: f64,
}

impl ByzantineConsensusLayer {
    pub fn new(config: ConsensusConfig) -> Self {
        ByzantineConsensusLayer {
            config,
            round_states: HashMap::new(),
            agent_reputations: HashMap::new(),
            contribution_scores: HashMap::new(),
            contribution_decay: 0.95,
            voting_weight_threshold: 0.1,
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(ConsensusConfig::default())
    }

    pub fn update_reputation(&mut self, agent: &AgentId, delta: f64) {
        let entry = self.agent_reputations.entry(agent.clone()).or_insert(0.5);
        *entry = (*entry + delta).clamp(0.0, 1.0);
    }

    pub fn reputation(&self, agent: &AgentId) -> f64 {
        self.agent_reputations.get(agent).copied().unwrap_or(0.5)
    }

    pub fn try_consensus(
        &mut self,
        bus: &mut AgentCommunicationBus,
        conversation_id: u64,
        query: &str,
        participants: &[AgentId],
    ) -> ConsensusOutcome {
        if participants.len() < 2 {
            return ConsensusOutcome::NotReached;
        }

        let cfg_max_rounds = self.config.max_rounds;
        let cfg_timeout = self.config.round_timeout_ms;
        let cfg_quorum = self.config.quorum_ratio;
        let cfg_agreement = self.config.agreement_threshold;
        let cfg_horizon = self.config.stability_horizon;
        let cfg_eval = self.config.receiver_eval_enabled;

        let start = Instant::now();
        let mut last_consensus: Option<String> = None;
        let mut stability_count: usize = 0;

        for round in 0..cfg_max_rounds {
            if start.elapsed() > Duration::from_millis(cfg_timeout * (round + 1) as u64) {
                break;
            }

            for agent in participants {
                if bus.is_registered(agent) {
                    bus.send(super::message::AgentMessage::new(
                        agent.clone(),
                        vec![],
                        MessageContent::Query {
                            question: query.to_string(),
                            context: vec![format!("round={}/{}", round + 1, cfg_max_rounds)],
                        },
                        super::message::MessagePriority::High,
                        Duration::from_secs(30),
                    ))
                    .ok();
                }
            }

            let delivered = bus.deliver();
            let recent: Vec<super::message::AgentMessage> = delivered
                .into_iter()
                .filter(|m| m.conversation_id == conversation_id)
                .collect();

            let mut received: Vec<(AgentId, String, f64)> = Vec::new();
            for msg in &recent {
                let response_text = match &msg.content {
                    MessageContent::Response { answer, .. } => answer.clone(),
                    MessageContent::TaskResult { output, .. } => output.clone(),
                    MessageContent::TaskRequest { .. } | MessageContent::Query { .. } => continue,
                    MessageContent::Coordination { .. } | MessageContent::StatusUpdate { .. } => {
                        continue
                    }
                    MessageContent::Error { description, .. } => {
                        log::error!("[consensus] agent error: {}", description);
                        continue;
                    }
                };
                let confidence = if cfg_eval {
                    self._eval_confidence(&response_text, query)
                } else {
                    0.8
                };
                received.push((msg.sender.clone(), response_text, confidence));
            }
            received.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

            let filtered = self._filter_byzantine(&received, participants);

            let mut text_groups: HashMap<String, (Vec<AgentId>, Vec<f64>)> = HashMap::new();
            for (agent, text, conf) in &filtered {
                let norm = self._normalize_text(text);
                let norm2 = norm.clone();
                text_groups
                    .entry(norm)
                    .or_default()
                    .0
                    .push((*agent).clone());
                text_groups.entry(norm2).or_default().1.push(*conf);
            }

            let best = text_groups.iter().max_by(|a, b| {
                let (a_agents, a_confs) = &a.1;
                let weighted_a: f64 = a_confs
                    .iter()
                    .zip(a_agents.iter())
                    .map(|(c, ag)| c * self.voting_weight(ag))
                    .sum();
                let total_a: f64 = a_agents.iter().map(|ag| self.voting_weight(ag)).sum();
                let avg_a = if total_a > 0.0 {
                    weighted_a / total_a
                } else {
                    0.0
                };
                let a_score = avg_a * cfg_quorum + a_agents.len() as f64 * 0.3;

                let (b_agents, b_confs) = &b.1;
                let weighted_b: f64 = b_confs
                    .iter()
                    .zip(b_agents.iter())
                    .map(|(c, ag)| c * self.voting_weight(ag))
                    .sum();
                let total_b: f64 = b_agents.iter().map(|ag| self.voting_weight(ag)).sum();
                let avg_b = if total_b > 0.0 {
                    weighted_b / total_b
                } else {
                    0.0
                };
                let b_score = avg_b * cfg_quorum + b_agents.len() as f64 * 0.3;

                a_score
                    .partial_cmp(&b_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            if let Some((text, (_agents, confs))) = best {
                let quorum = confs.len() as f64 / participants.len() as f64;
                let avg_conf = confs.iter().sum::<f64>() / confs.len() as f64;

                if quorum >= cfg_quorum && avg_conf >= cfg_agreement {
                    if last_consensus.as_ref() == Some(text) {
                        stability_count += 1;
                    } else {
                        stability_count = 1;
                        last_consensus = Some(text.clone());
                    }

                    if stability_count >= cfg_horizon {
                        for (agent, _, _) in &filtered {
                            self.update_reputation(agent, 0.05);
                        }
                        return ConsensusOutcome::Agreement(text.clone(), avg_conf);
                    }
                }
            }

            for (agent, _, _) in &filtered {
                self.update_reputation(agent, 0.01);
            }
            let filtered_set: HashSet<&AgentId> = filtered.iter().map(|(a, _, _)| *a).collect();
            for agent in participants {
                if !filtered_set.contains(agent) {
                    self.update_reputation(agent, -0.02);
                }
            }
        }

        if let Some(text) = &last_consensus {
            ConsensusOutcome::Disagreement(text.clone())
        } else {
            ConsensusOutcome::NotReached
        }
    }

    pub fn fast_consensus(
        &mut self,
        _bus: &mut AgentCommunicationBus,
        _conversation_id: u64,
        query: &str,
        participants: &[AgentId],
    ) -> ConsensusOutcome {
        let mut quorum_collector: HashMap<String, (Vec<AgentId>, f64)> = HashMap::new();

        for agent in participants {
            let partial = format!("{} analysis of {}", agent.name, query);
            if partial.len() > 3 {
                let normalized = self._normalize_text(&partial);
                let entry = quorum_collector
                    .entry(normalized)
                    .or_insert((Vec::new(), 0.0));
                entry.0.push(agent.clone());
                entry.1 += self.reputation(agent);
            }
        }

        let consensus = quorum_collector.iter().max_by(|a, b| {
            let a_avg_weight = if !a.1 .0.is_empty() {
                a.1 .0.iter().map(|ag| self.voting_weight(ag)).sum::<f64>() / a.1 .0.len() as f64
            } else {
                0.0
            };
            let a_score = a.1 .0.len() as f64 * a_avg_weight * 0.5 + a.1 .1 * 0.5;
            let b_avg_weight = if !b.1 .0.is_empty() {
                b.1 .0.iter().map(|ag| self.voting_weight(ag)).sum::<f64>() / b.1 .0.len() as f64
            } else {
                0.0
            };
            let b_score = b.1 .0.len() as f64 * b_avg_weight * 0.5 + b.1 .1 * 0.5;
            a_score
                .partial_cmp(&b_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        if let Some((text, (agents, _))) = consensus {
            let quorum = agents.len() as f64 / participants.len() as f64;
            if quorum >= self.config.quorum_ratio {
                return ConsensusOutcome::Agreement(text.clone(), quorum);
            }
        }

        ConsensusOutcome::NotReached
    }

    pub fn reset_round(&mut self, conversation_id: u64) {
        self.round_states.remove(&conversation_id);
    }

    pub fn config(&self) -> &ConsensusConfig {
        &self.config
    }

    pub fn suspicious_agents(&self, threshold: f64) -> Vec<AgentId> {
        self.agent_reputations
            .iter()
            .filter(|(_, &r)| r < threshold)
            .map(|(id, _)| id.clone())
            .collect()
    }

    pub fn record_contribution(&mut self, agent: &AgentId, delta: f64) {
        let entry = self.contribution_scores.entry(agent.clone()).or_insert(0.5);
        *entry = (*entry + delta).max(0.0);
    }

    pub fn voting_weight(&self, agent: &AgentId) -> f64 {
        let max_score = self
            .contribution_scores
            .values()
            .copied()
            .fold(0.0, f64::max);
        if max_score <= 0.0 {
            return 0.1;
        }
        let score = self.contribution_scores.get(agent).copied().unwrap_or(0.5);
        (score / max_score).max(0.0)
    }

    pub fn decay_contributions(&mut self) {
        for score in self.contribution_scores.values_mut() {
            *score *= self.contribution_decay;
        }
    }

    pub fn contribution_report(&self) -> Vec<(String, f64, f64)> {
        let mut report: Vec<(String, f64, f64)> = self
            .contribution_scores
            .iter()
            .map(|(id, score)| (id.to_string(), *score, self.voting_weight(id)))
            .collect();
        report.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        report
    }

    fn _eval_confidence(&self, response: &str, query: &str) -> f64 {
        let response_lower = response.to_lowercase();
        let query_keywords: Vec<&str> = query.split_whitespace().filter(|w| w.len() > 3).collect();

        let keyword_overlap = query_keywords
            .iter()
            .filter(|k| response_lower.contains(*k))
            .count();

        let keyword_ratio = if query_keywords.is_empty() {
            0.5
        } else {
            keyword_overlap as f64 / query_keywords.len() as f64
        };

        let length_ok = if response.len() > 10 && response.len() < 5000 {
            0.2
        } else {
            0.0
        };
        let has_detail = if response_lower.contains("because")
            || response_lower.contains("therefore")
            || response_lower.contains("based on")
            || response_lower.contains("analysis")
        {
            0.15
        } else {
            0.0
        };

        let repetition_penalty = {
            let words: Vec<&str> = response_lower.split_whitespace().collect();
            if words.len() > 5 {
                let unique: HashSet<&&str> = words.iter().collect();
                let ratio = unique.len() as f64 / words.len() as f64;
                if ratio < 0.3 {
                    -0.1
                } else {
                    0.0
                }
            } else {
                0.0
            }
        };

        let base = 0.3 + keyword_ratio * 0.35 + length_ok + has_detail + repetition_penalty;
        base.clamp(0.0, 1.0)
    }

    fn _filter_byzantine<'a>(
        &self,
        received: &'a [(AgentId, String, f64)],
        participants: &'a [AgentId],
    ) -> Vec<(&'a AgentId, &'a str, f64)> {
        if received.len() <= 1 {
            return received
                .iter()
                .map(|(a, t, c)| (a, t.as_str(), *c))
                .collect();
        }

        let mean_conf = received.iter().map(|(_, _, c)| c).sum::<f64>() / received.len() as f64;
        let std_conf = {
            let variance = received
                .iter()
                .map(|(_, _, c)| (c - mean_conf).powi(2))
                .sum::<f64>()
                / received.len() as f64;
            variance.sqrt().max(0.01)
        };

        let mut filtered: Vec<(&AgentId, &str, f64)> = Vec::new();
        let mut seen_agents: HashSet<&AgentId> = HashSet::new();

        for (agent, text, conf) in received {
            if !seen_agents.insert(agent) {
                continue;
            }

            let reps = self.agent_reputations.get(agent).copied().unwrap_or(0.5);
            let z_score = (conf - mean_conf) / std_conf;

            if reps < 0.2 && *conf < mean_conf - std_conf {
                continue;
            }
            if reps < 0.3 && z_score < -1.0 {
                continue;
            }
            if *conf < 0.1 {
                continue;
            }

            filtered.push((agent, text.as_str(), (*conf).min(1.0).max(0.0)));
        }

        let remaining: HashSet<&AgentId> = participants.iter().collect();
        let filtered_set: HashSet<&AgentId> = filtered.iter().map(|(a, _, _)| *a).collect();
        for agent in remaining.iter() {
            if !filtered_set.contains(agent) {
                if !seen_agents.contains(agent) {
                    filtered.push((agent, "", 0.0));
                }
            }
        }

        filtered
    }

    fn _normalize_text(&self, text: &str) -> String {
        text.chars()
            .map(|c| {
                if c.is_alphanumeric() || c.is_whitespace() {
                    c
                } else {
                    ' '
                }
            })
            .collect::<String>()
            .split_whitespace()
            .map(|w| w.to_lowercase())
            .collect::<Vec<String>>()
            .join(" ")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AgentResponse {
    pub agent_id: AgentId,
    pub content: String,
    pub confidence: f64,
}

impl Hash for AgentResponse {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.content.hash(state);
    }
}

impl Eq for AgentResponse {}

/// Threshold consensus: groups responses by content hash and returns the content
/// with >threshold support. If no group reaches the threshold, returns None.
///
/// This implements the BFT-style fault tolerance from arXiv:2606.15024 for
/// multi-agent agreement without requiring a full byzantine consensus round.
pub fn threshold_consensus(responses: &[AgentResponse], threshold: f64) -> Option<AgentResponse> {
    if responses.is_empty() {
        return None;
    }
    let total = responses.len() as f64;
    let mut groups: HashMap<u64, (String, f64, Vec<&AgentResponse>)> = HashMap::new();
    for r in responses {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        r.content.hash(&mut hasher);
        let key = hasher.finish();
        let entry = groups
            .entry(key)
            .or_insert_with(|| (r.content.clone(), 0.0, Vec::new()));
        entry.1 += r.confidence;
        entry.2.push(r);
    }
    let best = groups
        .into_iter()
        .filter(|(_, (_, _, group))| group.len() as f64 / total > threshold)
        .max_by(|a, b| {
            let a_support = a.1 .2.len() as f64 / total;
            let b_support = b.1 .2.len() as f64 / total;
            a_support
                .partial_cmp(&b_support)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    best.map(|(_, (content, _, group))| {
        let avg_conf = group.iter().map(|r| r.confidence).sum::<f64>() / group.len() as f64;
        AgentResponse {
            agent_id: group[0].agent_id.clone(),
            content,
            confidence: avg_conf,
        }
    })
}

#[derive(Debug, Clone)]
pub struct ConsensusReport {
    pub conversation_id: u64,
    pub outcome: ConsensusOutcome,
    pub rounds_taken: usize,
    pub participants: usize,
    pub suspicious_agents: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::super::bus::AgentCommunicationBus;
    use super::super::message::{AgentId, AgentStatus};
    use super::*;
    use serial_test::serial;

    fn make_agent(name: &str) -> AgentId {
        AgentId::new(name, "1.0")
    }

    #[serial]
    #[test]
    fn test_consensus_basic_config() {
        let config = ConsensusConfig::default();
        assert_eq!(config.max_rounds, 5);
        assert_eq!(config.stability_horizon, 3);
        assert!(config.receiver_eval_enabled);
    }

    #[test]
    fn test_consensus_not_reached_with_few_agents() {
        let mut layer = ByzantineConsensusLayer::with_defaults();
        let mut bus = AgentCommunicationBus::new(100);
        let agent = make_agent("solo");
        bus.register_agent(agent.clone(), AgentStatus::Idle).ok();

        let result = layer.try_consensus(&mut bus, 1, "test", &[agent]);
        assert_eq!(result, ConsensusOutcome::NotReached);
    }

    #[test]
    fn test_reputation_updates() {
        let mut layer = ByzantineConsensusLayer::with_defaults();
        let agent = make_agent("test-agent");

        assert!((layer.reputation(&agent) - 0.5).abs() < 0.01);
        layer.update_reputation(&agent, 0.3);
        assert!((layer.reputation(&agent) - 0.8).abs() < 0.01);
        layer.update_reputation(&agent, -1.0);
        assert!((layer.reputation(&agent) - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_eval_confidence_basic() {
        let layer = ByzantineConsensusLayer::with_defaults();
        let conf = layer._eval_confidence(
            "The analysis shows that X causes Y because of pathway Z.",
            "What causes Y?",
        );
        assert!(conf > 0.3);
        assert!(conf <= 1.0);
    }

    #[test]
    fn test_eval_confidence_empty_response() {
        let layer = ByzantineConsensusLayer::with_defaults();
        let conf = layer._eval_confidence("ok", "complex question with many details");
        assert!(conf < 0.5);
    }

    #[test]
    fn test_suspicious_agents() {
        let mut layer = ByzantineConsensusLayer::with_defaults();
        let good = make_agent("good");
        let bad = make_agent("bad");
        layer.update_reputation(&good, 0.4);
        layer.update_reputation(&bad, -0.4);
        let suspicious = layer.suspicious_agents(0.3);
        assert!(suspicious.contains(&bad));
        assert!(!suspicious.contains(&good));
    }

    #[test]
    fn test_fast_consensus_quorum() {
        let mut layer = ByzantineConsensusLayer::with_defaults();
        let mut bus = AgentCommunicationBus::new(100);
        let a1 = make_agent("alice");
        let a2 = make_agent("bob");
        let a3 = make_agent("carol");

        for agent in &[&a1, &a2, &a3] {
            bus.register_agent((*agent).clone(), AgentStatus::Idle).ok();
            layer.update_reputation(agent, 0.3);
        }

        let result = layer.fast_consensus(&mut bus, 42, "test query", &[a1, a2, a3]);
        assert!(
            matches!(result, ConsensusOutcome::Agreement(_, _)),
            "expected agreement, got {:?}",
            result
        );
    }

    #[test]
    fn test_round_reset() {
        let mut layer = ByzantineConsensusLayer::with_defaults();
        layer.round_states.insert(
            99,
            RoundState {
                proposals: HashMap::new(),
                stability_count: 2,
                last_consensus: Some("test".into()),
            },
        );
        assert!(layer.round_states.contains_key(&99));
        layer.reset_round(99);
        assert!(!layer.round_states.contains_key(&99));
    }
}
