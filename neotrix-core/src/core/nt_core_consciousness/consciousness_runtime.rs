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
        // Re-borrow world_item for critique
        let critique = match self.specious_present.current() {
            Some(current) => self.critic.evaluate(current, current, Some(&self.specious_present)),
            None => return None,
        };
        Some(critique)
    }
}

impl Default for ConsciousnessRuntime {
    fn default() -> Self {
        Self::new()
    }
}
