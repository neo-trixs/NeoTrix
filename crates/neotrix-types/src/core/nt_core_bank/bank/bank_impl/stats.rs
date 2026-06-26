use std::collections::VecDeque;
use std::sync::atomic::Ordering;

use chrono::Utc;

use super::ReasoningBank;
use crate::core::nt_core_bank::{
    LifecycleAction, LifecycleConfig, MemoryDetailedStats, MemoryIterationResult, MemoryTier,
    ReasoningBankStats, ReasoningMemory,
};

impl ReasoningBank {
    pub fn stats(&self) -> ReasoningBankStats {
        let total = self.memories.len();
        let successes = self.memories.iter().filter(|m| m.success).count();
        ReasoningBankStats {
            total_memories: total,
            success_count: successes,
            success_rate: if total > 0 { successes as f64 / total as f64 } else { 0.0 },
        }
    }

    pub fn stats_detailed(&self) -> MemoryDetailedStats {
        let total = self.memories.len();
        let mut tier_working = 0;
        let mut tier_episodic = 0;
        let mut tier_semantic = 0;
        let mut tier_procedural = 0;
        let mut total_confidence = 0.0;
        let mut total_importance = 0.0;
        for m in &self.memories {
            match m.tier {
                MemoryTier::Working => tier_working += 1,
                MemoryTier::Episodic => tier_episodic += 1,
                MemoryTier::Semantic => tier_semantic += 1,
                MemoryTier::Procedural => tier_procedural += 1,
            }
            total_confidence += m.lifecycle.confidence;
            total_importance += m.lifecycle.importance;
        }
        MemoryDetailedStats {
            total,
            tier_working, tier_episodic, tier_semantic, tier_procedural,
            avg_confidence: if total > 0 { total_confidence / total as f64 } else { 0.0 },
            avg_importance: if total > 0 { total_importance / total as f64 } else { 0.0 },
        }
    }

    pub fn memories(&self) -> &VecDeque<ReasoningMemory> { &self.memories }

    pub fn iterate_memories(&mut self, similarity_threshold: f64, min_reward: f64) -> MemoryIterationResult {
        let before = self.stats();
        let merged = self.consolidate_similar(similarity_threshold);
        let pruned = self.prune_low_value(min_reward);
        let replayed = self.replay_high_value();
        let promoted = self.promote_tiers();
        let expired = self.evict_expired();
        self.bm25_dirty.store(true, Ordering::SeqCst);
        MemoryIterationResult {
            before,
            after: self.stats(),
            merged_count: merged,
            pruned_count: pruned,
            replayed_count: replayed,
            promoted_count: promoted,
            expired_count: expired,
        }
    }

    pub fn promote_tiers(&mut self) -> usize {
        let now = Utc::now().timestamp();
        let mut promoted = 0;
        for m in &mut self.memories {
            let age_hours = (now - m.timestamp) as f64 / 3600.0;
            let should_promote = match m.tier {
                MemoryTier::Working => age_hours > 1.0 || m.lifecycle.access_count >= 3,
                MemoryTier::Episodic => age_hours > 24.0 || m.lifecycle.access_count >= 10,
                MemoryTier::Semantic => age_hours > 168.0 || m.lifecycle.access_count >= 30,
                MemoryTier::Procedural => false,
            };
            if should_promote {
                if let Some(new_tier) = m.tier.promote() {
                    m.tier = new_tier;
                    promoted += 1;
                }
            }
        }
        promoted
    }

    pub fn prune_expired(&mut self, now: i64) -> usize {
        let before = self.memories.len();
        self.memories.retain(|m| {
            if let Some(ttl) = m.lifecycle.ttl_seconds { now - m.lifecycle.created_at <= ttl } else { true }
        });
        before - self.memories.len()
    }

    fn evict_expired(&mut self) -> usize {
        let expired = self.prune_expired(Utc::now().timestamp());
        let before2 = self.memories.len();
        let now = Utc::now().timestamp();
        let one_week: i64 = 604800;
        self.memories.retain(|m| {
            let age = now - m.timestamp;
            if age > one_week && m.lifecycle.importance < 0.3 && m.lifecycle.access_count < 2 && m.reward < 0.4 { return false; }
            if age > one_week * 4 && m.lifecycle.importance < 0.5 && m.reward < 0.3 { return false; }
            true
        });
        expired + (before2 - self.memories.len())
    }

    pub fn apply_lifecycle_policy(&mut self, config: &LifecycleConfig) -> (usize, usize, usize, usize, usize) {
        let now = Utc::now().timestamp() as u64;
        let mut promotions = 0;
        let mut demotions = 0;
        let mut archives = 0;
        let mut evictions = 0;
        let mut decays = 0;

        let mut i = self.memories.len();
        while i > 0 {
            i -= 1;
            let age_secs = now.saturating_sub(self.memories[i].lifecycle.created_at as u64);
            let last_access_days = now.saturating_sub(self.memories[i].lifecycle.last_accessed as u64) / 86400;
            let action = config.evaluate(
                self.memories[i].tier,
                self.memories[i].lifecycle.access_count as u32,
                age_secs,
                last_access_days,
            );
            match action {
                LifecycleAction::Promote(t) => { self.memories[i].tier = t; promotions += 1; }
                LifecycleAction::Demote(t) => { self.memories[i].tier = t; demotions += 1; }
                LifecycleAction::Archive => { self.memories[i].tier = MemoryTier::Semantic; archives += 1; }
                LifecycleAction::Evict => { self.memories.remove(i); evictions += 1; }
                LifecycleAction::Retain => {}
            }
        }

        for m in &mut self.memories {
            let decayed = config.apply_decay(m.tier, m.lifecycle.importance, 1);
            if (decayed - m.lifecycle.importance).abs() > 1e-12 {
                m.lifecycle.importance = decayed;
                decays += 1;
            }
        }

        (promotions, demotions, archives, evictions, decays)
    }

    pub fn consolidate_similar(&mut self, threshold: f64) -> usize {
        let mut merged = 0;
        let mut i = 0;
        while i < self.memories.len() {
            let mut j = i + 1;
            while j < self.memories.len() {
                let same_type = self.memories[i].task_type == self.memories[j].task_type;
                let reward_sim = 1.0 - (self.memories[i].reward - self.memories[j].reward).abs();
                if same_type && reward_sim > threshold {
                    let reward = (self.memories[i].reward + self.memories[j].reward) / 2.0;
                    let desc = format!("{}; {}", self.memories[i].task_description, self.memories[j].task_description);
                    let mut edits = self.memories[i].micro_edits.clone();
                    edits.extend(self.memories[j].micro_edits.clone());
                    let mut memory = ReasoningMemory::new(&desc, self.memories[i].task_type, &edits, reward);
                    if let Some(ref emb) = self.memories[i].embedding { memory.embedding = Some(emb.clone()); }
                    self.memories[i] = memory;
                    self.memories.remove(j);
                    merged += 1;
                } else {
                    j += 1;
                }
            }
            i += 1;
        }
        merged
    }

    pub fn prune_low_value(&mut self, min_reward: f64) -> usize {
        let before = self.memories.len();
        self.memories.retain(|m| m.reward >= min_reward);
        before - self.memories.len()
    }

    pub fn replay_high_value(&mut self) -> usize {
        let threshold = 0.8;
        let mut replayed = 0;
        let high_value: Vec<ReasoningMemory> = self.memories.iter()
            .filter(|m| m.reward > threshold).cloned().collect();
        for mut mem in high_value {
            mem.timestamp = Utc::now().timestamp();
            if self.memories.len() < self.max_memories {
                self.memories.push_back(mem);
                replayed += 1;
            }
        }
        replayed
    }
}
