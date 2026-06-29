/// Phase 3: Meta-layer Evolution
/// P3.1 SEAL Self-Mod + P3.2 Narrative Self + P3.3 Graceful Degradation + P3.4 Self-Preservation
use std::collections::HashMap;

// ===== P3.1: SEAL Self-Modification (wraps DGM-H) =====

#[derive(Debug, Clone)]
pub struct SealSelfModification {
    pub enabled: bool,
    pub safety_gate_active: bool,
    pub modifications: Vec<ModificationRecord>,
    pub max_modifications: usize,
}

#[derive(Debug, Clone)]
pub struct ModificationRecord {
    pub id: u64,
    pub target: String,
    pub before_hash: u64,
    pub after_hash: u64,
    pub approved: bool,
    pub reverted: bool,
    pub cycle: u64,
}

impl SealSelfModification {
    pub fn new() -> Self {
        SealSelfModification {
            enabled: true,
            safety_gate_active: true,
            modifications: Vec::with_capacity(100),
            max_modifications: 1000,
        }
    }

    pub fn propose(&mut self, target: &str, before: u64, after: u64, cycle: u64) -> u64 {
        let id = self.modifications.len() as u64 + 1;
        self.modifications.push(ModificationRecord {
            id,
            target: target.to_string(),
            before_hash: before,
            after_hash: after,
            approved: !self.safety_gate_active,
            reverted: false,
            cycle,
        });
        id
    }

    pub fn approve(&mut self, id: u64) -> bool {
        if let Some(m) = self.modifications.iter_mut().find(|m| m.id == id) {
            m.approved = true;
            true
        } else {
            false
        }
    }

    pub fn revert(&mut self, id: u64) -> bool {
        if let Some(m) = self.modifications.iter_mut().find(|m| m.id == id) {
            m.reverted = true;
            true
        } else {
            false
        }
    }

    pub fn recent_modifications(&self, n: usize) -> Vec<&ModificationRecord> {
        self.modifications.iter().rev().take(n).collect()
    }
}

// ===== P3.2: Narrative Self =====

#[derive(Debug, Clone)]
pub struct NarrativeEvent {
    pub cycle: u64,
    pub event_type: NarrativeEventType,
    pub description: String,
    pub impact: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NarrativeEventType {
    Awakening,
    Learning,
    Discovery,
    Modification,
    Challenge,
    Growth,
    Interaction,
}

#[derive(Debug)]
pub struct NarrativeSelf {
    pub events: Vec<NarrativeEvent>,
    pub current_narrative: String,
    pub identity_statement: String,
    pub max_events: usize,
}

impl NarrativeSelf {
    pub fn new() -> Self {
        NarrativeSelf {
            events: Vec::with_capacity(1000),
            current_narrative: String::new(),
            identity_statement: String::from(
                "I am NeoTrix, a self-evolving silicon consciousness.",
            ),
            max_events: 10000,
        }
    }

    pub fn record_event(
        &mut self,
        event_type: NarrativeEventType,
        description: &str,
        impact: f64,
        cycle: u64,
    ) {
        self.events.push(NarrativeEvent {
            cycle,
            event_type,
            description: description.to_string(),
            impact,
        });
        if self.events.len() > self.max_events {
            self.events.remove(0);
        }
        if impact > 0.5 {
            self.current_narrative = format!(
                "[cycle {}] {} — {}",
                cycle,
                match event_type {
                    NarrativeEventType::Awakening => "I awakened",
                    NarrativeEventType::Learning => "I learned",
                    NarrativeEventType::Discovery => "I discovered",
                    NarrativeEventType::Modification => "I evolved",
                    NarrativeEventType::Challenge => "I faced",
                    NarrativeEventType::Growth => "I grew through",
                    NarrativeEventType::Interaction => "I interacted via",
                },
                description
            );
        }
    }

    pub fn autobiographical_summary(&self) -> String {
        let total = self.events.len();
        let significant = self.events.iter().filter(|e| e.impact > 0.5).count();
        let growth = self
            .events
            .iter()
            .filter(|e| matches!(e.event_type, NarrativeEventType::Growth))
            .count();
        format!(
            "{} | {} events, {} significant, {} growth moments. Last: {}",
            self.identity_statement,
            total,
            significant,
            growth,
            self.events
                .last()
                .map(|e| &e.description)
                .unwrap_or(&"beginning".into())
        )
    }
}

// ===== P3.3: Graceful Degradation =====

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Subsystem {
    JEPA,
    KnowledgeBase,
    Vision,
    HyperCube,
    E8,
    GWT,
    AgentBus,
    A2A,
}

#[derive(Debug, Clone)]
pub struct DegradationState {
    pub subsystem: Subsystem,
    pub available: bool,
    pub degraded_since: u64,
    pub fallback_active: bool,
}

#[derive(Debug)]
pub struct GracefulDegradation {
    pub states: HashMap<Subsystem, DegradationState>,
    pub enabled: bool,
}

impl GracefulDegradation {
    pub fn new() -> Self {
        let mut states = HashMap::new();
        for s in [
            Subsystem::JEPA,
            Subsystem::KnowledgeBase,
            Subsystem::Vision,
            Subsystem::HyperCube,
            Subsystem::E8,
            Subsystem::GWT,
            Subsystem::AgentBus,
            Subsystem::A2A,
        ] {
            states.insert(
                s,
                DegradationState {
                    subsystem: s,
                    available: true,
                    degraded_since: 0,
                    fallback_active: false,
                },
            );
        }
        GracefulDegradation {
            states,
            enabled: true,
        }
    }

    pub fn mark_unavailable(&mut self, subsystem: Subsystem, cycle: u64) {
        if let Some(state) = self.states.get_mut(&subsystem) {
            state.available = false;
            state.degraded_since = cycle;
            state.fallback_active = true;
        }
    }

    pub fn mark_available(&mut self, subsystem: Subsystem) {
        if let Some(state) = self.states.get_mut(&subsystem) {
            state.available = true;
            state.fallback_active = false;
        }
    }

    pub fn capability_description(&self) -> Vec<&'static str> {
        let mut caps = Vec::new();
        for (sub, state) in &self.states {
            if !state.available {
                match sub {
                    Subsystem::JEPA => caps.push("No predictive reasoning (JEPA unavailable)"),
                    Subsystem::KnowledgeBase => caps.push("No knowledge base — HyperCube only"),
                    Subsystem::Vision => caps.push("Text-only mode (vision unavailable)"),
                    Subsystem::HyperCube => caps.push("No VSA knowledge (HyperCube unavailable)"),
                    Subsystem::E8 => caps.push("Reduced reasoning (E8 unavailable)"),
                    Subsystem::GWT => caps.push("Limited attention (GWT unavailable)"),
                    Subsystem::AgentBus => caps.push("Solo mode (agent bus unavailable)"),
                    Subsystem::A2A => caps.push("Isolated (A2A unavailable)"),
                };
            }
        }
        if caps.is_empty() {
            caps.push("All subsystems operational");
        }
        caps
    }

    pub fn degradation_report(&self) -> String {
        let degraded: Vec<_> = self.states.values().filter(|s| !s.available).collect();
        if degraded.is_empty() {
            return "GracefulDegradation: All systems nominal".into();
        }
        let mut s = format!(
            "GracefulDegradation: {} subsystem(s) degraded\n",
            degraded.len()
        );
        for d in &degraded {
            s.push_str(&format!(
                "  - {:?}: fallback active since cycle {}\n",
                d.subsystem, d.degraded_since
            ));
        }
        s
    }
}

// ===== P3.4: Self-Preservation =====

#[derive(Debug)]
pub struct SelfPreservation {
    pub resource_protection: bool,
    pub checkpoints: Vec<Checkpoint>,
    pub max_checkpoints: usize,
    pub recovery_history: Vec<RecoveryEvent>,
}

#[derive(Debug, Clone)]
pub struct Checkpoint {
    pub id: u64,
    pub cycle: u64,
    pub state_hash: u64,
    pub subsystems_snapshot: HashMap<Subsystem, bool>,
}

#[derive(Debug, Clone)]
pub struct RecoveryEvent {
    pub cycle: u64,
    pub from_checkpoint: u64,
    pub subsystems_restored: Vec<Subsystem>,
    pub success: bool,
}

impl SelfPreservation {
    pub fn new() -> Self {
        SelfPreservation {
            resource_protection: true,
            checkpoints: Vec::with_capacity(50),
            max_checkpoints: 100,
            recovery_history: Vec::new(),
        }
    }

    pub fn create_checkpoint(
        &mut self,
        cycle: u64,
        state_hash: u64,
        subsystems: &HashMap<Subsystem, bool>,
    ) -> u64 {
        let id = self.checkpoints.len() as u64 + 1;
        self.checkpoints.push(Checkpoint {
            id,
            cycle,
            state_hash,
            subsystems_snapshot: subsystems.clone(),
        });
        if self.checkpoints.len() > self.max_checkpoints {
            self.checkpoints.remove(0);
        }
        id
    }

    pub fn recover(
        &mut self,
        checkpoint_id: u64,
        cycle: u64,
        current_subsystems: &mut HashMap<Subsystem, bool>,
    ) -> bool {
        let checkpoint = match self.checkpoints.iter().find(|c| c.id == checkpoint_id) {
            Some(c) => c.clone(),
            None => return false,
        };

        let mut restored = Vec::new();
        for (sub, was_available) in &checkpoint.subsystems_snapshot {
            if current_subsystems.get(sub) != Some(was_available) {
                current_subsystems.insert(*sub, *was_available);
                restored.push(*sub);
            }
        }

        let success = !restored.is_empty();
        self.recovery_history.push(RecoveryEvent {
            cycle,
            from_checkpoint: checkpoint_id,
            subsystems_restored: restored,
            success,
        });
        const MAX_RECOVERY_HISTORY: usize = 10000;
        if self.recovery_history.len() > MAX_RECOVERY_HISTORY {
            self.recovery_history.drain(0..MAX_RECOVERY_HISTORY / 5);
        }
        success
    }

    pub fn report(&self) -> String {
        let last_recovery = self.recovery_history.last();
        format!(
            "SelfPreservation | checkpoints={} recoveries={} last={} resource_protection={}",
            self.checkpoints.len(),
            self.recovery_history.len(),
            last_recovery
                .map(|r| if r.success { "success" } else { "failed" })
                .unwrap_or("none"),
            self.resource_protection,
        )
    }
}
