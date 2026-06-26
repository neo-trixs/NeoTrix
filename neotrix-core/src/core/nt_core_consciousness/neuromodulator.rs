// REVIVED Task 1 — dead_code removed 2026-06-24

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NeuromodulatorType {
    ACh,
    DA,
    NE,
    Serotonin5HT,
    Orexin,
    GABA,
}

#[derive(Debug, Clone)]
pub struct Neuromodulator {
    pub mod_type: NeuromodulatorType,
    pub level: f64,
    pub baseline: f64,
    pub decay_rate: f64,
}

impl Neuromodulator {
    pub fn new(mod_type: NeuromodulatorType) -> Self {
        let (baseline, decay_rate) = match mod_type {
            NeuromodulatorType::ACh => (0.5, 0.05),
            NeuromodulatorType::DA => (0.4, 0.08),
            NeuromodulatorType::NE => (0.3, 0.10),
            NeuromodulatorType::Serotonin5HT => (0.6, 0.03),
            NeuromodulatorType::Orexin => (0.5, 0.04),
            NeuromodulatorType::GABA => (0.3, 0.06),
        };
        Self {
            mod_type,
            level: baseline,
            baseline,
            decay_rate,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NeuromodulatorySystem {
    pub modulators: HashMap<NeuromodulatorType, Neuromodulator>,
    pub tick: u64,
}

impl NeuromodulatorySystem {
    pub fn new() -> Self {
        let mut modulators = HashMap::new();
        for mt in &[
            NeuromodulatorType::ACh,
            NeuromodulatorType::DA,
            NeuromodulatorType::NE,
            NeuromodulatorType::Serotonin5HT,
            NeuromodulatorType::Orexin,
            NeuromodulatorType::GABA,
        ] {
            modulators.insert(*mt, Neuromodulator::new(*mt));
        }
        Self {
            modulators,
            tick: 0,
        }
    }

    pub fn set_level(&mut self, mod_type: NeuromodulatorType, level: f64) {
        let level = level.clamp(0.0, 1.0);
        if let Some(m) = self.modulators.get_mut(&mod_type) {
            m.level = level;
        }
    }

    pub fn get_level(&self, mod_type: NeuromodulatorType) -> f64 {
        self.modulators.get(&mod_type).map_or(0.0, |m| m.level)
    }

    pub fn tick_decay(&mut self) {
        self.tick += 1;
        for m in self.modulators.values_mut() {
            m.level += (m.baseline - m.level) * m.decay_rate;
        }
    }

    pub fn phasic_burst(&mut self, mod_type: NeuromodulatorType, burst: f64) {
        if let Some(m) = self.modulators.get_mut(&mod_type) {
            m.level = (m.level + burst).clamp(0.0, 1.0);
        }
    }

    pub fn consciousness_influence(&self) -> ConsciousnessModulation {
        ConsciousnessModulation {
            attention_width: self.get_level(NeuromodulatorType::ACh),
            motivation: self.get_level(NeuromodulatorType::DA),
            arousal: self.get_level(NeuromodulatorType::NE),
            mood_stability: self.get_level(NeuromodulatorType::Serotonin5HT),
            wakefulness: self.get_level(NeuromodulatorType::Orexin),
            inhibition: self.get_level(NeuromodulatorType::GABA),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConsciousnessModulation {
    pub attention_width: f64,
    pub motivation: f64,
    pub arousal: f64,
    pub mood_stability: f64,
    pub wakefulness: f64,
    pub inhibition: f64,
}

impl ConsciousnessModulation {
    pub fn new() -> Self {
        Self {
            attention_width: 0.5,
            motivation: 0.4,
            arousal: 0.3,
            mood_stability: 0.6,
            wakefulness: 0.5,
            inhibition: 0.3,
        }
    }
}

/// Individual modulator level accessible as `.level` field.
#[derive(Debug, Clone)]
pub struct ModulatorLevel {
    pub level: f64,
}

impl ModulatorLevel {
    pub fn new(level: f64) -> Self {
        Self { level }
    }
}

/// Stats snapshot returned by `NeuromodulatorEngine::stats()`.
#[derive(Debug, Clone)]
pub struct NeuromodulatorStats {
    pub da: f64,
    pub ne: f64,
    pub ht: f64,
    pub ach: f64,
    pub arousal: f64,
    pub valence_bias: f64,
}

/// Engine wrapping NeuromodulatorySystem with meta-control parameters.
/// Backward-compatible with existing callers that access `.da.level`, `.ne.level`, etc.
#[derive(Debug, Clone)]
pub struct NeuromodulatorEngine {
    pub system: NeuromodulatorySystem,
    pub curiosity_rate: f64,
    pub da: ModulatorLevel,
    pub ne: ModulatorLevel,
    pub ht: ModulatorLevel,
    pub ach: ModulatorLevel,
}

impl NeuromodulatorEngine {
    pub fn new() -> Self {
        Self {
            system: NeuromodulatorySystem::new(),
            curiosity_rate: 0.5,
            da: ModulatorLevel::new(0.4),
            ne: ModulatorLevel::new(0.3),
            ht: ModulatorLevel::new(0.6),
            ach: ModulatorLevel::new(0.5),
        }
    }

    pub fn stats(&self) -> NeuromodulatorStats {
        NeuromodulatorStats {
            da: self.da.level,
            ne: self.ne.level,
            ht: self.ht.level,
            ach: self.ach.level,
            arousal: self.arousal_contribution(),
            valence_bias: 0.5,
        }
    }

    pub fn tick(&mut self, _delta: f64) {
        self.system.tick_decay();
        self.da.level = self.system.get_level(NeuromodulatorType::DA);
        self.ne.level = self.system.get_level(NeuromodulatorType::NE);
        self.ht.level = self.system.get_level(NeuromodulatorType::Serotonin5HT);
        self.ach.level = self.system.get_level(NeuromodulatorType::ACh);
    }

    pub fn arousal_contribution(&self) -> f64 {
        self.system.get_level(NeuromodulatorType::NE)
    }

    pub fn plasticity(&self) -> f64 {
        let ach = self.system.get_level(NeuromodulatorType::ACh);
        let da = self.system.get_level(NeuromodulatorType::DA);
        let gaba = self.system.get_level(NeuromodulatorType::GABA);
        ((ach + da) * 0.5 * (1.0 - gaba * 0.3)).clamp(0.0, 1.0)
    }
}

#[derive(Debug, Clone)]
pub struct NeuromodulatedAttention {
    pub attention_focus: Vec<f64>,
    pub modulation: ConsciousnessModulation,
}

impl NeuromodulatedAttention {
    pub fn new(modulation: ConsciousnessModulation) -> Self {
        Self {
            attention_focus: Vec::new(),
            modulation,
        }
    }

    pub fn select(&self, candidates: &[(String, f64)]) -> Vec<(String, f64)> {
        let k = ((self.modulation.attention_width * candidates.len() as f64).ceil() as usize)
            .max(1)
            .min(candidates.len());
        let mut sorted: Vec<_> = candidates.to_vec();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        sorted.into_iter().take(k).collect()
    }

    pub fn salience_boost(&self, base_salience: f64) -> f64 {
        base_salience * (1.0 + self.modulation.motivation * 0.5)
    }

    pub fn arousal_gate(&self, base_arousal: f64) -> f64 {
        base_arousal * self.modulation.arousal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        let ns = NeuromodulatorySystem::new();
        assert_eq!(ns.modulators.len(), 6);
        assert_eq!(ns.tick, 0);
        assert!((ns.get_level(NeuromodulatorType::ACh) - 0.5).abs() < 1e-6);
        assert!((ns.get_level(NeuromodulatorType::DA) - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_set_and_get() {
        let mut ns = NeuromodulatorySystem::new();
        ns.set_level(NeuromodulatorType::DA, 0.9);
        assert!((ns.get_level(NeuromodulatorType::DA) - 0.9).abs() < 1e-6);
        ns.set_level(NeuromodulatorType::DA, 1.5);
        assert!((ns.get_level(NeuromodulatorType::DA) - 1.0).abs() < 1e-6);
        ns.set_level(NeuromodulatorType::DA, -0.5);
        assert!((ns.get_level(NeuromodulatorType::DA) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_decay_reduces_level() {
        let mut ns = NeuromodulatorySystem::new();
        ns.set_level(NeuromodulatorType::NE, 0.9);
        for _ in 0..30 {
            ns.tick_decay();
        }
        let lvl = ns.get_level(NeuromodulatorType::NE);
        assert!(lvl < 0.5);
        assert!(lvl > 0.25);
    }

    #[test]
    fn test_phasic_burst() {
        let mut ns = NeuromodulatorySystem::new();
        ns.set_level(NeuromodulatorType::ACh, 0.3);
        ns.phasic_burst(NeuromodulatorType::ACh, 0.6);
        assert!((ns.get_level(NeuromodulatorType::ACh) - 0.9).abs() < 1e-6);
        ns.phasic_burst(NeuromodulatorType::ACh, 0.3);
        assert!((ns.get_level(NeuromodulatorType::ACh) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_consciousness_influence() {
        let ns = NeuromodulatorySystem::new();
        let cm = ns.consciousness_influence();
        assert!((cm.attention_width - 0.5).abs() < 1e-6);
        assert!((cm.motivation - 0.4).abs() < 1e-6);
        assert!((cm.arousal - 0.3).abs() < 1e-6);
        assert!((cm.mood_stability - 0.6).abs() < 1e-6);
        assert!((cm.wakefulness - 0.5).abs() < 1e-6);
        assert!((cm.inhibition - 0.3).abs() < 1e-6);
    }

    #[test]
    fn test_attention_select_filters() {
        let cm = ConsciousnessModulation {
            attention_width: 0.3,
            ..ConsciousnessModulation::new()
        };
        let attn = NeuromodulatedAttention::new(cm);
        let candidates = vec![
            ("a".to_string(), 0.9),
            ("b".to_string(), 0.7),
            ("c".to_string(), 0.5),
            ("d".to_string(), 0.3),
            ("e".to_string(), 0.1),
        ];
        let selected = attn.select(&candidates);
        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0].0, "a");
        assert_eq!(selected[1].0, "b");
    }

    #[test]
    fn test_salience_boost() {
        let cm = ConsciousnessModulation {
            motivation: 0.8,
            ..ConsciousnessModulation::new()
        };
        let attn = NeuromodulatedAttention::new(cm);
        let boosted = attn.salience_boost(0.5);
        assert!((boosted - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_arousal_gate() {
        let cm = ConsciousnessModulation {
            arousal: 0.0,
            ..ConsciousnessModulation::new()
        };
        let attn = NeuromodulatedAttention::new(cm);
        assert!((attn.arousal_gate(0.8) - 0.0).abs() < 1e-6);
        let cm2 = ConsciousnessModulation {
            arousal: 0.5,
            ..ConsciousnessModulation::new()
        };
        let attn2 = NeuromodulatedAttention::new(cm2);
        assert!((attn2.arousal_gate(0.8) - 0.4).abs() < 1e-6);
    }

    #[test]
    fn test_neuromodulator_new() {
        let nm = Neuromodulator::new(NeuromodulatorType::Serotonin5HT);
        assert_eq!(nm.mod_type, NeuromodulatorType::Serotonin5HT);
        assert!((nm.level - 0.6).abs() < 1e-6);
        assert!((nm.baseline - 0.6).abs() < 1e-6);
        assert!((nm.decay_rate - 0.03).abs() < 1e-6);
    }

    #[test]
    fn test_integrated_attention() {
        let mut ns = NeuromodulatorySystem::new();
        ns.set_level(NeuromodulatorType::ACh, 0.8);
        ns.set_level(NeuromodulatorType::DA, 0.9);
        ns.set_level(NeuromodulatorType::NE, 0.1);
        let cm = ns.consciousness_influence();
        let attn = NeuromodulatedAttention::new(cm);
        let candidates = vec![
            ("x".to_string(), 0.8),
            ("y".to_string(), 0.6),
            ("z".to_string(), 0.4),
        ];
        let selected = attn.select(&candidates);
        assert_eq!(selected.len(), 3);
        let boosted = attn.salience_boost(0.5);
        assert!((boosted - 0.5 * (1.0 + 0.9 * 0.5)).abs() < 1e-6);
        let gated = attn.arousal_gate(1.0);
        assert!((gated - 0.1).abs() < 1e-6);
    }
}
