use std::collections::VecDeque;

use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

use super::vsa_tag::VsaTagged;

const DEFAULT_WINDOW_SIZE: usize = 5;
const MIN_WINDOW_SIZE: usize = 3;
const MAX_WINDOW_SIZE: usize = 10;

/// Cross-step temporal binding buffer with phase synchrony.
/// Maintains phase-locked oscillatory binding across consciousness steps.
/// Reference: ScienceDirect 2026 phenomenal binding, phase synchrony gamma binding.
#[derive(Debug, Clone)]
pub struct TemporalBindingBuffer {
    /// Phase angles for each bound content item (0-2π)
    pub phases: Vec<f64>,
    /// Binding strengths (0.0-1.0) — how strongly each item is bound
    pub binding_strengths: Vec<f64>,
    /// Phase coherence (0.0-1.0) — overall synchrony level
    pub phase_coherence: f64,
    /// Oscillation frequency (Hz, default 40 for gamma)
    pub frequency: f64,
    /// Maximum number of concurrent bindings
    pub capacity: usize,
}

impl Default for TemporalBindingBuffer {
    fn default() -> Self {
        Self {
            phases: Vec::with_capacity(8),
            binding_strengths: Vec::with_capacity(8),
            phase_coherence: 0.0,
            frequency: 40.0,
            capacity: 8,
        }
    }
}

impl TemporalBindingBuffer {
    pub fn new(frequency: f64, capacity: usize) -> Self {
        Self {
            phases: Vec::with_capacity(capacity),
            binding_strengths: Vec::with_capacity(capacity),
            phase_coherence: 0.0,
            frequency,
            capacity,
        }
    }

    /// Bind a new content item with its VSA vector hash as the phase seed.
    /// The phase is derived from the vector to ensure consistent binding.
    pub fn bind(&mut self, vector: &[u8], strength: f64) {
        let phase = self.derive_phase(vector);

        if self.phases.len() >= self.capacity {
            let weakest = self
                .binding_strengths
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.phases[weakest] = phase;
            self.binding_strengths[weakest] = strength.min(1.0);
        } else {
            self.phases.push(phase);
            self.binding_strengths.push(strength.min(1.0));
        }

        self.update_coherence();
    }

    /// Advance phases by one time step (oscillation update).
    pub fn tick(&mut self) {
        for p in self.phases.iter_mut() {
            *p = (*p + 2.0 * std::f64::consts::PI * self.frequency / 100.0)
                % (2.0 * std::f64::consts::PI);
        }
        self.update_coherence();
    }

    /// Compute phase coherence using Kuramoto order parameter R.
    fn update_coherence(&mut self) {
        if self.phases.is_empty() {
            self.phase_coherence = 0.0;
            return;
        }
        let (sum_cos, sum_sin): (f64, f64) = self
            .phases
            .iter()
            .map(|p| (p.cos(), p.sin()))
            .fold((0.0, 0.0), |(c, s), (pc, ps)| (c + pc, s + ps));
        let n = self.phases.len() as f64;
        self.phase_coherence = (sum_cos.powi(2) + sum_sin.powi(2)).sqrt() / n;
    }

    /// Whether the buffer has achieved phase synchrony (R > 0.7).
    pub fn is_synchronized(&self) -> bool {
        self.phase_coherence > 0.7 && !self.phases.is_empty()
    }

    /// Get the current mean phase (for cross-step binding reference).
    pub fn mean_phase(&self) -> f64 {
        if self.phases.is_empty() {
            return 0.0;
        }
        let (sum_cos, sum_sin): (f64, f64) = self
            .phases
            .iter()
            .map(|p| (p.cos(), p.sin()))
            .fold((0.0, 0.0), |(c, s), (pc, ps)| (c + pc, s + ps));
        sum_sin.atan2(sum_cos)
    }

    fn derive_phase(&self, vector: &[u8]) -> f64 {
        let hash: u64 = vector
            .iter()
            .fold(0u64, |acc, &b| acc.wrapping_mul(31).wrapping_add(b as u64));
        (hash as f64 / u64::MAX as f64) * 2.0 * std::f64::consts::PI
    }
}

#[derive(Debug, Clone)]
pub struct SpeciousPresent {
    window: VecDeque<VsaTagged>,
    window_size: usize,
    coherence_trace: VecDeque<f64>,
    step_counter: u64,
    /// Temporal binding buffer for cross-step phase synchrony
    pub binding_buffer: TemporalBindingBuffer,
}

impl Default for SpeciousPresent {
    fn default() -> Self {
        Self::new(DEFAULT_WINDOW_SIZE)
    }
}

/// Identity persistence scores based on Stack Theory (arXiv 2603.09043).
/// Weak persistence (p_weak): identity components appear within the current window.
/// Strong persistence (p_strong): identity components co-instantiate at a single decision point.
/// Invariant: p_strong ≤ p_weak always holds for sequential substrates.
#[derive(Debug, Clone, Copy)]
pub struct IdentityPersistenceScores {
    /// Fraction of identity components occurring within the current window (RECORD accumulation)
    pub p_weak: f64,
    /// Fraction of identity components co-instantiating at the most recent decision point (ACT)
    pub p_strong: f64,
    /// Alert signal: > 0 when p_strong < 0.5, proportional to deficit
    pub identity_fragmentation_risk: f64,
}

impl IdentityPersistenceScores {
    pub fn new(p_weak: f64, p_strong: f64) -> Self {
        let identity_fragmentation_risk = if p_strong < 0.5 {
            1.0 - (p_strong / 0.5).min(1.0)
        } else {
            0.0
        };
        Self {
            p_weak,
            p_strong,
            identity_fragmentation_risk,
        }
    }
}

impl Default for IdentityPersistenceScores {
    fn default() -> Self {
        Self {
            p_weak: 1.0,
            p_strong: 1.0,
            identity_fragmentation_risk: 0.0,
        }
    }
}

impl SpeciousPresent {
    /// Calculate identity persistence scores per Stack Theory.
    ///
    /// `self_model_present`: whether a SelfModel snapshot exists this cycle.
    /// `narrative_entry_present`: whether a NarrativeSelf entry exists this cycle.
    ///
    /// p_weak = identity components present anywhere in the RECORD window / total components.
    /// p_strong = identity components co-instantiated at the most recent ACT decision / total.
    ///
    /// Invariant: p_strong ≤ p_weak (strictly less for sequential substrates).
    pub fn calculate_persistence(
        &self,
        self_model_present: bool,
        narrative_entry_present: bool,
    ) -> IdentityPersistenceScores {
        const NUM_COMPONENTS: f64 = 3.0;

        // Identity component 1: Self-tagged VSA items in the RECORD window
        let vsa_self_in_window = self.window.iter().filter(|t| t.tag.is_self()).count();
        let vsa_self_in_window_present = vsa_self_in_window > 0;

        // p_weak: component appeared *anywhere* in the window over RECORD steps
        let weak_components = [
            self_model_present,
            narrative_entry_present,
            vsa_self_in_window_present,
        ];
        let weak_count = weak_components.iter().filter(|&&c| c).count() as f64;
        let p_weak = weak_count / NUM_COMPONENTS;

        // p_strong: components co-instantiate at the CURRENT decision point.
        // The latest window entry represents the ACT step's bound content.
        let vsa_self_in_latest = self.window.back().map(|t| t.tag.is_self()).unwrap_or(false);

        let strong_components = [
            self_model_present,
            narrative_entry_present,
            vsa_self_in_latest,
        ];
        let strong_count = strong_components.iter().filter(|&&c| c).count() as f64;
        let p_strong = strong_count / NUM_COMPONENTS;

        IdentityPersistenceScores::new(p_weak, p_strong)
    }

    /// RAG identity coherence: proportion of Self-tagged items in a retrieval result.
    /// If < 30% of retrieved items are Self-tagged, identity is fragmented by external content.
    /// Reference: Stack Theory Theorems E.1/E.2 (arXiv 2603.09043).
    pub fn check_retrieval_identity_coherence(
        &self,
        retrieved_count: usize,
        self_tagged_count: usize,
    ) -> f64 {
        if retrieved_count == 0 {
            return 1.0;
        }
        self_tagged_count as f64 / retrieved_count as f64
    }

    pub fn new(window_size: usize) -> Self {
        let size = window_size.clamp(MIN_WINDOW_SIZE, MAX_WINDOW_SIZE);
        Self {
            window: VecDeque::with_capacity(size),
            window_size: size,
            coherence_trace: VecDeque::with_capacity(size),
            step_counter: 0,
            binding_buffer: TemporalBindingBuffer::default(),
        }
    }

    pub fn push(&mut self, tagged: VsaTagged) {
        self.step_counter += 1;
        // Advance phase synchrony for cross-step binding
        self.binding_buffer.tick();
        // Bind new content with strength proportional to coherence
        let bind_strength = self.binding_buffer.phase_coherence.max(0.3);
        self.binding_buffer.bind(&tagged.vector, bind_strength);
        if self.window.len() >= self.window_size {
            self.window.pop_front();
        }
        if let Some(prev) = self.window.back() {
            let coherence = QuantizedVSA::similarity(&prev.vector, &tagged.vector);
            self.coherence_trace.push_back(coherence);
            if self.coherence_trace.len() > self.window_size {
                self.coherence_trace.pop_front();
            }
        }
        self.window.push_back(tagged);
    }

    pub fn current(&self) -> Option<&VsaTagged> {
        self.window.back()
    }

    pub fn previous(&self, steps_back: usize) -> Option<&VsaTagged> {
        if steps_back == 0 || steps_back >= self.window.len() {
            return None;
        }
        self.window.iter().nth(self.window.len() - 1 - steps_back)
    }

    pub fn temporal_integral(&self) -> Option<Vec<u8>> {
        if self.window.is_empty() {
            return None;
        }
        let refs: Vec<&[u8]> = self.window.iter().map(|t| t.vector.as_slice()).collect();
        Some(QuantizedVSA::bundle(&refs))
    }

    pub fn temporal_difference(&self) -> Option<f64> {
        if self.window.len() < 2 {
            return None;
        }
        let first = self.window.front()?;
        let last = self.window.back()?;
        Some(1.0 - QuantizedVSA::similarity(&first.vector, &last.vector))
    }

    pub fn average_coherence(&self) -> f64 {
        if self.coherence_trace.is_empty() {
            return 0.0;
        }
        self.coherence_trace.iter().sum::<f64>() / self.coherence_trace.len() as f64
    }

    pub fn is_temporally_stable(&self) -> bool {
        if self.window.len() < 2 {
            return true;
        }
        self.average_coherence() > 0.5
    }

    pub fn len(&self) -> usize {
        self.window.len()
    }

    pub fn is_empty(&self) -> bool {
        self.window.is_empty()
    }

    pub fn step_counter(&self) -> u64 {
        self.step_counter
    }

    pub fn window_size(&self) -> usize {
        self.window_size
    }

    pub fn window(&self) -> &VecDeque<VsaTagged> {
        &self.window
    }

    pub fn most_recent(&self) -> Option<&VsaTagged> {
        self.window.back()
    }

    pub fn clear(&mut self) {
        self.window.clear();
        self.coherence_trace.clear();
    }

    /// Compact the specious present window using soft/hard thresholds.
    /// When length > hard_threshold, remove oldest entries down to soft_threshold.
    /// Default thresholds: soft=512, hard=768.
    pub fn compact(&mut self, soft_threshold: usize, hard_threshold: usize) -> usize {
        let before = self.window.len();
        if before > hard_threshold {
            let remove_count = before - soft_threshold;
            for _ in 0..remove_count {
                self.window.pop_front();
            }
            // Keep coherence_trace in sync
            if self.window.len() < self.coherence_trace.len() {
                let coh_remove = self.coherence_trace.len() - self.window.len();
                for _ in 0..coh_remove {
                    self.coherence_trace.pop_front();
                }
            }
            return remove_count;
        }
        0
    }

    /// Fold adjacent entries with VSA cosine similarity > threshold.
    /// Merged pairs are replaced by their bundle vector.
    /// Returns count of folded pairs.
    pub fn compact_fold(&mut self, threshold: f64, min_keep: usize) -> usize {
        let before = self.window.len();
        if before < 4 {
            return 0;
        }
        let entries: Vec<VsaTagged> = self.window.iter().cloned().collect();
        let mut merged: Vec<VsaTagged> = Vec::with_capacity(entries.len());
        let mut i = 0;
        let mut fold_count = 0;
        while i < entries.len() {
            if i + 1 < entries.len() {
                let sim = QuantizedVSA::similarity(&entries[i].vector, &entries[i + 1].vector);
                if sim > threshold {
                    let bundled =
                        QuantizedVSA::bundle(&[&entries[i].vector, &entries[i + 1].vector]);
                    let mut merged_entry = entries[i + 1].clone();
                    merged_entry.vector = bundled;
                    merged.push(merged_entry);
                    i += 2;
                    fold_count += 1;
                    continue;
                }
            }
            merged.push(entries[i].clone());
            i += 1;
        }

        // Enforce balance: keep at least min_keep from each half
        if merged.len() < min_keep * 2 && before >= min_keep * 2 {
            // Ensure first min_keep from old half
            for j in 0..min_keep.min(before / 2) {
                let need = entries[j].clone();
                if !merged.iter().any(|e| e.vector == need.vector) {
                    merged.insert(j, need);
                }
            }
            // Ensure last min_keep from recent half
            for j in (before.saturating_sub(min_keep))..before {
                let need = entries[j].clone();
                if !merged.iter().any(|e| e.vector == need.vector) {
                    merged.push(need);
                }
            }
        }

        self.window.clear();
        for e in merged {
            self.window.push_back(e);
        }
        fold_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_consciousness::vsa_tag::{VsaOrigin, VsaSelfCategory};
    use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

    fn tagged(v: Vec<u8>) -> VsaTagged {
        VsaTagged::new(v, VsaOrigin::Self_(VsaSelfCategory::Thought))
    }

    #[test]
    fn test_new_specious_present_empty() {
        let sp = SpeciousPresent::new(5);
        assert!(sp.is_empty());
        assert_eq!(sp.window_size(), 5);
    }

    #[test]
    fn test_push_adds_element() {
        let mut sp = SpeciousPresent::new(3);
        sp.push(tagged(vec![1; 100]));
        assert_eq!(sp.len(), 1);
        assert!(sp.current().is_some());
    }

    #[test]
    fn test_window_does_not_exceed_size() {
        let mut sp = SpeciousPresent::new(3);
        for _ in 0..10 {
            sp.push(tagged(QuantizedVSA::random_binary()));
        }
        assert_eq!(sp.len(), 3);
    }

    #[test]
    fn test_previous_returns_correct() {
        let mut sp = SpeciousPresent::new(5);
        let v1 = QuantizedVSA::random_binary();
        let v2 = QuantizedVSA::random_binary();
        let v3 = QuantizedVSA::random_binary();
        sp.push(tagged(v1.clone()));
        sp.push(tagged(v2.clone()));
        sp.push(tagged(v3.clone()));
        assert_eq!(sp.previous(1).unwrap().vector, v2);
        assert_eq!(sp.previous(2).unwrap().vector, v1);
    }

    #[test]
    fn test_temporal_integral_returns_bundle() {
        let mut sp = SpeciousPresent::new(3);
        sp.push(tagged(vec![1; 100]));
        sp.push(tagged(vec![1; 100]));
        sp.push(tagged(vec![1; 100]));
        let integral = sp.temporal_integral();
        assert!(integral.is_some());
        assert_eq!(integral.unwrap().len(), 100);
    }

    #[test]
    fn test_temporal_difference() {
        let mut sp = SpeciousPresent::new(5);
        let v1 = vec![0; 100];
        let v2 = vec![1; 100];
        sp.push(tagged(v1));
        sp.push(tagged(v2));
        let diff = sp.temporal_difference();
        assert!(diff.is_some());
        assert!(diff.unwrap() > 0.5);
    }

    #[test]
    fn test_empty_integral_returns_none() {
        let sp = SpeciousPresent::new(3);
        assert!(sp.temporal_integral().is_none());
    }

    #[test]
    fn test_stability_detection() {
        let mut sp = SpeciousPresent::new(3);
        sp.push(tagged(vec![1; 100]));
        sp.push(tagged(vec![1; 100]));
        sp.push(tagged(vec![1; 100]));
        assert!(sp.is_temporally_stable());
    }

    #[test]
    fn test_clear_resets_state() {
        let mut sp = SpeciousPresent::new(3);
        sp.push(tagged(vec![1; 100]));
        sp.clear();
        assert!(sp.is_empty());
    }

    #[test]
    fn test_step_counter_increments() {
        let mut sp = SpeciousPresent::new(3);
        sp.push(tagged(vec![1; 100]));
        sp.push(tagged(vec![1; 100]));
        assert_eq!(sp.step_counter(), 2);
    }

    #[test]
    fn test_window_size_clamping() {
        let sp = SpeciousPresent::new(100);
        assert_eq!(sp.window_size(), MAX_WINDOW_SIZE);
        let sp = SpeciousPresent::new(1);
        assert_eq!(sp.window_size(), MIN_WINDOW_SIZE);
    }
}
