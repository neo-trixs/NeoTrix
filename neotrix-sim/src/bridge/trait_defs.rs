use crate::agents::{AgentAction, Personality};
use crate::consciousness::ConsciousnessState;
use crate::world_sim::EvolutionRecord;
use crate::foundation::SimEvent;
use crate::world_sim::WorldSnapshot;
use serde::{Deserialize, Serialize};

/// Direction of information flow across the bridge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BridgeDirection {
    SimToCore,
    CoreToSim,
    Bidirectional,
}

/// Value-weighted goal injected by the core.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreDirective {
    pub goal: String,
    pub weight: f64,
    pub constraint: Option<String>,
    pub ttl: u64,
    pub issued_at: u64,
}

impl CoreDirective {
    pub fn new(goal: &str, weight: f64, ttl: u64, issued_at: u64) -> Self {
        Self {
            goal: goal.to_string(),
            weight: weight.clamp(0.0, 1.0),
            constraint: None,
            ttl,
            issued_at,
        }
    }

    pub fn with_constraint(mut self, constraint: &str) -> Self {
        self.constraint = Some(constraint.to_string());
        self
    }

    pub fn is_expired(&self, current_tick: u64) -> bool {
        current_tick.saturating_sub(self.issued_at) >= self.ttl
    }
}

/// Sim report sent to core each cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimReport {
    pub snapshot: WorldSnapshot,
    pub consciousness: ConsciousnessState,
    pub phi_trend: Vec<f64>,
    pub evolution_records: Vec<EvolutionRecord>,
    pub notable_events: Vec<SimEvent>,
    pub agent_highlights: Vec<AgentHighlight>,
    pub recommended_core_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHighlight {
    pub agent_id: u64,
    pub position: [f32; 2],
    pub fitness: f64,
    pub phi: f64,
    pub dominant_trait: String,
    pub notable_action: String,
}

/// Trait NeoTrix core implements to receive sim reports.
pub trait SimObserver {
    fn on_sim_report(&mut self, report: &SimReport);
    fn on_consciousness_event(&mut self, agent_id: u64, phi_before: f64, phi_after: f64);
    fn on_evolution_event(&mut self, record: &EvolutionRecord);
}

/// Trait the sim implements to receive core directives.
pub trait CoreReceiver {
    fn poll_directives(&self, current_tick: u64) -> Vec<CoreDirective>;
    fn apply_core_adjustment(&mut self, adjustment: CoreAdjustment);
}

/// Core→Sim adjustments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoreAdjustment {
    SelectionPressure(f64),
    MutationRate(f64),
    TickSpeed(f64),
    InjectAgent {
        position: [f32; 2],
        personality: Personality,
        energy: f32,
    },
    EvolutionEnabled(bool),
    OverrideAction {
        agent_id: u64,
        action: AgentAction,
    },
}
