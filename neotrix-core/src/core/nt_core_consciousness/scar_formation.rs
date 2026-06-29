// REVIVED Task 1 — dead_code removed 2026-06-24

use std::collections::VecDeque;

/// Severity of a scar event
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScarSeverity {
    Minor,
    Significant,
    Critical,
}

/// A persistent scar — critical event that left a durable mark
#[derive(Debug, Clone)]
pub struct Scar {
    pub id: u64,
    pub description: String,
    pub severity: ScarSeverity,
    pub tick: u64,
    pub impact: f64,
    pub healed: bool,
}

/// Scar formation system — critical events leave persistent markers
#[derive(Debug, Clone)]
pub struct ScarFormation {
    pub scars: VecDeque<Scar>,
    pub max_scars: usize,
    pub next_id: u64,
    pub healing_rate: f64,
}

impl ScarFormation {
    pub fn new() -> Self {
        ScarFormation {
            scars: VecDeque::with_capacity(32),
            max_scars: 100,
            next_id: 0,
            healing_rate: 0.001,
        }
    }

    pub fn form_scar(
        &mut self,
        description: &str,
        severity: ScarSeverity,
        impact: f64,
        tick: u64,
    ) -> Scar {
        if self.scars.len() >= self.max_scars {
            self.scars.pop_front();
        }
        let scar = Scar {
            id: self.next_id,
            description: description.into(),
            severity,
            tick,
            impact: impact.clamp(0.0, 1.0),
            healed: false,
        };
        self.next_id += 1;
        self.scars.push_back(scar.clone());
        scar
    }

    pub fn heal_scar(&mut self, id: u64) {
        if let Some(scar) = self.scars.iter_mut().find(|s| s.id == id) {
            scar.healed = true;
        }
    }

    pub fn tick_healing(&mut self) {
        for scar in self.scars.iter_mut() {
            if !scar.healed {
                scar.impact = (scar.impact - self.healing_rate).max(0.0);
                if scar.impact <= 0.0 {
                    scar.healed = true;
                }
            }
        }
    }

    pub fn active_scars(&self) -> Vec<&Scar> {
        self.scars.iter().filter(|s| !s.healed).collect()
    }

    pub fn critical_scars(&self) -> Vec<&Scar> {
        self.scars
            .iter()
            .filter(|s| !s.healed && s.severity == ScarSeverity::Critical)
            .collect()
    }

    pub fn total_impact(&self) -> f64 {
        self.scars
            .iter()
            .filter(|s| !s.healed)
            .map(|s| s.impact)
            .sum()
    }

    pub fn scar_count(&self) -> usize {
        self.scars.len()
    }

    pub fn report(&self) -> String {
        let active = self.active_scars().len();
        let critical = self.critical_scars().len();
        let total_impact = self.total_impact();
        format!(
            "ScarFormation: {} total, {} active, {} critical, total_impact={:.2}",
            self.scars.len(),
            active,
            critical,
            total_impact,
        )
    }
}

/// Context surrounding a scar — what triggered it, what it affects
#[derive(Debug, Clone)]
pub struct ScarContext {
    pub scar_id: u64,
    pub vsa_vector: Vec<f64>,
    pub associated_memories: Vec<String>,
    pub behavior_impact: f64,
}

/// A scar that actively shapes future behavior
#[derive(Debug, Clone)]
pub struct LearningScar {
    pub scar: Scar,
    pub context: ScarContext,
    pub avoidance_strength: f64,
    pub generalization: f64,
}

/// Scar-based behavior modification system
#[derive(Clone)]
pub struct ScarGuidedLearning {
    pub learning_scars: Vec<LearningScar>,
    pub max_scars: usize,
    pub formation: ScarFormation,
}

impl ScarGuidedLearning {
    pub fn new(max_scars: usize) -> Self {
        ScarGuidedLearning {
            learning_scars: Vec::with_capacity(max_scars.min(100)),
            max_scars,
            formation: ScarFormation::new(),
        }
    }

    pub fn form_learning_scar(
        &mut self,
        description: &str,
        severity: ScarSeverity,
        vsa: Vec<f64>,
        impact: f64,
        tick: u64,
    ) -> u64 {
        let scar = self
            .formation
            .form_scar(description, severity, impact, tick);
        let learning = LearningScar {
            context: ScarContext {
                scar_id: scar.id,
                vsa_vector: vsa,
                associated_memories: Vec::new(),
                behavior_impact: impact,
            },
            avoidance_strength: impact * 0.8,
            generalization: 0.3,
            scar,
        };
        if self.learning_scars.len() >= self.max_scars {
            self.learning_scars.remove(0);
        }
        let id = learning.context.scar_id;
        self.learning_scars.push(learning);
        id
    }

    pub fn associate_memory(&mut self, scar_id: u64, memory: &str) {
        if let Some(ls) = self
            .learning_scars
            .iter_mut()
            .find(|ls| ls.context.scar_id == scar_id)
        {
            if !ls.context.associated_memories.contains(&memory.to_string()) {
                ls.context.associated_memories.push(memory.to_string());
            }
        }
    }

    pub fn avoidance_signal(&self, current_vsa: &[f64]) -> f64 {
        let mut total = 0.0;
        for ls in &self.learning_scars {
            if !ls.scar.healed {
                let sim = self.cosine_similarity(current_vsa, &ls.context.vsa_vector);
                total += sim * ls.avoidance_strength;
            }
        }
        total
    }

    pub fn generalize_scar(&mut self, scar_id: u64, new_vsa: Vec<f64>) {
        if let Some(ls) = self
            .learning_scars
            .iter_mut()
            .find(|ls| ls.context.scar_id == scar_id)
        {
            let old = &ls.context.vsa_vector;
            let g = ls.generalization;
            let blended: Vec<f64> = old
                .iter()
                .zip(new_vsa.iter())
                .map(|(a, b)| a * (1.0 - g) + b * g)
                .collect();
            ls.context.vsa_vector = blended;
            ls.generalization = (ls.generalization + 0.1).min(1.0);
        }
    }

    pub fn healed_but_influential(&self) -> Vec<&LearningScar> {
        self.learning_scars
            .iter()
            .filter(|ls| ls.scar.healed && ls.context.behavior_impact > 0.3)
            .collect()
    }

    pub fn behavior_change(&self) -> f64 {
        self.learning_scars
            .iter()
            .filter(|ls| !ls.scar.healed)
            .map(|ls| ls.context.behavior_impact)
            .sum()
    }

    fn cosine_similarity(&self, a: &[f64], b: &[f64]) -> f64 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }
        let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let na: f64 = a.iter().map(|x| x * x).sum();
        let nb: f64 = b.iter().map(|x| x * x).sum();
        let denom = na.sqrt() * nb.sqrt();
        if denom == 0.0 {
            0.0
        } else {
            (dot / denom).clamp(0.0, 1.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_scar_creates_entry() {
        let mut sf = ScarFormation::new();
        sf.form_scar("critical failure", ScarSeverity::Critical, 0.9, 0);
        assert_eq!(sf.scar_count(), 1);
    }

    #[test]
    fn test_heal_scar_marks_healed() {
        let mut sf = ScarFormation::new();
        let scar = sf.form_scar("test", ScarSeverity::Minor, 0.5, 0);
        sf.heal_scar(scar.id);
        assert!(sf.scars[0].healed);
    }

    #[test]
    fn test_active_scars_excludes_healed() {
        let mut sf = ScarFormation::new();
        let s = sf.form_scar("test", ScarSeverity::Minor, 0.5, 0);
        assert_eq!(sf.active_scars().len(), 1);
        sf.heal_scar(s.id);
        assert_eq!(sf.active_scars().len(), 0);
    }

    #[test]
    fn test_critical_scars_filter() {
        let mut sf = ScarFormation::new();
        sf.form_scar("minor", ScarSeverity::Minor, 0.3, 0);
        sf.form_scar("critical", ScarSeverity::Critical, 0.9, 0);
        assert_eq!(sf.critical_scars().len(), 1);
    }

    #[test]
    fn test_tick_healing_reduces_impact() {
        let mut sf = ScarFormation::new();
        sf.healing_rate = 0.5;
        sf.form_scar("test", ScarSeverity::Significant, 1.0, 0);
        sf.tick_healing();
        assert!(sf.scars[0].impact < 1.0);
    }

    #[test]
    fn test_total_impact() {
        let mut sf = ScarFormation::new();
        sf.form_scar("s1", ScarSeverity::Minor, 0.3, 0);
        sf.form_scar("s2", ScarSeverity::Significant, 0.6, 0);
        assert!((sf.total_impact() - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_report() {
        let sf = ScarFormation::new();
        let r = sf.report();
        assert!(r.contains("ScarFormation"));
    }

    #[test]
    fn test_form_learning_scar() {
        let mut sgl = ScarGuidedLearning::new(10);
        let id = sgl.form_learning_scar("trauma", ScarSeverity::Critical, vec![1.0, 0.0], 0.9, 1);
        assert_eq!(sgl.learning_scars.len(), 1);
        assert_eq!(sgl.learning_scars[0].context.scar_id, id);
        assert!((sgl.learning_scars[0].avoidance_strength - 0.72).abs() < 0.01);
    }

    #[test]
    fn test_associate_memory() {
        let mut sgl = ScarGuidedLearning::new(10);
        let id = sgl.form_learning_scar("test", ScarSeverity::Minor, vec![0.5, 0.5], 0.3, 0);
        sgl.associate_memory(id, "dark room");
        assert_eq!(sgl.learning_scars[0].context.associated_memories.len(), 1);
        assert_eq!(
            sgl.learning_scars[0].context.associated_memories[0],
            "dark room"
        );
    }

    #[test]
    fn test_avoidance_signal_high_when_similar() {
        let mut sgl = ScarGuidedLearning::new(10);
        sgl.form_learning_scar("pain", ScarSeverity::Critical, vec![1.0, 0.0, 0.0], 1.0, 0);
        let signal = sgl.avoidance_signal(&[1.0, 0.0, 0.0]);
        assert!(signal > 0.5);
    }

    #[test]
    fn test_avoidance_signal_low_when_different() {
        let mut sgl = ScarGuidedLearning::new(10);
        sgl.form_learning_scar("pain", ScarSeverity::Critical, vec![1.0, 0.0, 0.0], 1.0, 0);
        let signal = sgl.avoidance_signal(&[0.0, 1.0, 0.0]);
        assert!(signal < 0.01);
    }

    #[test]
    fn test_generalize_scar() {
        let mut sgl = ScarGuidedLearning::new(10);
        let id =
            sgl.form_learning_scar("burn", ScarSeverity::Critical, vec![1.0, 0.0, 0.0], 0.8, 0);
        assert!((sgl.learning_scars[0].generalization - 0.3).abs() < 0.01);
        sgl.generalize_scar(id, vec![0.0, 1.0, 0.0]);
        assert!(sgl.learning_scars[0].generalization > 0.3);
        assert!(sgl.learning_scars[0].context.vsa_vector[0] < 1.0);
    }

    #[test]
    fn test_healed_but_influential() {
        let mut sgl = ScarGuidedLearning::new(10);
        let id = sgl.form_learning_scar("past", ScarSeverity::Significant, vec![0.5; 4], 0.5, 0);
        sgl.formation.heal_scar(id);
        let healed = sgl.healed_but_influential();
        assert_eq!(healed.len(), 1);
        assert_eq!(healed[0].scar.id, id);
    }

    #[test]
    fn test_behavior_change() {
        let mut sgl = ScarGuidedLearning::new(10);
        sgl.form_learning_scar("a", ScarSeverity::Minor, vec![0.0; 4], 0.3, 0);
        sgl.form_learning_scar("b", ScarSeverity::Significant, vec![0.0; 4], 0.6, 0);
        let bc = sgl.behavior_change();
        assert!((bc - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_cosine_similarity() {
        let sgl = ScarGuidedLearning::new(10);
        assert!((sgl.cosine_similarity(&[1.0, 0.0], &[1.0, 0.0]) - 1.0).abs() < 0.01);
        assert!((sgl.cosine_similarity(&[1.0, 0.0], &[0.0, 1.0]) - 0.0).abs() < 0.01);
        assert!((sgl.cosine_similarity(&[], &[]) - 0.0).abs() < 0.01);
    }
}
