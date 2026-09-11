use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegotiationProposal {
    pub proposer: u32,
    pub target: u32,
    pub offer: Vec<(String, f32)>,
    pub demand: Vec<(String, f32)>,
    pub accepted: bool,
    pub round: u32,
}

pub struct NegotiationEngine {
    pub max_rounds: u32,
    pub concession_rate: f32,
}

impl NegotiationEngine {
    pub fn new() -> Self {
        Self {
            max_rounds: 5,
            concession_rate: 0.1,
        }
    }

    pub fn start_negotiation(
        &self,
        proposer: u32,
        target: u32,
        offer: Vec<(String, f32)>,
        demand: Vec<(String, f32)>,
    ) -> NegotiationProposal {
        NegotiationProposal {
            proposer,
            target,
            offer,
            demand,
            accepted: false,
            round: 0,
        }
    }

    pub fn evaluate_proposal(
        &self,
        proposal: &NegotiationProposal,
        responder_inventory: &[(String, f32)],
    ) -> bool {
        let has_resources = proposal.demand.iter().all(|(item, amount)| {
            responder_inventory
                .iter()
                .any(|(i, a)| i == item && a >= amount)
        });

        if !has_resources {
            return false;
        }

        let fairness = self.calculate_fairness(proposal);
        fairness > 0.3
    }

    fn calculate_fairness(&self, proposal: &NegotiationProposal) -> f32 {
        let offer_value: f32 = proposal.offer.iter().map(|(_, v)| v).sum();
        let demand_value: f32 = proposal.demand.iter().map(|(_, v)| v).sum();

        if demand_value == 0.0 {
            return 1.0;
        }

        (offer_value / demand_value).min(1.0)
    }

    pub fn counter_proposal(&self, proposal: &NegotiationProposal) -> NegotiationProposal {
        let mut new_offer = proposal.demand.clone();
        let mut new_demand = proposal.offer.clone();

        for (_, amount) in &mut new_offer {
            *amount *= 1.0 - self.concession_rate;
        }
        for (_, amount) in &mut new_demand {
            *amount *= 1.0 - self.concession_rate;
        }

        NegotiationProposal {
            proposer: proposal.target,
            target: proposal.proposer,
            offer: new_offer,
            demand: new_demand,
            accepted: false,
            round: proposal.round + 1,
        }
    }

    pub fn is_finished(&self, proposal: &NegotiationProposal) -> bool {
        proposal.accepted || proposal.round >= self.max_rounds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_negotiation_creation() {
        let engine = NegotiationEngine::new();
        let proposal = engine.start_negotiation(
            0,
            1,
            vec![("food".to_string(), 10.0)],
            vec![("wood".to_string(), 5.0)],
        );
        assert_eq!(proposal.proposer, 0);
        assert_eq!(proposal.round, 0);
    }

    #[test]
    fn test_counter_proposal() {
        let engine = NegotiationEngine::new();
        let proposal = engine.start_negotiation(
            0,
            1,
            vec![("food".to_string(), 10.0)],
            vec![("wood".to_string(), 5.0)],
        );
        let counter = engine.counter_proposal(&proposal);
        assert_eq!(counter.proposer, 1);
        assert_eq!(counter.round, 1);
    }

    #[test]
    fn test_fairness() {
        let engine = NegotiationEngine::new();
        let proposal = engine.start_negotiation(
            0,
            1,
            vec![("food".to_string(), 10.0)],
            vec![("wood".to_string(), 10.0)],
        );
        let fairness = engine.calculate_fairness(&proposal);
        assert_eq!(fairness, 1.0);
    }
}
