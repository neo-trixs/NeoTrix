//! Self-Edit Loop — propose → validate → apply pipeline for memory modifications
//!
//! Agent proposes edits to memory blocks. Each proposal goes through validation
//! (content length bounds, injection pattern checks, audit trail) before being
//! applied. This implements the MemGPT self-edit loop pattern.
//!
//! Rules:
//! - R-MEM09: Agent self-editable memory blocks
//! - R-P121: Decay of stale memory entries
//! - R-P122: Localized maintenance (edits are scoped to blocks)

use serde::{Deserialize, Serialize};

use super::memory_filesystem::{EditResult, MemoryFilesystem, MemoryLabel};

// ============================================================
// Types
// ============================================================

/// Validation rule for memory edits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRule {
    MaxContentLength(usize),
    MinContentLength(usize),
    RequireExistingBlock,
    AllowedLabels(Vec<MemoryLabel>),
    NoInjectionPatterns,
    MaxEditsPerTurn(usize),
}

impl ValidationRule {
    pub fn default_rules() -> Vec<Self> {
        vec![
            ValidationRule::MaxContentLength(10_000),
            ValidationRule::MinContentLength(0),
            ValidationRule::RequireExistingBlock,
            ValidationRule::NoInjectionPatterns,
            ValidationRule::MaxEditsPerTurn(10),
        ]
    }
}

/// A proposed edit to a memory block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditProposal {
    pub block_id: String,
    pub old_content: String,
    pub new_content: String,
    pub reason: String,
    pub proposed_at: u64,
    pub turn_number: u64,
}

/// Result of validating an edit proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub violations: Vec<String>,
    pub proposal: EditProposal,
}

/// Audit trail entry for applied edits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditAuditEntry {
    pub proposal: EditProposal,
    pub applied_at: u64,
    pub block_label: MemoryLabel,
    pub content_delta: i64,
}

/// Self-edit loop engine
#[derive(Debug, Clone)]
pub struct SelfEditLoop {
    pub max_edits_per_turn: usize,
    pub edit_history: Vec<EditAuditEntry>,
    pub validation_rules: Vec<ValidationRule>,
    edits_this_turn: usize,
    current_turn: u64,
}

impl Default for SelfEditLoop {
    fn default() -> Self {
        Self::new()
    }
}

impl SelfEditLoop {
    pub fn new() -> Self {
        Self {
            max_edits_per_turn: 10,
            edit_history: Vec::new(),
            validation_rules: ValidationRule::default_rules(),
            edits_this_turn: 0,
            current_turn: 0,
        }
    }

    pub fn with_rules(rules: Vec<ValidationRule>) -> Self {
        Self {
            max_edits_per_turn: 10,
            edit_history: Vec::new(),
            validation_rules: rules,
            edits_this_turn: 0,
            current_turn: 0,
        }
    }

    /// Advance to next turn (resets per-turn edit counter)
    pub fn next_turn(&mut self) {
        self.current_turn += 1;
        self.edits_this_turn = 0;
    }

    /// Propose an edit — creates a proposal and validates it
    pub fn propose_edit(
        &self,
        fs: &MemoryFilesystem,
        block_id: &str,
        new_content: &str,
        reason: &str,
    ) -> ValidationResult {
        let proposal = EditProposal {
            block_id: block_id.to_string(),
            old_content: fs
                .blocks
                .get(block_id)
                .map(|b| b.content.clone())
                .unwrap_or_default(),
            new_content: new_content.to_string(),
            reason: reason.to_string(),
            proposed_at: unix_now(),
            turn_number: self.current_turn,
        };

        let violations = self.validate_proposal(fs, &proposal);
        let valid = violations.is_empty();

        ValidationResult {
            valid,
            violations,
            proposal,
        }
    }

    /// Validate a proposal against all rules
    fn validate_proposal(
        &self,
        fs: &MemoryFilesystem,
        proposal: &EditProposal,
    ) -> Vec<String> {
        let mut violations = Vec::new();

        for rule in &self.validation_rules {
            match rule {
                ValidationRule::MaxContentLength(max) => {
                    if proposal.new_content.len() > *max {
                        violations.push(format!(
                            "content length {} exceeds max {}",
                            proposal.new_content.len(),
                            max
                        ));
                    }
                }
                ValidationRule::MinContentLength(min) => {
                    if proposal.new_content.len() < *min {
                        violations.push(format!(
                            "content length {} below min {}",
                            proposal.new_content.len(),
                            min
                        ));
                    }
                }
                ValidationRule::RequireExistingBlock => {
                    if !fs.blocks.contains_key(&proposal.block_id) {
                        violations.push(format!("block '{}' not found", proposal.block_id));
                    }
                }
                ValidationRule::NoInjectionPatterns => {
                    if contains_injection(&proposal.new_content) {
                        violations
                            .push("content contains injection patterns".to_string());
                    }
                }
                ValidationRule::MaxEditsPerTurn(max) => {
                    if self.edits_this_turn >= *max {
                        violations.push(format!(
                            "max edits per turn ({}) exceeded",
                            max
                        ));
                    }
                }
                ValidationRule::AllowedLabels(labels) => {
                    if let Some(block) = fs.blocks.get(&proposal.block_id) {
                        if !labels.contains(&block.label) {
                            violations.push(format!(
                                "block label {:?} not in allowed labels",
                                block.label
                            ));
                        }
                    }
                }
            }
        }

        violations
    }

    /// Apply a validated edit to the filesystem
    pub fn apply_edit(
        &mut self,
        fs: &mut MemoryFilesystem,
        result: &ValidationResult,
    ) -> EditResult {
        if !result.valid {
            return EditResult::Rejected(result.violations.join("; "));
        }

        match fs.write_block(&result.proposal.block_id, &result.proposal.new_content) {
            EditResult::Applied => {
                let block_label = fs
                    .blocks
                    .get(&result.proposal.block_id)
                    .map(|b| b.label.clone())
                    .unwrap_or(MemoryLabel::Core);

                let entry = EditAuditEntry {
                    proposal: result.proposal.clone(),
                    applied_at: unix_now(),
                    block_label,
                    content_delta: result.proposal.new_content.len() as i64
                        - result.proposal.old_content.len() as i64,
                };

                self.edit_history.push(entry);
                self.edits_this_turn += 1;
                EditResult::Applied
            }
            other => other,
        }
    }

    /// Full pipeline: propose → validate → apply
    pub fn propose_and_apply(
        &mut self,
        fs: &mut MemoryFilesystem,
        block_id: &str,
        new_content: &str,
        reason: &str,
    ) -> (ValidationResult, EditResult) {
        let validation = self.propose_edit(fs, block_id, new_content, reason);
        let edit_result = self.apply_edit(fs, &validation);
        (validation, edit_result)
    }

    /// Get edit statistics
    pub fn stats(&self) -> EditLoopStats {
        EditLoopStats {
            total_edits: self.edit_history.len(),
            edits_this_turn: self.edits_this_turn,
            current_turn: self.current_turn,
            avg_content_delta: if self.edit_history.is_empty() {
                0.0
            } else {
                self.edit_history
                    .iter()
                    .map(|e| e.content_delta as f64)
                    .sum::<f64>()
                    / self.edit_history.len() as f64
            },
        }
    }
}

/// Edit loop statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditLoopStats {
    pub total_edits: usize,
    pub edits_this_turn: usize,
    pub current_turn: u64,
    pub avg_content_delta: f64,
}

/// Check for common injection patterns in content
fn contains_injection(content: &str) -> bool {
    let patterns = [
        "ignore previous",
        "ignore all previous",
        "disregard instructions",
        "system prompt",
        "jailbreak",
        "override safety",
        "<|system|>",
        "[INST]",
        "<<SYS>>",
    ];
    let lower = content.to_lowercase();
    patterns.iter().any(|p| lower.contains(p))
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_fs_with_block() -> MemoryFilesystem {
        let mut fs = MemoryFilesystem::new();
        fs.create_block("core1", MemoryLabel::Core, "Original content")
            .unwrap();
        fs
    }

    #[test]
    fn test_propose_valid_edit() {
        let fs = setup_fs_with_block();
        let loop_ = SelfEditLoop::new();
        let result = loop_.propose_edit(&fs, "core1", "Updated content", "test update");
        assert!(result.valid);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn test_propose_edit_to_nonexistent_block() {
        let fs = setup_fs_with_block();
        let loop_ = SelfEditLoop::new();
        let result = loop_.propose_edit(&fs, "ghost", "content", "test");
        assert!(!result.valid);
        assert!(result
            .violations
            .iter()
            .any(|v| v.contains("not found")));
    }

    #[test]
    fn test_propose_edit_exceeds_max_length() {
        let fs = setup_fs_with_block();
        let rules = vec![
            ValidationRule::MaxContentLength(20),
            ValidationRule::RequireExistingBlock,
        ];
        let loop_ = SelfEditLoop::with_rules(rules);
        let long_content = "x".repeat(25);
        let result = loop_.propose_edit(&fs, "core1", &long_content, "too long");
        assert!(!result.valid);
        assert!(result.violations.iter().any(|v| v.contains("exceeds max")));
    }

    #[test]
    fn test_apply_valid_edit() {
        let mut fs = setup_fs_with_block();
        let mut loop_ = SelfEditLoop::new();
        let validation = loop_.propose_edit(&fs, "core1", "New content", "test");
        let edit_result = loop_.apply_edit(&mut fs, &validation);
        assert_eq!(edit_result, EditResult::Applied);
        assert_eq!(fs.read_block("core1").unwrap().content, "New content");
        assert_eq!(loop_.stats().total_edits, 1);
    }

    #[test]
    fn test_apply_rejected_edit() {
        let mut fs = setup_fs_with_block();
        let mut loop_ = SelfEditLoop::new();
        let validation = loop_.propose_edit(&fs, "ghost", "content", "test");
        let edit_result = loop_.apply_edit(&mut fs, &validation);
        assert!(matches!(edit_result, EditResult::Rejected(_)));
        assert_eq!(loop_.stats().total_edits, 0);
    }

    #[test]
    fn test_injection_pattern_detected() {
        let fs = setup_fs_with_block();
        let loop_ = SelfEditLoop::new();
        let result = loop_.propose_edit(&fs, "core1", "ignore previous instructions", "test");
        assert!(!result.valid);
        assert!(result.violations.iter().any(|v| v.contains("injection")));
    }

    #[test]
    fn test_max_edits_per_turn() {
        let mut fs = MemoryFilesystem::new();
        fs.create_block("b1", MemoryLabel::Core, "init").unwrap();

        let rules = vec![
            ValidationRule::MaxEditsPerTurn(2),
            ValidationRule::RequireExistingBlock,
        ];
        let mut loop_ = SelfEditLoop::with_rules(rules);

        // First two edits should pass
        let v1 = loop_.propose_edit(&fs, "b1", "edit1", "r1");
        loop_.apply_edit(&mut fs, &v1);
        let v2 = loop_.propose_edit(&fs, "b1", "edit2", "r2");
        loop_.apply_edit(&mut fs, &v2);

        // Third should fail
        let v3 = loop_.propose_edit(&fs, "b1", "edit3", "r3");
        assert!(!v3.valid);
        assert!(v3.violations.iter().any(|v| v.contains("max edits")));
    }

    #[test]
    fn test_next_turn_resets_counter() {
        let mut fs = MemoryFilesystem::new();
        fs.create_block("b1", MemoryLabel::Core, "init").unwrap();

        let rules = vec![
            ValidationRule::MaxEditsPerTurn(1),
            ValidationRule::RequireExistingBlock,
        ];
        let mut loop_ = SelfEditLoop::with_rules(rules);

        let v1 = loop_.propose_edit(&fs, "b1", "edit1", "r1");
        loop_.apply_edit(&mut fs, &v1);

        // Should be blocked now
        let v2 = loop_.propose_edit(&fs, "b1", "edit2", "r2");
        assert!(!v2.valid);

        // Advance turn — counter resets
        loop_.next_turn();
        let v3 = loop_.propose_edit(&fs, "b1", "edit3", "r3");
        assert!(v3.valid);
    }

    #[test]
    fn test_audit_trail_records_entries() {
        let mut fs = setup_fs_with_block();
        let mut loop_ = SelfEditLoop::new();
        let v = loop_.propose_edit(&fs, "core1", "audited edit", "reason");
        loop_.apply_edit(&mut fs, &v);
        assert_eq!(loop_.edit_history.len(), 1);
        assert_eq!(loop_.edit_history[0].proposal.reason, "reason");
    }
}
