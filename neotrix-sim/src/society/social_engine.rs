use crate::agents::sim_agent::SimAgent;
use super::faction::FactionManager;
use super::gossip::GossipProtocol;
use super::communication::CommunicationChannel;
use super::norm_evolution::NormEvolution;
use super::conflict::{ConflictManager, Conflict};
use super::hierarchy::Hierarchy;
use super::negotiation::Negotiation;
use super::relationship_graph::RelationshipGraph;

#[derive(Debug, Clone)]
pub struct SocialDynamicsReport {
    pub active_conflicts: usize,
    pub norms_compliance: f32,
    pub gossip_messages: usize,
    pub pending_messages: usize,
    pub active_negotiations: usize,
    pub leaders_count: usize,
    pub faction_count: usize,
}

pub struct SocialEngine {
    pub faction_mgr: FactionManager,
    pub gossip: GossipProtocol,
    pub communication: CommunicationChannel,
    pub norms: NormEvolution,
    pub conflict_mgr: ConflictManager,
    pub hierarchy: Hierarchy,
    pub negotiations: Vec<Negotiation>,
    pub tick: u64,
}

impl SocialEngine {
    pub fn new() -> Self {
        Self {
            faction_mgr: FactionManager::new(),
            gossip: GossipProtocol::new(),
            communication: CommunicationChannel::new(),
            norms: NormEvolution::new(),
            conflict_mgr: ConflictManager::new(),
            hierarchy: Hierarchy::new(),
            negotiations: Vec::new(),
            tick: 0,
        }
    }

    pub fn tick(&mut self, agents: &[SimAgent], relationships: &mut RelationshipGraph) {
        self.tick = self.tick.wrapping_add(1);

        self.faction_mgr.update(agents);

        self.gossip.propagate(agents, relationships);

        self.communication.tick();

        self.norms.update(agents);
        for agent in &mut self.agents_mut_placeholder(agents) {
            self.norms.enforce(agent);
        }

        let detected = self.conflict_mgr.detect_conflicts(agents, relationships, self.tick);
        for conflict in detected {
            self.conflict_mgr.register_conflict(conflict);
        }

        // Determine actions for each conflict without borrowing conflict_mgr
        let actions: Vec<(usize, bool, bool)> = self.conflict_mgr.active_conflicts.iter().enumerate()
            .filter(|(_, c)| c.resolution.is_none())
            .map(|(i, c)| (i, c.intensity > 0.6, c.intensity <= 0.6))
            .collect();
        for (idx, needs_resolve, needs_escalate) in actions {
            if needs_resolve || needs_escalate {
                // Extract conflict, process, put back
                let mut conflict = std::mem::take(&mut self.conflict_mgr.active_conflicts[idx]);
                if needs_resolve {
                    self.conflict_mgr.resolve(&mut conflict, agents);
                } else {
                    self.conflict_mgr.escalate(&mut conflict);
                }
                self.conflict_mgr.active_conflicts[idx] = conflict;
            }
        }
        self.conflict_mgr.cleanup_resolved();

        self.hierarchy.update(agents, self.tick);

        self.norms.evolve();

        for negotiation in &mut self.negotiations {
            if negotiation.status == super::negotiation::NegotiationStatus::Active {
                negotiation.resolve();
            }
        }
        self.negotiations.retain(|n| {
            n.status == super::negotiation::NegotiationStatus::Active
        });

        self.gossip.tick();
        self.communication.tick();
    }

    fn agents_mut_placeholder(&self, agents: &[SimAgent]) -> Vec<SimAgent> {
        agents.to_vec()
    }

    pub fn start_negotiation(&mut self, parties: Vec<String>) {
        let neg = Negotiation::new(parties);
        self.negotiations.push(neg);
    }

    pub fn get_report(&self) -> SocialDynamicsReport {
        SocialDynamicsReport {
            active_conflicts: self.conflict_mgr.active_count(),
            norms_compliance: self.norms.overall_compliance(),
            gossip_messages: self.gossip.messages.len(),
            pending_messages: self.communication.total_messages(),
            active_negotiations: self.negotiations.len(),
            leaders_count: self.hierarchy.leaders.len(),
            faction_count: self.faction_mgr.factions.len(),
        }
    }
}

impl Default for SocialEngine {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::foundation::math_bridge::Vec2;

    fn make_agents(count: u64) -> Vec<SimAgent> {
        (0..count)
            .map(|i| SimAgent::new(i, Vec2::new(i as f32 * 20.0, 0.0)))
            .collect()
    }

    #[test]
    fn test_social_engine_creation() {
        let engine = SocialEngine::new();
        assert_eq!(engine.tick, 0);
        assert_eq!(engine.faction_mgr.factions.len(), 0);
    }

    #[test]
    fn test_social_engine_tick() {
        let mut engine = SocialEngine::new();
        let agents = make_agents(5);
        let mut rg = RelationshipGraph::new();
        engine.tick(&agents, &mut rg);

        assert_eq!(engine.tick, 1);
        assert!(engine.norms.norms.len() > 0);
    }

    #[test]
    fn test_social_report() {
        let engine = SocialEngine::new();
        let report = engine.get_report();
        assert_eq!(report.active_conflicts, 0);
        assert!(report.norms_compliance > 0.0);
    }

    #[test]
    fn test_start_negotiation() {
        let mut engine = SocialEngine::new();
        engine.start_negotiation(vec!["a".into(), "b".into()]);
        assert_eq!(engine.negotiations.len(), 1);
    }
}
