// SimulationBus - Unified event bus for NT-WORLD-SIM
// Modeled after tokio::sync::broadcast but simulation-aware
//
// Key design: Every simulation event flows through this bus.
// It provides:
// - Typed event channels (not raw bytes)
// - SimTime-aware event ordering
// - Event history for reflection/memory systems
// - Priority levels for GWT routing

use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use serde::{Serialize, Deserialize};

/// SimTime - Forward declaration, defined in sim_time.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SimTime {
    pub tick: u64,
    pub day: u32,
    pub hour: u8,
    pub minute: u8,
    pub season: Season,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

/// Priority for GWT attention routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EventPriority {
    Critical = 0,
    High = 1,
    Normal = 2,
    Low = 3,
    Debug = 4,
}

/// All simulation event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimEvent {
    // Environment events
    TimeAdvanced { time: SimTime },
    WeatherChanged { weather: String },
    SeasonChanged { season: Season },
    ResourceDepleted { resource_id: String, position: (f32, f32) },
    ResourceRegenerated { resource_id: String, position: (f32, f32) },

    // Agent events
    AgentSpawned { agent_id: String, position: (f32, f32) },
    AgentDied { agent_id: String, cause: String },
    AgentMoved { agent_id: String, from: (f32, f32), to: (f32, f32) },
    AgentActed { agent_id: String, action: String, result: String },
    AgentInteracted { agent_a: String, agent_b: String, interaction: String },

    // Social events
    RelationshipFormed { agent_a: String, agent_b: String, kind: String },
    RelationshipBroken { agent_a: String, agent_b: String },
    CultureSpread { meme: String, from: String, to: String },
    EconomyTransaction { buyer: String, seller: String, item: String, amount: u32 },

    // Consciousness events
    PhiComputed { agent_id: String, phi: f64 },
    CoherenceShift { agent_id: String, old: f64, new: f64 },
    GwtBroadcast { source: String, content: String, salience: f64 },

    // Evolution events
    FitnessScored { agent_id: String, fitness: f64 },
    MutationOccurred { agent_id: String, mutation_type: String },
    SpeciationEvent { group: String, new_species: String },

    // Safety events
    AgentNearDeath { agent_id: String, energy: f64 },
    EnvironmentHazard { position: (f32, f32), hazard_type: String },
}

/// Event record with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRecord {
    pub event: SimEvent,
    pub priority: EventPriority,
    pub timestamp: SimTime,
    pub source: String,
}

/// History buffer for reflection/memory systems
#[derive(Debug, Default)]
pub struct EventHistory {
    records: Vec<EventRecord>,
    max_size: usize,
}

impl EventHistory {
    pub fn new(max_size: usize) -> Self {
        Self { records: Vec::with_capacity(max_size), max_size }
    }

    pub fn push(&mut self, record: EventRecord) {
        if self.records.len() >= self.max_size {
            self.records.remove(0);
        }
        self.records.push(record);
    }

    pub fn recent(&self, n: usize) -> &[EventRecord] {
        let start = self.records.len().saturating_sub(n);
        &self.records[start..]
    }

    pub fn by_agent(&self, agent_id: &str) -> Vec<&EventRecord> {
        self.records.iter().filter(|r| {
            match &r.event {
                SimEvent::AgentSpawned { agent_id: id, .. } |
                SimEvent::AgentDied { agent_id: id, .. } |
                SimEvent::AgentMoved { agent_id: id, .. } |
                SimEvent::AgentActed { agent_id: id, .. } |
                SimEvent::PhiComputed { agent_id: id, .. } |
                SimEvent::CoherenceShift { agent_id: id, .. } |
                SimEvent::FitnessScored { agent_id: id, .. } |
                SimEvent::MutationOccurred { agent_id: id, .. } => id == agent_id,
                SimEvent::AgentInteracted { agent_a, agent_b, .. } => agent_a == agent_id || agent_b == agent_id,
                _ => false,
            }
        }).collect()
    }

    pub fn by_priority(&self, priority: EventPriority) -> Vec<&EventRecord> {
        self.records.iter().filter(|r| r.priority == priority).collect()
    }
}

/// The main simulation bus
pub struct SimulationBus {
    sender: broadcast::Sender<EventRecord>,
    history: Arc<RwLock<EventHistory>>,
    subscribers: Arc<RwLock<BTreeMap<String, usize>>>,
}

impl SimulationBus {
    pub fn new(history_size: usize) -> Self {
        let (sender, _) = broadcast::channel(1024);
        Self {
            sender,
            history: Arc::new(RwLock::new(EventHistory::new(history_size))),
            subscribers: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    /// Emit an event onto the bus
    pub async fn emit(&self, event: SimEvent, priority: EventPriority, time: SimTime, source: &str) {
        let record = EventRecord {
            event,
            priority,
            timestamp: time,
            source: source.to_string(),
        };

        self.history.write().await.push(record.clone());
        let _ = self.sender.send(record);
    }

    /// Subscribe to events
    pub fn subscribe(&self, name: &str) -> broadcast::Receiver<EventRecord> {
        self.subscribers.blocking_write().entry(name.to_string()).and_modify(|c| *c += 1).or_insert(1);
        self.sender.subscribe()
    }

    /// Get event history
    pub fn history(&self) -> &Arc<RwLock<EventHistory>> {
        &self.history
    }

    /// Get recent events
    pub async fn recent_events(&self, n: usize) -> Vec<EventRecord> {
        self.history.read().await.recent(n).to_vec()
    }

    /// Get events for a specific agent
    pub async fn agent_events(&self, agent_id: &str) -> Vec<EventRecord> {
        self.history.read().await.by_agent(agent_id).into_iter().cloned().collect()
    }

    /// Get subscriber count
    pub async fn subscriber_count(&self) -> usize {
        self.subscribers.read().await.values().sum()
    }
}

impl Clone for SimulationBus {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            history: Arc::clone(&self.history),
            subscribers: Arc::clone(&self.subscribers),
        }
    }
}
