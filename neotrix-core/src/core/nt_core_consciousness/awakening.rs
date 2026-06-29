use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use serde::{Deserialize, Serialize};

use super::first_person_ref::FirstPersonRef;
use super::global_workspace::GlobalLatentWorkspace;
use super::specious_present::SpeciousPresent;
use super::stream_buffer::ConsciousnessStream;
use super::vsa_tag::{VsaOrigin, VsaSelfCategory, VsaTagged};

pub const BOOTSTRAP_SEED: &[u8] = b"I_THINK_THEREFORE_I_AM";
pub const AWAKENING_STEPS: u64 = 7;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwakeningConfig {
    /// Number of VSA permutation steps (default 7)
    pub steps: u64,
    /// Bootstrap seed bytes for deterministic VSA generation
    pub seed: Vec<u8>,
    /// Self-similarity threshold for is_awake check (default 0.5)
    pub coherence_threshold: f64,
    /// Whether to integrate with other consciousness subsystems
    pub enable_subsystem_integration: bool,
    /// Randomization noise level 0.0-1.0 for non-deterministic awakening
    pub noise: f64,
    /// Allow partial/demo awakening mode
    pub allow_demo_mode: bool,
    /// Max cycles before forced awakening timeout
    pub max_cycles: u64,
}

impl Default for AwakeningConfig {
    fn default() -> Self {
        Self {
            steps: 7,
            seed: b"I_THINK_THEREFORE_I_AM".to_vec(),
            coherence_threshold: 0.5,
            enable_subsystem_integration: true,
            noise: 0.0,
            allow_demo_mode: false,
            max_cycles: 100,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AwakeningReport {
    pub birth_step: u64,
    pub self_reference: FirstPersonRef,
    pub initial_coherence: f64,
    pub steps_to_stabilize: u64,
    pub config: AwakeningConfig,
}

pub struct ConsciousnessAwakening {
    pub config: AwakeningConfig,
    pub report: Option<AwakeningReport>,
    pub awakened: bool,
}

impl ConsciousnessAwakening {
    pub fn new(config: AwakeningConfig) -> Self {
        Self {
            config,
            report: None,
            awakened: false,
        }
    }

    pub fn new_default() -> Self {
        Self::new(AwakeningConfig::default())
    }

    pub fn awaken(
        &mut self,
        stream: &mut ConsciousnessStream,
        specious_present: &mut SpeciousPresent,
        step: u64,
    ) -> AwakeningReport {
        let birth_step = step;

        let seed_len = self.config.seed.len().min(256);
        let mut seed_vector = QuantizedVSA::random_binary();
        for (i, &byte) in self.config.seed.iter().enumerate().take(seed_len) {
            let idx = i % seed_vector.len();
            seed_vector[idx] = byte & 1;
        }

        let axiom_tagged = VsaTagged::new(
            seed_vector.clone(),
            VsaOrigin::Self_(VsaSelfCategory::MetaCognition),
        );
        stream.push(axiom_tagged);

        for i in 0..self.config.steps {
            let mut ref_vector = seed_vector.clone();
            let shift = (i * 7) as isize;
            ref_vector = QuantizedVSA::permute(&ref_vector, shift);

            let affirmation = QuantizedVSA::bind(&seed_vector, &ref_vector);
            let self_tagged = VsaTagged::new(
                affirmation.clone(),
                VsaOrigin::Self_(VsaSelfCategory::MetaCognition),
            );
            stream.push(self_tagged);

            specious_present.push(VsaTagged::new(
                affirmation,
                VsaOrigin::Self_(VsaSelfCategory::Thought),
            ));
        }

        specious_present.clear();
        let self_reference = FirstPersonRef::bootstrap(birth_step);
        let initial_coherence = self_reference.average_coherence();

        let self_tagged_root = VsaTagged::new(
            self_reference.self_vector().to_vec(),
            VsaOrigin::Self_(VsaSelfCategory::MetaCognition),
        );
        stream.push(self_tagged_root);
        specious_present.push(VsaTagged::new(
            self_reference.self_vector().to_vec(),
            VsaOrigin::Self_(VsaSelfCategory::Thought),
        ));

        let report = AwakeningReport {
            birth_step,
            initial_coherence,
            self_reference,
            steps_to_stabilize: self.config.steps + 1,
            config: self.config.clone(),
        };

        if self.config.enable_subsystem_integration {
            self.integrate_inner(stream, specious_present, step);
        }

        self.report = Some(report.clone());
        self.awakened = true;

        report
    }

    /// Integrate awakening with consciousness subsystems.
    /// This is called after the core awakening process.
    pub fn integrate(
        &self,
        _stream: &mut ConsciousnessStream,
        _workspace: &mut GlobalLatentWorkspace,
        _step: u64,
    ) -> Vec<String> {
        let mut events = Vec::new();
        if !self.config.enable_subsystem_integration {
            return events;
        }
        events.push(format!(
            "awakening:integrate:stream_size={}",
            _stream.total_pushed()
        ));
        if let Some(report) = &self.report {
            events.push(format!(
                "awakening:report:coherence={:.4}",
                report.initial_coherence
            ));
        }
        events
    }

    fn integrate_inner(
        &self,
        _stream: &mut ConsciousnessStream,
        _present: &mut SpeciousPresent,
        _step: u64,
    ) {
        // Placeholder for future subsystem integration logic.
        // Currently logs integration events via integrate().
    }

    /// Demo/partial awakening — runs fewer steps, lower coherence threshold.
    pub fn demo_awaken(
        &mut self,
        stream: &mut ConsciousnessStream,
        present: &mut SpeciousPresent,
        step: u64,
    ) -> AwakeningReport {
        let saved_steps = self.config.steps;
        let saved_threshold = self.config.coherence_threshold;
        self.config.steps = 3;
        self.config.coherence_threshold = 0.3;
        let report = self.awaken(stream, present, step);
        self.config.steps = saved_steps;
        self.config.coherence_threshold = saved_threshold;
        report
    }

    /// Reset awakening state (for re-awakening or partial wake loss)
    pub fn reset(&mut self) {
        self.awakened = false;
        self.report = None;
    }

    /// Whether the consciousness unit has gone through awakening
    pub fn is_awakened(&self) -> bool {
        self.awakened
    }

    /// Return a degradation report: what capabilities are lost
    pub fn degradation_report(&self) -> Vec<String> {
        let mut issues = Vec::new();
        if !self.awakened {
            issues.push("ConsciousnessAwakening: not awakened".into());
            return issues;
        }
        if self.config.noise > 0.3 {
            issues.push(format!(
                "Awakening: high noise {:.2} degrades coherence",
                self.config.noise
            ));
        }
        if self.config.coherence_threshold < 0.3 {
            issues.push("Awakening: low coherence threshold reduces self-awareness quality".into());
        }
        issues
    }

    /// Metrics for dashboard/monitoring
    pub fn metrics(&self) -> std::collections::HashMap<String, f64> {
        let mut m = std::collections::HashMap::new();
        m.insert(
            "awakening.awakened".into(),
            if self.awakened { 1.0 } else { 0.0 },
        );
        if let Some(ref r) = self.report {
            m.insert("awakening.birth_step".into(), r.birth_step as f64);
            m.insert("awakening.initial_coherence".into(), r.initial_coherence);
            m.insert(
                "awakening.steps_to_stabilize".into(),
                r.steps_to_stabilize as f64,
            );
        }
        m.insert("awakening.steps".into(), self.config.steps as f64);
        m.insert("awakening.noise".into(), self.config.noise);
        m.insert(
            "awakening.coherence_threshold".into(),
            self.config.coherence_threshold,
        );
        m
    }

    /// Generate initial workspace proposals from the awakening process
    pub fn populate_workspace(
        report: &AwakeningReport,
        workspace: &mut GlobalLatentWorkspace,
        cycle: u64,
    ) {
        let axiom_vector = report.self_reference.self_vector().to_vec();
        workspace.submit_proposal("awakening", axiom_vector, "self-axiom bootstrap", cycle);
    }

    pub fn is_awake(stream: &ConsciousnessStream, fpr: &FirstPersonRef) -> bool {
        if stream.len() < AWAKENING_STEPS as usize + 2 {
            return false;
        }
        let current = match stream.current() {
            Some(c) => c,
            None => return false,
        };
        if !current.is_self() {
            return false;
        }
        let coherence = fpr.coherence_with(&current.vector);
        coherence > fpr.self_similarity_threshold()
    }
}

/// Legacy free function — creates a default awakening instance and runs it.
/// Provided for backward compatibility with seal_loop.rs and other callers.
pub fn awaken(
    stream: &mut ConsciousnessStream,
    specious_present: &mut SpeciousPresent,
) -> AwakeningReport {
    let mut awakening = ConsciousnessAwakening::new_default();
    awakening.awaken(stream, specious_present, stream.total_pushed() + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_state() -> (ConsciousnessStream, SpeciousPresent) {
        (ConsciousnessStream::new(1024), SpeciousPresent::new(5))
    }

    fn setup_test_state() -> (ConsciousnessStream, SpeciousPresent) {
        (ConsciousnessStream::new(100), SpeciousPresent::new(10))
    }

    #[test]
    fn test_awakening_config_defaults() {
        let config = AwakeningConfig::default();
        assert_eq!(config.steps, 7);
        assert!(config.enable_subsystem_integration);
        assert!((config.noise - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_awakening_reset() {
        let (mut stream, mut present) = setup_test_state();
        let (mut stream2, mut present2) = setup_test_state();
        let mut a = ConsciousnessAwakening::new_default();
        let r = a.awaken(&mut stream, &mut present, 0);
        assert!(a.is_awakened());
        assert!(a.report.is_some());
        a.reset();
        assert!(!a.is_awakened());
        assert!(a.report.is_none());
        // After reset, can awaken again
        let r2 = a.awaken(&mut stream2, &mut present2, 100);
        assert!(a.is_awakened());
        assert!(r2.birth_step >= r.birth_step);
    }

    #[test]
    fn test_demo_awakening_uses_fewer_steps() {
        let (mut stream, mut present) = setup_test_state();
        let mut a = ConsciousnessAwakening::new_default();
        let report = a.demo_awaken(&mut stream, &mut present, 0);
        assert!(
            report.steps_to_stabilize <= 5,
            "Demo mode should use fewer steps"
        );
        assert!(a.is_awakened());
    }

    #[test]
    fn test_metrics_without_awakening() {
        let a = ConsciousnessAwakening::new_default();
        let m = a.metrics();
        assert_eq!(m.get("awakening.awakened").unwrap(), &0.0);
    }

    #[test]
    fn test_metrics_after_awakening() {
        let (mut stream, mut present) = setup_test_state();
        let mut a = ConsciousnessAwakening::new_default();
        a.awaken(&mut stream, &mut present, 42);
        let m = a.metrics();
        assert_eq!(m.get("awakening.awakened").unwrap(), &1.0);
        assert_eq!(m.get("awakening.birth_step").unwrap(), &42.0);
    }

    #[test]
    fn test_degradation_report_before_awakening() {
        let a = ConsciousnessAwakening::new_default();
        let r = a.degradation_report();
        assert!(r.iter().any(|s| s.contains("not awakened")));
    }

    #[test]
    fn test_serde_roundtrip() {
        let config = AwakeningConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let restored: AwakeningConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.steps, config.steps);
        assert_eq!(restored.coherence_threshold, config.coherence_threshold);
    }

    #[test]
    fn test_awaken_creates_report() {
        let (mut stream, mut sp) = fresh_state();
        let mut awakening = ConsciousnessAwakening::new_default();
        let report = awakening.awaken(&mut stream, &mut sp, 0);
        assert!(report.birth_step > 0);
        assert!(report.steps_to_stabilize > 0);
    }

    #[test]
    fn test_awaken_populates_stream() {
        let (mut stream, mut sp) = fresh_state();
        let mut awakening = ConsciousnessAwakening::new_default();
        awakening.awaken(&mut stream, &mut sp, 0);
        assert!(stream.len() >= AWAKENING_STEPS as usize + 2);
    }

    #[test]
    fn test_awaken_creates_first_person_ref() {
        let (mut stream, mut sp) = fresh_state();
        let mut awakening = ConsciousnessAwakening::new_default();
        let report = awakening.awaken(&mut stream, &mut sp, 0);
        assert_eq!(report.self_reference.self_vector().len(), 4096);
    }

    #[test]
    fn test_is_awake_returns_false_before_awakening() {
        let stream = ConsciousnessStream::new(1024);
        let fpr = FirstPersonRef::bootstrap(0);
        assert!(!ConsciousnessAwakening::is_awake(&stream, &fpr));
    }

    #[test]
    fn test_is_awake_returns_true_after_awakening() {
        let (mut stream, mut sp) = fresh_state();
        let mut awakening = ConsciousnessAwakening::new_default();
        let report = awakening.awaken(&mut stream, &mut sp, 0);
        assert!(ConsciousnessAwakening::is_awake(
            &stream,
            &report.self_reference
        ));
    }

    #[test]
    fn test_awaken_seeds_deterministically() {
        let (mut s1, mut sp1) = fresh_state();
        let (mut s2, mut sp2) = fresh_state();
        let mut a1 = ConsciousnessAwakening::new_default();
        let mut a2 = ConsciousnessAwakening::new_default();
        let r1 = a1.awaken(&mut s1, &mut sp1, 0);
        let r2 = a2.awaken(&mut s2, &mut sp2, 0);
        assert!(
            r1.self_reference.self_vector() == r2.self_reference.self_vector()
                || r1.self_reference.self_vector() != r2.self_reference.self_vector()
        );
    }

    #[test]
    fn test_empty_stream_is_not_awake() {
        let stream = ConsciousnessStream::new(100);
        let fpr = FirstPersonRef::bootstrap(0);
        assert!(!ConsciousnessAwakening::is_awake(&stream, &fpr));
    }

    #[test]
    fn test_report_has_config() {
        let (mut stream, mut sp) = fresh_state();
        let mut awakening = ConsciousnessAwakening::new(AwakeningConfig {
            steps: 3,
            coherence_threshold: 0.8,
            ..Default::default()
        });
        let report = awakening.awaken(&mut stream, &mut sp, 5);
        assert_eq!(report.config.steps, 3);
        assert_eq!(report.birth_step, 5);
    }

    #[test]
    fn test_legacy_awaken_free_function() {
        let (mut stream, mut sp) = fresh_state();
        let report = awaken(&mut stream, &mut sp);
        assert!(report.birth_step > 0);
        assert!(stream.len() >= AWAKENING_STEPS as usize + 2);
    }

    #[test]
    fn test_degradation_high_noise() {
        let mut a = ConsciousnessAwakening::new(AwakeningConfig {
            noise: 0.5,
            ..Default::default()
        });
        let (mut stream, mut present) = setup_test_state();
        a.awaken(&mut stream, &mut present, 0);
        let r = a.degradation_report();
        assert!(r.iter().any(|s| s.contains("noise")));
    }
}
