use crate::core::nt_core_hcube::vsa_quantized::QuantizedVSA;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

pub const DEFAULT_WM_CAPACITY: usize = 7;
pub const WM_DECAY_TICK: Duration = Duration::from_secs(30);

#[derive(Debug, Clone)]
pub struct WorkingMemoryItem {
    pub id: u64,
    pub content: String,
    pub vsa_vector: Option<Vec<u8>>,
    pub importance: f64,
    pub confidence: f64,
    pub created: Instant,
    pub last_access: Instant,
    pub access_count: u64,
    pub decay_rate: f64,
    pub chunk_label: Option<String>,
}

#[derive(Debug, Clone)]
pub enum BindingOp {
    Rehearse,
    Chunk,
    Prune,
    Integrate,
}

#[derive(Debug, Clone)]
pub struct SRMUStats {
    pub total_gated: u64,
    pub total_rehearsals: u64,
    pub entropy: f64,
    pub load_factor: f64,
    pub decay_factor: f64,
}

pub struct WorkingMemory {
    slots: Vec<Option<WorkingMemoryItem>>,
    capacity: usize,
    next_id: u64,
    binding_history: VecDeque<BindingOp>,
    coherence_trace: VecDeque<f64>,
    total_rehearsals: u64,
    gated_suppressions: u64,
    decay_factor: f64,
}

impl Default for WorkingMemory {
    fn default() -> Self {
        Self::new(DEFAULT_WM_CAPACITY)
    }
}

impl WorkingMemory {
    pub fn new(capacity: usize) -> Self {
        Self {
            slots: vec![None; capacity],
            capacity,
            next_id: 1,
            binding_history: VecDeque::with_capacity(32),
            coherence_trace: VecDeque::with_capacity(capacity),
            total_rehearsals: 0,
            gated_suppressions: 0,
            decay_factor: 0.95,
        }
    }

    pub fn push(&mut self, content: String, importance: f64, vsa_vector: Option<Vec<u8>>) {
        if let Some(ref new_vsa) = vsa_vector {
            for slot in &mut self.slots {
                if let Some(ref mut existing) = slot {
                    if let Some(ref existing_vsa) = existing.vsa_vector {
                        if QuantizedVSA::similarity(new_vsa, existing_vsa) > 0.85 {
                            existing.importance = existing.importance * 0.7 + importance * 0.3;
                            existing.access_count += 1;
                            existing.last_access = Instant::now();
                            self.gated_suppressions += 1;
                            self.tick_decay();
                            return;
                        }
                    }
                }
            }
        }

        let item = WorkingMemoryItem {
            id: self.next_id,
            content,
            vsa_vector,
            importance,
            confidence: 0.5,
            created: Instant::now(),
            last_access: Instant::now(),
            access_count: 1,
            decay_rate: 0.1 * (1.0 - importance),
            chunk_label: None,
        };
        self.next_id += 1;

        let empty_slot = self.slots.iter().position(|s| s.is_none());
        if let Some(idx) = empty_slot {
            self.slots[idx] = Some(item);
        } else {
            let lowest = self.find_lowest_priority();
            if let Some(idx) = lowest {
                self.binding_history.push_back(BindingOp::Prune);
                self.slots[idx] = Some(item);
            }
        }
        self.tick_decay();
    }

    pub fn read(&mut self, id: u64) -> Option<&WorkingMemoryItem> {
        let item = self
            .slots
            .iter_mut()
            .filter_map(|s| s.as_mut())
            .find(|item| item.id == id)?;
        item.last_access = Instant::now();
        item.access_count += 1;
        self.binding_history.push_back(BindingOp::Rehearse);
        self.total_rehearsals += 1;
        Some(item)
    }

    pub fn peek(&self, id: u64) -> Option<&WorkingMemoryItem> {
        self.slots
            .iter()
            .filter_map(|s| s.as_ref())
            .find(|item| item.id == id)
    }

    pub fn current_content(&self) -> Vec<(&WorkingMemoryItem, f64)> {
        let mut items: Vec<_> = self
            .slots
            .iter()
            .filter_map(|s| s.as_ref())
            .map(|item| {
                let elapsed = item.last_access.elapsed().as_secs_f64();
                let decay = (-elapsed / 60.0).exp();
                let salience = item.importance * 0.6 + item.confidence * 0.2 + decay * 0.2;
                (item, salience)
            })
            .collect();
        items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        items
    }

    pub fn chunk(&mut self, ids: &[u64], label: String) -> Option<WorkingMemoryItem> {
        if ids.len() < 2 {
            return None;
        }
        let mut chunk_content = String::new();
        let mut vectors: Vec<&[u8]> = Vec::new();
        let mut total_importance = 0.0;

        for id in ids {
            if let Some(item) = self.peek(*id) {
                chunk_content.push_str(&item.content);
                chunk_content.push('\n');
                if let Some(ref v) = item.vsa_vector {
                    vectors.push(v.as_slice());
                }
                total_importance += item.importance;
            }
        }

        let bundle = if !vectors.is_empty() {
            Some(QuantizedVSA::bundle(&vectors))
        } else {
            None
        };

        let chunk = WorkingMemoryItem {
            id: self.next_id,
            content: chunk_content,
            vsa_vector: bundle,
            importance: total_importance / ids.len() as f64,
            confidence: 0.6,
            created: Instant::now(),
            last_access: Instant::now(),
            access_count: 1,
            decay_rate: 0.05,
            chunk_label: Some(label),
        };
        self.next_id += 1;

        for id in ids {
            self.remove(*id);
        }

        let empty_slot = self.slots.iter().position(|s| s.is_none());
        if let Some(idx) = empty_slot {
            self.binding_history.push_back(BindingOp::Chunk);
            self.slots[idx] = Some(chunk);
            self.tick_decay();
            self.slots[idx].clone()
        } else {
            let lowest = self.find_lowest_priority();
            if let Some(idx) = lowest {
                self.binding_history.push_back(BindingOp::Chunk);
                self.slots[idx] = Some(chunk);
                self.tick_decay();
                self.slots[idx].clone()
            } else {
                self.tick_decay();
                None
            }
        }
    }

    pub fn remove(&mut self, id: u64) {
        if let Some(slot) = self
            .slots
            .iter_mut()
            .find(|s| s.as_ref().map(|item| item.id == id).unwrap_or(false))
        {
            *slot = None;
            self.binding_history.push_back(BindingOp::Prune);
        }
    }

    pub fn clear(&mut self) {
        for slot in &mut self.slots {
            *slot = None;
        }
        self.binding_history.clear();
        self.coherence_trace.clear();
    }

    pub fn load(&self) -> f64 {
        let occupied = self.slots.iter().filter(|s| s.is_some()).count();
        occupied as f64 / self.capacity as f64
    }

    pub fn coherence(&self) -> f64 {
        let vectors: Vec<&[u8]> = self
            .slots
            .iter()
            .filter_map(|s| s.as_ref())
            .filter_map(|item| item.vsa_vector.as_ref())
            .map(|v| v.as_slice())
            .collect();
        if vectors.len() < 2 {
            return 1.0;
        }
        let mut total_sim = 0.0;
        let mut pairs = 0;
        for i in 0..vectors.len() {
            for j in (i + 1)..vectors.len() {
                total_sim += QuantizedVSA::similarity(vectors[i], vectors[j]);
                pairs += 1;
            }
        }
        total_sim / pairs as f64
    }

    pub fn item_count(&self) -> usize {
        self.slots.iter().filter(|s| s.is_some()).count()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn total_rehearsals(&self) -> u64 {
        self.total_rehearsals
    }

    pub fn gated_suppressions(&self) -> u64 {
        self.gated_suppressions
    }

    pub fn binding_history(&self) -> &VecDeque<BindingOp> {
        &self.binding_history
    }

    pub fn entropy(&self) -> f64 {
        let occupied: Vec<f64> = self
            .slots
            .iter()
            .filter_map(|s| s.as_ref())
            .map(|item| item.importance)
            .collect();

        if occupied.is_empty() {
            return 0.0;
        }

        let total: f64 = occupied.iter().sum();
        if total <= 0.0 {
            return 0.0;
        }

        let h = occupied
            .iter()
            .map(|&v| {
                let p = v / total;
                if p > 0.0 {
                    -p * p.log2()
                } else {
                    0.0
                }
            })
            .sum::<f64>();

        let max = (occupied.len() as f64).log2();
        if max <= 0.0 {
            0.0
        } else {
            (h / max).clamp(0.0, 1.0)
        }
    }

    pub fn relevance_scores(&self) -> Vec<(u64, f64)> {
        let now = Instant::now();
        self.slots
            .iter()
            .filter_map(|s| s.as_ref())
            .map(|item| {
                let elapsed_secs = now.duration_since(item.last_access).as_secs_f64();
                let recency_decay = (-elapsed_secs / 60.0).exp();
                let relevance = item.importance * 0.5 + item.confidence * 0.3 + recency_decay * 0.2;
                (item.id, relevance)
            })
            .collect()
    }

    pub fn srmu_stats(&self) -> SRMUStats {
        SRMUStats {
            total_gated: self.gated_suppressions,
            total_rehearsals: self.total_rehearsals,
            entropy: self.entropy(),
            load_factor: self.load(),
            decay_factor: self.decay_factor,
        }
    }

    pub fn apply_time_decay(&mut self) {
        for slot in &mut self.slots {
            if let Some(ref mut item) = slot {
                item.importance *= self.decay_factor;
            }
        }
        for slot in &mut self.slots {
            if slot.as_ref().map_or(false, |item| item.importance <= 0.05) {
                *slot = None;
            }
        }
    }

    pub fn set_decay_factor(&mut self, factor: f64) {
        self.decay_factor = factor.clamp(0.1, 0.999);
    }

    fn find_lowest_priority(&self) -> Option<usize> {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(i, s)| s.as_ref().map(|item| (i, item.importance)))
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
    }

    fn tick_decay(&mut self) {
        for slot in &mut self.slots {
            if let Some(ref mut item) = slot {
                let elapsed = item.last_access.elapsed().as_secs_f64() / 60.0;
                let decay_amount = item.decay_rate * elapsed;
                item.importance = (item.importance - decay_amount).max(0.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_count() {
        let mut wm = WorkingMemory::new(5);
        wm.push("item1".into(), 0.9, None);
        wm.push("item2".into(), 0.8, None);
        assert_eq!(wm.item_count(), 2);
        assert_eq!(wm.load(), 0.4);
    }

    #[test]
    fn test_capacity_eviction() {
        let mut wm = WorkingMemory::new(3);
        wm.push("a".into(), 0.9, None);
        wm.push("b".into(), 0.8, None);
        wm.push("c".into(), 0.7, None);
        wm.push("d".into(), 1.0, None);
        assert_eq!(wm.item_count(), 3);
        assert!(wm.peek(4).is_some());
    }

    #[test]
    fn test_read_updates_access() {
        let mut wm = WorkingMemory::new(5);
        wm.push("test".into(), 0.5, None);
        let item = wm.read(1).unwrap();
        assert_eq!(item.access_count, 1);
    }

    #[test]
    fn test_current_content_ordered() {
        let mut wm = WorkingMemory::new(5);
        wm.push("low".into(), 0.3, None);
        wm.push("high".into(), 0.9, None);
        let content = wm.current_content();
        assert!(content[0].0.importance >= content[1].0.importance);
    }

    #[test]
    fn test_chunk_combines_items() {
        let mut wm = WorkingMemory::new(5);
        wm.push("part1".into(), 0.7, None);
        wm.push("part2".into(), 0.6, None);
        let chunk = wm.chunk(&[1, 2], "combined".into());
        assert!(chunk.is_some());
        assert_eq!(chunk.unwrap().chunk_label.unwrap(), "combined");
    }

    #[test]
    fn test_coherence_calculation() {
        let mut wm = WorkingMemory::new(3);
        let v1 = QuantizedVSA::random_binary();
        let v2 = QuantizedVSA::random_binary();
        wm.push("a".into(), 0.5, Some(v1));
        wm.push("b".into(), 0.5, Some(v2));
        let coherence = wm.coherence();
        assert!(coherence >= 0.0 && coherence <= 1.0);
    }

    #[test]
    fn test_empty_coherence() {
        let wm = WorkingMemory::new(5);
        assert!((wm.coherence() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_clear_resets_all() {
        let mut wm = WorkingMemory::new(5);
        wm.push("x".into(), 0.5, None);
        wm.push("y".into(), 0.5, None);
        wm.clear();
        assert_eq!(wm.item_count(), 0);
        assert_eq!(wm.load(), 0.0);
    }

    #[test]
    fn test_binding_history() {
        let mut wm = WorkingMemory::new(3);
        wm.push("a".into(), 0.9, None);
        wm.push("b".into(), 0.8, None);
        wm.push("c".into(), 0.7, None);
        wm.push("d".into(), 1.0, None);
        let history = wm.binding_history();
        assert!(history.iter().any(|op| matches!(op, BindingOp::Prune)));
    }

    #[test]
    fn test_srmu_relevance_gate_suppresses_duplicate() {
        let mut wm = WorkingMemory::new(5);
        let v = vec![1u8; 4096];
        wm.push("original".into(), 0.8, Some(v.clone()));
        assert_eq!(wm.item_count(), 1);
        assert_eq!(wm.gated_suppressions(), 0);

        wm.push("copy".into(), 0.9, Some(v.clone()));
        assert_eq!(wm.item_count(), 1);
        assert_eq!(wm.gated_suppressions(), 1);
    }

    #[test]
    fn test_srmu_gate_blends_importance() {
        let mut wm = WorkingMemory::new(5);
        let v = vec![1u8; 4096];
        wm.push("first".into(), 0.8, Some(v.clone()));
        let _orig_imp = wm.peek(1).unwrap().importance;

        wm.push("second".into(), 1.0, Some(v.clone()));
        let updated = wm.peek(1).unwrap();
        let expected = 0.8 * 0.7 + 1.0 * 0.3;
        assert!((updated.importance - expected).abs() < 1e-9);
        assert_eq!(updated.access_count, 2);
    }

    #[test]
    fn test_srmu_gate_does_not_affect_dissimilar() {
        let mut wm = WorkingMemory::new(5);
        let v1 = QuantizedVSA::random_binary();
        let v2 = QuantizedVSA::random_binary();
        assert!(QuantizedVSA::similarity(&v1, &v2) < 0.85);

        wm.push("a".into(), 0.5, Some(v1));
        wm.push("b".into(), 0.5, Some(v2));
        assert_eq!(wm.item_count(), 2);
        assert_eq!(wm.gated_suppressions(), 0);
    }

    #[test]
    fn test_srmu_entropy_uniform() {
        let mut wm = WorkingMemory::new(3);
        wm.push("a".into(), 0.8, None);
        wm.push("b".into(), 0.8, None);
        wm.push("c".into(), 0.8, None);
        let e = wm.entropy();
        assert!((e - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_srmu_entropy_skewed() {
        let mut wm = WorkingMemory::new(3);
        wm.push("a".into(), 0.0, None);
        wm.push("b".into(), 0.0, None);
        wm.push("c".into(), 1.0, None);
        let e = wm.entropy();
        assert!(e < 1.0);
    }

    #[test]
    fn test_srmu_entropy_empty() {
        let wm: WorkingMemory = WorkingMemory::new(5);
        assert!((wm.entropy() - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_srmu_relevance_scores() {
        let mut wm = WorkingMemory::new(5);
        wm.push("x".into(), 0.9, None);
        wm.push("y".into(), 0.3, None);
        let scores = wm.relevance_scores();
        assert_eq!(scores.len(), 2);
        assert!(scores[0].1 >= scores[1].1);
    }

    #[test]
    fn test_srmu_stats_structure() {
        let mut wm = WorkingMemory::new(5);
        let v = vec![1u8; 4096];
        wm.push("a".into(), 0.5, Some(v.clone()));
        wm.push("b".into(), 0.5, Some(v.clone()));
        let stats = wm.srmu_stats();
        assert_eq!(stats.total_gated, 1);
        assert!(stats.entropy == 0.0);
        assert!((stats.load_factor - 0.2).abs() < 1e-9);
    }

    #[test]
    fn test_srmu_gate_no_vsa_falls_through() {
        let mut wm = WorkingMemory::new(5);
        wm.push("a".into(), 0.5, None);
        wm.push("b".into(), 0.5, None);
        assert_eq!(wm.item_count(), 2);
        assert_eq!(wm.gated_suppressions(), 0);
    }

    #[test]
    fn test_time_decay() {
        let mut wm = WorkingMemory::new(5);
        for i in 0..5 {
            wm.push(format!("item_{}", i), 0.9, None);
        }
        assert_eq!(wm.item_count(), 5);
        wm.apply_time_decay();
        // After one decay step with default 0.95, items should stay
        assert_eq!(wm.item_count(), 5);
        // Aggressive decay to trigger pruning
        wm.set_decay_factor(0.5);
        for _ in 0..10 {
            wm.apply_time_decay();
        }
        // Items should be pruned by now
        assert!(wm.item_count() < 5);
    }
}
