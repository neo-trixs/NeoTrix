/// Multi-agent fleet simulation — "鸭子合唱"
///
/// Multiple SimState instances communicating via BLE beacons.
/// Demonstrates group emotion field and consensus protocol.
///
/// Based on Microduck's chorale: multiple ducks singing in harmony,
/// synchronized via BLE beacons with no shared clock.

use crate::SimState;
use crate::feel::{EmotionType, SystemEvent, PadVector};

/// BLE beacon — the only shared state between agents
#[derive(Debug, Clone)]
pub struct BleBeacon {
    pub agent_id: u32,
    pub position: [f32; 3],
    pub dominant_emotion: Option<EmotionType>,
    pub emotion_intensity: f32,
    pub pad: PadVector,
    pub name: String,
}

/// Group emotion field — emergent emotional state of the fleet
#[derive(Debug, Clone)]
pub struct GroupEmotionField {
    /// Average PAD across all agents
    pub avg_pad: PadVector,
    /// Emotional consensus (0=disagree, 1=unanimous)
    pub consensus: f32,
    /// Dominant emotion of the group
    pub dominant: Option<EmotionType>,
    /// Trust network strength
    pub trust_field: f32,
    /// Resonance (emotional synchronization)
    pub resonance: f32,
}

impl GroupEmotionField {
    pub fn new() -> Self {
        Self {
            avg_pad: PadVector::default(),
            consensus: 0.0,
            dominant: None,
            trust_field: 0.0,
            resonance: 0.0,
        }
    }
}

/// A fleet of coordinated agents
pub struct SimFleet {
    pub agents: Vec<FleetAgent>,
    pub group_field: GroupEmotionField,
    pub tick: u64,
    pub ble_range: f32,
}

pub struct FleetAgent {
    pub id: u32,
    pub state: SimState,
    pub name: String,
    pub received_beacons: Vec<BleBeacon>,
}

impl SimFleet {
    pub fn new(names: &[&str]) -> Self {
        let agents = names.iter().enumerate().map(|(i, &name)| {
            FleetAgent {
                id: i as u32,
                state: SimState::new(),
                name: name.to_string(),
                received_beacons: Vec::new(),
            }
        }).collect();

        Self {
            agents,
            group_field: GroupEmotionField::new(),
            tick: 0,
            ble_range: 2.0, // 2 meter BLE range
        }
    }

    /// Run one tick for all agents
    pub fn tick(&mut self) {
        self.tick += 1;

        // 1. Each agent runs its own control loop
        for agent in &mut self.agents {
            agent.state.tick();
        }

        // 2. Broadcast beacons
        let beacons: Vec<BleBeacon> = self.agents.iter().map(|a| {
            BleBeacon {
                agent_id: a.id,
                position: [
                    a.state.physics.position.x,
                    a.state.physics.position.y,
                    a.state.physics.position.z,
                ],
                dominant_emotion: a.state.feel.dominant_emotion(),
                emotion_intensity: a.state.feel.dominant_emotion()
                    .map(|e| a.state.feel.get_emotion(e))
                    .unwrap_or(0.0),
                pad: a.state.feel.pad,
                name: a.name.clone(),
            }
        }).collect();

        // 3. Deliver beacons to agents in range
        for agent in &mut self.agents {
            agent.received_beacons.clear();
            let my_pos = [
                agent.state.physics.position.x,
                agent.state.physics.position.y,
            ];

            for beacon in &beacons {
                if beacon.agent_id == agent.id {
                    continue; // no self-talk
                }
                let dx = beacon.position[0] - my_pos[0];
                let dy = beacon.position[1] - my_pos[1];
                let dist = (dx * dx + dy * dy).sqrt();

                if dist < self.ble_range {
                    agent.received_beacons.push(beacon.clone());

                    // Trigger social emotions from peers
                    if let Some(peer_emotion) = beacon.dominant_emotion {
                        match peer_emotion {
                            EmotionType::Joy => {
                                agent.state.feel.process_events(&[
                                    SystemEvent::PeerConnected {
                                        id: beacon.name.clone(),
                                    },
                                ]);
                            }
                            EmotionType::Frustration | EmotionType::Anxiety => {
                                agent.state.feel.process_events(&[
                                    SystemEvent::PeerStruggling {
                                        peer_id: beacon.name.clone(),
                                    },
                                ]);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // 4. Consensus protocol: agents move toward group average
        self.update_group_field();

        // 5. Resonance effect: high consensus → positive feedback
        if self.group_field.consensus > 0.7 {
            for agent in &mut self.agents {
                agent.state.feel.process_events(&[
                    SystemEvent::ConsensusReached {
                        agreement: self.group_field.consensus,
                    },
                ]);
            }
        }
    }

    pub fn update_group_field(&mut self) {
        let n = self.agents.len() as f32;
        if n == 0.0 {
            return;
        }

        // Average PAD
        let mut avg_v = 0.0f32;
        let mut avg_a = 0.0f32;
        let mut avg_d = 0.0f32;
        for agent in &self.agents {
            avg_v += agent.state.feel.pad.valence;
            avg_a += agent.state.feel.pad.arousal;
            avg_d += agent.state.feel.pad.dominance;
        }
        self.group_field.avg_pad = PadVector {
            valence: avg_v / n,
            arousal: avg_a / n,
            dominance: avg_d / n,
        };

        // Consensus: 1 - std_dev / range
        let var_v: f32 = self.agents.iter()
            .map(|a| (a.state.feel.pad.valence - self.group_field.avg_pad.valence).powi(2))
            .sum::<f32>() / n;
        let consensus = (1.0 - var_v.sqrt()).max(0.0);
        self.group_field.consensus = consensus;

        // Dominant emotion of the group
        let mut emotion_counts = std::collections::HashMap::new();
        for agent in &self.agents {
            if let Some(e) = agent.state.feel.dominant_emotion() {
                *emotion_counts.entry(e).or_insert(0) += 1;
            }
        }
        self.group_field.dominant = emotion_counts.into_iter()
            .max_by_key(|(_, c)| *c)
            .map(|(e, _)| e);

        // Trust field: average trust across agents
        self.group_field.trust_field = self.agents.iter()
            .map(|a| a.state.feel.get_emotion(EmotionType::Trust))
            .sum::<f32>() / n;

        // Resonance: average resonance
        self.group_field.resonance = self.agents.iter()
            .map(|a| a.state.feel.get_emotion(EmotionType::Resonance))
            .sum::<f32>() / n;
    }

    /// Print fleet status
    pub fn print_status(&self) {
        println!("  Fleet ({} agents, tick {}):", self.agents.len(), self.tick);
        for agent in &self.agents {
            let dominant = agent.state.feel.dominant_emotion()
                .map(|e| format!("{:?}", e))
                .unwrap_or_else(|| "None".into());
            let peers = agent.received_beacons.len();
            println!("    [{}] emotion={} peers={} pad=({:.2},{:.2},{:.2})",
                agent.name, dominant, peers,
                agent.state.feel.pad.valence,
                agent.state.feel.pad.arousal,
                agent.state.feel.pad.dominance);
        }
        println!("  Group: consensus={:.2} dominant={:?} trust={:.2} resonance={:.2}",
            self.group_field.consensus,
            self.group_field.dominant,
            self.group_field.trust_field,
            self.group_field.resonance);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fleet_creation() {
        let fleet = SimFleet::new(&["Alice", "Bob", "Charlie"]);
        assert_eq!(fleet.agents.len(), 3);
    }

    #[test]
    fn fleet_tick() {
        let mut fleet = SimFleet::new(&["A", "B"]);
        fleet.tick();
        assert_eq!(fleet.tick, 1);
    }

    #[test]
    fn group_field_updates() {
        let mut fleet = SimFleet::new(&["A", "B", "C"]);
        fleet.tick();
        // Consensus should be high since all agents start identical
        assert!(fleet.group_field.consensus > 0.5);
    }

    #[test]
    fn ble_delivery() {
        let mut fleet = SimFleet::new(&["A", "B"]);
        fleet.tick();
        // Agents in range should receive beacons
        // Default positions are same, so they should see each other
        let total_beacons: usize = fleet.agents.iter()
            .map(|a| a.received_beacons.len())
            .sum();
        assert!(total_beacons > 0);
    }
}
