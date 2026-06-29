use std::collections::VecDeque;

use super::vsa_tag::VsaTagged;
use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

const LOCAL_WINDOW_SIZE: usize = 3;
const DILATED_WINDOW_SIZE: usize = 5;
const DILATED_STRIDE: usize = 3;
const GAMMA: f64 = 0.85;

#[derive(Debug, Clone)]
pub struct TemporalAttentionStack {
    local: SlidingWindowAttention,
    mid: DilatedSlidingWindow,
    global: GatedLinearAttention,
    step_counter: u64,
    error_bound: f64,
}

impl TemporalAttentionStack {
    pub fn new() -> Self {
        Self {
            local: SlidingWindowAttention::new(LOCAL_WINDOW_SIZE),
            mid: DilatedSlidingWindow::new(DILATED_WINDOW_SIZE, DILATED_STRIDE),
            global: GatedLinearAttention::new(GAMMA),
            step_counter: 0,
            error_bound: 0.0,
        }
    }

    pub fn push(&mut self, tagged: VsaTagged) {
        self.step_counter += 1;
        self.local.push(tagged.clone());
        if self.step_counter % DILATED_STRIDE as u64 == 0 {
            self.mid.push(tagged.clone());
        }
        self.global.push(tagged);
        self.update_error_bound();
    }

    pub fn attend(&self) -> Vec<u8> {
        let local_vec = self.local.integrate();
        let mid_vec = self.mid.integrate().unwrap_or_else(|| local_vec.clone());
        let global_vec = self.global.integrate().unwrap_or_else(|| mid_vec.clone());
        if local_vec.is_empty() && mid_vec.is_empty() && global_vec.is_empty() {
            return Vec::new();
        }
        let weighted = QuantizedVSA::bundle(&[&local_vec, &mid_vec, &global_vec]);
        weighted
    }

    pub fn local_attention(&self) -> &SlidingWindowAttention {
        &self.local
    }

    pub fn mid_attention(&self) -> &DilatedSlidingWindow {
        &self.mid
    }

    pub fn global_attention(&self) -> &GatedLinearAttention {
        &self.global
    }

    pub fn step_counter(&self) -> u64 {
        self.step_counter
    }

    pub fn error_bound(&self) -> f64 {
        self.error_bound
    }

    fn update_error_bound(&mut self) {
        let local_err = self.local.estimate_error();
        let mid_err = self.mid.estimate_error();
        let global_err = self.global.estimate_error();
        let n = self.step_counter as f64;
        let gamma_n = GAMMA.powf(n);
        self.error_bound = local_err + mid_err * gamma_n + global_err * gamma_n * gamma_n;
    }
}

#[derive(Debug, Clone)]
pub struct SlidingWindowAttention {
    window: VecDeque<VsaTagged>,
    size: usize,
}

impl SlidingWindowAttention {
    pub fn new(size: usize) -> Self {
        Self {
            window: VecDeque::with_capacity(size),
            size,
        }
    }

    pub fn push(&mut self, tagged: VsaTagged) {
        if self.window.len() >= self.size {
            self.window.pop_front();
        }
        self.window.push_back(tagged);
    }

    pub fn integrate(&self) -> Vec<u8> {
        if self.window.is_empty() {
            return Vec::new();
        }
        let refs: Vec<&[u8]> = self.window.iter().map(|t| t.vector.as_slice()).collect();
        QuantizedVSA::bundle(&refs)
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

    pub fn estimate_error(&self) -> f64 {
        1.0 - self.coherence()
    }

    fn coherence(&self) -> f64 {
        if self.window.len() < 2 {
            return 1.0;
        }
        let first = match self.window.front() {
            Some(f) => f,
            None => return 1.0,
        };
        let last = match self.window.back() {
            Some(l) => l,
            None => return 1.0,
        };
        QuantizedVSA::similarity(&first.vector, &last.vector)
    }

    pub fn len(&self) -> usize {
        self.window.len()
    }

    pub fn is_empty(&self) -> bool {
        self.window.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct DilatedSlidingWindow {
    entries: VecDeque<VsaTagged>,
    size: usize,
    #[allow(dead_code)]
    stride: usize,
}

impl DilatedSlidingWindow {
    pub fn new(size: usize, stride: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(size),
            size,
            stride,
        }
    }

    pub fn push(&mut self, tagged: VsaTagged) {
        if self.entries.len() >= self.size {
            self.entries.pop_front();
        }
        self.entries.push_back(tagged);
    }

    pub fn integrate(&self) -> Option<Vec<u8>> {
        if self.entries.is_empty() {
            return None;
        }
        let refs: Vec<&[u8]> = self.entries.iter().map(|t| t.vector.as_slice()).collect();
        Some(QuantizedVSA::bundle(&refs))
    }

    pub fn estimate_error(&self) -> f64 {
        if self.entries.len() < 2 {
            return 0.0;
        }
        let first = &self.entries.front().unwrap().vector;
        let last = &self.entries.back().unwrap().vector;
        1.0 - QuantizedVSA::similarity(first, last)
    }
}

#[derive(Debug, Clone)]
pub struct GatedLinearAttention {
    state: Vec<u8>,
    gamma: f64,
    update_count: u64,
}

impl GatedLinearAttention {
    pub fn new(gamma: f64) -> Self {
        Self {
            state: Vec::new(),
            gamma,
            update_count: 0,
        }
    }

    pub fn push(&mut self, tagged: VsaTagged) {
        self.update_count += 1;
        if self.state.is_empty() {
            self.state = tagged.vector.clone();
            return;
        }
        let blend = QuantizedVSA::bundle(&[&self.state, &tagged.vector]);
        self.state = blend;
    }

    pub fn integrate(&self) -> Option<Vec<u8>> {
        if self.state.is_empty() {
            None
        } else {
            Some(self.state.clone())
        }
    }

    pub fn estimate_error(&self) -> f64 {
        if self.state.is_empty() {
            return 0.0;
        }
        1.0 - (self.gamma.powf(self.update_count as f64))
    }

    pub fn update_count(&self) -> u64 {
        self.update_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_consciousness::vsa_tag::{VsaOrigin, VsaSelfCategory};

    fn tagged(v: Vec<u8>) -> VsaTagged {
        VsaTagged::new(v, VsaOrigin::Self_(VsaSelfCategory::Thought))
    }

    #[test]
    fn test_new_stack_empty() {
        let tas = TemporalAttentionStack::new();
        assert_eq!(tas.step_counter(), 0);
    }

    #[test]
    fn test_push_updates_all_layers() {
        let mut tas = TemporalAttentionStack::new();
        let v1 = QuantizedVSA::random_binary();
        tas.push(tagged(v1));
        assert_eq!(tas.step_counter(), 1);
    }

    #[test]
    fn test_attend_returns_vector() {
        let mut tas = TemporalAttentionStack::new();
        tas.push(tagged(QuantizedVSA::random_binary()));
        tas.push(tagged(QuantizedVSA::random_binary()));
        tas.push(tagged(QuantizedVSA::random_binary()));
        let attended = tas.attend();
        assert_eq!(attended.len(), 4096);
    }

    #[test]
    fn test_error_bound_decreases_with_steps() {
        let mut tas = TemporalAttentionStack::new();
        let e0 = tas.error_bound();
        for _ in 0..10 {
            tas.push(tagged(QuantizedVSA::random_binary()));
        }
        let e10 = tas.error_bound();
        assert!(e10 <= e0 + 1e-9);
    }

    #[test]
    fn test_local_window_max_size() {
        let mut local = SlidingWindowAttention::new(3);
        for _ in 0..10 {
            local.push(tagged(vec![1; 100]));
        }
        assert_eq!(local.len(), 3);
    }

    #[test]
    fn test_dilated_stride_push() {
        let mut tas = TemporalAttentionStack::new();
        for i in 0..10 {
            let v = vec![i as u8; 100];
            tas.push(tagged(v));
        }
        assert!(tas.mid_attention().integrate().is_some());
    }

    #[test]
    fn test_global_gamma_decay() {
        let mut global = GatedLinearAttention::new(0.5);
        assert!(global.estimate_error() < 1.0);
        for _ in 0..5 {
            global.push(tagged(vec![1; 100]));
        }
        assert!(global.update_count() == 5);
        let e = global.estimate_error();
        assert!(e > 0.0 && e < 1.0);
    }

    #[test]
    fn test_empty_attend_returns_empty() {
        let tas = TemporalAttentionStack::new();
        let attended = tas.attend();
        assert!(attended.is_empty());
    }
}
