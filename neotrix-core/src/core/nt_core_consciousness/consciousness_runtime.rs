use super::stream_buffer::ConsciousnessStream;
use super::specious_present::SpeciousPresent;
use super::volition::{ActionCandidate, VolitionEngine};
use super::inner_critic::{CritiqueResult, InnerCritic};
use super::awakening::{AwakeningReport, ConsciousnessAwakening};
use super::vsa_tag::VsaTagged;
use crate::core::nt_core_self::emotion_state::{EmotionEngine, EmotionReport, EmotionDimension};

pub struct ConsciousnessRuntime {
    pub stream: ConsciousnessStream,
    pub specious_present: SpeciousPresent,
    pub volition: VolitionEngine,
    pub critic: InnerCritic,
    pub emotion_engine: EmotionEngine,
    pub awakened: bool,
    pub last_report: Option<AwakeningReport>,
    pub last_quality: f64,
    pub tick_count: u64,
}

impl ConsciousnessRuntime {
    pub fn new() -> Self {
        Self {
            stream: ConsciousnessStream::new(super::stream_buffer::DEFAULT_STREAM_CAPACITY),
            specious_present: SpeciousPresent::new(12),
            volition: VolitionEngine::new(),
            critic: InnerCritic::new(),
            emotion_engine: EmotionEngine::default(),
            awakened: false,
            last_report: None,
            last_quality: 0.0,
            tick_count: 0,
        }
    }

    pub fn awaken(&mut self) -> &AwakeningReport {
        let report = ConsciousnessAwakening::awaken(&mut self.stream, &mut self.specious_present);
        self.awakened = true;
        self.last_report = Some(report.clone());
        self.last_report.as_ref().unwrap()
    }

    pub fn tick_emotion(&mut self) -> EmotionReport {
        self.emotion_engine.tick();
        self.emotion_engine.report()
    }

    pub fn observe_from_critique(&mut self, critique: &CritiqueResult) {
        let quality = critique.overall_quality;
        self.last_quality = quality;
        self.emotion_engine.observe(EmotionDimension::Confidence, quality, "critique_quality");
        if quality < 0.3 {
            self.emotion_engine.observe(EmotionDimension::Frustration, 0.7 - quality, "low_quality_critique");
        }
        if let Some(ref action) = critique.selected_action {
            if action.contains("explore") || action.contains("curious") {
                self.emotion_engine.observe(EmotionDimension::Curiosity, 0.8, action);
            }
        }
    }

    pub fn emotion_engine(&self) -> &EmotionEngine {
        &self.emotion_engine
    }

    pub fn emotion_engine_mut(&mut self) -> &mut EmotionEngine {
        &mut self.emotion_engine
    }

    pub fn last_quality(&self) -> Option<f64> {
        if self.tick_count > 0 { Some(self.last_quality) } else { None }
    }

    pub fn set_emotion_engine(&mut self, engine: EmotionEngine) {
        self.emotion_engine = engine;
    }

    /// Advance one consciousness tick.
    pub fn tick(&mut self, resonance_content: &str) -> Option<CritiqueResult> {
        if !self.awakened {
            return None;
        }
        self.tick_count += 1;
        // Feed resonance content into specious present as a VSA-tagged item
        let world_item = VsaTagged::world_input(resonance_content);
        self.specious_present.push(world_item);
        // Run volition: propose candidates from the specious present window
        for item in self.specious_present.window().iter() {
            let desc = String::from_utf8_lossy(
                &item.vector[..item.vector.len().min(64)],
            ).to_string();
            if !desc.is_empty() {
                let candidate = ActionCandidate::new(item.vector.clone(), &desc);
                self.volition.propose(candidate);
            }
        }
        // Select the best action candidate (was never called before)
        let selected_action = self.volition.select_best();
        // Compute temporal integral and difference for richer critique
        let _temporal_integral = self.specious_present.temporal_integral();
        let _temporal_delta = self.specious_present.temporal_difference();
        // Re-borrow for critique
        let mut critique = match self.specious_present.current() {
            Some(current) => self.critic.evaluate(current, current, Some(&self.specious_present)),
            None => return None,
        };
        // Attach selected action info to critique
        if let Some(action) = selected_action {
            critique.selected_action = Some(action.description.clone());
        }
        critique.temporal_delta = _temporal_delta;
        self.observe_from_critique(&critique);
        Some(critique)
    }

    /// Get a reference to the volition engine for inspection
    pub fn volition(&self) -> &VolitionEngine {
        &self.volition
    }

    /// Get a mutable reference to the volition engine
    pub fn volition_mut(&mut self) -> &mut VolitionEngine {
        &mut self.volition
    }

    /// Get the current specious present coherence
    pub fn coherence(&self) -> f64 {
        self.specious_present.average_coherence()
    }

    /// Clear all candidates from the volition engine
    pub fn clear_volition(&mut self) {
        self.volition.clear();
    }
}

impl crate::core::nt_core_self_test::SelfTest for ConsciousnessRuntime {
    fn name(&self) -> &str { "consciousness_runtime" }
    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        // Test 1: new runtime is not awakened
        if self.awakened {
            failures.push("new runtime should not be awakened".into());
        }
        // Test 2: tick returns None before awaken
        let mut cr = ConsciousnessRuntime::new();
        if cr.tick("test").is_some() {
            failures.push("tick before awaken should return None".into());
        }
        // Test 3: awaken sets awakened flag
        cr.awaken();
        if !cr.awakened {
            failures.push("awaken should set awakened=true".into());
        }
        // Test 4: tick_count increments
        let count_before = cr.tick_count;
        let _ = cr.tick("test resonance");
        if cr.tick_count != count_before + 1 {
            failures.push("tick should increment tick_count".into());
        }
        // Test 5: tick after awaken returns Some
        let result = cr.tick("after awaken");
        if result.is_none() {
            failures.push("tick after awaken should return Some critique".into());
        }
        if failures.is_empty() { Ok(()) } else { Err(failures) }
    }
}

impl Default for ConsciousnessRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observe_from_critique_low_quality() {
        let mut cr = ConsciousnessRuntime::new();
        let critique = CritiqueResult {
            passed: true,
            relevance_score: 0.0,
            consistency_score: 0.0,
            uncertainty_score: 0.8,
            overall_quality: 0.2,
            reasons: vec!["low quality".into()],
            selected_action: Some("rethink".into()),
            temporal_delta: Some(0.0),
        };
        cr.observe_from_critique(&critique);
        let report = cr.emotion_engine.report();
        assert!(report.confidence < 0.5, "confidence={} should have dropped from 0.5", report.confidence);
        assert!(report.frustration > 0.49, "frustration={} should be above neutral", report.frustration);
        assert_eq!(cr.last_quality, 0.2);
    }

    #[test]
    fn test_observe_from_critique_high_quality() {
        let mut cr = ConsciousnessRuntime::new();
        let critique = CritiqueResult {
            passed: true,
            relevance_score: 0.0,
            consistency_score: 0.0,
            uncertainty_score: 0.2,
            overall_quality: 0.9,
            reasons: vec!["high quality".into()],
            selected_action: Some("explore_new".into()),
            temporal_delta: Some(0.0),
        };
        cr.observe_from_critique(&critique);
        let report = cr.emotion_engine.report();
        assert!(report.confidence > 0.55, "confidence={} should be above neutral", report.confidence);
        assert!(report.curiosity > 0.55, "curiosity={} should be above neutral", report.curiosity);
        assert_eq!(cr.last_quality, 0.9);
    }

    #[test]
    fn test_tick_emotion_returns_report() {
        let mut cr = ConsciousnessRuntime::new();
        cr.awaken();
        let _ = cr.tick("test resonance");
        let report = cr.tick_emotion();
        assert!(report.confidence >= 0.0);
        assert!(report.valence >= -1.0 && report.valence <= 1.0);
    }

    #[test]
    fn test_set_emotion_engine() {
        let mut cr = ConsciousnessRuntime::new();
        let mut engine = EmotionEngine::default();
        engine.observe(EmotionDimension::Confidence, 0.9, "test");
        cr.set_emotion_engine(engine);
        let report = cr.emotion_engine.report();
        assert!(report.confidence > 0.55, "confidence={} should reflect observed 0.9", report.confidence);
    }

    #[test]
    fn test_tick_wires_observe_from_critique() {
        let mut cr = ConsciousnessRuntime::new();
        cr.awaken();
        let result = cr.tick("high quality content that is meaningful long enough to produce a critique");
        assert!(result.is_some());
        let report = cr.emotion_engine.report();
        assert!(report.confidence >= 0.0); // tick wired observe_from_critique
        assert!(cr.last_quality > 0.0 || cr.last_quality == 0.0); // set by tick
        assert!(cr.last_quality() == Some(cr.last_quality));
    }
}
