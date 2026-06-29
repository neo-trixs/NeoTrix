//! # Working Memory — SpeciousPresent Buffer
//!
//! Short-term buffer for the current cognitive cycle.
//! Items persist for the SpeciousPresent window (~200ms / ~10 cycles),
//! then decay or get consolidated into episodic memory.

use std::collections::VecDeque;

const DEFAULT_CAPACITY: usize = 64;
const TTL_CYCLES: u64 = 10;

#[derive(Debug, Clone)]
pub struct WorkingMemoryItem {
    pub vector: Vec<f64>,
    pub tag: String,
    pub cycle: u64,
    pub salience: f64,
}

#[derive(Debug, Clone)]
pub struct WorkingMemory {
    items: VecDeque<WorkingMemoryItem>,
    capacity: usize,
}

impl WorkingMemory {
    pub fn new(capacity: usize) -> Self {
        Self {
            items: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, item: WorkingMemoryItem) {
        if self.items.len() >= self.capacity {
            self.items.pop_front();
        }
        self.items.push_back(item);
    }

    pub fn prune(&mut self, current_cycle: u64) -> usize {
        let before = self.items.len();
        self.items.retain(|item| current_cycle - item.cycle < TTL_CYCLES);
        before - self.items.len()
    }

    pub fn decay(&mut self, factor: f64) {
        for item in &mut self.items {
            item.salience *= factor;
        }
        self.items.retain(|item| item.salience > 0.01);
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn items(&self) -> &VecDeque<WorkingMemoryItem> {
        &self.items
    }

    pub fn most_salient(&self) -> Option<&WorkingMemoryItem> {
        self.items.iter().max_by(|a, b| {
            a.salience.partial_cmp(&b.salience).unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }
}

impl Default for WorkingMemory {
    fn default() -> Self {
        Self::new(DEFAULT_CAPACITY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_working_memory_push_pop() {
        let mut wm = WorkingMemory::new(10);
        wm.push(WorkingMemoryItem {
            vector: vec![1.0; 4],
            tag: "test".into(),
            cycle: 0,
            salience: 0.8,
        });
        assert_eq!(wm.len(), 1);
    }

    #[test]
    fn test_working_memory_capacity() {
        let mut wm = WorkingMemory::new(2);
        for i in 0..5 {
            wm.push(WorkingMemoryItem {
                vector: vec![i as f64; 4],
                tag: format!("item_{}", i),
                cycle: i,
                salience: 0.5,
            });
        }
        assert_eq!(wm.len(), 2);
    }

    #[test]
    fn test_prune_expired() {
        let mut wm = WorkingMemory::new(10);
        wm.push(WorkingMemoryItem {
            vector: vec![1.0; 4],
            tag: "old".into(),
            cycle: 0,
            salience: 0.8,
        });
        let pruned = wm.prune(100);
        assert_eq!(pruned, 1);
        assert!(wm.is_empty());
    }

    #[test]
    fn test_most_salient() {
        let mut wm = WorkingMemory::new(10);
        wm.push(WorkingMemoryItem {
            vector: vec![1.0; 4],
            tag: "low".into(),
            cycle: 0,
            salience: 0.2,
        });
        wm.push(WorkingMemoryItem {
            vector: vec![2.0; 4],
            tag: "high".into(),
            cycle: 1,
            salience: 0.9,
        });
        assert_eq!(wm.most_salient().unwrap().tag, "high");
    }

    #[test]
    fn test_decay_removes_low() {
        let mut wm = WorkingMemory::new(10);
        wm.push(WorkingMemoryItem {
            vector: vec![1.0; 4],
            tag: "low".into(),
            cycle: 0,
            salience: 0.02,
        });
        wm.decay(0.5);
        assert!(wm.is_empty());
    }
}
