use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

// ── Config ──

#[derive(Debug, Clone)]
pub struct DecentMemConfig {
    pub e_pool_capacity: usize,
    pub x_pool_capacity: usize,
    pub lru_threshold: usize,
    pub vector_dim: usize,
    pub rl_learning_rate: f64,
    pub rl_exploration_rate: f64,
}

impl Default for DecentMemConfig {
    fn default() -> Self {
        Self {
            e_pool_capacity: 1000,
            x_pool_capacity: 500,
            lru_threshold: 3,
            vector_dim: 64,
            rl_learning_rate: 0.1,
            rl_exploration_rate: 0.2,
        }
    }
}

// ── SurpriseScorer ──

#[derive(Debug, Clone)]
pub struct SurpriseScorer {
    pub access_frequency_weight: f64,
    pub recency_weight: f64,
    pub content_novelty_weight: f64,
}

impl Default for SurpriseScorer {
    fn default() -> Self {
        Self {
            access_frequency_weight: 0.4,
            recency_weight: 0.3,
            content_novelty_weight: 0.3,
        }
    }
}

impl SurpriseScorer {
    pub fn new(
        access_frequency_weight: f64,
        recency_weight: f64,
        content_novelty_weight: f64,
    ) -> Self {
        Self {
            access_frequency_weight,
            recency_weight,
            content_novelty_weight,
        }
    }

    /// Score how surprising/valuable an entry is. Higher = more worth keeping.
    pub fn score(
        &self,
        entry: &EpisodicEntry,
        pool: &VecDeque<EpisodicEntry>,
        now_secs: u64,
    ) -> f64 {
        let max_access = pool.iter().map(|e| e.access_count).max().unwrap_or(0);
        let freq = if max_access > 0 {
            entry.access_count as f64 / max_access as f64
        } else {
            0.0
        };

        let max_age = pool
            .iter()
            .map(|e| now_secs.saturating_sub(e.timestamp))
            .max()
            .unwrap_or(0);
        let recency = if max_age > 0 {
            1.0 - (now_secs.saturating_sub(entry.timestamp) as f64 / max_age as f64)
        } else {
            1.0
        };

        let novelty = Self::compute_content_novelty(entry, pool);

        self.access_frequency_weight * freq
            + self.recency_weight * recency
            + self.content_novelty_weight * novelty
    }

    fn compute_content_novelty(entry: &EpisodicEntry, pool: &VecDeque<EpisodicEntry>) -> f64 {
        let words: HashSet<&str> = entry.content.split_whitespace().collect();
        if words.is_empty() || pool.len() <= 1 {
            return 1.0;
        }

        let mut total_sim = 0.0;
        let mut count = 0;
        for other in pool {
            if other.id == entry.id {
                continue;
            }
            let other_words: HashSet<&str> = other.content.split_whitespace().collect();
            let intersection = words.intersection(&other_words).count();
            let union = words.union(&other_words).count();
            if union > 0 {
                total_sim += intersection as f64 / union as f64;
                count += 1;
            }
        }

        if count == 0 {
            1.0
        } else {
            1.0 - (total_sim / count as f64)
        }
    }
}

// ── EpisodicEntry ──

#[derive(Debug, Clone)]
pub struct EpisodicEntry {
    pub id: u64,
    pub timestamp: u64,
    pub agent_id: String,
    pub content: String,
    pub context_tags: Vec<String>,
    pub access_count: usize,
}

// ── ExchangeEntry ──

#[derive(Debug, Clone)]
pub struct ExchangeEntry {
    pub id: u64,
    pub timestamp: u64,
    pub source_agent: String,
    pub content: String,
    pub tags: Vec<String>,
    pub embedding: Vec<f64>,
    pub relevance_score: f64,
}

// ── EPool ──

pub struct EPool {
    capacity: usize,
    entries: VecDeque<EpisodicEntry>,
    access_count: HashMap<u64, usize>,
    next_id: u64,
}

impl EPool {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            entries: VecDeque::with_capacity(capacity),
            access_count: HashMap::new(),
            next_id: 1,
        }
    }
}

// ── XPool ──

pub struct XPool {
    capacity: usize,
    entries: Vec<ExchangeEntry>,
    tag_index: HashMap<String, Vec<usize>>,
    next_id: u64,
}

impl XPool {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            entries: Vec::with_capacity(capacity),
            tag_index: HashMap::new(),
            next_id: 1,
        }
    }
}

// ── RLMemoryPolicy ──

type StateKey = (u8, u8, u8, u8);

#[derive(Debug, Clone)]
pub struct RLMemoryPolicy {
    q_table: HashMap<StateKey, [f64; 2]>,
    pub learning_rate: f64,
    pub exploration_rate: f64,
    discount_factor: f64,
}

impl RLMemoryPolicy {
    pub fn new(learning_rate: f64, exploration_rate: f64) -> Self {
        Self {
            q_table: HashMap::new(),
            learning_rate,
            exploration_rate,
            discount_factor: 0.9,
        }
    }

    pub fn discretize_surprise(score: f64) -> u8 {
        if score < 0.25 {
            0
        } else if score < 0.5 {
            1
        } else if score < 0.75 {
            2
        } else {
            3
        }
    }

    pub fn discretize_access(count: usize) -> u8 {
        match count {
            0 => 0,
            1 => 1,
            2 => 2,
            3..=5 => 3,
            _ => 4,
        }
    }

    pub fn discretize_age(age_ratio: f64) -> u8 {
        if age_ratio < 0.25 {
            0
        } else if age_ratio < 0.5 {
            1
        } else if age_ratio < 0.75 {
            2
        } else {
            3
        }
    }

    pub fn discretize_length(len: usize) -> u8 {
        match len {
            0..=50 => 0,
            51..=200 => 1,
            _ => 2,
        }
    }

    pub fn compute_state(
        entry: &EpisodicEntry,
        pool: &VecDeque<EpisodicEntry>,
        scorer: &SurpriseScorer,
        now_secs: u64,
    ) -> StateKey {
        let access_bucket = Self::discretize_access(entry.access_count);

        let max_age = pool
            .iter()
            .map(|e| now_secs.saturating_sub(e.timestamp))
            .max()
            .unwrap_or(0);
        let age_ratio = if max_age > 0 {
            (now_secs.saturating_sub(entry.timestamp) as f64 / max_age as f64).min(1.0)
        } else {
            0.0
        };
        let age_bucket = Self::discretize_age(age_ratio);

        let surprise = scorer.score(entry, pool, now_secs);
        let surprise_bucket = Self::discretize_surprise(surprise);
        let len_bucket = Self::discretize_length(entry.content.len());

        (access_bucket, age_bucket, surprise_bucket, len_bucket)
    }

    /// Decision-time Q value for eviction: learned value if available,
    /// otherwise a sensible prior based on access count and age.
    fn get_evict_q(&self, state: &StateKey) -> f64 {
        let (access, age, _, _) = state;
        if let Some(q) = self.q_table.get(state) {
            return q[1];
        }
        // Prior: low access + high age = better eviction candidate
        let access_prior = match access {
            0 => 0.5,
            1 => 0.2,
            2 => 0.0,
            _ => -0.3,
        };
        let age_prior = if *age >= 2 { 0.2 } else { 0.0 };
        access_prior + age_prior
    }

    /// Decide which entry to evict using epsilon-greedy Q-learning policy.
    /// Returns the index of the entry to evict.
    pub fn decide_eviction(
        &self,
        pool: &VecDeque<EpisodicEntry>,
        scorer: &SurpriseScorer,
        now_secs: u64,
    ) -> usize {
        if pool.is_empty() {
            return 0;
        }

        if self.exploration_rate > 0.0 && fastrand::f64() < self.exploration_rate {
            return fastrand::usize(..pool.len());
        }

        let mut best_idx = 0;
        let mut best_value = f64::NEG_INFINITY;

        for (i, entry) in pool.iter().enumerate() {
            let state = Self::compute_state(entry, pool, scorer, now_secs);
            let q_evict = self.get_evict_q(&state);
            if q_evict > best_value {
                best_value = q_evict;
                best_idx = i;
            }
        }

        best_idx
    }

    /// Q-learning update: Q(s,a) += lr * (reward + discount * max_a' Q(s',a') - Q(s,a))
    pub fn train(&mut self, reward: f64, state: StateKey, action: u8, next_state: StateKey) {
        let next_q = self.q_table.get(&next_state).copied().unwrap_or([0.0; 2]);
        let max_next = next_q[0].max(next_q[1]);
        let q = self.q_table.entry(state).or_insert([0.0; 2]);
        let td_error = reward + self.discount_factor * max_next - q[action as usize];
        q[action as usize] += self.learning_rate * td_error;
    }
}

// ── MemStats ──

#[derive(Debug, Clone, PartialEq)]
pub struct MemStats {
    pub e_pool_entries: usize,
    pub x_pool_entries: usize,
    pub e_pool_capacity: usize,
    pub x_pool_capacity: usize,
    pub total_accesses: u64,
    pub consolidation_count: u64,
}

// ── DecentMem ──

pub struct DecentMem {
    pub agent_id: String,
    pub e_pool: EPool,
    pub x_pool: XPool,
    pub config: DecentMemConfig,
    pub scorer: SurpriseScorer,
    pub rl_policy: RLMemoryPolicy,
    consolidation_count: u64,
}

impl DecentMem {
    pub fn new(agent_id: &str, config: DecentMemConfig) -> Self {
        Self {
            agent_id: agent_id.to_string(),
            e_pool: EPool::new(config.e_pool_capacity),
            x_pool: XPool::new(config.x_pool_capacity),
            consolidation_count: 0,
            scorer: SurpriseScorer::default(),
            rl_policy: RLMemoryPolicy::new(config.rl_learning_rate, config.rl_exploration_rate),
            config,
        }
    }

    /// Store an episodic memory entry.
    pub fn store_episodic(&mut self, content: &str, tags: &[String]) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let id = self.e_pool.next_id;
        self.e_pool.next_id += 1;

        if self.e_pool.entries.len() >= self.e_pool.capacity {
            self.evict_episodic();
        }

        self.e_pool.access_count.insert(id, 0);
        self.e_pool.entries.push_back(EpisodicEntry {
            id,
            timestamp: now,
            agent_id: self.agent_id.clone(),
            content: content.to_string(),
            context_tags: tags.to_vec(),
            access_count: 0,
        });
        id
    }

    fn evict_episodic(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if self.e_pool.entries.is_empty() {
            return;
        }

        let evict_idx = self
            .rl_policy
            .decide_eviction(&self.e_pool.entries, &self.scorer, now);

        let evict_idx = evict_idx.min(self.e_pool.entries.len().saturating_sub(1));
        let evict_id = self.e_pool.entries.remove(evict_idx).unwrap().id;
        self.e_pool.access_count.remove(&evict_id);
    }

    /// Store an exchange (cross-agent) memory entry.
    pub fn store_exchange(&mut self, content: &str, tags: &[String], embedding: &[f64]) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let id = self.x_pool.next_id;
        self.x_pool.next_id += 1;

        if self.x_pool.entries.len() >= self.x_pool.capacity {
            self.evict_exchange();
        }

        let idx = self.x_pool.entries.len();
        for tag in tags {
            self.x_pool
                .tag_index
                .entry(tag.clone())
                .or_default()
                .push(idx);
        }

        self.x_pool.entries.push(ExchangeEntry {
            id,
            timestamp: now,
            source_agent: self.agent_id.clone(),
            content: content.to_string(),
            tags: tags.to_vec(),
            embedding: embedding.to_vec(),
            relevance_score: 0.5,
        });
        id
    }

    fn evict_exchange(&mut self) {
        if self.x_pool.entries.is_empty() {
            return;
        }
        let (evict_idx, _) = self
            .x_pool
            .entries
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                a.relevance_score
                    .partial_cmp(&b.relevance_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap();

        self.x_pool.entries.swap_remove(evict_idx);

        // Rebuild tag index after removal
        self.x_pool.tag_index.clear();
        for (i, e) in self.x_pool.entries.iter().enumerate() {
            for tag in &e.tags {
                self.x_pool
                    .tag_index
                    .entry(tag.clone())
                    .or_default()
                    .push(i);
            }
        }
    }

    /// Recall episodic entries by substring match on content or tags.
    pub fn recall_episodic(&mut self, query: &str, max_results: usize) -> Vec<&EpisodicEntry> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let ids: Vec<u64> = self
            .e_pool
            .entries
            .iter()
            .filter(|e| {
                e.content.contains(query) || e.context_tags.iter().any(|t| t.contains(query))
            })
            .map(|e| e.id)
            .collect();

        for id in &ids {
            *self.e_pool.access_count.entry(*id).or_insert(0) += 1;
        }
        for entry in self.e_pool.entries.iter_mut() {
            if ids.contains(&entry.id) {
                entry.access_count = *self.e_pool.access_count.get(&entry.id).unwrap_or(&0);
            }
        }

        let mut training_states: Vec<StateKey> = Vec::new();
        let mut match_count = 0;
        for entry in self.e_pool.entries.iter() {
            if ids.contains(&entry.id) && match_count < max_results {
                let state =
                    RLMemoryPolicy::compute_state(entry, &self.e_pool.entries, &self.scorer, now);
                training_states.push(state);
                match_count += 1;
            }
        }
        for state in &training_states {
            self.rl_policy.train(0.5, *state, 0, *state);
        }

        let mut results: Vec<&EpisodicEntry> = Vec::new();
        for entry in self.e_pool.entries.iter() {
            if ids.contains(&entry.id) && results.len() < max_results {
                results.push(entry);
            }
        }
        results
    }

    /// Recall exchange entries by tag matching.
    pub fn recall_exchange(&self, tags: &[String], max_results: usize) -> Vec<&ExchangeEntry> {
        if tags.is_empty() {
            return Vec::new();
        }
        let mut seen = std::collections::HashSet::new();
        let mut results = Vec::new();
        for tag in tags {
            if let Some(indices) = self.x_pool.tag_index.get(tag) {
                for &idx in indices {
                    if seen.insert(idx) && results.len() < max_results {
                        results.push(&self.x_pool.entries[idx]);
                    }
                }
            }
        }
        results
    }

    /// Consolidate: promote frequently-accessed episodic entries into the exchange pool.
    pub fn consolidate(&mut self) {
        let threshold = self.config.lru_threshold;
        let mut promoted: Vec<(EpisodicEntry, usize)> = Vec::new();

        let mut keep = VecDeque::new();
        while let Some(mut entry) = self.e_pool.entries.pop_front() {
            let entry_id = entry.id;
            let count = self
                .e_pool
                .access_count
                .get(&entry_id)
                .copied()
                .unwrap_or(0);
            if count >= threshold {
                entry.access_count = count;
                self.e_pool.access_count.remove(&entry_id);
                promoted.push((entry, count));
            } else {
                keep.push_back(entry);
            }
        }
        self.e_pool.entries = keep;

        for (entry, _) in promoted {
            if self.x_pool.entries.len() >= self.x_pool.capacity {
                self.evict_exchange();
            }
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let exchange = ExchangeEntry {
                id: self.x_pool.next_id,
                timestamp: now,
                source_agent: entry.agent_id,
                content: entry.content,
                tags: entry.context_tags,
                embedding: vec![0.0; self.config.vector_dim],
                relevance_score: 0.5,
            };
            self.x_pool.next_id += 1;
            let idx = self.x_pool.entries.len();
            for tag in &exchange.tags {
                self.x_pool
                    .tag_index
                    .entry(tag.clone())
                    .or_default()
                    .push(idx);
            }
            self.x_pool.entries.push(exchange);
        }

        self.consolidation_count += 1;
    }

    /// Return current memory statistics.
    pub fn stats(&self) -> MemStats {
        let total_accesses: u64 = self.e_pool.access_count.values().map(|&v| v as u64).sum();
        MemStats {
            e_pool_entries: self.e_pool.entries.len(),
            x_pool_entries: self.x_pool.entries.len(),
            e_pool_capacity: self.e_pool.capacity,
            x_pool_capacity: self.x_pool.capacity,
            total_accesses,
            consolidation_count: self.consolidation_count,
        }
    }
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    fn small_config() -> DecentMemConfig {
        DecentMemConfig {
            e_pool_capacity: 5,
            x_pool_capacity: 5,
            lru_threshold: 3,
            vector_dim: 64,
            rl_learning_rate: 0.1,
            rl_exploration_rate: 0.0,
        }
    }

    #[test]
    fn test_store_recall_episodic() {
        let mut dm = DecentMem::new("agent_a", small_config());
        dm.store_episodic("the cat sat on the mat", &[]);
        dm.store_episodic("the dog ran in the park", &[]);
        let results = dm.recall_episodic("cat", 10);
        assert_eq!(results.len(), 1);
        assert!(results[0].content.contains("cat"));
    }

    #[test]
    fn test_store_recall_episodic_by_tag() {
        let mut dm = DecentMem::new("agent_a", small_config());
        dm.store_episodic(
            "weather report",
            &["weather".to_string(), "daily".to_string()],
        );
        dm.store_episodic("stock market", &["finance".to_string()]);
        let results = dm.recall_episodic("weath", 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "weather report");
    }

    #[test]
    fn test_store_recall_exchange() {
        let mut dm = DecentMem::new("agent_a", small_config());
        let emb = vec![0.1, 0.2, 0.3, 0.4];
        dm.store_exchange("hello from agent b", &["greeting".to_string()], &emb);
        dm.store_exchange("task completed", &["status".to_string()], &emb);
        let results = dm.recall_exchange(&["greeting".to_string()], 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "hello from agent b");
    }

    #[test]
    fn test_epool_fifo_eviction() {
        let mut dm = DecentMem::new("agent_a", small_config());
        for i in 0..5 {
            dm.store_episodic(&format!("entry {}", i), &[]);
        }
        assert_eq!(dm.e_pool.entries.len(), 5);
        dm.store_episodic("new entry", &[]);
        assert_eq!(dm.e_pool.entries.len(), 5);
        let results = dm.recall_episodic("entry 0", 10);
        assert_eq!(results.len(), 0, "entry 0 should have been evicted");
        let found = dm.recall_episodic("entry 1", 10);
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn test_epool_eviction_high_access_keeps_oldest() {
        let mut dm = DecentMem::new(
            "agent_a",
            DecentMemConfig {
                e_pool_capacity: 3,
                x_pool_capacity: 5,
                lru_threshold: 3,
                vector_dim: 64,
                rl_learning_rate: 0.1,
                rl_exploration_rate: 0.0,
            },
        );
        dm.store_episodic("alpha", &[]);
        dm.store_episodic("beta", &[]);
        dm.store_episodic("gamma", &[]);
        for _ in 0..3 {
            dm.recall_episodic("alpha", 10);
        }
        dm.store_episodic("delta", &[]);
        let r = dm.recall_episodic("beta", 10);
        assert_eq!(r.len(), 0, "beta should have been evicted");
        let r = dm.recall_episodic("alpha", 10);
        assert_eq!(r.len(), 1, "alpha should still be present");
    }

    #[test]
    fn test_xpool_tag_retrieval() {
        let mut dm = DecentMem::new("agent_a", small_config());
        let emb = vec![0.5; 64];
        dm.store_exchange("msg one", &["a".to_string(), "b".to_string()], &emb);
        dm.store_exchange("msg two", &["b".to_string(), "c".to_string()], &emb);
        dm.store_exchange("msg three", &["c".to_string()], &emb);
        let results = dm.recall_exchange(&["a".to_string(), "c".to_string()], 10);
        assert_eq!(results.len(), 2);
        let contents: Vec<&str> = results.iter().map(|e| e.content.as_str()).collect();
        assert!(contents.contains(&"msg one"));
        assert!(contents.contains(&"msg three"));
    }

    #[test]
    fn test_consolidation_promotion() {
        let mut dm = DecentMem::new("agent_a", small_config());
        for i in 0..4 {
            dm.store_episodic(&format!("entry {}", i), &["tag_a".to_string()]);
        }
        for _ in 0..3 {
            dm.recall_episodic("entry 0", 10);
        }
        let stats_before = dm.stats();
        assert_eq!(stats_before.e_pool_entries, 4);
        assert_eq!(stats_before.x_pool_entries, 0);

        dm.consolidate();

        let stats_after = dm.stats();
        assert_eq!(stats_after.e_pool_entries, 3, "entry 0 promoted away");
        assert_eq!(
            stats_after.x_pool_entries, 1,
            "one entry promoted to X-pool"
        );
        assert_eq!(stats_after.consolidation_count, 1);
        let results = dm.recall_exchange(&["tag_a".to_string()], 10);
        assert_eq!(results.len(), 1);
        assert!(results[0].content.contains("entry 0"));
    }

    #[test]
    fn test_empty_pool_episodic() {
        let mut dm = DecentMem::new("agent_a", small_config());
        let results = dm.recall_episodic("anything", 10);
        assert!(results.is_empty());
    }

    #[test]
    fn test_empty_pool_exchange() {
        let dm = DecentMem::new("agent_a", small_config());
        let results = dm.recall_exchange(&["tag".to_string()], 10);
        assert!(results.is_empty());
    }

    #[test]
    fn test_stats_accuracy() {
        let mut dm = DecentMem::new("agent_a", small_config());
        let s0 = dm.stats();
        assert_eq!(s0.e_pool_entries, 0);
        assert_eq!(s0.x_pool_entries, 0);
        assert_eq!(s0.e_pool_capacity, 5);
        assert_eq!(s0.x_pool_capacity, 5);
        assert_eq!(s0.total_accesses, 0);
        assert_eq!(s0.consolidation_count, 0);

        dm.store_episodic("hello", &["greet".to_string()]);
        dm.store_episodic("world", &[]);
        dm.recall_episodic("hello", 10);
        dm.recall_episodic("hello", 10);
        dm.recall_episodic("world", 10);

        let s1 = dm.stats();
        assert_eq!(s1.e_pool_entries, 2);
        assert_eq!(s1.total_accesses, 3);

        let emb = vec![0.0; 64];
        dm.store_exchange("exchange one", &["x".to_string()], &emb);
        let s2 = dm.stats();
        assert_eq!(s2.x_pool_entries, 1);
        assert_eq!(s2.e_pool_entries, 2);

        dm.consolidate();
        let s3 = dm.stats();
        assert_eq!(s3.consolidation_count, 1);
    }

    #[test]
    fn test_xpool_eviction_lowest_relevance() {
        let mut dm = DecentMem::new(
            "agent_a",
            DecentMemConfig {
                e_pool_capacity: 5,
                x_pool_capacity: 3,
                lru_threshold: 3,
                vector_dim: 64,
                rl_learning_rate: 0.1,
                rl_exploration_rate: 0.0,
            },
        );
        let emb = vec![0.0; 64];
        dm.store_exchange("entry a", &["t1".to_string()], &emb);
        dm.store_exchange("entry b", &["t1".to_string()], &emb);
        dm.store_exchange("entry c", &["t1".to_string()], &emb);
        dm.store_exchange("entry d", &["t2".to_string()], &emb);
        assert_eq!(dm.x_pool.entries.len(), 3);
        // All have same relevance 0.5; min_by returns the first encountered (index 0, "entry a")
        let results = dm.recall_exchange(&["t1".to_string()], 10);
        assert_eq!(results.len(), 2);
        assert!(!results.iter().any(|e| e.content == "entry a"));
    }

    #[test]
    fn test_surprise_scorer_frequency() {
        let scorer = SurpriseScorer::new(1.0, 0.0, 0.0);
        let now = 1000;
        let mut pool = VecDeque::new();
        pool.push_back(EpisodicEntry {
            id: 1,
            timestamp: 100,
            agent_id: "a".into(),
            content: "frequent".into(),
            context_tags: vec![],
            access_count: 10,
        });
        pool.push_back(EpisodicEntry {
            id: 2,
            timestamp: 100,
            agent_id: "a".into(),
            content: "rare".into(),
            context_tags: vec![],
            access_count: 0,
        });

        let s_high = scorer.score(&pool[0], &pool, now);
        let s_low = scorer.score(&pool[1], &pool, now);
        assert!(s_high > s_low, "frequent entry should score higher");
        assert!((s_high - 1.0).abs() < 1e-9, "freq=1.0 * weight=1.0");
        assert!((s_low - 0.0).abs() < 1e-9, "freq=0.0 * weight=1.0");
    }

    #[test]
    fn test_rl_policy_learns_eviction() {
        let mut policy = RLMemoryPolicy::new(0.5, 0.0);
        let scorer = SurpriseScorer::default();
        let now = 1000;

        let mut pool = VecDeque::new();
        pool.push_back(EpisodicEntry {
            id: 1,
            timestamp: 100,
            agent_id: "a".into(),
            content: "low value".into(),
            context_tags: vec![],
            access_count: 0,
        });
        pool.push_back(EpisodicEntry {
            id: 2,
            timestamp: 200,
            agent_id: "a".into(),
            content: "high value".into(),
            context_tags: vec![],
            access_count: 5,
        });

        let idx = policy.decide_eviction(&pool, &scorer, now);
        assert_eq!(idx, 0, "prior prefers low-access entry");

        let state0 = RLMemoryPolicy::compute_state(&pool[0], &pool, &scorer, now);
        policy.train(-2.0, state0, 1, state0);

        let idx2 = policy.decide_eviction(&pool, &scorer, now);
        assert_eq!(idx2, 1, "policy learned to avoid evicting state 0");
    }

    #[test]
    fn test_rl_integration_store_recall() {
        let mut dm = DecentMem::new("test_agent", small_config());
        let id1 = dm.store_episodic("hello world", &["greeting".to_string()]);
        let id2 = dm.store_episodic("goodbye world", &["farewell".to_string()]);

        let results = dm.recall_episodic("hello", 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "hello world");
        assert_eq!(results[0].id, id1);

        let results2 = dm.recall_episodic("goodbye", 10);
        assert_eq!(results2.len(), 1);
        assert_eq!(results2[0].content, "goodbye world");
        assert_eq!(results2[0].id, id2);

        let results3 = dm.recall_episodic("nonexistent", 10);
        assert!(results3.is_empty());

        // RL policy was trained during recall calls
        assert!(
            !dm.rl_policy.q_table.is_empty(),
            "RL q_table should have entries"
        );
    }

    #[test]
    fn test_rl_store_triggers_eviction() {
        let mut dm = DecentMem::new(
            "test_agent",
            DecentMemConfig {
                e_pool_capacity: 3,
                x_pool_capacity: 5,
                lru_threshold: 3,
                vector_dim: 64,
                rl_learning_rate: 0.1,
                rl_exploration_rate: 0.0,
            },
        );
        dm.store_episodic("entry a", &[]);
        dm.store_episodic("entry b", &[]);
        dm.store_episodic("entry c", &[]);
        assert_eq!(dm.e_pool.entries.len(), 3);

        dm.store_episodic("entry d", &[]);
        assert_eq!(
            dm.e_pool.entries.len(),
            3,
            "eviction should keep pool at capacity"
        );
        let results = dm.recall_episodic("entry a", 10);
        assert_eq!(results.len(), 0, "oldest entry (prior) evicted");
    }

    #[test]
    fn test_config_defaults() {
        let config = DecentMemConfig::default();
        assert!((config.rl_learning_rate - 0.1).abs() < 1e-6);
        assert!((config.rl_exploration_rate - 0.2).abs() < 1e-6);
    }
}
