use crate::foundation::simulation_bus::{SimEvent, EventPriority};
use crate::agents::sim_agent::AgentAction;
use crate::agents::pheromone::PheromoneType;
use super::WorldSim;

impl WorldSim {
    pub(crate) async fn execute_action(&mut self, agent_id: &str, action: &AgentAction) {
        let agent_idx = self.agents.iter().position(|a| &a.core.id == agent_id);
        let Some(idx) = agent_idx else { return };

        match action {
            AgentAction::Move { target } => {
                let agent = &mut self.agents[idx];
                let dir = (*target - agent.core.position).normalize();
                let speed = 5.0;
                agent.core.position = agent.core.position + dir * speed;
                agent.core.position.x = agent.core.position.x.clamp(0.0, self.config.world_width);
                agent.core.position.y = agent.core.position.y.clamp(0.0, self.config.world_height);
            }
            AgentAction::Eat { resource_id } => {
                if let Some(res) = self.resources.nodes.iter_mut().find(|r| &r.id == resource_id) {
                    let amount = res.harvest(20.0);
                    if amount > 0.0 {
                        self.agents[idx].core.eat(amount);
                    }
                }
            }
            AgentAction::Rest => {
                self.agents[idx].core.rest(5.0);
            }
            AgentAction::Talk { target_id, message } => {
                let agent_id_owned = agent_id.to_string();
                let target_id_owned = target_id.clone();
                self.relationships.update_interaction(&agent_id_owned, &target_id_owned, 0.1, self.tick);
                self.phi_bridge.record_interaction(crate::consciousness::phi_bridge::InteractionRecord {
                    agent_a: agent_id_owned.clone(),
                    agent_b: target_id_owned.clone(),
                    interaction_type: "talk".to_string(),
                    timestamp: self.tick,
                    success: true,
                });
                let meme = self.culture.create_meme(message, &agent_id_owned, self.tick);
                let meme_id = meme.id.clone();
                self.culture.spread_meme(&meme_id, &target_id_owned, 0.5);
            }
            AgentAction::Explore { direction } => {
                let agent = &mut self.agents[idx];
                let dir = direction.normalize() * 10.0;
                agent.core.position = agent.core.position + dir;
                agent.core.position.x = agent.core.position.x.clamp(0.0, self.config.world_width);
                agent.core.position.y = agent.core.position.y.clamp(0.0, self.config.world_height);
            }
            AgentAction::Trade { target_id, item, amount } => {
                let agent_id_owned = agent_id.to_string();
                let target_id_owned = target_id.clone();
                let agent_inv = {
                    let agent = &self.agents[idx];
                    self.build_inventory_near(agent.core.position.x, agent.core.position.y, 150.0)
                };
                let target_inv = {
                    if let Some(target_agent) = self.agents.iter().find(|a| &a.core.id == &target_id_owned) {
                        self.build_inventory_near(target_agent.core.position.x, target_agent.core.position.y, 150.0)
                    } else {
                        crate::society::economy::Inventory::new()
                    }
                };
                let offer = crate::society::economy::TradeOffer {
                    from: agent_id_owned.clone(),
                    to: target_id_owned.clone(),
                    offer: crate::society::economy::ResourceType::Food,
                    offer_amount: *amount as f32,
                    want: crate::society::economy::ResourceType::Wood,
                    want_amount: *amount as f32 * 0.5,
                };
                let mut buyer_inv = agent_inv;
                let mut seller_inv = target_inv;
                let trade_ok = self.economy.execute_trade(offer, &mut buyer_inv, &mut seller_inv);
                self.relationships.update_interaction(&agent_id_owned, &target_id_owned, 0.15, self.tick);
                if trade_ok {
                    self.bus.emit(
                        SimEvent::EconomyTransaction {
                            buyer: agent_id_owned,
                            seller: target_id_owned,
                            item: item.clone(),
                            amount: *amount,
                        },
                        EventPriority::Normal,
                        self.clock.current,
                        "world_sim",
                    ).await;
                }
            }
            AgentAction::Build { position, structure_type } => {
                use crate::environment::structures::StructureType;
                let struct_type = match structure_type.as_str() {
                    "shelter" => StructureType::Shelter,
                    "farm" => StructureType::Farm,
                    "workshop" => StructureType::Workshop,
                    "watchtower" => StructureType::Watchtower,
                    "market" => StructureType::Market,
                    "wall" => StructureType::Wall,
                    "road" => StructureType::Road,
                    _ => StructureType::Shelter,
                };
                let _ = self.structures.build(
                    struct_type,
                    [position.x, position.y],
                    agent_id,
                    self.tick,
                );
            }
            AgentAction::Harvest { resource_id } => {
                if let Some(res) = self.resources.nodes.iter_mut().find(|r| &r.id == resource_id) {
                    let harvested = res.harvest(30.0);
                    if harvested > 0.0 {
                        let agent = &mut self.agents[idx];
                        let nutrition = res.resource_type.nutrition_value();
                        let energy_gain = res.resource_type.energy_value();
                        if nutrition > 0.0 {
                            agent.core.eat(harvested * nutrition / 30.0);
                        } else {
                            agent.core.energy = (agent.core.energy + energy_gain * harvested / 30.0).min(100.0);
                        }
                        self.bus.emit(
                            SimEvent::AgentActed {
                                agent_id: agent_id.to_string(),
                                action: "harvest".to_string(),
                                result: format!("harvested {:.1} from {}", harvested, resource_id),
                            },
                            EventPriority::Normal,
                            self.clock.current,
                            "world_sim",
                        ).await;
                        if res.depleted {
                            self.bus.emit(
                                SimEvent::ResourceDepleted {
                                    resource_id: res.id.clone(),
                                    position: res.position,
                                },
                                EventPriority::Normal,
                                self.clock.current,
                                "world_sim",
                            ).await;
                        }
                    }
                }
            }
            _ => {}
        }

        {
            let action_label = format!("{:?}_{}", agent_id, self.tick);
            let mut attrs = std::collections::HashMap::new();
            attrs.insert("action".to_string(), format!("{:?}", action));
            attrs.insert("agent".to_string(), agent_id.to_string());
            self.dual_repr.add(&action_label, "event", attrs, self.tick);
        }

        self.deposit_pheromones_for_action(agent_id, action);
    }

    pub(crate) fn deposit_pheromones_for_action(&mut self, agent_id: &str, action: &AgentAction) {
        let pos = if let Some(agent) = self.agents.iter().find(|a| &a.core.id == agent_id) {
            [agent.core.position.x, agent.core.position.y]
        } else {
            return;
        };

        match action {
            AgentAction::Eat { resource_id } => {
                if let Some(res) = self.resources.nodes.iter().find(|r| &r.id == resource_id) {
                    self.pheromone_field.deposit(
                        PheromoneType::Food,
                        [res.position.0, res.position.1],
                        agent_id,
                        self.tick,
                    );
                }
            }
            AgentAction::Harvest { resource_id } => {
                if let Some(res) = self.resources.nodes.iter().find(|r| &r.id == resource_id) {
                    self.pheromone_field.deposit(
                        PheromoneType::Food,
                        [res.position.0, res.position.1],
                        agent_id,
                        self.tick,
                    );
                }
            }
            AgentAction::Rest => {
                self.pheromone_field.deposit(PheromoneType::Rest, pos, agent_id, self.tick);
            }
            AgentAction::Talk { .. } => {
                self.pheromone_field.deposit(PheromoneType::Social, pos, agent_id, self.tick);
            }
            AgentAction::Attack { .. } => {
                self.pheromone_field.deposit(PheromoneType::Danger, pos, agent_id, self.tick);
            }
            AgentAction::Explore { .. } => {
                self.pheromone_field.deposit(PheromoneType::Explore, pos, agent_id, self.tick);
            }
            AgentAction::Build { position, .. } => {
                self.pheromone_field.deposit(
                    PheromoneType::Territory,
                    [position.x, position.y],
                    agent_id,
                    self.tick,
                );
            }
            AgentAction::Move { .. } | AgentAction::Trade { .. } | AgentAction::Think => {}
        }
    }

    pub(crate) async fn compute_consciousness_metrics(&mut self) {
        for agent in &self.agents {
            if !agent.core.alive { continue; }
            self.phi_bridge.update(&agent.core.id, self.tick, &agent.recent_actions);

            if let Some(state) = self.phi_bridge.get_state(&agent.core.id) {
                self.bus.emit(
                    SimEvent::PhiComputed {
                        agent_id: agent.core.id.clone(),
                        phi: state.phi,
                    },
                    EventPriority::Normal,
                    self.clock.current,
                    "phi_bridge",
                ).await;
            }
        }

        let global = self.coherence_tracker.compute(self.phi_bridge.all_states(), self.tick);

        if let Some((best_id, best_phi)) = self.phi_bridge.most_conscious() {
            println!("[Tick {}] GlobalCoherence: mean_phi={:.4}, best={}: {:.4}, pop={}, species={}",
                self.tick, global.mean_phi, best_id, best_phi, global.population,
                self.speciation.species_count());
        }
    }
}
