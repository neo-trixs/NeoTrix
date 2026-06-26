use chrono::Utc;
use std::collections::HashMap;
use std::f64;

const MAX_ENTRIES: usize = 10000;

/// Memory tier: L1=Working, L2=Episodic, L3=Semantic, L4=Consolidated, L5=Fleet
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryTier {
    L1,
    L2,
    L3,
    L4,
    L5,
}

impl MemoryTier {
    pub fn retention(&self) -> f64 {
        match self {
            MemoryTier::L1 => 0.3,
            MemoryTier::L2 => 0.6,
            MemoryTier::L3 => 0.8,
            MemoryTier::L4 => 0.95,
            MemoryTier::L5 => 0.99,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemoryEntry {
    pub id: String,
    pub content: String,
    pub tier: MemoryTier,
    pub created_at: u64,
    pub last_accessed: u64,
    pub access_count: u64,
    pub reward: f64,
    pub embedding: Vec<f64>,
    pub source_agent: Option<String>,
    pub tags: Vec<String>,
    pub vsa: Option<crate::core::nt_core_hcube::VsaVector<4096>>,
}

pub struct CognitiveMemory {
    entries: Vec<MemoryEntry>,
    cross_agent_agreement: HashMap<String, usize>,
}

impl Default for CognitiveMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl CognitiveMemory {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            cross_agent_agreement: HashMap::new(),
        }
    }

    pub fn store(
        &mut self,
        content: &str,
        tier: MemoryTier,
        source_agent: Option<&str>,
        tags: Vec<String>,
    ) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        if self.entries.len() >= MAX_ENTRIES {
            self.entries.remove(0);
        }
        self.entries.push(MemoryEntry {
            id: id.clone(),
            content: content.to_string(),
            tier,
            created_at: Utc::now().timestamp() as u64,
            last_accessed: Utc::now().timestamp() as u64,
            access_count: 0,
            reward: 0.0,
            embedding: Self::simple_embed(content),
            source_agent: source_agent.map(|s| s.to_string()),
            tags,
            vsa: None,
        });
        id
    }

    pub fn retrieve(&mut self, query: &str, top_k: usize) -> Vec<usize> {
        let query_emb = Self::simple_embed(query);
        let mut scored: Vec<usize> = (0..self.entries.len()).collect();
        scored.sort_by(|&a, &b| {
            let sa = self.score_entry(a, &query_emb);
            let sb = self.score_entry(b, &query_emb);
            sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
        });
        let indices: Vec<usize> = scored.into_iter().take(top_k).collect();
        for &i in &indices {
            self.entries[i].access_count += 1;
            self.entries[i].last_accessed = Utc::now().timestamp() as u64;
        }
        indices
    }

    fn score_entry(&self, idx: usize, query_emb: &[f64]) -> f64 {
        let e = &self.entries[idx];
        let sim = Self::cosine_sim(&e.embedding, query_emb);
        sim * e.tier.retention() * (0.5 + e.reward * 0.5)
    }

    fn cosine_sim(a: &[f64], b: &[f64]) -> f64 {
        let dot: f64 = a.iter().zip(b).map(|(x, y)| x * y).sum();
        let na: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let nb: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
        if na == 0.0 || nb == 0.0 {
            0.0
        } else {
            dot / (na * nb)
        }
    }

    fn simple_embed(text: &str) -> Vec<f64> {
        let chars: Vec<char> = text.chars().collect();
        let mut emb = vec![0.0; 16];
        for (i, &c) in chars.iter().enumerate() {
            emb[i % 16] += c as u8 as f64 / 255.0;
        }
        let norm: f64 = emb.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm > 0.0 {
            for v in &mut emb {
                *v /= norm;
            }
        }
        emb
    }

    pub fn consolidate(&mut self, promotion_threshold: f64) -> usize {
        let mut promoted = 0;
        for entry in &mut self.entries {
            if entry.reward > promotion_threshold && entry.access_count >= 3 {
                let new_tier = match entry.tier {
                    MemoryTier::L1 => {
                        promoted += 1;
                        MemoryTier::L2
                    }
                    MemoryTier::L2 => {
                        promoted += 1;
                        MemoryTier::L3
                    }
                    MemoryTier::L3 => {
                        promoted += 1;
                        MemoryTier::L4
                    }
                    other => other,
                };
                entry.tier = new_tier;
            }
        }
        promoted
    }

    pub fn active_consolidation(&mut self, staleness_threshold_secs: u64) -> (usize, usize, usize) {
        let now = Utc::now().timestamp() as u64;
        let mut removed = 0;
        let mut merged = 0;
        self.entries.retain(|e| {
            if e.tier == MemoryTier::L1
                && now.saturating_sub(e.created_at) > staleness_threshold_secs
            {
                removed += 1;
                return false;
            }
            true
        });
        let mut i = 0;
        while i < self.entries.len() {
            let mut j = i + 1;
            while j < self.entries.len() {
                if Self::cosine_sim(&self.entries[i].embedding, &self.entries[j].embedding) > 0.85 {
                    let other_count = self.entries[j].access_count;
                    self.entries[i].access_count += other_count;
                    self.entries.remove(j);
                    merged += 1;
                } else {
                    j += 1;
                }
            }
            i += 1;
        }
        (removed, merged, self.consolidate(0.7))
    }

    pub fn theory_of_mind(&self, agent_name: &str) -> Vec<&MemoryEntry> {
        self.entries
            .iter()
            .filter(|e| e.source_agent.as_deref() == Some(agent_name))
            .collect()
    }

    pub fn flash_reasoning(&self, start_tag: &str, max_depth: usize) -> Vec<Vec<&MemoryEntry>> {
        let mut paths = Vec::new();
        let start: Vec<&MemoryEntry> = self
            .entries
            .iter()
            .filter(|e| e.tags.iter().any(|t| t.contains(start_tag)))
            .collect();
        for entry in start {
            let mut path = vec![entry];
            let mut depth = 0;
            while depth < max_depth {
                let last = path.last().copied().expect("trace_association: path always has >=1 entry because it starts with seed");
                let next = self
                    .entries
                    .iter()
                    .filter(|e| e.tags.iter().any(|t| last.tags.contains(t)) && e.id != last.id)
                    .max_by(|a, b| {
                        a.reward
                            .partial_cmp(&b.reward)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                if let Some(n) = next {
                    path.push(n);
                    depth += 1;
                } else {
                    break;
                }
            }
            paths.push(path);
        }
        paths
    }

    pub fn cross_agent_synthesis(&mut self) -> Vec<String> {
        let mut synthesized = Vec::new();
        for entry in &self.entries {
            if let Some(ref _agent) = entry.source_agent {
                let key = format!("{}:{}", entry.content, entry.tier as u8);
                let count = self.cross_agent_agreement.entry(key).or_insert(0);
                *count += 1;
                if *count >= 3 && entry.tier != MemoryTier::L5 {
                    synthesized.push(entry.id.clone());
                }
            }
        }
        for id in &synthesized {
            if let Some(e) = self.entries.iter_mut().find(|e| e.id == *id) {
                e.tier = MemoryTier::L5;
            }
        }
        synthesized
    }

    pub fn provide_reward(&mut self, id: &str, reward: f64) -> bool {
        if let Some(e) = self.entries.iter_mut().find(|e| e.id == id) {
            e.reward = e.reward * 0.7 + reward * 0.3;
            true
        } else {
            false
        }
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }
    pub fn count_by_tier(&self, tier: MemoryTier) -> usize {
        self.entries.iter().filter(|e| e.tier == tier).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_retrieve() {
        let mut mem = CognitiveMemory::new();
        let id1 = mem.store("hello world", MemoryTier::L2, None, vec![]);
        let id2 = mem.store("goodbye world", MemoryTier::L2, None, vec![]);
        assert_eq!(mem.entry_count(), 2);
        let results = mem.retrieve("hello", 5);
        assert_eq!(results.len(), 2);
        assert_eq!(mem.entries[results[0]].content, "hello world");
        assert_eq!(mem.entries[results[1]].content, "goodbye world");
        let _ = id1;
        let _ = id2;
    }

    #[test]
    fn test_rl_consolidation() {
        let mut mem = CognitiveMemory::new();
        let id = mem.store("persistent knowledge", MemoryTier::L1, None, vec![]);
        // EMA needs ~5 iterations of 0.9 to cross 0.7 threshold
        mem.provide_reward(&id, 1.0);
        mem.provide_reward(&id, 1.0);
        mem.provide_reward(&id, 1.0);
        mem.provide_reward(&id, 1.0);
        for _ in 0..3 {
            mem.retrieve("persistent", 5);
        }
        let promoted = mem.consolidate(0.7);
        assert_eq!(promoted, 1);
        assert_eq!(mem.count_by_tier(MemoryTier::L2), 1);
    }

    #[test]
    fn test_active_consolidation() {
        let mut mem = CognitiveMemory::new();
        mem.store("unique knowledge", MemoryTier::L2, None, vec![]);
        mem.store("unique knowledge", MemoryTier::L2, None, vec![]);
        assert_eq!(mem.entry_count(), 2);
        let (removed, merged, promoted) = mem.active_consolidation(3600);
        assert_eq!(removed, 0);
        assert!(merged >= 1);
        assert_eq!(promoted, 0);
    }

    #[test]
    fn test_flash_reasoning() {
        let mut mem = CognitiveMemory::new();
        mem.store(
            "alpha concept",
            MemoryTier::L3,
            None,
            vec!["math".into(), "alpha".into()],
        );
        mem.store(
            "beta concept",
            MemoryTier::L3,
            None,
            vec!["math".into(), "beta".into()],
        );
        mem.store(
            "gamma concept",
            MemoryTier::L3,
            None,
            vec!["science".into(), "gamma".into()],
        );
        let paths = mem.flash_reasoning("alpha", 2);
        assert!(!paths.is_empty());
        // first path should start with alpha concept
        assert!(paths[0][0].tags.contains(&"alpha".to_string()));
    }

    #[test]
    fn test_cross_agent_synthesis() {
        let mut mem = CognitiveMemory::new();
        let content = "shared truth";
        mem.store(content, MemoryTier::L3, Some("agent-a"), vec![]);
        mem.store(content, MemoryTier::L3, Some("agent-b"), vec![]);
        mem.store(content, MemoryTier::L3, Some("agent-c"), vec![]);
        let synthesized = mem.cross_agent_synthesis();
        assert!(!synthesized.is_empty());
        // At least one entry should be promoted to L5
        assert!(mem.count_by_tier(MemoryTier::L5) >= 1);
    }

    #[test]
    fn test_theory_of_mind() {
        let mut mem = CognitiveMemory::new();
        mem.store("agent a data", MemoryTier::L2, Some("agent-a"), vec![]);
        mem.store("agent b data", MemoryTier::L2, Some("agent-b"), vec![]);
        mem.store("more a data", MemoryTier::L2, Some("agent-a"), vec![]);
        let agent_a_knowledge = mem.theory_of_mind("agent-a");
        assert_eq!(agent_a_knowledge.len(), 2);
        let agent_b_knowledge = mem.theory_of_mind("agent-b");
        assert_eq!(agent_b_knowledge.len(), 1);
    }
}
