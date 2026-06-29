#![allow(dead_code)]

/// Consensus round state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RoundState {
    Collecting,
    Voting,
    Finalizing,
    Complete,
}

/// A vote in the consensus engine
#[derive(Debug, Clone)]
pub struct Vote {
    pub voter_id: u64,
    pub decision: bool,
    pub confidence: f64,
    pub rationale: String,
}

/// Consensus result
#[derive(Debug, Clone)]
pub struct ConsensusResult {
    pub accepted: bool,
    pub votes_for: usize,
    pub votes_against: usize,
    pub confidence: f64,
    pub majority: f64,
}

/// Multi-agent BFT-style consensus engine
#[derive(Debug, Clone)]
pub struct ConsensusEngine {
    pub round: u64,
    pub state: RoundState,
    pub votes: Vec<Vote>,
    pub voter_ids: Vec<u64>,
    pub threshold: f64,
    pub min_voters: usize,
}

impl ConsensusEngine {
    pub fn new(voter_ids: Vec<u64>, threshold: f64) -> Self {
        let min = (voter_ids.len() as f64 * 0.67).ceil() as usize;
        ConsensusEngine {
            round: 0,
            state: RoundState::Collecting,
            votes: Vec::new(),
            voter_ids,
            threshold,
            min_voters: min,
        }
    }

    pub fn start_round(&mut self) {
        self.round += 1;
        self.state = RoundState::Collecting;
        self.votes.clear();
    }

    pub fn cast_vote(
        &mut self,
        voter_id: u64,
        decision: bool,
        confidence: f64,
        rationale: &str,
    ) -> bool {
        if self.state != RoundState::Collecting {
            return false;
        }
        if !self.voter_ids.contains(&voter_id) {
            return false;
        }
        self.votes.push(Vote {
            voter_id,
            decision,
            confidence: confidence.clamp(0.0, 1.0),
            rationale: rationale.into(),
        });
        if self.votes.len() >= self.min_voters {
            self.state = RoundState::Voting;
        }
        true
    }

    pub fn finalize(&mut self) -> ConsensusResult {
        if self.state == RoundState::Collecting && self.votes.len() < self.min_voters {
            return ConsensusResult {
                accepted: false,
                votes_for: 0,
                votes_against: 0,
                confidence: 0.0,
                majority: 0.0,
            };
        }
        self.state = RoundState::Finalizing;
        let total = self.votes.len();
        let for_count = self.votes.iter().filter(|v| v.decision).count();
        let against_count = total - for_count;
        let avg_conf = self.votes.iter().map(|v| v.confidence).sum::<f64>() / total as f64;
        let majority = for_count as f64 / total as f64;
        let accepted = majority >= self.threshold;

        let result = ConsensusResult {
            accepted,
            votes_for: for_count,
            votes_against: against_count,
            confidence: avg_conf,
            majority,
        };

        self.state = RoundState::Complete;
        result
    }

    pub fn voter_count(&self) -> usize {
        self.voter_ids.len()
    }

    pub fn participation_rate(&self) -> f64 {
        if self.voter_ids.is_empty() {
            return 0.0;
        }
        self.votes.len() as f64 / self.voter_ids.len() as f64
    }

    pub fn report(&self) -> String {
        format!(
            "ConsensusEngine: round={}, state={:?}, votes={}/{}, threshold={}",
            self.round,
            self.state,
            self.votes.len(),
            self.voter_ids.len(),
            self.threshold,
        )
    }
}

/// Vote value for governance consensus
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VoteValue {
    For,
    Against,
    Abstain,
}

/// Proposal lifecycle status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProposalStatus {
    Draft,
    Voting,
    Approved,
    Rejected,
    Executed,
}

/// A governance proposal for BFT-style multi-round consensus
#[derive(Debug, Clone)]
pub struct ConsensusProposal {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub proposed_by: u64,
    pub voting_power_required: f64,
    pub created_tick: u64,
    pub status: ProposalStatus,
}

/// BFT-style multi-round governance consensus
#[derive(Debug, Clone)]
pub struct GovernanceConsensus {
    pub proposals: Vec<ConsensusProposal>,
    pub engines: Vec<ConsensusEngine>,
    pub next_proposal_id: u64,
    pub round: u64,
    pub voter_ids: Vec<u64>,
    threshold: f64,
}

impl GovernanceConsensus {
    pub fn new(voter_ids: Vec<u64>, threshold: f64) -> Self {
        GovernanceConsensus {
            proposals: Vec::new(),
            engines: Vec::new(),
            next_proposal_id: 1,
            round: 0,
            voter_ids,
            threshold,
        }
    }

    pub fn create_proposal(
        &mut self,
        title: &str,
        description: &str,
        proposed_by: u64,
        tick: u64,
    ) -> u64 {
        let id = self.next_proposal_id;
        self.next_proposal_id += 1;
        self.proposals.push(ConsensusProposal {
            id,
            title: title.into(),
            description: description.into(),
            proposed_by,
            voting_power_required: self.threshold,
            created_tick: tick,
            status: ProposalStatus::Voting,
        });
        self.round += 1;
        let mut engine = ConsensusEngine::new(self.voter_ids.clone(), self.threshold);
        engine.start_round();
        self.engines.push(engine);
        id
    }

    pub fn vote_on_proposal(&mut self, proposal_id: u64, voter: u64, vote: VoteValue) -> bool {
        let idx = self.proposals.iter().position(|p| p.id == proposal_id);
        let Some(idx) = idx else { return false };
        if idx >= self.engines.len() {
            return false;
        }
        if self.proposals[idx].status != ProposalStatus::Voting {
            return false;
        }
        match vote {
            VoteValue::For => self.engines[idx].cast_vote(voter, true, 1.0, "for"),
            VoteValue::Against => self.engines[idx].cast_vote(voter, false, 1.0, "against"),
            VoteValue::Abstain => {
                // abstain counts as a vote but not for or against
                if !self.engines[idx].voter_ids.contains(&voter) {
                    return false;
                }
                if self.engines[idx].state != RoundState::Collecting {
                    return false;
                }
                // Add a neutral vote so participation is tracked
                self.engines[idx].votes.push(Vote {
                    voter_id: voter,
                    decision: true,
                    confidence: 0.0,
                    rationale: "abstain".into(),
                });
                if self.engines[idx].votes.len() >= self.engines[idx].min_voters {
                    self.engines[idx].state = RoundState::Voting;
                }
                true
            }
        }
    }

    pub fn tally_proposal(&mut self, proposal_id: u64) -> Option<ProposalStatus> {
        let idx = self.proposals.iter().position(|p| p.id == proposal_id)?;
        if idx >= self.engines.len() {
            return None;
        }
        if self.proposals[idx].status != ProposalStatus::Voting {
            return None;
        }
        let result = self.engines[idx].finalize();
        let status = if result.accepted {
            ProposalStatus::Approved
        } else {
            ProposalStatus::Rejected
        };
        self.proposals[idx].status = status;
        Some(status)
    }

    pub fn execute_proposal(&mut self, proposal_id: u64) -> bool {
        let Some(p) = self.proposals.iter_mut().find(|p| p.id == proposal_id) else {
            return false;
        };
        if p.status != ProposalStatus::Approved {
            return false;
        }
        p.status = ProposalStatus::Executed;
        true
    }

    pub fn pending_proposals(&self) -> Vec<&ConsensusProposal> {
        self.proposals
            .iter()
            .filter(|p| p.status == ProposalStatus::Voting)
            .collect()
    }

    pub fn approved_proposals(&self) -> Vec<&ConsensusProposal> {
        self.proposals
            .iter()
            .filter(|p| p.status == ProposalStatus::Approved)
            .collect()
    }

    pub fn proposal_by_id(&self, id: u64) -> Option<&ConsensusProposal> {
        self.proposals.iter().find(|p| p.id == id)
    }

    pub fn voter_turnout(&self, proposal_id: u64) -> f64 {
        let Some(idx) = self.proposals.iter().position(|p| p.id == proposal_id) else {
            return 0.0;
        };
        if idx >= self.engines.len() {
            return 0.0;
        }
        if self.voter_ids.is_empty() {
            return 0.0;
        }
        self.engines[idx].votes.len() as f64 / self.voter_ids.len() as f64
    }

    pub fn consensus_health(&self) -> f64 {
        if self.proposals.is_empty() {
            return 1.0;
        }
        let approved = self
            .proposals
            .iter()
            .filter(|p| {
                p.status == ProposalStatus::Approved || p.status == ProposalStatus::Executed
            })
            .count();
        approved as f64 / self.proposals.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_engine() {
        let e = ConsensusEngine::new(vec![1, 2, 3], 0.67);
        assert_eq!(e.voter_count(), 3);
        assert_eq!(e.state, RoundState::Collecting);
    }

    #[test]
    fn test_cast_and_finalize() {
        let mut e = ConsensusEngine::new(vec![1, 2, 3], 0.5);
        e.start_round();
        e.cast_vote(1, true, 0.8, "agree");
        e.cast_vote(2, true, 0.7, "agree");
        let result = e.finalize();
        assert!(result.accepted);
        assert_eq!(result.votes_for, 2);
    }

    #[test]
    fn test_rejected_when_below_threshold() {
        let mut e = ConsensusEngine::new(vec![1, 2, 3], 0.8);
        e.start_round();
        e.cast_vote(1, true, 0.6, "ok");
        e.cast_vote(2, false, 0.5, "no");
        let result = e.finalize();
        assert!(!result.accepted);
    }

    #[test]
    fn test_unknown_voter_rejected() {
        let mut e = ConsensusEngine::new(vec![1, 2], 0.5);
        e.start_round();
        assert!(!e.cast_vote(99, true, 0.5, "hack"));
    }

    #[test]
    fn test_participation_rate() {
        let mut e = ConsensusEngine::new(vec![1, 2, 3, 4], 0.5);
        e.start_round();
        e.cast_vote(1, true, 0.5, "a");
        e.cast_vote(2, true, 0.5, "b");
        assert!((e.participation_rate() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_report() {
        let mut e = ConsensusEngine::new(vec![1, 2], 0.5);
        e.start_round();
        let r = e.report();
        assert!(r.contains("ConsensusEngine"));
    }

    #[test]
    fn test_start_round_increments() {
        let mut e = ConsensusEngine::new(vec![1], 0.5);
        e.start_round();
        assert_eq!(e.round, 1);
        e.start_round();
        assert_eq!(e.round, 2);
    }

    #[test]
    fn test_create_proposal() {
        let mut gc = GovernanceConsensus::new(vec![1, 2, 3], 0.67);
        let id = gc.create_proposal("Test", "A test proposal", 1, 100);
        assert_eq!(id, 1);
        let p = gc.proposal_by_id(id).unwrap();
        assert_eq!(p.title, "Test");
        assert_eq!(p.status, ProposalStatus::Voting);
        assert_eq!(gc.round, 1);
    }

    #[test]
    fn test_vote_and_tally_approves() {
        let mut gc = GovernanceConsensus::new(vec![1, 2, 3], 0.5);
        let id = gc.create_proposal("Approve", "Will pass", 1, 100);
        assert!(gc.vote_on_proposal(id, 1, VoteValue::For));
        assert!(gc.vote_on_proposal(id, 2, VoteValue::For));
        assert_eq!(gc.tally_proposal(id), Some(ProposalStatus::Approved));
    }

    #[test]
    fn test_vote_and_tally_rejects() {
        let mut gc = GovernanceConsensus::new(vec![1, 2, 3], 0.8);
        let id = gc.create_proposal("Reject", "Will fail", 1, 100);
        assert!(gc.vote_on_proposal(id, 1, VoteValue::For));
        assert!(gc.vote_on_proposal(id, 2, VoteValue::Against));
        assert_eq!(gc.tally_proposal(id), Some(ProposalStatus::Rejected));
    }

    #[test]
    fn test_execute_proposal() {
        let mut gc = GovernanceConsensus::new(vec![1, 2, 3], 0.5);
        let id = gc.create_proposal("Exec", "Will execute", 1, 100);
        gc.vote_on_proposal(id, 1, VoteValue::For);
        gc.vote_on_proposal(id, 2, VoteValue::For);
        gc.tally_proposal(id);
        assert!(gc.execute_proposal(id));
        assert_eq!(
            gc.proposal_by_id(id).unwrap().status,
            ProposalStatus::Executed
        );
        assert!(!gc.execute_proposal(id)); // already executed
    }

    #[test]
    fn test_pending_proposals() {
        let mut gc = GovernanceConsensus::new(vec![1, 2], 0.5);
        gc.create_proposal("P1", "First", 1, 100);
        gc.create_proposal("P2", "Second", 1, 100);
        assert_eq!(gc.pending_proposals().len(), 2);
    }

    #[test]
    fn test_approved_proposals() {
        let mut gc = GovernanceConsensus::new(vec![1, 2], 0.5);
        let id = gc.create_proposal("P1", "First", 1, 100);
        gc.vote_on_proposal(id, 1, VoteValue::For);
        gc.vote_on_proposal(id, 2, VoteValue::For);
        gc.tally_proposal(id);
        let approved = gc.approved_proposals();
        assert_eq!(approved.len(), 1);
        assert_eq!(approved[0].id, id);
    }

    #[test]
    fn test_voter_turnout() {
        let mut gc = GovernanceConsensus::new(vec![1, 2, 3, 4], 0.5);
        let id = gc.create_proposal("Turnout", "Check turnout", 1, 100);
        gc.vote_on_proposal(id, 1, VoteValue::For);
        gc.vote_on_proposal(id, 2, VoteValue::Against);
        let turnout = gc.voter_turnout(id);
        assert!((turnout - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_consensus_health() {
        let mut gc = GovernanceConsensus::new(vec![1, 2], 0.5);
        assert!((gc.consensus_health() - 1.0).abs() < 0.01);
        let id1 = gc.create_proposal("P1", "First", 1, 100);
        gc.vote_on_proposal(id1, 1, VoteValue::For);
        gc.vote_on_proposal(id1, 2, VoteValue::For);
        gc.tally_proposal(id1);
        let id2 = gc.create_proposal("P2", "Second", 1, 100);
        gc.vote_on_proposal(id2, 1, VoteValue::Against);
        gc.vote_on_proposal(id2, 2, VoteValue::Against);
        gc.tally_proposal(id2);
        let h = gc.consensus_health();
        assert!((h - 0.5).abs() < 0.01);
    }
}
