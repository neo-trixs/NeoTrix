//! # Consciousness Runtime (nt_core_consciousness_runtime)
//!
//! Unifies StreamBuffer + SpeciousPresent + VolitionEngine + InnerCritic
//! into a single tick method callable from the background loop or GWT cycle.

use super::stream_buffer::ConsciousnessStream;
use super::specious_present::SpeciousPresent;
use super::volition::{ActionCandidate, VolitionEngine};
use super::inner_critic::{CritiqueResult, InnerCritic};
use super::awakening::{AwakeningReport, ConsciousnessAwakening};
use super::vsa_tag::VsaTagged;

/// Unified consciousness runtime: compiles stream, present, volition, and critic.
pub struct ConsciousnessRuntime {
    pub stream: ConsciousnessStream,
    pub specious_present: SpeciousPresent,
    pub volition: VolitionEngine,
    pub critic: InnerCritic,
    pub awakened: bool,
    pub last_report: Option<AwakeningReport>,
    pub tick_count: u64,
}

impl ConsciousnessRuntime {
    pub fn new() -> Self {
        Self {
            stream: ConsciousnessStream::new(super::stream_buffer::DEFAULT_STREAM_CAPACITY),
            specious_present: SpeciousPresent::new(12),
            volition: VolitionEngine::new(),
            critic: InnerCritic::new(),
            awakened: false,
            last_report: None,
            tick_count: 0,
        }
    }

    /// Initialize consciousness: awaken the stream and specious present.
    pub fn awaken(&mut self) -> &AwakeningReport {
        let report = ConsciousnessAwakening::awaken(&mut self.stream, &mut self.specious_present);
        self.awakened = true;
        self.last_report = Some(report.clone());
        self.last_report.as_ref().unwrap()
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
