use std::collections::VecDeque;

use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

use super::vsa_tag::VsaTagged;

const DEFAULT_WINDOW_SIZE: usize = 5;
const MIN_WINDOW_SIZE: usize = 3;
const MAX_WINDOW_SIZE: usize = 10;

#[derive(Debug, Clone)]
pub struct SpeciousPresent {
    window: VecDeque<VsaTagged>,
    window_size: usize,
    coherence_trace: VecDeque<f64>,
    step_counter: u64,
}

impl Default for SpeciousPresent {
    fn default() -> Self {
        Self::new(DEFAULT_WINDOW_SIZE)
    }
}

impl SpeciousPresent {
    pub fn new(window_size: usize) -> Self {
        let size = window_size.clamp(MIN_WINDOW_SIZE, MAX_WINDOW_SIZE);
        Self {
            window: VecDeque::with_capacity(size),
            window_size: size,
            coherence_trace: VecDeque::with_capacity(size),
            step_counter: 0,
        }
    }

    pub fn push(&mut self, tagged: VsaTagged) {
        self.step_counter += 1;
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

    pub fn clear(&mut self) {
        self.window.clear();
        self.coherence_trace.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
    use crate::core::nt_core_consciousness::vsa_tag::{VsaOrigin, VsaSelfCategory};

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
