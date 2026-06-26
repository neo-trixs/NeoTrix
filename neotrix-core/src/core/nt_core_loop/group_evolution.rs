use std::collections::{HashSet, VecDeque};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExperienceRecord {
    pub id: u64,
    pub branch: String,
    pub context: String,
    pub action: String,
    pub outcome: String,
    pub success: bool,
    pub utility: f64,
    pub timestamp: u64,
    pub transfer_count: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GroupExperiencePool {
    pub pool: VecDeque<ExperienceRecord>,
    pub max_pool: usize,
    pub next_id: u64,
    pub transfer_events: u64,
}

impl GroupExperiencePool {
    pub fn new(max_pool: usize) -> Self {
        Self {
            pool: VecDeque::with_capacity(max_pool),
            max_pool,
            next_id: 0,
            transfer_events: 0,
        }
    }

    pub fn add_experience(
        &mut self,
        branch: &str,
        context: &str,
        action: &str,
        outcome: &str,
        success: bool,
        utility: f64,
        cycle: u64,
    ) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.pool.push_back(ExperienceRecord {
            id,
            branch: branch.to_string(),
            context: context.to_string(),
            action: action.to_string(),
            outcome: outcome.to_string(),
            success,
            utility,
            timestamp: cycle,
            transfer_count: 0,
        });
        if self.pool.len() > self.max_pool {
            self.pool.pop_front();
        }
        id
    }

    pub fn find_transferable(
        &self,
        current_branch: &str,
        context: &str,
        min_utility: f64,
    ) -> Vec<&ExperienceRecord> {
        self.pool
            .iter()
            .filter(|e| {
                e.branch != current_branch
                    && e.success
                    && e.utility >= min_utility
                    && (e.context.contains(context) || context.contains(&e.context))
            })
            .collect()
    }

    pub fn transfer_experience(
        &mut self,
        id: u64,
        target_branch: &str,
    ) -> Option<ExperienceRecord> {
        if let Some(record) = self.pool.iter_mut().find(|e| e.id == id) {
            record.transfer_count += 1;
            self.transfer_events += 1;
            Some(ExperienceRecord {
                branch: target_branch.to_string(),
                ..record.clone()
            })
        } else {
            None
        }
    }

    pub fn cross_seed(
        &mut self,
        current_branch: &str,
        context: &str,
        cycle: u64,
    ) -> Vec<ExperienceRecord> {
        let candidate_ids: Vec<u64> = self
            .find_transferable(current_branch, context, 0.5)
            .iter()
            .take(3)
            .map(|e| e.id)
            .collect();
        let mut seeded = Vec::new();
        for id in candidate_ids {
            if let Some(record) = self.transfer_experience(id, current_branch) {
                seeded.push(record);
            }
        }
        if !seeded.is_empty() && cycle % 10 == 0 {
            log::debug!(
                "[gea] cross-seed: {} -> {}, {} transfers",
                current_branch,
                seeded[0].context,
                self.transfer_events
            );
        }
        seeded
    }

    pub fn success_rate(&self, branch: &str) -> f64 {
        let branch_experiences: Vec<&ExperienceRecord> =
            self.pool.iter().filter(|e| e.branch == branch).collect();
        if branch_experiences.is_empty() {
            return 0.0;
        }
        let successes = branch_experiences.iter().filter(|e| e.success).count();
        successes as f64 / branch_experiences.len() as f64
    }

    pub fn diversity_score(&self) -> f64 {
        if self.pool.is_empty() {
            return 0.0;
        }
        let mut branches: HashSet<&str> = HashSet::new();
        for e in &self.pool {
            branches.insert(e.branch.as_str());
        }
        branches.len() as f64 / self.pool.len().max(1) as f64
    }

    pub fn stats(&self) -> GroupEvolutionStats {
        GroupEvolutionStats {
            pool_size: self.pool.len(),
            max_pool: self.max_pool,
            transfer_events: self.transfer_events,
            diversity: self.diversity_score(),
            next_id: self.next_id,
        }
    }
}

impl Default for GroupExperiencePool {
    fn default() -> Self {
        Self::new(200)
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GroupEvolutionStats {
    pub pool_size: usize,
    pub max_pool: usize,
    pub transfer_events: u64,
    pub diversity: f64,
    pub next_id: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_pool() {
        let pool = GroupExperiencePool::new(100);
        assert_eq!(pool.pool.len(), 0);
        assert_eq!(pool.max_pool, 100);
    }

    #[test]
    fn test_add_experience() {
        let mut pool = GroupExperiencePool::new(100);
        let id = pool.add_experience("branch_a", "context_x", "action_y", "success", true, 0.8, 1);
        assert_eq!(id, 0);
        assert_eq!(pool.pool.len(), 1);
        assert_eq!(pool.pool[0].branch, "branch_a");
    }

    #[test]
    fn test_find_transferable() {
        let mut pool = GroupExperiencePool::new(100);
        pool.add_experience(
            "branch_a",
            "debug_error",
            "fix_bug",
            "resolved",
            true,
            0.9,
            1,
        );
        pool.add_experience("branch_b", "other", "nothing", "done", false, 0.1, 1);
        let transferable = pool.find_transferable("branch_c", "debug_error", 0.5);
        assert_eq!(transferable.len(), 1);
        assert_eq!(transferable[0].branch, "branch_a");
    }

    #[test]
    fn test_transfer_self_branch_excluded() {
        let mut pool = GroupExperiencePool::new(100);
        pool.add_experience("branch_a", "test", "action", "ok", true, 0.8, 1);
        let transferable = pool.find_transferable("branch_a", "test", 0.5);
        assert_eq!(transferable.len(), 0);
    }

    #[test]
    fn test_cross_seed() {
        let mut pool = GroupExperiencePool::new(100);
        pool.add_experience("branch_a", "common_task", "solution", "good", true, 0.9, 1);
        let seeded = pool.cross_seed("branch_b", "common_task", 1);
        assert_eq!(seeded.len(), 1);
        assert_eq!(seeded[0].branch, "branch_b");
    }

    #[test]
    fn test_success_rate() {
        let mut pool = GroupExperiencePool::new(100);
        pool.add_experience("branch_a", "c1", "a1", "ok", true, 0.8, 1);
        pool.add_experience("branch_a", "c2", "a2", "fail", false, 0.2, 1);
        pool.add_experience("branch_b", "c3", "a3", "ok", true, 0.7, 1);
        assert!((pool.success_rate("branch_a") - 0.5).abs() < 0.01);
        assert!((pool.success_rate("branch_b") - 1.0).abs() < 0.01);
        assert!((pool.success_rate("branch_c") - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_diversity() {
        let mut pool = GroupExperiencePool::new(100);
        pool.add_experience("branch_a", "c1", "a1", "ok", true, 0.5, 1);
        pool.add_experience("branch_b", "c2", "a2", "ok", true, 0.5, 1);
        let d = pool.diversity_score();
        assert!(d > 0.0 && d <= 1.0);
    }

    #[test]
    fn test_pool_eviction() {
        let mut pool = GroupExperiencePool::new(3);
        for i in 0..10 {
            pool.add_experience("branch_a", &format!("ctx_{}", i), "act", "ok", true, 0.5, i);
        }
        assert_eq!(pool.pool.len(), 3);
    }

    #[test]
    fn test_stats() {
        let pool = GroupExperiencePool::new(100);
        let s = pool.stats();
        assert_eq!(s.pool_size, 0);
        assert_eq!(s.max_pool, 100);
    }
}
