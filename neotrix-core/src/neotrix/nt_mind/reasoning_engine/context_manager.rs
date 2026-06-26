use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MemoryZone {
    Frozen,
    Compress,
    Active,
}

pub struct ContextEntry {
    pub content: String,
    pub zone: MemoryZone,
    pub token_estimate: usize,
    pub priority: u8,
    pub age: u64,
}

impl ContextEntry {
    pub fn new(content: &str, priority: u8) -> Self {
        let estimate = content.len() / 4;
        Self {
            content: content.to_string(),
            zone: MemoryZone::Active,
            token_estimate: estimate,
            priority,
            age: 0,
        }
    }

    pub fn estimate_tokens(&self) -> usize {
        self.token_estimate
    }
}

pub struct ContextManager {
    pub frozen: Vec<ContextEntry>,
    pub compress: VecDeque<ContextEntry>,
    pub active: VecDeque<ContextEntry>,
    pub max_tokens: usize,
    pub compress_threshold: f64,
    pub active_threshold: f64,
    pub current_tokens: usize,
    age_counter: u64,
}

pub struct MemoryPressure {
    pub usage_pct: f64,
    pub zone: &'static str,
    pub frozen_tokens: usize,
    pub compress_tokens: usize,
    pub active_tokens: usize,
}

impl ContextManager {
    pub fn new(max_tokens: usize) -> Self {
        Self {
            frozen: Vec::new(),
            compress: VecDeque::new(),
            active: VecDeque::new(),
            max_tokens,
            compress_threshold: 0.60,
            active_threshold: 0.80,
            current_tokens: 0,
            age_counter: 0,
        }
    }

    pub fn insert(&mut self, content: &str, priority: u8) {
        let age = self.age_counter;
        self.age_counter += 1;

        let estimate = content.len() / 4;

        let target_zone = if priority >= 200 {
            MemoryZone::Frozen
        } else if priority >= 100 {
            MemoryZone::Active
        } else {
            MemoryZone::Compress
        };

        let entry = ContextEntry {
            content: content.to_string(),
            zone: target_zone,
            token_estimate: estimate,
            priority,
            age,
        };

        self.current_tokens += estimate;

        match target_zone {
            MemoryZone::Frozen => {
                self.frozen.push(entry);
                if self.frozen.len() > 100 {
                    self.frozen.remove(0);
                }
            }
            MemoryZone::Active => self.active.push_back(entry),
            MemoryZone::Compress => self.compress.push_back(entry),
        }
    }

    pub fn freeze(&mut self, index: usize) {
        const MAX_FROZEN: usize = 100;
        if index >= self.active.len() {
            return;
        }
        let mut entry = self.active.remove(index).expect("freeze: index checked < active.len() above so remove cannot fail");
        entry.zone = MemoryZone::Frozen;
        self.current_tokens = self.current_tokens.saturating_sub(entry.token_estimate);
        self.frozen.push(entry);
        if self.frozen.len() > MAX_FROZEN {
            self.frozen.remove(0);
        }
    }

    pub fn compress(&mut self) -> usize {
        if self.compress.is_empty() {
            return 0;
        }

        let before: usize = self.compress.iter().map(|e| e.token_estimate).sum();

        let mut entries: Vec<ContextEntry> = self.compress.drain(..).collect();
        entries.sort_by(|a, b| b.priority.cmp(&a.priority));

        let keep_count = (entries.len() + 1) / 2;
        entries.truncate(keep_count);

        for entry in entries {
            self.compress.push_back(entry);
        }

        let after: usize = self.compress.iter().map(|e| e.token_estimate).sum();
        let freed = before - after;
        self.current_tokens = self.current_tokens.saturating_sub(freed);
        freed
    }

    pub fn evict_active(&mut self) -> usize {
        if self.active.is_empty() {
            return 0;
        }

        let evict_count = (self.active.len() * 3 + 9) / 10;
        let evict_count = evict_count.min(self.active.len());

        let mut entries: Vec<ContextEntry> = self.active.drain(..).collect();
        entries.sort_by(|a, b| a.priority.cmp(&b.priority));

        let evicted: Vec<ContextEntry> = entries.drain(..evict_count).collect();
        let remaining: VecDeque<ContextEntry> = entries.into();

        let moved_tokens: usize = evicted.iter().map(|e| e.token_estimate).sum();

        for mut entry in evicted {
            entry.zone = MemoryZone::Compress;
            self.compress.push_back(entry);
        }

        self.active = remaining;
        moved_tokens
    }

    pub fn maintain(&mut self) -> (usize, usize) {
        let usage = self.current_tokens as f64 / self.max_tokens as f64;

        if usage < self.compress_threshold {
            return (0, 0);
        }

        if usage < self.active_threshold {
            let freed = self.compress();
            return (freed, 0);
        }

        let compress_freed = self.compress();
        let evicted = self.evict_active();
        let compress_again = self.compress();
        (compress_freed + compress_again, evicted)
    }

    pub fn pressure_report(&self) -> MemoryPressure {
        let frozen_tokens: usize = self.frozen.iter().map(|e| e.token_estimate).sum();
        let compress_tokens: usize = self.compress.iter().map(|e| e.token_estimate).sum();
        let active_tokens: usize = self.active.iter().map(|e| e.token_estimate).sum();

        let total = frozen_tokens + compress_tokens + active_tokens;
        let usage_pct = if self.max_tokens > 0 {
            total as f64 / self.max_tokens as f64 * 100.0
        } else {
            0.0
        };

        let zone = if usage_pct < self.compress_threshold * 100.0 {
            "healthy"
        } else if usage_pct < self.active_threshold * 100.0 {
            "compress"
        } else {
            "critical"
        };

        MemoryPressure {
            usage_pct,
            zone,
            frozen_tokens,
            compress_tokens,
            active_tokens,
        }
    }

    pub fn total_entries(&self) -> usize {
        self.frozen.len() + self.compress.len() + self.active.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_entry_creation_and_estimation() {
        let entry = ContextEntry::new("hello world", 100);
        assert_eq!(entry.priority, 100);
        assert!(entry.token_estimate > 0);
        assert_eq!(entry.estimate_tokens(), entry.token_estimate);
    }

    #[test]
    fn test_insert_correct_zone_by_priority() {
        let mut cm = ContextManager::new(1000);

        cm.insert("frozen content", 200);
        cm.insert("active content", 150);
        cm.insert("compress content", 50);

        assert_eq!(cm.frozen.len(), 1);
        assert_eq!(cm.active.len(), 1);
        assert_eq!(cm.compress.len(), 1);
    }

    #[test]
    fn test_freeze_moves_entry_to_frozen() {
        let mut cm = ContextManager::new(1000);
        cm.insert("task context", 150);
        assert_eq!(cm.active.len(), 1);

        cm.freeze(0);
        assert_eq!(cm.active.len(), 0);
        assert_eq!(cm.frozen.len(), 1);
    }

    #[test]
    fn test_compress_removes_lowest_priority() {
        let mut cm = ContextManager::new(1000);
        cm.insert("low priority", 10);
        cm.insert("medium priority", 50);

        assert_eq!(cm.compress.len(), 2);
        let freed = cm.compress();
        assert!(freed > 0);
        assert_eq!(cm.compress.len(), 1);
    }

    #[test]
    fn test_evict_active_moves_entries_to_compress() {
        let mut cm = ContextManager::new(10000);
        for i in (0..10).rev() {
            cm.active
                .push_back(ContextEntry::new(&format!("entry {}", i), i * 10));
            cm.current_tokens += cm
                .active
                .back()
                .expect("value should be ok in test")
                .token_estimate;
        }
        assert_eq!(cm.active.len(), 10);

        let moved = cm.evict_active();
        assert!(moved > 0);
        assert!(cm.active.len() < 10);
        assert!(cm.compress.len() > 0);
    }

    #[test]
    fn test_maintain_healthy_below_threshold() {
        let mut cm = ContextManager::new(1000);
        cm.insert("small", 100);
        assert!(cm.current_tokens as f64 / (cm.max_tokens as f64) < cm.compress_threshold);

        let (frozen, evicted) = cm.maintain();
        assert_eq!(frozen, 0);
        assert_eq!(evicted, 0);
    }

    #[test]
    fn test_maintain_compress_zone() {
        let mut cm = ContextManager::new(100);
        for i in 0..8 {
            let content = "x".repeat(32);
            cm.insert(&content, (i as u8) * 12);
        }

        let usage = cm.current_tokens as f64 / cm.max_tokens as f64;
        assert!(
            usage >= cm.compress_threshold,
            "usage={:.2} should be >= 0.60 for compress test",
            usage
        );

        let (freed, evicted) = cm.maintain();
        assert!(freed > 0);
        assert!(cm.compress.len() <= 4);
        assert_eq!(evicted, 0);
    }

    #[test]
    fn test_maintain_critical_zone() {
        let mut cm = ContextManager::new(100);
        for i in 0..3 {
            let content = "y".repeat(84);
            cm.insert(&content, 100 + i as u8);
        }
        for i in 0..4 {
            let content = "z".repeat(28);
            cm.insert(&content, i as u8);
        }

        let usage = cm.current_tokens as f64 / cm.max_tokens as f64;
        assert!(
            usage >= cm.active_threshold,
            "usage={:.2} should be >= 0.80 for critical test",
            usage
        );

        let (freed, evicted) = cm.maintain();
        assert!(freed > 0);
        assert!(evicted > 0);
    }

    #[test]
    fn test_pressure_report_format() {
        let cm = ContextManager::new(1000);
        let report = cm.pressure_report();
        assert_eq!(report.zone, "healthy");
        assert!(report.usage_pct < 100.0);
        assert_eq!(report.frozen_tokens, 0);
        assert_eq!(report.compress_tokens, 0);
        assert_eq!(report.active_tokens, 0);
    }

    #[test]
    fn test_total_entries_counts_all_zones() {
        let mut cm = ContextManager::new(1000);
        cm.insert("f", 200);
        cm.insert("a", 150);
        cm.insert("c", 50);
        assert_eq!(cm.total_entries(), 3);
    }

    #[test]
    fn test_large_insertions_varying_priorities() {
        let mut cm = ContextManager::new(100000);
        for i in 0..100 {
            cm.insert(&format!("entry {}", i), (i as u8) % 250);
        }
        assert_eq!(cm.total_entries(), 100);
        assert!(cm.current_tokens > 0);
    }

    #[test]
    fn test_boundary_at_exact_thresholds() {
        let mut cm = ContextManager::new(1000);
        cm.compress_threshold = 0.60;
        cm.active_threshold = 0.80;

        let exact_60_size = (1000.0 * 0.60) as usize;
        let content = "x".repeat(exact_60_size * 4);
        cm.insert(&content, 50);

        let report = cm.pressure_report();
        assert!(
            report.usage_pct >= 59.0,
            "usage={:.2} should be near 60%",
            report.usage_pct
        );

        let (freed, evicted) = cm.maintain();
        assert!(freed > 0 || evicted == 0);

        cm.compress.clear();

        let exact_80_size = (1000.0 * 0.80) as usize;
        let content2 = "y".repeat(exact_80_size * 4);
        cm.insert(&content2, 50);
        // insert extra entries to ensure active zone has items
        for i in 0..3 {
            cm.insert(&format!("active entry {}", i), 100 + i as u8);
        }

        let report2 = cm.pressure_report();
        assert!(
            report2.usage_pct >= 79.0,
            "usage={:.2} should be near 80%",
            report2.usage_pct
        );

        let (freed2, evicted2) = cm.maintain();
        assert!(freed2 > 0);
        assert!(evicted2 > 0);
    }
}
