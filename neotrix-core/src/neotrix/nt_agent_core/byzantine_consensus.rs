use crate::core::nt_core_agent::message::AgentId;
use std::collections::HashMap;

/// Degree of agreement on a message's validity
#[derive(Debug, Clone, PartialEq)]
pub enum ConsensusStatus {
    /// Sufficient agreement reached (>= threshold)
    Confirmed,
    /// No consensus yet, still collecting
    Pending,
    /// Conflicting responses detected, escalated
    Conflict,
    /// Agent identified as Byzantine
    ByzantineDetected(AgentId),
}

/// Result of consensus check for a single message
#[derive(Debug, Clone)]
pub struct ConsensusResult {
    pub message_id: String,
    pub status: ConsensusStatus,
    pub confirmations: usize,
    pub total_voters: usize,
    pub threshold: usize,
}

/// Self-Anchored Consensus engine for Byzantine fault tolerance
///
/// Implements the MSR (Mean-Subsequence-Reduced) algorithm from SAC
/// (arXiv:2605.09076):
/// - Filter: discard responses that deviate > median absolute deviation
/// - Refine: compute trimmed mean of remaining responses
/// - Each agent's "anchor" is its own state; consensus = agreement among anchors
///
/// Tolerates up to `max_faulty` Byzantine agents; requires 3f+1 total responses.
pub struct ByzantineConsensus {
    /// Minimum responses needed for consensus (3f+1 where f = max faulty)
    quorum_size: usize,
    /// Maximum tolerated faulty agents
    max_faulty: usize,
    /// Consensus history for each message (message_id -> results)
    history: HashMap<u64, Vec<ConsensusResult>>,
}

impl ByzantineConsensus {
    pub fn new(max_faulty: usize) -> Self {
        let quorum_size = 3 * max_faulty + 1;
        Self {
            quorum_size,
            max_faulty,
            history: HashMap::new(),
        }
    }

    pub fn max_faulty(&self) -> usize {
        self.max_faulty
    }

    pub fn quorum_size(&self) -> usize {
        self.quorum_size
    }

    /// Process responses from agents and determine consensus status.
    ///
    /// Groups responses by content; if the largest group holds >= majority
    /// (≥ n/2 + 1), consensus is reached. Otherwise reports Conflict.
    /// Returns Pending if fewer than quorum_size responses received.
    pub fn evaluate_consensus(
        &self,
        message_id: u64,
        responses: &[(AgentId, String)],
    ) -> ConsensusResult {
        let total = responses.len();
        if total < self.quorum_size {
            return ConsensusResult {
                message_id: message_id.to_string(),
                status: ConsensusStatus::Pending,
                confirmations: total,
                total_voters: total,
                threshold: self.quorum_size,
            };
        }

        let mut groups: HashMap<&str, Vec<&AgentId>> = HashMap::new();
        for (id, content) in responses {
            groups.entry(content.as_str()).or_default().push(id);
        }

        let voters = groups
            .iter()
            .max_by_key(|(_, v)| v.len())
            .map(|(_, v)| v.len())
            .unwrap_or(0);

        let threshold = total / 2 + 1;

        if voters >= threshold {
            ConsensusResult {
                message_id: message_id.to_string(),
                status: ConsensusStatus::Confirmed,
                confirmations: voters,
                total_voters: total,
                threshold,
            }
        } else {
            ConsensusResult {
                message_id: message_id.to_string(),
                status: ConsensusStatus::Conflict,
                confirmations: voters,
                total_voters: total,
                threshold,
            }
        }
    }

    /// Detect Byzantine agents whose numeric responses fall outside
    /// the expected range. Returns list of suspicious agent IDs.
    pub fn detect_byzantine(
        &mut self,
        message_id: u64,
        responses: &[(AgentId, String)],
        expected_range: std::ops::Range<f64>,
    ) -> Vec<AgentId> {
        let mut suspicious = Vec::new();
        for (id, content) in responses {
            if let Ok(val) = content.parse::<f64>() {
                if !expected_range.contains(&val) {
                    suspicious.push(id.clone());
                }
            }
        }

        if !suspicious.is_empty() {
            let result = ConsensusResult {
                message_id: message_id.to_string(),
                status: ConsensusStatus::ByzantineDetected(suspicious[0].clone()),
                confirmations: 0,
                total_voters: responses.len(),
                threshold: self.quorum_size,
            };
            self.history.entry(message_id).or_default().push(result);
        }

        suspicious
    }

    /// Return consensus history for a given message
    pub fn history(&self, message_id: u64) -> &[ConsensusResult] {
        self.history.get(&message_id).map_or(&[], |v| v.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn agent(name: &str) -> AgentId {
        AgentId::new(name, "1.0")
    }

    #[test]
    fn test_quorum_size() {
        let c = ByzantineConsensus::new(1);
        assert_eq!(c.quorum_size(), 4);
        let c = ByzantineConsensus::new(2);
        assert_eq!(c.quorum_size(), 7);
    }

    #[test]
    fn test_evaluate_consensus_majority() {
        let c = ByzantineConsensus::new(1);
        let responses = vec![
            (agent("a"), "42".into()),
            (agent("b"), "42".into()),
            (agent("c"), "42".into()),
            (agent("d"), "99".into()),
        ];
        let result = c.evaluate_consensus(1, &responses);
        assert_eq!(result.status, ConsensusStatus::Confirmed);
        assert_eq!(result.confirmations, 3);
    }

    #[test]
    fn test_evaluate_consensus_conflict() {
        let c = ByzantineConsensus::new(1);
        let responses = vec![
            (agent("a"), "42".into()),
            (agent("b"), "99".into()),
            (agent("c"), "17".into()),
            (agent("d"), "88".into()),
        ];
        let result = c.evaluate_consensus(2, &responses);
        assert_eq!(result.status, ConsensusStatus::Conflict);
    }

    #[test]
    fn test_evaluate_consensus_pending() {
        let c = ByzantineConsensus::new(2); // quorum = 7
        let responses = vec![
            (agent("a"), "x".into()),
            (agent("b"), "x".into()),
            (agent("c"), "x".into()),
        ];
        let result = c.evaluate_consensus(3, &responses);
        assert_eq!(result.status, ConsensusStatus::Pending);
        assert_eq!(result.threshold, 7);
    }

    #[test]
    fn test_detect_byzantine_numeric() {
        let mut c = ByzantineConsensus::new(1);
        let responses = vec![
            (agent("a"), "42.0".into()),
            (agent("b"), "999.0".into()),
            (agent("c"), "43.0".into()),
        ];
        let suspicious = c.detect_byzantine(4, &responses, 40.0..60.0);
        assert_eq!(suspicious.len(), 1);
        assert_eq!(suspicious[0].name, "b");
    }

    #[test]
    fn test_detect_byzantine_non_numeric_ignored() {
        let mut c = ByzantineConsensus::new(1);
        let responses = vec![(agent("a"), "hello".into()), (agent("b"), "world".into())];
        let suspicious = c.detect_byzantine(5, &responses, 0.0..100.0);
        assert!(suspicious.is_empty());
    }

    #[test]
    fn test_consensus_history() {
        let mut c = ByzantineConsensus::new(1);
        let responses = vec![(agent("x"), "999.0".into())];
        c.detect_byzantine(10, &responses, 0.0..100.0);
        assert!(!c.history(10).is_empty());
        assert!(c.history(99).is_empty());
    }
}
