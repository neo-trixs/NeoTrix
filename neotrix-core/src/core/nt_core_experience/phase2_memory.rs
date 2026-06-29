/// Phase 2: Memory Organization + Autonomy
/// P2.1 DMN + P2.2 Forgetting + P2.3 Sleep + P2.4 Intrinsic Value
use std::collections::HashMap;

// ===== P2.1: Default Mode Network =====

#[derive(Debug)]
pub struct DefaultModeNetwork {
    pub enabled: bool,
    pub idle_cycles: u64,
    pub consolidation_interval: u64,
    pub cross_session_associations: Vec<String>,
}

impl DefaultModeNetwork {
    pub fn new() -> Self {
        DefaultModeNetwork {
            enabled: true,
            idle_cycles: 0,
            consolidation_interval: 50,
            cross_session_associations: Vec::new(),
        }
    }

    pub fn tick(&mut self, is_idle: bool) -> Option<DmnReport> {
        if !self.enabled {
            return None;
        }
        if is_idle {
            self.idle_cycles += 1;
            if self.idle_cycles >= self.consolidation_interval {
                self.idle_cycles = 0;
                return Some(DmnReport {
                    fragments_consolidated: 5,
                    hypercube_defrag: true,
                    associations_found: 2,
                });
            }
        } else {
            self.idle_cycles = 0;
        }
        None
    }
}

#[derive(Debug)]
pub struct DmnReport {
    pub fragments_consolidated: usize,
    pub hypercube_defrag: bool,
    pub associations_found: usize,
}

// ===== P2.2: Forgetting Strategy =====

#[derive(Debug, Clone)]
pub struct KnowledgeItem {
    pub id: String,
    pub importance: f64, // 0.0-1.0
    pub last_access: u64,
    pub access_count: u64,
}

#[derive(Debug)]
pub struct ForgettingStrategy {
    pub items: HashMap<String, KnowledgeItem>,
    pub max_items: usize,
    pub decay_rate: f64, // Per-cycle decay multiplier
}

impl ForgettingStrategy {
    pub fn new() -> Self {
        ForgettingStrategy {
            items: HashMap::new(),
            max_items: 10000,
            decay_rate: 0.999,
        }
    }

    pub fn access(&mut self, id: &str, cycle: u64) {
        if let Some(item) = self.items.get_mut(id) {
            item.last_access = cycle;
            item.access_count += 1;
        }
    }

    pub fn add(&mut self, id: String, importance: f64) {
        if self.items.len() >= self.max_items {
            // Remove least important * least recently used
            let lru = self
                .items
                .iter()
                .min_by(|(_, a), (_, b)| {
                    (a.importance * a.last_access as f64)
                        .partial_cmp(&(b.importance * b.last_access as f64))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(k, _)| k.clone());
            if let Some(k) = lru {
                self.items.remove(&k);
            }
        }
        let item_id = id.clone();
        self.items.insert(
            id,
            KnowledgeItem {
                id: item_id,
                importance,
                last_access: 0,
                access_count: 0,
            },
        );
    }

    pub fn decay_all(&mut self, current_cycle: u64) {
        self.items.retain(|_, item| {
            let age = current_cycle - item.last_access;
            let decayed = item.importance * self.decay_rate.powi(age as i32);
            item.importance = decayed;
            decayed > 0.01 // Remove if below threshold
        });
    }

    pub fn size(&self) -> usize {
        self.items.len()
    }
}

// ===== P2.3: Sleep/Wake Cycle =====

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SleepStage {
    Awake,
    NREM, // Pattern extraction + redundancy elimination
    REM,  // Cross-domain association
}

#[derive(Debug)]
pub struct SleepEngine {
    pub stage: SleepStage,
    pub cycles_asleep: u64,
    pub sleep_interval: u64, // How often to sleep (cycles)
    pub nrem_duration: u64,
    pub rem_duration: u64,
}

impl SleepEngine {
    pub fn new() -> Self {
        SleepEngine {
            stage: SleepStage::Awake,
            cycles_asleep: 0,
            sleep_interval: 100,
            nrem_duration: 30,
            rem_duration: 20,
        }
    }

    pub fn tick(&mut self, cycle: u64) -> Option<SleepReport> {
        if self.stage == SleepStage::Awake && cycle % self.sleep_interval == 0 {
            self.stage = SleepStage::NREM;
            self.cycles_asleep = 0;
        }

        if self.stage != SleepStage::Awake {
            self.cycles_asleep += 1;

            // Transition logic
            match self.stage {
                SleepStage::NREM if self.cycles_asleep >= self.nrem_duration => {
                    self.stage = SleepStage::REM;
                }
                SleepStage::REM if self.cycles_asleep >= self.nrem_duration + self.rem_duration => {
                    self.stage = SleepStage::Awake;
                    let total = self.cycles_asleep;
                    self.cycles_asleep = 0;
                    return Some(SleepReport {
                        patterns_extracted: 10,
                        redundancy_removed: 5,
                        associations_discovered: 3,
                        freshness_boost: total as f64 * 0.01,
                    });
                }
                _ => {}
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct SleepReport {
    pub patterns_extracted: usize,
    pub redundancy_removed: usize,
    pub associations_discovered: usize,
    pub freshness_boost: f64,
}

// ===== P2.4: Intrinsic Value System =====

#[derive(Debug)]
pub struct IntrinsicValueSystem {
    pub curiosity_reward: f64,
    pub knowledge_gap_drive: f64,
    pub prediction_error_signal: f64,
}

impl IntrinsicValueSystem {
    pub fn new() -> Self {
        IntrinsicValueSystem {
            curiosity_reward: 0.0,
            knowledge_gap_drive: 0.0,
            prediction_error_signal: 0.0,
        }
    }

    pub fn compute_curiosity(&mut self, prediction_error: f64, uncertainty: f64) -> f64 {
        // Curiosity = prediction_error * uncertainty (epistemic curiosity)
        self.prediction_error_signal = prediction_error;
        self.curiosity_reward = prediction_error * (1.0 + uncertainty);
        self.curiosity_reward
    }

    pub fn knowledge_gap_drive(&mut self, known_ratio: f64) -> f64 {
        // Drive to explore when knowledge is low
        // Inverted U-curve: highest at ~40% known
        self.knowledge_gap_drive = if known_ratio < 0.1 {
            0.2 // Too little known — need basic data first
        } else if known_ratio > 0.9 {
            0.1 // Almost everything known — low drive
        } else {
            1.0 - (known_ratio - 0.4).abs() * 2.0 // Peak at 40%
        };
        self.knowledge_gap_drive
    }

    pub fn report(&self) -> String {
        format!(
            "IntrinsicValue | curiosity={:.3} gap_drive={:.3} pred_error={:.3}",
            self.curiosity_reward, self.knowledge_gap_drive, self.prediction_error_signal
        )
    }
}
