/// Status of a search branch in the tree.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum BranchStatus {
    Active,
    Pruned,
    Completed,
}

/// A single branch in the tree-structured search.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TreeSeekerBranch {
    pub id: u64,
    pub parent_id: Option<u64>,
    pub depth: u32,
    pub source_label: String,
    pub value: f64,
    pub uncertainty: f64,
    pub risk: f64,
    pub visits: u64,
    pub created_at: u64,
    pub status: BranchStatus,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TreeSeekerConfig {
    pub ucb_explore_weight: f64,
    pub ucb_risk_penalty: f64,
    pub return_threshold: f64,
    pub max_depth: u32,
    pub max_branches: usize,
}

impl Default for TreeSeekerConfig {
    fn default() -> Self {
        Self {
            ucb_explore_weight: 0.5,
            ucb_risk_penalty: 0.3,
            return_threshold: 0.8,
            max_depth: 5,
            max_branches: 20,
        }
    }
}

/// Tree-structured search manager with UCB-based branch selection
/// and automatic return-to-parent when a branch stagnates.
///
/// Distilled from: arXiv:2606.11662 TreeSeeker.
/// Adapts textual UCB signals (value/uncertainty/risk) to VSA-native
/// exploration scoring using negentropy, prediction error, and storm score.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TreeSeekerManager {
    branches: Vec<TreeSeekerBranch>,
    next_id: u64,
    pub config: TreeSeekerConfig,
    cycle: u64,
}

impl TreeSeekerManager {
    pub fn new() -> Self {
        Self {
            branches: Vec::new(),
            next_id: 1,
            config: TreeSeekerConfig::default(),
            cycle: 0,
        }
    }

    pub fn with_config(config: TreeSeekerConfig) -> Self {
        Self {
            branches: Vec::new(),
            next_id: 1,
            config,
            cycle: 0,
        }
    }

    /// Start a new branch. If parent_id is None, this is a root branch.
    pub fn start_branch(
        &mut self,
        parent_id: Option<u64>,
        source_label: &str,
        value: f64,
        uncertainty: f64,
        risk: f64,
    ) -> u64 {
        let depth = parent_id
            .and_then(|pid| self.branches.iter().find(|b| b.id == pid))
            .map(|p| p.depth + 1)
            .unwrap_or(0);

        if depth > self.config.max_depth {
            return 0;
        }
        if self.active_branches().len() >= self.config.max_branches {
            return 0;
        }

        let id = self.next_id;
        self.next_id += 1;
        self.branches.push(TreeSeekerBranch {
            id,
            parent_id,
            depth,
            source_label: source_label.to_string(),
            value,
            uncertainty,
            risk,
            visits: 0,
            created_at: self.cycle,
            status: BranchStatus::Active,
        });
        id
    }

    /// Update the scores of an existing branch.
    pub fn update_branch(&mut self, id: u64, value: f64, uncertainty: f64, risk: f64) {
        if let Some(b) = self.branches.iter_mut().find(|b| b.id == id) {
            b.value = value;
            b.uncertainty = uncertainty;
            b.risk = risk;
            b.visits += 1;
        }
    }

    /// Compute the UCB score for a branch.
    /// ucb = value + explore_weight * sqrt(ln(total_active) / visits)
    ///       + uncertainty * explore_weight
    ///       - risk_penalty * risk
    pub fn ucb_score(&self, id: u64) -> f64 {
        let branch = match self.branches.iter().find(|b| b.id == id) {
            Some(b) => b,
            None => return 0.0,
        };
        let total = self.active_branches().len().max(1) as f64;
        let visit_bonus = if branch.visits == 0 {
            self.config.ucb_explore_weight * 2.0 // encourage unvisited branches
        } else {
            self.config.ucb_explore_weight * (total.ln() / branch.visits as f64).sqrt()
        };
        branch.value + visit_bonus + self.config.ucb_explore_weight * branch.uncertainty
            - self.config.ucb_risk_penalty * branch.risk
    }

    /// Select the best branch by UCB score.
    pub fn select_branch(&self) -> Option<u64> {
        let active: Vec<u64> = self
            .branches
            .iter()
            .filter(|b| b.status == BranchStatus::Active)
            .map(|b| b.id)
            .collect();
        if active.is_empty() {
            return None;
        }
        active.into_iter().max_by(|&a, &b| {
            self.ucb_score(a)
                .partial_cmp(&self.ucb_score(b))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Check if a branch should return to its parent (prune self).
    /// Returns true when parent UCB * threshold > child UCB.
    pub fn should_return(&self, id: u64) -> bool {
        let branch = match self.branches.iter().find(|b| b.id == id) {
            Some(b) => b,
            None => return false,
        };
        if branch.visits < 2 {
            return false;
        }
        let parent_id = match branch.parent_id {
            Some(pid) => pid,
            None => return false,
        };
        let child_ucb = self.ucb_score(id);
        let parent_ucb = self.ucb_score(parent_id);
        parent_ucb * self.config.return_threshold > child_ucb
    }

    /// Mark a branch as pruned.
    pub fn prune_branch(&mut self, id: u64) {
        if let Some(b) = self.branches.iter_mut().find(|b| b.id == id) {
            b.status = BranchStatus::Pruned;
        }
    }

    /// Mark a branch as completed.
    pub fn complete_branch(&mut self, id: u64) {
        if let Some(b) = self.branches.iter_mut().find(|b| b.id == id) {
            b.status = BranchStatus::Completed;
        }
    }

    /// Advance the internal cycle counter.
    pub fn tick(&mut self) {
        self.cycle += 1;
    }

    /// Active branches.
    pub fn active_branches(&self) -> Vec<&TreeSeekerBranch> {
        self.branches
            .iter()
            .filter(|b| b.status == BranchStatus::Active)
            .collect()
    }

    pub fn active_branch_count(&self) -> usize {
        self.branches
            .iter()
            .filter(|b| b.status == BranchStatus::Active)
            .count()
    }

    /// Get a branch by id.
    pub fn get_branch(&self, id: u64) -> Option<&TreeSeekerBranch> {
        self.branches.iter().find(|b| b.id == id)
    }

    /// Get the root branch (first created, depth 0).
    pub fn root_branch(&self) -> Option<&TreeSeekerBranch> {
        self.branches.iter().find(|b| b.depth == 0)
    }

    /// Get children of a branch.
    pub fn children_of(&self, parent_id: u64) -> Vec<&TreeSeekerBranch> {
        self.branches
            .iter()
            .filter(|b| b.parent_id == Some(parent_id))
            .collect()
    }

    pub fn stats(&self) -> TreeSeekerStats {
        let active = self.active_branch_count();
        let total = self.branches.len();
        let pruned = self
            .branches
            .iter()
            .filter(|b| b.status == BranchStatus::Pruned)
            .count();
        let completed = self
            .branches
            .iter()
            .filter(|b| b.status == BranchStatus::Completed)
            .count();
        let max_depth = self.branches.iter().map(|b| b.depth).max().unwrap_or(0);
        let best_id = self.select_branch();
        let best_ucb = best_id.map(|id| self.ucb_score(id));
        TreeSeekerStats {
            active_branches: active,
            total_branches: total,
            pruned_branches: pruned,
            completed_branches: completed,
            max_depth,
            best_branch_ucb: best_ucb.unwrap_or(0.0),
            cycle: self.cycle,
        }
    }
}

impl Default for TreeSeekerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TreeSeekerStats {
    pub active_branches: usize,
    pub total_branches: usize,
    pub pruned_branches: usize,
    pub completed_branches: usize,
    pub max_depth: u32,
    pub best_branch_ucb: f64,
    pub cycle: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_root_branch() {
        let mut m = TreeSeekerManager::new();
        let id = m.start_branch(None, "root_search", 0.8, 0.3, 0.1);
        assert_eq!(id, 1);
        assert_eq!(m.active_branch_count(), 1);
        assert_eq!(m.get_branch(id).unwrap().depth, 0);
    }

    #[test]
    fn test_start_child_branch() {
        let mut m = TreeSeekerManager::new();
        let root = m.start_branch(None, "root", 0.8, 0.3, 0.1);
        let child = m.start_branch(Some(root), "child", 0.6, 0.5, 0.2);
        assert_eq!(m.get_branch(child).unwrap().depth, 1);
    }

    #[test]
    fn test_ucb_score_encourages_unvisited() {
        let mut m = TreeSeekerManager::new();
        let a = m.start_branch(None, "a", 0.7, 0.2, 0.1);
        let b = m.start_branch(None, "b", 0.5, 0.8, 0.1);
        // Both unvisited, but b has higher uncertainty → higher UCB
        let ucb_a = m.ucb_score(a);
        let ucb_b = m.ucb_score(b);
        assert!(ucb_b > ucb_a); // b's uncertainty bonus outweighs lower value
    }

    #[test]
    fn test_ucb_score_risk_penalty() {
        let mut m = TreeSeekerManager::new();
        let low_risk = m.start_branch(None, "safe", 0.6, 0.3, 0.1);
        let high_risk = m.start_branch(None, "risky", 0.7, 0.3, 0.9);
        let ucb_safe = m.ucb_score(low_risk);
        let ucb_risky = m.ucb_score(high_risk);
        assert!(ucb_safe > ucb_risky);
    }

    #[test]
    fn test_select_branch_returns_best() {
        let mut m = TreeSeekerManager::new();
        m.start_branch(None, "low_value", 0.2, 0.1, 0.1);
        m.start_branch(None, "high_value", 0.9, 0.3, 0.1);
        // Both unvisited with same uncertainty, the high value branch wins
        // Actually the uncertainty differs, but value dominates at this scale
        let best = m.select_branch().unwrap();
        assert_eq!(m.get_branch(best).unwrap().source_label, "high_value");
    }

    #[test]
    fn test_should_return_after_visits() {
        let mut m = TreeSeekerManager::new();
        let root = m.start_branch(None, "root", 0.8, 0.2, 0.1);
        let child = m.start_branch(Some(root), "weak_child", 0.3, 0.2, 0.1);
        // Child needs 2+ visits before return check
        m.update_branch(child, 0.3, 0.2, 0.1);
        m.update_branch(child, 0.2, 0.2, 0.1);
        assert!(m.should_return(child));
    }

    #[test]
    fn test_prune_and_complete() {
        let mut m = TreeSeekerManager::new();
        let id = m.start_branch(None, "test", 0.5, 0.3, 0.1);
        assert_eq!(m.active_branch_count(), 1);
        m.prune_branch(id);
        assert_eq!(m.active_branch_count(), 0);
    }

    #[test]
    fn test_depth_limit_blocks_deep_branches() {
        let mut m = TreeSeekerManager::with_config(TreeSeekerConfig {
            max_depth: 2,
            ..TreeSeekerConfig::default()
        });
        let r = m.start_branch(None, "root", 0.5, 0.3, 0.1);
        let c1 = m.start_branch(Some(r), "child1", 0.5, 0.3, 0.1);
        let c2 = m.start_branch(Some(c1), "child2", 0.5, 0.3, 0.1);
        let c3 = m.start_branch(Some(c2), "too_deep", 0.5, 0.3, 0.1);
        assert_eq!(c3, 0); // blocked by depth
    }

    #[test]
    fn test_stats_reflect_state() {
        let mut m = TreeSeekerManager::new();
        let a = m.start_branch(None, "a", 0.8, 0.2, 0.1);
        let _b = m.start_branch(None, "b", 0.6, 0.4, 0.1);
        m.complete_branch(a);
        m.tick();
        let s = m.stats();
        assert_eq!(s.active_branches, 1);
        assert_eq!(s.total_branches, 2);
        assert_eq!(s.completed_branches, 1);
        assert_eq!(s.cycle, 1);
    }

    #[test]
    fn test_children_of() {
        let mut m = TreeSeekerManager::new();
        let root = m.start_branch(None, "root", 0.5, 0.3, 0.1);
        let c1 = m.start_branch(Some(root), "c1", 0.5, 0.3, 0.1);
        let _c2 = m.start_branch(Some(root), "c2", 0.5, 0.3, 0.1);
        assert_eq!(m.children_of(root).len(), 2);
        assert_eq!(m.children_of(c1).len(), 0);
    }
}
