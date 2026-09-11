use crate::agents::sim_agent::{SimAgent, AgentObservation, NearbyAgent, NearbyResource, Threat};
use crate::feel::SystemEvent;
use super::WorldSim;

impl WorldSim {
    pub(crate) fn build_observation(&self, agent: &SimAgent, nearby_ids: &[&str]) -> AgentObservation {
        let nearby_agents: Vec<NearbyAgent> = nearby_ids.iter()
            .filter_map(|id| {
                self.agents.iter().find(|a| &a.core.id == *id && a.core.id != agent.core.id)
                    .map(|a| NearbyAgent {
                        id: a.core.id.clone(),
                        distance: agent.core.position.distance_to(&a.core.position),
                        relationship: self.relationships.sentiment_between(&agent.core.id, &a.core.id),
                        apparent_health: a.core.health / 100.0,
                    })
            })
            .collect();

        let nearby_resources: Vec<NearbyResource> = self.resources.nodes.iter()
            .filter(|r| !r.depleted)
            .filter(|r| {
                let dx = r.position.0 - agent.core.position.x;
                let dy = r.position.1 - agent.core.position.y;
                (dx * dx + dy * dy).sqrt() < 100.0
            })
            .map(|r| NearbyResource {
                id: r.id.clone(),
                resource_type: format!("{:?}", r.resource_type),
                distance: ((r.position.0 - agent.core.position.x).powi(2)
                    + (r.position.1 - agent.core.position.y).powi(2)).sqrt(),
                amount: r.amount,
            })
            .collect();

        let biome = self.biome_map.biome_at(
            agent.core.position.x,
            agent.core.position.y,
            self.config.world_width,
            self.config.world_height,
        );
        let terrain_type = format!("{:?}", biome);

        let height = self.heightmap.height_at(agent.core.position.x, agent.core.position.y);
        let mut threats: Vec<Threat> = vec![];
        if self.heightmap.is_mountain(agent.core.position.x, agent.core.position.y) {
            threats.push(Threat {
                threat_type: "mountain".to_string(),
                position: agent.core.position,
                severity: (height - self.heightmap.config().mountain_level) / (1.0 - self.heightmap.config().mountain_level).max(0.01),
            });
        }
        if self.heightmap.is_water(agent.core.position.x, agent.core.position.y) {
            threats.push(Threat {
                threat_type: "water".to_string(),
                position: agent.core.position,
                severity: 0.3,
            });
        }

        AgentObservation {
            position: agent.core.position,
            nearby_agents,
            nearby_resources,
            terrain_type,
            time_of_day: format!("{}", self.clock.day_phase()),
            season: format!("{:?}", self.clock.current.season),
            threats,
        }
    }

    pub(crate) fn build_inventory_near(&self, x: f32, y: f32, radius: f32) -> crate::society::economy::Inventory {
        use crate::environment::terrain::resources::ResourceType as RT;
        let mut inv = crate::society::economy::Inventory::new();
        for r in self.resources.nodes.iter().filter(|r| !r.depleted) {
            let dx = r.position.0 - x;
            let dy = r.position.1 - y;
            if (dx * dx + dy * dy).sqrt() >= radius {
                continue;
            }
            match r.resource_type {
                RT::Berries | RT::Fish | RT::Meat => {
                    inv.add(crate::society::economy::ResourceType::Food, r.amount * 0.3);
                }
                RT::Wood => {
                    inv.add(crate::society::economy::ResourceType::Wood, r.amount * 0.3);
                }
                RT::Stone | RT::Ore => {
                    inv.add(crate::society::economy::ResourceType::Stone, r.amount * 0.2);
                }
                RT::Water => {
                    inv.add(crate::society::economy::ResourceType::Water, r.amount * 0.3);
                }
                _ => {}
            }
        }
        inv
    }

    pub(crate) fn collect_world_events(&self) -> Vec<SystemEvent> {
        let mut events = Vec::new();

        let alive = self.agents.iter().filter(|a| a.core.alive).count();
        let total = self.agents.len().max(1);
        let mortality = 1.0 - (alive as f32 / total as f32);
        if mortality > 0.1 {
            events.push(SystemEvent::GoalBlocked { attempts: (mortality * 10.0) as u32 });
        }

        let depleted_ratio = if self.resources.total_nodes() > 0 {
            self.resources.depleted_nodes() as f32 / self.resources.total_nodes() as f32
        } else {
            0.0
        };
        if depleted_ratio > 0.3 {
            events.push(SystemEvent::ErrorRate { rate: depleted_ratio });
        }

        if let Some(global) = self.coherence_tracker.current() {
            if global.mean_phi > 0.5 {
                events.push(SystemEvent::NoveltyDetected { score: global.mean_phi as f32 });
            }
            if global.mean_coherence > 0.6 {
                events.push(SystemEvent::ConsensusReached { agreement: global.mean_coherence as f32 });
            }
        }

        let mean_health: f32 = self.agents.iter()
            .filter(|a| a.core.alive)
            .map(|a| a.core.health)
            .sum::<f32>()
            .max(0.01)
            / alive.max(1) as f32;
        if mean_health < 40.0 {
            events.push(SystemEvent::BatteryLow { level: mean_health / 100.0 });
        }

        let social_count = self.relationships.total_relationships();
        if social_count > 5 {
            events.push(SystemEvent::PeerConnected { id: "community".into() });
        }

        events
    }
}
