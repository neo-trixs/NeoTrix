use std::collections::HashSet;

const MAX_ENTRIES: usize = 1000;
const CONSOLIDATE_AGE_SECS: u64 = 3600; // 1 hour

#[derive(Clone, Debug)]
pub struct EmotionalMemoryTag {
    pub valence: f64,
    pub arousal: f64,
    pub dominance: f64,
    pub label: String,
}

#[derive(Clone, Debug)]
pub struct EmotionalMemoryEntry {
    pub id: u64,
    pub timestamp: std::time::Instant,
    pub text: String,
    pub emotion: EmotionalMemoryTag,
    pub vsa_hash: u64,
    pub access_count: u64,
    pub last_accessed: std::time::Instant,
}

#[derive(Clone, Debug, Default)]
pub struct EmotionalMemoryStats {
    pub total_entries: usize,
    pub consolidated: usize,
    pub avg_valence: f64,
    pub avg_arousal: f64,
}

pub struct EmotionalMemory {
    entries: Vec<EmotionalMemoryEntry>,
    next_id: u64,
}

impl EmotionalMemory {
    pub fn new() -> Self {
        Self {
            entries: Vec::with_capacity(MAX_ENTRIES),
            next_id: 1,
        }
    }

    fn vsa_hash(text: &str) -> u64 {
        let mut h: u64 = 5381;
        for b in text.bytes() {
            h = h.wrapping_mul(33).wrapping_add(b as u64);
        }
        h
    }

    pub fn store(&mut self, text: &str, emotion: EmotionalMemoryTag) {
        let vsa_hash = Self::vsa_hash(text);
        let now = std::time::Instant::now();

        let entry = EmotionalMemoryEntry {
            id: self.next_id,
            timestamp: now,
            text: text.to_string(),
            emotion,
            vsa_hash,
            access_count: 0,
            last_accessed: now,
        };
        self.next_id += 1;

        if self.entries.len() >= MAX_ENTRIES {
            self.evict_lru();
        }
        self.entries.push(entry);
    }

    fn evict_lru(&mut self) {
        if let Some(idx) = self
            .entries
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                a.access_count
                    .cmp(&b.access_count)
                    .then(a.last_accessed.cmp(&b.last_accessed))
            })
            .map(|(i, _)| i)
        {
            self.entries.swap_remove(idx);
        }
    }

    pub fn retrieve_by_emotion(&self, valence_threshold: f64) -> Vec<&EmotionalMemoryEntry> {
        self.entries
            .iter()
            .filter(|e| e.emotion.valence.abs() > valence_threshold)
            .collect()
    }

    pub fn retrieve_recent(&self, n: usize) -> Vec<&EmotionalMemoryEntry> {
        let mut sorted: Vec<_> = self.entries.iter().collect();
        sorted.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        sorted.truncate(n);
        sorted
    }

    pub fn search_similar(&self, query: &str, n: usize) -> Vec<&EmotionalMemoryEntry> {
        let query_words: HashSet<&str> = query.split_whitespace().collect();
        if query_words.is_empty() {
            return Vec::new();
        }
        let mut scored: Vec<_> = self
            .entries
            .iter()
            .map(|e| {
                let entry_words: HashSet<&str> = e.text.split_whitespace().collect();
                let overlap = query_words.intersection(&entry_words).count();
                let score = overlap as f64 / query_words.len() as f64;
                (score, e)
            })
            .filter(|(s, _)| *s > 0.0)
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(n);
        scored.into_iter().map(|(_, e)| e).collect()
    }

    pub fn tick(&mut self) -> EmotionalMemoryStats {
        let n_before = self.entries.len();
        let now = std::time::Instant::now();

        self.entries.retain(|e| {
            if e.access_count == 0 {
                let age = now.duration_since(e.timestamp).as_secs();
                age < CONSOLIDATE_AGE_SECS
            } else {
                true
            }
        });

        let consolidated = n_before - self.entries.len();
        let total = self.entries.len();

        let (sum_v, sum_a) = if total > 0 {
            let sv: f64 = self.entries.iter().map(|e| e.emotion.valence).sum();
            let sa: f64 = self.entries.iter().map(|e| e.emotion.arousal).sum();
            (sv, sa)
        } else {
            (0.0, 0.0)
        };

        EmotionalMemoryStats {
            total_entries: total,
            consolidated,
            avg_valence: if total > 0 { sum_v / total as f64 } else { 0.0 },
            avg_arousal: if total > 0 { sum_a / total as f64 } else { 0.0 },
        }
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
}

impl Default for EmotionalMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tag(valence: f64, arousal: f64, label: &str) -> EmotionalMemoryTag {
        EmotionalMemoryTag {
            valence,
            arousal,
            dominance: 0.5,
            label: label.to_string(),
        }
    }

    #[test]
    fn test_store_and_retrieve() {
        let mut em = EmotionalMemory::new();
        em.store("happy day", tag(0.8, 0.6, "joy"));
        em.store("sad moment", tag(-0.7, 0.3, "sadness"));
        assert_eq!(em.entry_count(), 2);
    }

    #[test]
    fn test_eviction() {
        let mut em = EmotionalMemory::new();
        for i in 0..1001 {
            em.store(&format!("entry {}", i), tag(0.1, 0.1, "neutral"));
        }
        assert_eq!(em.entry_count(), 1000);
    }

    #[test]
    fn test_retrieve_by_emotion() {
        let mut em = EmotionalMemory::new();
        em.store("very happy", tag(0.9, 0.8, "joy"));
        em.store("neutral", tag(0.1, 0.1, "neutral"));
        em.store("very sad", tag(-0.8, 0.2, "sadness"));
        let strong = em.retrieve_by_emotion(0.5);
        assert_eq!(strong.len(), 2);
    }

    #[test]
    fn test_retrieve_recent() {
        let mut em = EmotionalMemory::new();
        em.store("first", tag(0.1, 0.1, "a"));
        em.store("second", tag(0.1, 0.1, "b"));
        em.store("third", tag(0.1, 0.1, "c"));
        let recent = em.retrieve_recent(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].text, "third");
        assert_eq!(recent[1].text, "second");
    }

    #[test]
    fn test_search_similar() {
        let mut em = EmotionalMemory::new();
        em.store("happy joyful day", tag(0.8, 0.6, "joy"));
        em.store("sad rainy day", tag(-0.6, 0.2, "sadness"));
        em.store("neutral weather", tag(0.1, 0.1, "neutral"));
        let results = em.search_similar("happy joyful", 2);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].text, "happy joyful day");
    }

    #[test]
    fn test_empty_state() {
        let em = EmotionalMemory::new();
        assert_eq!(em.entry_count(), 0);
        assert!(em.retrieve_by_emotion(0.5).is_empty());
        assert!(em.retrieve_recent(5).is_empty());
        assert!(em.search_similar("anything", 5).is_empty());
    }

    #[test]
    fn test_tick_consolidation() {
        let mut em = EmotionalMemory::new();
        // Entries with access_count=0 will be consolidated if old
        em.store("fresh", tag(0.5, 0.3, "a"));
        // Manually add an entry with old timestamp and zero access
        let old_stamp =
            std::time::Instant::now() - std::time::Duration::from_secs(CONSOLIDATE_AGE_SECS + 10);
        em.entries.push(EmotionalMemoryEntry {
            id: 999,
            timestamp: old_stamp,
            text: "stale".to_string(),
            emotion: tag(0.1, 0.1, "b"),
            vsa_hash: 0,
            access_count: 0,
            last_accessed: old_stamp,
        });
        let stats = em.tick();
        assert!(stats.consolidated >= 1);
        assert_eq!(em.entry_count(), 1);
        assert!(stats.avg_valence > 0.0);
    }

    #[test]
    fn test_entry_count_bounded() {
        let mut em = EmotionalMemory::new();
        for i in 0..2000 {
            em.store(&format!("entry {}", i), tag(0.1, 0.1, "x"));
        }
        assert_eq!(em.entry_count(), 1000);
    }

    #[test]
    fn test_search_similar_no_match() {
        let mut em = EmotionalMemory::new();
        em.store("cats and dogs", tag(0.5, 0.3, "pets"));
        let results = em.search_similar("quantum physics", 5);
        assert!(results.is_empty());
    }
}
