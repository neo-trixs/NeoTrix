use super::MetaAgent;

impl MetaAgent {
    /// Should this meta-agent continue improving?
    pub fn should_continue(&self) -> bool {
        self.iteration < self.config.budget as u64
    }

    /// Rollback mechanism: if score drops below parent, discard
    pub fn should_rollback(&self, child_score: f64, parent_score: f64) -> bool {
        child_score < parent_score * 0.9
    }
}
