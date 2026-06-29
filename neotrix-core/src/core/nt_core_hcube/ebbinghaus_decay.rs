use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;

#[derive(Debug, Clone)]
pub struct DecayConfig {
    pub initial_stability: f64,
    pub min_stability: f64,
    pub decay_rate: f64,
    pub retrieval_boost: f64,
    pub spacing_factor: f64,
}

pub const LONG_TERM_DAYS: f64 = 30.0;
pub const DEFAULT_30_DAY_HALF_LIFE: f64 = 30.0;

impl Default for DecayConfig {
    fn default() -> Self {
        Self {
            initial_stability: 1.0,
            min_stability: 0.1,
            decay_rate: 0.1,
            retrieval_boost: 1.5,
            spacing_factor: 1.2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemoryTrace {
    pub id: u64,
    pub vector: Vec<u8>,
    pub label: String,
    pub stability: f64,
    pub age: f64,
    pub retrieval_count: u64,
    pub last_retrieval_age: f64,
}

#[derive(Debug, Clone)]
pub struct EbbinghausDecay {
    memories: Vec<MemoryTrace>,
    config: DecayConfig,
    next_id: u64,
    global_time: f64,
}

impl EbbinghausDecay {
    pub fn new(config: DecayConfig) -> Self {
        Self {
            memories: Vec::new(),
            config,
            next_id: 0,
            global_time: 0.0,
        }
    }

    pub fn add_memory(&mut self, vector: Vec<u8>, label: &str) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.memories.push(MemoryTrace {
            id,
            vector,
            label: label.to_string(),
            stability: self.config.initial_stability,
            age: 0.0,
            retrieval_count: 0,
            last_retrieval_age: 0.0,
        });
        id
    }

    pub fn retrieve(&mut self, id: u64) -> Option<Vec<u8>> {
        let trace = self.memories.iter_mut().find(|m| m.id == id)?;
        trace.retrieval_count += 1;
        let age_since_last = self.global_time - trace.last_retrieval_age;
        let boost =
            self.config.retrieval_boost * (age_since_last * self.config.spacing_factor).max(1.0);
        trace.stability = (trace.stability * boost).max(self.config.min_stability);
        trace.last_retrieval_age = self.global_time;
        Some(trace.vector.clone())
    }

    pub fn compute_retention(memory: &MemoryTrace) -> f64 {
        if memory.stability <= 0.0 {
            return 0.0;
        }
        (-0.1 * memory.age / memory.stability).exp().clamp(0.0, 1.0)
    }

    pub fn apply_decay_to_memory(&mut self, id: u64) {
        if let Some(m) = self.memories.iter_mut().find(|m| m.id == id) {
            let decay = 0.1 * m.age / m.stability.max(1e-12);
            m.stability = (m.stability - decay).max(self.config.min_stability);
        }
    }

    pub fn tick(&mut self, dt: f64) {
        self.global_time += dt;
        for m in &mut self.memories {
            let old_age = m.age;
            m.age = self.global_time;
            let dt_mem = (m.age - old_age).max(0.0);
            let decay = self.config.decay_rate * dt_mem / m.stability.max(1e-12);
            m.stability = (m.stability - decay).max(self.config.min_stability);
        }
    }

    pub fn forget_below_threshold(&mut self, threshold: f64) {
        self.memories
            .retain(|m| Self::compute_retention(m) >= threshold);
    }

    pub fn find_nearest(&self, query: &[u8], k: usize) -> Vec<(u64, f64)> {
        let mut sims: Vec<(u64, f64)> = self
            .memories
            .iter()
            .map(|m| (m.id, QuantizedVSA::similarity(&m.vector, query)))
            .collect();
        sims.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        sims.truncate(k);
        sims
    }

    pub fn consolidate(&mut self, ids: &[u64]) -> Vec<u8> {
        let vectors: Vec<&[u8]> = self
            .memories
            .iter()
            .filter(|m| ids.contains(&m.id))
            .map(|m| m.vector.as_slice())
            .collect();
        QuantizedVSA::majority_bundle(&vectors)
    }

    pub fn stability_distribution(&self) -> (f64, f64, f64) {
        if self.memories.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        let sum: f64 = self.memories.iter().map(|m| m.stability).sum();
        let mean = sum / self.memories.len() as f64;
        let min = self
            .memories
            .iter()
            .map(|m| m.stability)
            .fold(f64::INFINITY, f64::min);
        let max = self
            .memories
            .iter()
            .map(|m| m.stability)
            .fold(f64::NEG_INFINITY, f64::max);
        (mean, min, max)
    }

    pub fn with_half_life(half_life_days: f64) -> Self {
        Self::new(DecayConfig {
            decay_rate: std::f64::consts::LN_2 / half_life_days.max(1.0),
            ..DecayConfig::default()
        })
    }

    pub fn preset_long_term() -> Self {
        Self::with_half_life(LONG_TERM_DAYS)
    }

    pub fn memory_count(&self) -> usize {
        self.memories.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_hcube::vsa_quantized::VSA_DIM;

    fn test_decay() -> EbbinghausDecay {
        EbbinghausDecay::new(DecayConfig::default())
    }

    fn random_vsa() -> Vec<u8> {
        (0..VSA_DIM).map(|i| (i as u8) & 1).collect()
    }

    #[test]
    fn test_add_memory() {
        let mut d = test_decay();
        let v = random_vsa();
        let id = d.add_memory(v.clone(), "test");
        assert_eq!(d.memory_count(), 1);
        let trace = &d.memories[0];
        assert_eq!(trace.id, id);
        assert!((trace.stability - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_ebbinghaus_curve() {
        let mut m = MemoryTrace {
            id: 0,
            vector: vec![0; VSA_DIM],
            label: "test".into(),
            stability: 1.0,
            age: 0.0,
            retrieval_count: 0,
            last_retrieval_age: 0.0,
        };
        let r0 = EbbinghausDecay::compute_retention(&m);
        assert!((r0 - 1.0).abs() < 1e-6);
        m.age = 10.0;
        let r10 = EbbinghausDecay::compute_retention(&m);
        assert!(r10 < r0);
        m.stability = 5.0;
        let r_high_stab = EbbinghausDecay::compute_retention(&m);
        assert!(r_high_stab > r10);
    }

    #[test]
    fn test_retrieval_boost() {
        let mut d = test_decay();
        let v = random_vsa();
        let id = d.add_memory(v, "boost");
        d.global_time = 5.0;
        let _ = d.retrieve(id);
        let s_after = d.memories[0].stability;
        assert!(s_after > 1.0);
    }

    #[test]
    fn test_tick_decay() {
        let mut d = test_decay();
        let v = random_vsa();
        let _id = d.add_memory(v, "tick");
        let s_before = d.memories[0].stability;
        d.tick(5.0);
        let s_after = d.memories[0].stability;
        assert!(s_after < s_before);
        assert!(s_after >= d.config.min_stability);
    }

    #[test]
    fn test_forget_below_threshold() {
        let mut d = test_decay();
        d.add_memory(random_vsa(), "keep");
        d.add_memory(random_vsa(), "forget");
        d.memories[1].stability = 0.01;
        d.memories[1].age = 100.0;
        d.forget_below_threshold(0.5);
        assert_eq!(d.memory_count(), 1);
        assert_eq!(d.memories[0].label, "keep");
    }

    #[test]
    fn test_find_nearest() {
        let mut d = test_decay();
        let v1 = vec![1u8; VSA_DIM];
        let v2 = vec![0u8; VSA_DIM];
        let id1 = d.add_memory(v1.clone(), "ones");
        let _id2 = d.add_memory(v2.clone(), "zeros");
        let near = d.find_nearest(&v1, 2);
        assert_eq!(near.len(), 2);
        assert_eq!(near[0].0, id1);
    }

    #[test]
    fn test_consolidate() {
        let mut d = test_decay();
        let id1 = d.add_memory(vec![1u8; VSA_DIM], "all_ones");
        let id2 = d.add_memory(vec![0u8; VSA_DIM], "all_zeros");
        let bundled = d.consolidate(&[id1, id2]);
        assert_eq!(bundled.len(), VSA_DIM);
        let ones_count = bundled.iter().filter(|&&x| x > 0).count();
        assert!(ones_count == 0 || ones_count == VSA_DIM);
    }

    #[test]
    fn test_spacing_effect() {
        let mut d = test_decay();
        let v = random_vsa();
        let id = d.add_memory(v, "spacing");
        d.global_time = 1.0;
        let _ = d.retrieve(id);
        let s_short = d.memories[0].stability;
        d.memories[0].stability = 1.0;
        d.memories[0].retrieval_count = 0;
        d.memories[0].last_retrieval_age = 0.0;
        d.global_time = 10.0;
        let _ = d.retrieve(id);
        let s_long = d.memories[0].stability;
        assert!(s_long > s_short);
    }

    #[test]
    fn test_stability_distribution() {
        let mut d = test_decay();
        assert_eq!(d.stability_distribution(), (0.0, 0.0, 0.0));
        d.add_memory(random_vsa(), "a");
        d.add_memory(random_vsa(), "b");
        d.memories[0].stability = 0.5;
        d.memories[1].stability = 2.0;
        let (mean, min, max) = d.stability_distribution();
        assert!((mean - 1.25).abs() < 1e-6);
        assert!((min - 0.5).abs() < 1e-6);
        assert!((max - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_apply_decay_to_memory() {
        let mut d = test_decay();
        let v = random_vsa();
        let id = d.add_memory(v, "decay_target");
        d.memories[0].age = 10.0;
        d.apply_decay_to_memory(id);
        let s = d.memories[0].stability;
        assert!(s < 1.0);
        assert!(s >= d.config.min_stability);
    }
}
