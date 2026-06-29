use std::collections::VecDeque;

/// Signals that subsystems can broadcast to each other.
/// This is the "corpus callosum" of the cognitive architecture.
#[derive(Debug, Clone)]
pub enum IntegrationSignal {
    /// Temporal prediction divergence detected
    DivergenceDetected {
        error: f64,
        volatility: f64,
        cycle: u64,
    },
    /// Curiosity/exploration pressure
    CuriositySignal {
        score: f64,
        action_bonus: f64,
        cycle: u64,
    },
    /// Free energy curiosity from curiosity engine
    FreeEnergyCuriositySignal {
        score: f64,
        action_bonus: f64,
        cycle: u64,
    },
    /// IIT Phi computed
    PhiSignal {
        max_phi: f64,
        avg_phi: f64,
        integrated_info: f64,
        cycle: u64,
    },
    /// SEAL evolution event
    EvolutionEvent {
        mutated: bool,
        metric_delta: f64,
        cycle: u64,
    },
    /// Awakening insight
    AwakeningInsight {
        phi: f64,
        hypotheses: usize,
        speed: f64,
        cycle: u64,
    },
    /// Cross-model distillation completed
    DistillationSignal {
        total_interactions: usize,
        patterns_found: usize,
        capabilities_ranked: usize,
        knowledge_fragments: usize,
        top_model: String,
        cycle: u64,
    },
    /// Multi-timeline research insight emerged
    TimelineEmergence {
        timeline_count: usize,
        hypothesis_count: usize,
        emergence_score: f64,
        cycle: u64,
    },
    /// Constellation formed from cross-timeline correlation
    ConstellationFormed {
        constellation_id: String,
        star_count: usize,
        emergence_score: f64,
        cycle: u64,
    },
    /// Cross-timeline integration completed
    IntegrationCompleted {
        solution_id: String,
        integrated_timelines: usize,
        integration_score: f64,
        cycle: u64,
    },
    /// Evolutionary prediction generated
    PredictionGenerated {
        prediction_id: String,
        target: String,
        confidence: f64,
        cycle: u64,
    },
    /// Deep digestion completed on a knowledge source
    DigestionCompleted {
        node_count: usize,
        domain: String,
        avg_confidence: f64,
        cycle: u64,
    },
    /// GWA semantic entropy signal for dynamic temperature regulation
    SemanticEntropySignal {
        entropy: f64,
        temperature: f64,
        cycle: u64,
    },
}

/// Top-down modulation commands from Meta step to subsystems.
/// This is the "executive control / top-down attention" path.
#[derive(Debug, Clone)]
pub enum ModulationCommand {
    /// Increase exploration rate
    ExploreMore(f64),
    /// Increase exploitation rate
    ExploitMore(f64),
    /// Adjust cognitive load threshold
    SetCognitiveLoad(f64),
    /// Reset subsystem by name
    ResetSubsystem(String),
    /// General parameter adjustment
    SetParam {
        subsystem: String,
        param: String,
        value: f64,
    },
    /// Trigger distillation run
    RunDistillation,
    /// Adjust distillation buffer capacity
    SetDistillationBuffer(usize),
    /// Enable/disable cross-model distillation
    SetDistillationEnabled(bool),
    /// Set temperature parameter (semantic entropy drive)
    SetTemperature(f64),
    /// Set reasoning exploration temperature (divergent thinking pressure)
    SetReasonTemperature(f64),
}

/// SubsystemIntegrationBus — routes signals between cognitive subsystems.
/// Each subsystem can broadcast to the bus, and subscribers receive signals
/// in the next cycle step.
#[derive(Debug, Clone)]
pub struct SubsystemIntegrationBus {
    /// Signal queue for the current cycle
    pending_signals: Vec<IntegrationSignal>,
    /// Signal history (for analytics)
    signal_history: VecDeque<IntegrationSignal>,
    /// Max history
    max_history: usize,
    /// Cycle counter
    cycle: u64,
    /// Top-down modulation command queue
    pending_modulations: VecDeque<ModulationCommand>,
}

impl SubsystemIntegrationBus {
    pub fn new(max_history: usize) -> Self {
        Self {
            pending_signals: Vec::new(),
            signal_history: VecDeque::with_capacity(max_history),
            pending_modulations: VecDeque::new(),
            max_history,
            cycle: 0,
        }
    }

    /// Broadcast a signal to all subscribers
    pub fn broadcast(&mut self, signal: IntegrationSignal) {
        self.cycle += 1;
        self.pending_signals.push(signal);
    }

    /// Drain all pending signals for processing
    pub fn drain_pending(&mut self) -> Vec<IntegrationSignal> {
        let drained = self.pending_signals.drain(..).collect::<Vec<_>>();
        for sig in &drained {
            if self.signal_history.len() >= self.max_history {
                self.signal_history.pop_front();
            }
            self.signal_history.push_back(sig.clone());
        }
        drained
    }

    /// Send a modulation command to be consumed by subsystems in the next cycle
    pub fn send_modulation(&mut self, cmd: ModulationCommand) {
        self.pending_modulations.push_back(cmd);
    }

    /// Drain all pending modulation commands
    pub fn drain_modulations(&mut self) -> Vec<ModulationCommand> {
        self.pending_modulations.drain(..).collect()
    }

    /// Get the latest signal of a specific variant
    pub fn latest(&self, variant: &str) -> Option<&IntegrationSignal> {
        self.signal_history.iter().rev().find(|s| {
            matches!(
                (variant, s),
                ("divergence", IntegrationSignal::DivergenceDetected { .. })
                    | ("curiosity", IntegrationSignal::CuriositySignal { .. })
                    | ("phi", IntegrationSignal::PhiSignal { .. })
                    | ("evolution", IntegrationSignal::EvolutionEvent { .. })
                    | ("awakening", IntegrationSignal::AwakeningInsight { .. })
                    | ("distillation", IntegrationSignal::DistillationSignal { .. })
                    | (
                        "free_energy_curiosity",
                        IntegrationSignal::FreeEnergyCuriositySignal { .. }
                    )
                    | (
                        "timeline_emergence",
                        IntegrationSignal::TimelineEmergence { .. }
                    )
                    | (
                        "constellation",
                        IntegrationSignal::ConstellationFormed { .. }
                    )
                    | (
                        "integration",
                        IntegrationSignal::IntegrationCompleted { .. }
                    )
                    | ("prediction", IntegrationSignal::PredictionGenerated { .. })
                    | ("digestion", IntegrationSignal::DigestionCompleted { .. })
                    | (
                        "semantic_entropy",
                        IntegrationSignal::SemanticEntropySignal { .. }
                    )
            )
        })
    }

    /// Count signals by type
    pub fn signal_count(&self) -> serde_json::Value {
        let mut counts = serde_json::Map::new();
        for sig in &self.signal_history {
            let key = match sig {
                IntegrationSignal::DivergenceDetected { .. } => "divergence",
                IntegrationSignal::CuriositySignal { .. } => "curiosity",
                IntegrationSignal::FreeEnergyCuriositySignal { .. } => "free_energy_curiosity",
                IntegrationSignal::PhiSignal { .. } => "phi",
                IntegrationSignal::EvolutionEvent { .. } => "evolution",
                IntegrationSignal::AwakeningInsight { .. } => "awakening",
                IntegrationSignal::DistillationSignal { .. } => "distillation",
                IntegrationSignal::TimelineEmergence { .. } => "timeline_emergence",
                IntegrationSignal::ConstellationFormed { .. } => "constellation",
                IntegrationSignal::IntegrationCompleted { .. } => "integration",
                IntegrationSignal::PredictionGenerated { .. } => "prediction",
                IntegrationSignal::DigestionCompleted { .. } => "digestion",
                IntegrationSignal::SemanticEntropySignal { .. } => "semantic_entropy",
            };
            *counts
                .entry(key.to_string())
                .or_insert(serde_json::Value::Number(0.into())) = serde_json::Value::Number(
                (counts.get(key).and_then(|v| v.as_u64()).unwrap_or(0) + 1).into(),
            );
        }
        serde_json::Value::Object(counts)
    }

    /// Metrics for dashboard
    pub fn metrics(&self) -> serde_json::Value {
        serde_json::json!({
            "pending": self.pending_signals.len(),
            "history": self.signal_history.len(),
            "max_history": self.max_history,
            "cycle": self.cycle,
            "signal_counts": self.signal_count(),
            "pending_modulations": self.pending_modulations.len(),
        })
    }
}
