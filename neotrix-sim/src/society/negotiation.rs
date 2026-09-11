use serde::{Deserialize, Serialize};

use crate::agents::sim_agent::SimAgent;
use super::economy::ResourceType;

pub type AgentId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceTransfer {
    pub resource: ResourceType,
    pub amount: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NegotiationStatus {
    Active,
    Accepted,
    Rejected,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub from: AgentId,
    pub offers: Vec<ResourceTransfer>,
    pub demands: Vec<ResourceTransfer>,
    pub concessions: f32,
    pub round: u32,
}

pub struct Negotiation {
    pub parties: Vec<AgentId>,
    pub proposals: Vec<Proposal>,
    pub round: u32,
    pub max_rounds: u32,
    pub status: NegotiationStatus,
}

impl Negotiation {
    pub fn new(parties: Vec<AgentId>) -> Self {
        Self {
            parties,
            proposals: Vec::new(),
            round: 0,
            max_rounds: 5,
            status: NegotiationStatus::Active,
        }
    }

    pub fn propose(&mut self, proposal: Proposal) -> NegotiationResult {
        if self.status != NegotiationStatus::Active {
            return NegotiationResult::Rejected;
        }
        if self.round >= self.max_rounds {
            self.status = NegotiationStatus::Expired;
            return NegotiationResult::Expired;
        }

        let fairness = self.calculate_fairness(&proposal);
        self.proposals.push(proposal);
        self.round += 1;

        if fairness > 0.7 {
            NegotiationResult::Accepted { fairness }
        } else if self.round >= self.max_rounds {
            self.status = NegotiationStatus::Expired;
            NegotiationResult::Expired
        } else {
            NegotiationResult::CounterProposed { fairness }
        }
    }

    pub fn evaluate(&self, agent: &SimAgent) -> f32 {
        if let Some(last) = self.proposals.last() {
            let offer_value: f32 = last.offers.iter().map(|o| o.amount).sum();
            let demand_value: f32 = last.demands.iter().map(|d| d.amount).sum();

            let mut utility = if demand_value > 0.0 {
                offer_value / demand_value
            } else {
                1.0
            };

            utility *= agent.personality.cooperativeness;
            utility.clamp(0.0, 1.0)
        } else {
            0.5
        }
    }

    pub fn resolve(&mut self) -> NegotiationStatus {
        if self.proposals.is_empty() {
            self.status = NegotiationStatus::Rejected;
            return self.status.clone();
        }

        let last = self.proposals.last().unwrap();
        let fairness = self.calculate_fairness(last);

        if fairness > 0.5 {
            self.status = NegotiationStatus::Accepted;
        } else {
            self.status = NegotiationStatus::Rejected;
        }

        self.status.clone()
    }

    fn calculate_fairness(&self, proposal: &Proposal) -> f32 {
        let offer_value: f32 = proposal.offers.iter().map(|o| o.amount).sum();
        let demand_value: f32 = proposal.demands.iter().map(|d| d.amount).sum();

        if demand_value == 0.0 {
            return 1.0;
        }

        (offer_value / demand_value).min(1.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NegotiationResult {
    Accepted { fairness: f32 },
    Rejected,
    CounterProposed { fairness: f32 },
    Expired,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_agent(id: u64, coop: f32) -> SimAgent {
        let mut agent = SimAgent::new(id, crate::foundation::math_bridge::Vec2::new(0.0, 0.0));
        agent.personality.cooperativeness = coop;
        agent
    }

    #[test]
    fn test_new_negotiation() {
        let neg = Negotiation::new(vec!["a".into(), "b".into()]);
        assert_eq!(neg.status, NegotiationStatus::Active);
        assert_eq!(neg.round, 0);
    }

    #[test]
    fn test_fair_proposal() {
        let mut neg = Negotiation::new(vec!["a".into(), "b".into()]);
        let result = neg.propose(Proposal {
            from: "a".into(),
            offers: vec![ResourceTransfer { resource: ResourceType::Food, amount: 10.0 }],
            demands: vec![ResourceTransfer { resource: ResourceType::Wood, amount: 10.0 }],
            concessions: 0.0,
            round: 0,
        });
        assert!(matches!(result, NegotiationResult::Accepted { .. }));
    }

    #[test]
    fn test_unfair_proposal() {
        let mut neg = Negotiation::new(vec!["a".into(), "b".into()]);
        let result = neg.propose(Proposal {
            from: "a".into(),
            offers: vec![ResourceTransfer { resource: ResourceType::Food, amount: 2.0 }],
            demands: vec![ResourceTransfer { resource: ResourceType::Wood, amount: 10.0 }],
            concessions: 0.0,
            round: 0,
        });
        assert!(matches!(result, NegotiationResult::CounterProposed { .. }));
    }

    #[test]
    fn test_evaluate_utility() {
        let mut neg = Negotiation::new(vec!["a".into(), "b".into()]);
        neg.proposals.push(Proposal {
            from: "a".into(),
            offers: vec![ResourceTransfer { resource: ResourceType::Food, amount: 10.0 }],
            demands: vec![ResourceTransfer { resource: ResourceType::Wood, amount: 5.0 }],
            concessions: 0.0,
            round: 0,
        });
        let agent = make_agent(0, 0.8);
        let utility = neg.evaluate(&agent);
        assert!(utility > 0.0);
    }

    #[test]
    fn test_resolve_empty() {
        let mut neg = Negotiation::new(vec!["a".into()]);
        let status = neg.resolve();
        assert_eq!(status, NegotiationStatus::Rejected);
    }
}
