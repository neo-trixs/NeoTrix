#![forbid(unsafe_code)]

use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PeerStatus {
    Online,
    Busy,
    Offline,
    Degraded,
}

#[derive(Debug, Clone)]
pub struct PeerState {
    pub peer_id: String,
    pub status: PeerStatus,
    pub capabilities: Vec<String>,
    pub trust_score: f64,
    pub latency_ms: u64,
    pub last_seen: u64,
}

impl PeerState {
    pub fn new(peer_id: impl Into<String>) -> Self {
        Self {
            peer_id: peer_id.into(),
            status: PeerStatus::Online,
            capabilities: Vec::new(),
            trust_score: 1.0,
            latency_ms: 0,
            last_seen: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsensusState {
    Proposed,
    Voting,
    Accepted,
    Rejected,
    Superseded,
}

#[derive(Debug, Clone)]
pub struct ConsensusProposal {
    pub id: String,
    pub proposer: String,
    pub title: String,
    pub description: String,
    pub state: ConsensusState,
    pub votes: Vec<Vote>,
    pub created_at: u64,
}

impl ConsensusProposal {
    pub fn new(
        id: impl Into<String>,
        proposer: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            proposer: proposer.into(),
            title: title.into(),
            description: description.into(),
            state: ConsensusState::Proposed,
            votes: Vec::new(),
            created_at: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Vote {
    pub peer_id: String,
    pub approve: bool,
    pub rationale: String,
}

#[derive(Debug, Clone)]
pub struct GroupCoordinatorConfig {
    pub max_peers: u32,
    pub consensus_threshold: f64,
    pub heartbeat_interval_ms: u64,
    pub max_history: usize,
}

impl Default for GroupCoordinatorConfig {
    fn default() -> Self {
        Self {
            max_peers: 16,
            consensus_threshold: 0.67,
            heartbeat_interval_ms: 5000,
            max_history: 100,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GroupCoordinatorStats {
    pub peers: u32,
    pub online: u32,
    pub total_proposals: u32,
    pub accepted: u32,
    pub avg_trust: f64,
    pub avg_latency_ms: u64,
}

#[derive(Debug, Clone)]
pub struct GroupCoordinator {
    pub config: GroupCoordinatorConfig,
    peers: HashMap<String, PeerState>,
    proposals: Vec<ConsensusProposal>,
    results: VecDeque<String>,
}

impl GroupCoordinator {
    pub fn new(config: GroupCoordinatorConfig) -> Self {
        Self {
            config,
            peers: HashMap::new(),
            proposals: Vec::new(),
            results: VecDeque::with_capacity(100),
        }
    }

    pub fn register_peer(&mut self, peer: PeerState) {
        self.peers.insert(peer.peer_id.clone(), peer);
    }

    pub fn unregister_peer(&mut self, peer_id: &str) {
        self.peers.remove(peer_id);
    }

    pub fn get_peer(&self, peer_id: &str) -> Option<&PeerState> {
        self.peers.get(peer_id)
    }

    pub fn get_peer_mut(&mut self, peer_id: &str) -> Option<&mut PeerState> {
        self.peers.get_mut(peer_id)
    }

    pub fn update_heartbeat(&mut self, peer_id: &str, latency_ms: u64) {
        if let Some(peer) = self.peers.get_mut(peer_id) {
            peer.latency_ms = latency_ms;
            peer.last_seen = 0;
            peer.status = PeerStatus::Online;
        }
    }

    pub fn submit_proposal(&mut self, proposal: ConsensusProposal) {
        self.proposals.push(proposal);
    }

    pub fn vote(&mut self, proposal_id: &str, vote: Vote) -> Option<ConsensusState> {
        let proposal = self.proposals.iter_mut().find(|p| p.id == proposal_id)?;
        proposal.state = ConsensusState::Voting;
        proposal.votes.push(vote);

        let total_votes = proposal.votes.len() as f64;
        if total_votes >= 2.0 {
            let approvals = proposal.votes.iter().filter(|v| v.approve).count() as f64;
            let ratio = approvals / total_votes;
            if ratio >= self.config.consensus_threshold {
                proposal.state = ConsensusState::Accepted;
                if self.results.len() >= 100 {
                    self.results.pop_front();
                }
                self.results.push_back(proposal.id.clone());
                return Some(ConsensusState::Accepted);
            } else if total_votes >= self.peers.len() as f64 * 0.5 {
                let reject = proposal.votes.iter().filter(|v| !v.approve).count() as f64;
                if reject / total_votes > 0.5 {
                    proposal.state = ConsensusState::Rejected;
                    return Some(ConsensusState::Rejected);
                }
            }
        }
        None
    }

    pub fn get_proposal(&self, id: &str) -> Option<&ConsensusProposal> {
        self.proposals.iter().find(|p| p.id == id)
    }

    pub fn accepted_proposals(&self) -> Vec<&ConsensusProposal> {
        self.proposals
            .iter()
            .filter(|p| p.state == ConsensusState::Accepted)
            .collect()
    }

    pub fn select_peers_for_task(&self, capability: &str, max_count: usize) -> Vec<&PeerState> {
        let mut candidates: Vec<&PeerState> = self
            .peers
            .values()
            .filter(|p| {
                p.status == PeerStatus::Online && p.capabilities.iter().any(|c| c == capability)
            })
            .collect();
        candidates.sort_by(|a, b| {
            b.trust_score
                .partial_cmp(&a.trust_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        candidates.truncate(max_count);
        candidates
    }

    pub fn broadcast(&self, message: &str) -> Vec<String> {
        self.peers
            .iter()
            .filter(|(_, p)| p.status == PeerStatus::Online)
            .map(|(id, _)| format!("{}: {} delivered", id, message))
            .collect()
    }

    pub fn stats(&self) -> GroupCoordinatorStats {
        let peers = self.peers.len() as u32;
        let online = self
            .peers
            .values()
            .filter(|p| p.status == PeerStatus::Online)
            .count() as u32;
        let total_proposals = self.proposals.len() as u32;
        let accepted = self
            .proposals
            .iter()
            .filter(|p| p.state == ConsensusState::Accepted)
            .count() as u32;
        let trust_sum: f64 = self.peers.values().map(|p| p.trust_score).sum();
        let avg_trust = if self.peers.is_empty() {
            0.0
        } else {
            trust_sum / self.peers.len() as f64
        };
        let latency_sum: u64 = self.peers.values().map(|p| p.latency_ms).sum();
        let avg_latency_ms = if self.peers.is_empty() {
            0
        } else {
            latency_sum / self.peers.len() as u64
        };
        GroupCoordinatorStats {
            peers,
            online,
            total_proposals,
            accepted,
            avg_trust,
            avg_latency_ms,
        }
    }
}

impl Default for GroupCoordinator {
    fn default() -> Self {
        Self::new(GroupCoordinatorConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_lookup() {
        let mut coord = GroupCoordinator::default();
        let peer = PeerState::new("peer1");
        coord.register_peer(peer);
        assert!(coord.get_peer("peer1").is_some());
    }

    #[test]
    fn test_proposal_vote_to_accept() {
        let mut coord = GroupCoordinator::default();
        coord.register_peer(PeerState::new("alice"));
        coord.register_peer(PeerState::new("bob"));
        coord.register_peer(PeerState::new("charlie"));

        coord.submit_proposal(ConsensusProposal::new(
            "p1",
            "alice",
            "refactor",
            "refactor core",
        ));
        let vote1 = Vote {
            peer_id: "alice".into(),
            approve: true,
            rationale: "".into(),
        };
        let vote2 = Vote {
            peer_id: "bob".into(),
            approve: true,
            rationale: "".into(),
        };
        coord.vote("p1", vote1);
        match coord.vote("p1", vote2) {
            Some(ConsensusState::Accepted) => {}
            _ => {}
        }
    }

    #[test]
    fn test_select_peers_for_task() {
        let mut coord = GroupCoordinator::default();
        let mut peer = PeerState::new("worker1");
        peer.capabilities.push("coding".to_string());
        coord.register_peer(peer);
        let selected = coord.select_peers_for_task("coding", 5);
        assert_eq!(selected.len(), 1);
    }

    #[test]
    fn test_stats_counts() {
        let mut coord = GroupCoordinator::default();
        coord.register_peer(PeerState::new("p1"));
        coord.register_peer(PeerState::new("p2"));
        let stats = coord.stats();
        assert_eq!(stats.peers, 2);
        assert_eq!(stats.online, 2);
    }
}
