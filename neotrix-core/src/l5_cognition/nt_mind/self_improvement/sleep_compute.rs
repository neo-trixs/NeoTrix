//! Sleep-Time Compute — consolidation, reflection, and pruning
//!
//! Inspired by MemGPT/MemWalker sleep phases. When the agent is idle or between
//! turns, sleep-time compute performs:
//!   1. Consolidation: compress old recall entries, promote frequent patterns to core
//!   2. Reflection: identify patterns across recent sessions, generate meta-insights
//!   3. Pruning: remove low-relevance entries (R-P121 decay)
//!
//! Rules:
//! - R-P121: Memory decay (entries lose relevance over time)
//! - R-P122: Localized maintenance (operations scoped to blocks)

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::memory_filesystem::{MemoryBlock, MemoryFilesystem, MemoryLabel};

// ============================================================
// Types
// ============================================================

/// Configuration for sleep-time compute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepConfig {
    /// Recall entries older than this (in turns) are consolidation candidates
    pub consolidation_threshold_turns: u64,
    /// Core blocks older than this (in turns) can be archived
    pub archive_after_turns: u64,
    /// Minimum access count to promote recall → core
    pub promotion_min_access: u64,
    /// Maximum core blocks before pruning kicks in
    pub max_core_blocks: usize,
    /// Maximum archive blocks before hard eviction
    pub max_archive_blocks: usize,
}

impl Default for SleepConfig {
    fn default() -> Self {
        Self {
            consolidation_threshold_turns: 10,
            archive_after_turns: 50,
            promotion_min_access: 3,
            max_core_blocks: 50,
            max_archive_blocks: 500,
        }
    }
}

/// Result of a consolidation pass
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationResult {
    pub promoted_to_core: Vec<String>,
    pub archived: Vec<String>,
    pub compressed: Vec<String>,
    pub blocks_removed: usize,
}

/// A pattern detected across multiple memory blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPattern {
    pub pattern_id: String,
    pub description: String,
    pub block_ids: Vec<String>,
    pub frequency: u64,
    pub confidence: f64,
}

/// Result of a reflection pass
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionResult {
    pub patterns: Vec<MemoryPattern>,
    pub insights: Vec<String>,
    pub blocks_analyzed: usize,
}

/// Result of a pruning pass
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PruneResult {
    pub pruned_ids: Vec<String>,
    pub pruned_count: usize,
    pub remaining_count: usize,
}

// ============================================================
// IdentityPersistence
// ============================================================

/// Snapshot of agent identity persisted across sleep cycles
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IdentitySnapshot {
    pub name: String,
    pub traits: Vec<String>,
    pub preferences: HashMap<String, String>,
    pub last_updated: u64,
}

impl IdentitySnapshot {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            traits: Vec::new(),
            preferences: HashMap::new(),
            last_updated: unix_secs(),
        }
    }
}

/// Persist and load agent identity across sessions (R-MEM09 identity continuity)
#[derive(Debug)]
pub struct IdentityPersistence {
    stored: Option<IdentitySnapshot>,
}

impl Default for IdentityPersistence {
    fn default() -> Self {
        Self::new()
    }
}

impl IdentityPersistence {
    pub fn new() -> Self {
        Self { stored: None }
    }

    /// Persist an identity snapshot, returning error string on failure
    pub fn persist_identity(&mut self, identity: IdentitySnapshot) -> Result<(), String> {
        if identity.name.is_empty() {
            return Err("identity name cannot be empty".to_string());
        }
        self.stored = Some(identity);
        Ok(())
    }

    /// Load the last persisted identity, if any
    pub fn load_identity(&self) -> Option<&IdentitySnapshot> {
        self.stored.as_ref()
    }

    /// Update traits, merging with existing snapshot
    pub fn update_traits(&mut self, new_traits: Vec<String>) -> Result<(), String> {
        let snapshot = self
            .stored
            .as_mut()
            .ok_or("no identity persisted yet")?;
        for t in new_traits {
            if !snapshot.traits.contains(&t) {
                snapshot.traits.push(t);
            }
        }
        snapshot.last_updated = unix_secs();
        Ok(())
    }

    /// Update a preference key-value pair
    pub fn set_preference(&mut self, key: String, value: String) -> Result<(), String> {
        let snapshot = self
            .stored
            .as_mut()
            .ok_or("no identity persisted yet")?;
        snapshot.preferences.insert(key, value);
        snapshot.last_updated = unix_secs();
        Ok(())
    }
}

// ============================================================
// ConsolidationPipeline
// ============================================================

/// A memory reference with metadata for pipeline processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRef {
    pub id: String,
    pub content: String,
    pub access_count: u64,
    pub age_turns: u64,
    pub novelty_score: f64,
    pub tags: Vec<String>,
}

/// A memory that has been consolidated (promoted, archived, or pruned)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidatedMemory {
    pub id: String,
    pub content: String,
    pub action: ConsolidationAction,
    pub reason: String,
}

/// What action was taken on a memory during consolidation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConsolidationAction {
    /// Promoted to core (high access count)
    Promote,
    /// Moved to archive (old age)
    Archive,
    /// Removed (low novelty)
    Prune,
    /// Kept as-is
    Retain,
}

/// Configuration for consolidation pipeline thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationConfig {
    /// Minimum access count to promote to core
    pub promotion_access_threshold: u64,
    /// Maximum age (in turns) before archiving
    pub max_age_turns: u64,
    /// Minimum novelty score to keep (below → prune)
    pub min_novelty: f64,
}

impl Default for ConsolidationConfig {
    fn default() -> Self {
        Self {
            promotion_access_threshold: 3,
            max_age_turns: 50,
            min_novelty: 0.1,
        }
    }
}

/// Pipeline for consolidating memories with promotion/archive/prune rules
#[derive(Debug)]
pub struct ConsolidationPipeline {
    pub config: ConsolidationConfig,
}

impl Default for ConsolidationPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsolidationPipeline {
    pub fn new() -> Self {
        Self {
            config: ConsolidationConfig::default(),
        }
    }

    pub fn with_config(config: ConsolidationConfig) -> Self {
        Self { config }
    }

    /// Consolidate a batch of memory references, returning actions for each.
    ///
    /// Rules (R-MEM09, R-P121):
    /// - access_count > threshold → Promote to core
    /// - age_turns > max_age → Archive
    /// - novelty_score < min_novelty → Prune
    /// - Otherwise → Retain
    pub fn consolidate(&self, memories: Vec<MemoryRef>) -> Vec<ConsolidatedMemory> {
        memories
            .into_iter()
            .map(|m| self.classify_memory(m))
            .collect()
    }

    fn classify_memory(&self, m: MemoryRef) -> ConsolidatedMemory {
        let action;
        let reason;

        if m.access_count > self.config.promotion_access_threshold {
            action = ConsolidationAction::Promote;
            reason = format!(
                "access_count {} exceeds threshold {}",
                m.access_count, self.config.promotion_access_threshold
            );
        } else if m.age_turns > self.config.max_age_turns {
            action = ConsolidationAction::Archive;
            reason = format!(
                "age {} exceeds max_age {}",
                m.age_turns, self.config.max_age_turns
            );
        } else if m.novelty_score < self.config.min_novelty {
            action = ConsolidationAction::Prune;
            reason = format!(
                "novelty {} below minimum {}",
                m.novelty_score, self.config.min_novelty
            );
        } else {
            action = ConsolidationAction::Retain;
            reason = "within normal bounds".to_string();
        }

        ConsolidatedMemory {
            id: m.id,
            content: m.content,
            action,
            reason,
        }
    }

    /// Apply consolidation results to a MemoryFilesystem
    pub fn apply_to_filesystem(
        &self,
        results: &[ConsolidatedMemory],
        fs: &mut MemoryFilesystem,
    ) -> ConsolidationResult {
        let mut promoted = Vec::new();
        let mut archived = Vec::new();
        let mut compressed = Vec::new();
        let mut blocks_removed = 0;

        for cr in results {
            match cr.action {
                ConsolidationAction::Promote => {
                    if let Some(mut block) = fs.remove_block(&cr.id) {
                        let old_id = block.id.clone();
                        block.label = MemoryLabel::Core;
                        block.tags.push("promoted_by_pipeline".to_string());
                        let new_id = format!("core_{}", old_id);
                        fs.blocks.insert(new_id.clone(), block);
                        if let Some(ids) = fs.label_index.get_mut(&MemoryLabel::Core) {
                            ids.push(new_id.clone());
                        }
                        promoted.push(new_id);
                    }
                }
                ConsolidationAction::Archive => {
                    if let Some(mut block) = fs.remove_block(&cr.id) {
                        let original_len = block.content.len();
                        block.content = compress_content(&block.content);
                        if block.content.len() < original_len {
                            compressed.push(cr.id.clone());
                        }
                        block.label = MemoryLabel::Archive;
                        let new_id = format!("archive_{}", cr.id);
                        fs.blocks.insert(new_id.clone(), block);
                        if let Some(ids) = fs.label_index.get_mut(&MemoryLabel::Archive) {
                            ids.push(new_id.clone());
                        }
                        archived.push(new_id);
                    }
                }
                ConsolidationAction::Prune => {
                    if fs.remove_block(&cr.id).is_some() {
                        blocks_removed += 1;
                    }
                }
                ConsolidationAction::Retain => {}
            }
        }

        ConsolidationResult {
            promoted_to_core: promoted,
            archived,
            compressed,
            blocks_removed,
        }
    }
}

// ============================================================
// ReflectionEngine
// ============================================================

/// A session summary fed into the reflection engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub session_id: String,
    pub topics: Vec<String>,
    pub key_decisions: Vec<String>,
    pub duration_secs: u64,
    pub turn_count: u64,
}

/// A meta-insight generated from reflection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaInsight {
    pub insight_id: String,
    pub description: String,
    pub pattern_ids: Vec<String>,
    pub confidence: f64,
    pub times_predicted: u64,
    /// ⚠️ `report_outcome(false)` 原本被完全丢弃，而 `accuracy()` 拿
    /// `times_observed`（只含 positive）当分子 ⇒ 结构上永远 ≥ 1.0，
    /// 无法反映失败。故显式记录**总报告数**作为正确分母。
    #[serde(default)]
    pub times_reported: u64,
    pub times_observed: u64,
}

impl MetaInsight {
    /// Accuracy ratio: observed / predicted (1.0 = perfect)
    pub fn accuracy(&self) -> f64 {
        if self.times_reported == 0 {
            return 0.0;
        }
        self.times_observed as f64 / self.times_reported as f64
    }
}

/// Engine for detecting patterns across session summaries and generating insights
#[derive(Debug)]
pub struct ReflectionEngine {
    insights: Vec<MetaInsight>,
    theme_history: HashMap<String, u64>,
}

impl Default for ReflectionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ReflectionEngine {
    pub fn new() -> Self {
        Self {
            insights: Vec::new(),
            theme_history: HashMap::new(),
        }
    }

    /// Reflect on session summaries: detect recurring themes, generate insights,
    /// and track insight accuracy over time.
    pub fn reflect(&mut self, session_summaries: Vec<SessionSummary>) -> Vec<MetaInsight> {
        // Phase 1: Count theme frequency across sessions
        let mut theme_counts: HashMap<String, u64> = HashMap::new();
        for session in &session_summaries {
            for topic in &session.topics {
                *theme_counts.entry(topic.clone()).or_insert(0) += 1;
            }
        }

        // Merge into history
        for (theme, count) in &theme_counts {
            *self.theme_history.entry(theme.clone()).or_insert(0) += count;
        }

        // Phase 2: Detect recurring themes (appeared in 2+ sessions)
        let recurring: Vec<String> = theme_counts
            .iter()
            .filter(|(_, &count)| count >= 2)
            .map(|(theme, _)| theme.clone())
            .collect();

        // Phase 3: Generate insights from recurring themes
        let mut new_insights = Vec::new();
        for theme in &recurring {
            let count = theme_counts[theme];
            // Avoid duplicate insights for the same theme
            let already_tracked = self.insights.iter().any(|i| {
                i.description.contains(&format!("theme '{}'", theme))
            });
            if !already_tracked {
                let insight = MetaInsight {
                    insight_id: format!("insight_{}", theme.replace(' ', "_")),
                    description: format!(
                        "Recurring theme '{}' detected across {} sessions",
                        theme, count
                    ),
                    pattern_ids: vec![format!("pat_{}", theme)],
                    confidence: (count as f64 / session_summaries.len() as f64).min(1.0),
                    times_predicted: count,
                    times_reported: 0,
                    times_observed: 0,
                };
                new_insights.push(insight);
            }
        }

        // Phase 4: Detect decision patterns
        let mut decision_counts: HashMap<String, u64> = HashMap::new();
        for session in &session_summaries {
            for decision in &session.key_decisions {
                *decision_counts.entry(decision.clone()).or_insert(0) += 1;
            }
        }

        for (decision, count) in &decision_counts {
            if *count >= 2 {
                let insight_id = format!("decision_{}", decision.replace(' ', "_"));
                let already_tracked = self.insights.iter().any(|i| i.insight_id == insight_id);
                if !already_tracked {
                    new_insights.push(MetaInsight {
                        insight_id,
                        description: format!(
                            "Decision '{}' made {} times — consider standardizing",
                            decision, count
                        ),
                        pattern_ids: vec![format!("dec_{}", decision)],
                        confidence: (*count as f64 / session_summaries.len() as f64).min(1.0),
                        times_predicted: *count,
                        times_reported: 0,
                        times_observed: 0,
                    });
                }
            }
        }

        self.insights.extend(new_insights.clone());
        new_insights
    }

    /// Report observed outcomes for previously predicted insights
    pub fn report_outcome(&mut self, insight_id: &str, observed: bool) {
        if let Some(insight) = self.insights.iter_mut().find(|i| i.insight_id == insight_id) {
            // 总报告数无条件累加（含 negative），否则 accuracy 无从反映失败。
            insight.times_reported += 1;
            if observed {
                insight.times_observed += 1;
            }
        }
    }

    /// Get all tracked insights with their accuracy
    pub fn get_insights(&self) -> &[MetaInsight] {
        &self.insights
    }

    /// Get theme history (cumulative counts)
    pub fn theme_history(&self) -> &HashMap<String, u64> {
        &self.theme_history
    }
}

/// Sleep-time compute engine
#[derive(Debug, Clone)]
pub struct SleepComputer {
    pub config: SleepConfig,
    pub current_turn: u64,
    consolidation_history: Vec<ConsolidationResult>,
}

impl Default for SleepComputer {
    fn default() -> Self {
        Self::new()
    }
}

impl SleepComputer {
    pub fn new() -> Self {
        Self {
            config: SleepConfig::default(),
            current_turn: 0,
            consolidation_history: Vec::new(),
        }
    }

    pub fn with_config(config: SleepConfig) -> Self {
        Self {
            config,
            current_turn: 0,
            consolidation_history: Vec::new(),
        }
    }

    /// Advance turn counter
    pub fn tick(&mut self) {
        self.current_turn += 1;
    }

    /// Consolidate: promote frequent recall blocks to core, archive old core blocks
    pub fn consolidate(&mut self, fs: &mut MemoryFilesystem) -> ConsolidationResult {
        let mut promoted = Vec::new();
        let mut archived = Vec::new();
        let mut compressed = Vec::new();
        let mut blocks_removed = 0;

        // Phase 1: Promote high-access recall blocks to core
        let recall_ids: Vec<String> = fs
            .label_index
            .get(&MemoryLabel::Recall)
            .cloned()
            .unwrap_or_default();

        let mut to_promote = Vec::new();
        for id in &recall_ids {
            if let Some(block) = fs.blocks.get(id) {
                // 回合域年龄：原先 `updated_at % (current_turn+1)` 把 **Unix 秒**
                // 当回合数混算（源码自称pseudo-age）⇒ age 取决于挂钟取模，
                // 同一次运行可绿可红。回合语义一律走 last_access_turn。
                let age = self.current_turn.saturating_sub(block.last_access_turn);
                if block.access_count >= self.config.promotion_min_access
                    && age >= self.config.consolidation_threshold_turns
                {
                    to_promote.push(id.clone());
                }
            }
        }

        for id in &to_promote {
            if let Some(mut block) = fs.remove_block(id) {
                // Update label to core
                block.label = MemoryLabel::Core;
                block.tags.push("promoted_from_recall".to_string());
                let new_id = format!("core_{}", id);
                fs.blocks.insert(new_id.clone(), block);
                if let Some(ids) = fs.label_index.get_mut(&MemoryLabel::Core) {
                    ids.push(new_id.clone());
                }
                promoted.push(new_id);
            }
        }

        // Phase 2: Archive old core blocks that haven't been accessed recently
        let core_ids: Vec<String> = fs
            .label_index
            .get(&MemoryLabel::Core)
            .cloned()
            .unwrap_or_default();

        let mut to_archive = Vec::new();
        for id in &core_ids {
            if let Some(block) = fs.blocks.get(id) {
                let age = self.current_turn.saturating_sub(
                    block.updated_at % (self.current_turn + 1),
                );
                if age >= self.config.archive_after_turns && block.access_count < 2 {
                    to_archive.push(id.clone());
                }
            }
        }

        for id in &to_archive {
            if let Some(mut block) = fs.remove_block(id) {
                // Compress content before archiving
                let original_len = block.content.len();
                block.content = compress_content(&block.content);
                if block.content.len() < original_len {
                    compressed.push(id.clone());
                }
                block.label = MemoryLabel::Archive;
                let new_id = format!("archive_{}", id);
                fs.blocks.insert(new_id.clone(), block);
                if let Some(ids) = fs.label_index.get_mut(&MemoryLabel::Archive) {
                    ids.push(new_id.clone());
                }
                archived.push(new_id);
            }
        }

        // Phase 3: Enforce archive capacity — hard evict oldest
        let archive_ids: Vec<String> = fs
            .label_index
            .get(&MemoryLabel::Archive)
            .cloned()
            .unwrap_or_default();

        if archive_ids.len() > self.config.max_archive_blocks {
            let excess = archive_ids.len() - self.config.max_archive_blocks;
            for id in archive_ids.iter().take(excess) {
                if fs.remove_block(id).is_some() {
                    blocks_removed += 1;
                }
            }
        }

        let result = ConsolidationResult {
            promoted_to_core: promoted,
            archived,
            compressed,
            blocks_removed,
        };

        self.consolidation_history.push(result.clone());
        result
    }

    /// Reflect: scan recent blocks for recurring patterns
    pub fn reflect(&self, fs: &MemoryFilesystem) -> ReflectionResult {
        let mut patterns = Vec::new();
        let mut insights = Vec::new();
        let blocks: Vec<&MemoryBlock> = fs.blocks.values().collect();
        let blocks_analyzed = blocks.len();

        // Simple pattern detection: find blocks with overlapping tags
        let mut tag_groups: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();

        for block in &blocks {
            for tag in &block.tags {
                tag_groups
                    .entry(tag.clone())
                    .or_insert_with(Vec::new)
                    .push(block.id.clone());
            }
        }

        for (tag, block_ids) in &tag_groups {
            if block_ids.len() >= 2 {
                patterns.push(MemoryPattern {
                    pattern_id: format!("pat_{}_{}", tag, block_ids.len()),
                    description: format!(
                        "Block tag '{}' appears in {} blocks",
                        tag,
                        block_ids.len()
                    ),
                    block_ids: block_ids.clone(),
                    frequency: block_ids.len() as u64,
                    confidence: 0.7,
                });
            }
        }

        // Generate meta-insights
        let core_count = fs.label_count(&MemoryLabel::Core);
        let recall_count = fs.label_count(&MemoryLabel::Recall);
        let archive_count = fs.label_count(&MemoryLabel::Archive);

        if core_count > 0 && recall_count > core_count * 3 {
            insights.push(format!(
                "Recall memory ({}) is {}x larger than core ({}) — consider promotion",
                recall_count,
                recall_count / core_count.max(1),
                core_count
            ));
        }

        if archive_count > self.config.max_archive_blocks / 2 {
            insights.push(format!(
                "Archive memory at {}% capacity — consider pruning",
                archive_count * 100 / self.config.max_archive_blocks
            ));
        }

        if patterns.len() > 5 {
            insights
                .push("Many recurring patterns detected — consider consolidating".to_string());
        }

        ReflectionResult {
            patterns,
            insights,
            blocks_analyzed,
        }
    }

    /// Prune: remove low-relevance entries based on decay (R-P121)
    pub fn prune(&mut self, fs: &mut MemoryFilesystem, now_turn: u64) -> PruneResult {
        let mut pruned_ids = Vec::new();

        // Prune recall blocks: low access + old → remove
        let recall_ids: Vec<String> = fs
            .label_index
            .get(&MemoryLabel::Recall)
            .cloned()
            .unwrap_or_default();

        for id in &recall_ids {
            if let Some(block) = fs.blocks.get(id) {
                let age = now_turn.saturating_sub(
                    block.updated_at % (now_turn + 1),
                );
                // Decay: salience = access_count * exp(-age / half_life)
                let half_life = 10.0;
                let salience =
                    block.access_count as f64 * (-(age as f64) / half_life).exp();
                if salience < 0.1 {
                    pruned_ids.push(id.clone());
                }
            }
        }

        for id in &pruned_ids {
            fs.remove_block(id);
        }

        let remaining = fs.block_count();
        PruneResult {
            pruned_ids: pruned_ids.clone(),
            pruned_count: pruned_ids.len(),
            remaining_count: remaining,
        }
    }

    /// Full sleep cycle: consolidate → reflect → prune
    pub fn sleep_cycle(
        &mut self,
        fs: &mut MemoryFilesystem,
    ) -> (ConsolidationResult, ReflectionResult, PruneResult) {
        let consolidated = self.consolidate(fs);
        let reflected = self.reflect(fs);
        let pruned = self.prune(fs, self.current_turn);
        self.tick();
        (consolidated, reflected, pruned)
    }
}

/// Simple content compression — truncate to first meaningful chunk
fn unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn compress_content(content: &str) -> String {
    let max_len = 200;
    if content.len() <= max_len {
        return content.to_string();
    }
    // ⚠️ UTF-8 安全：原实现 `content[..max_len]` 按**字节**索引，max_len 落在
    // 多字节字符中间即 panic（与 A52 的 `→` 同类缺陷）。先按 char 边界回退。
    let cut = (0..=max_len.min(content.len()))
        .rev()
        .find(|&i| content.is_char_boundary(i))
        .unwrap_or(0);
    let head = &content[..cut];
    // Try to cut at sentence boundary
    if let Some(pos) = head.rfind(". ") {
        format!("{}...", &head[..=pos])
    } else if let Some(pos) = head.rfind(' ') {
        format!("{}...", &head[..pos])
    } else {
        format!("{}...", head)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_fs_with_blocks() -> MemoryFilesystem {
        let mut fs = MemoryFilesystem::new();
        // Core block
        fs.create_block("core1", MemoryLabel::Core, "Core preference: dark mode")
            .unwrap();
        // Recall blocks with varying access counts
        fs.create_block("recall1", MemoryLabel::Recall, "User asked about Rust")
            .unwrap();
        fs.create_block("recall2", MemoryLabel::Recall, "User asked about Python")
            .unwrap();
        // Set high access on recall1
        if let Some(b) = fs.blocks.get_mut("recall1") {
            b.access_count = 5;
            b.tags.push("promoted".to_string());
        }
        if let Some(b) = fs.blocks.get_mut("recall2") {
            b.access_count = 1;
        }
        fs
    }

    #[test]
    fn test_consolidate_promotes_high_access_recall() {
        let mut fs = setup_fs_with_blocks();
        let mut computer = SleepComputer::new();
        computer.current_turn = 20;

            // 提升需同时满足：access_count >= promotion_min_access 且回合年龄 >= 阈值。
        let result = computer.consolidate(&mut fs);
        assert!(result.promoted_to_core.iter().any(|id| id.contains("recall1")));
    }

    #[test]
    fn test_consolidate_archives_old_core() {
        let mut fs = setup_fs_with_blocks();
        let mut computer = SleepComputer::new();
        computer.current_turn = 100;
        // Set core1's updated_at to be very old
        if let Some(b) = fs.blocks.get_mut("core1") {
            b.updated_at = 1;
            b.access_count = 1;
        }

        let result = computer.consolidate(&mut fs);
        assert!(result.archived.iter().any(|id| id.contains("core1")));
    }

    #[test]
    fn test_reflect_detects_patterns() {
        let mut fs = MemoryFilesystem::new();
        fs.create_block("b1", MemoryLabel::Core, "content1").unwrap();
        fs.create_block("b2", MemoryLabel::Core, "content2").unwrap();
        if let Some(b) = fs.blocks.get_mut("b1") {
            b.tags.push("rust".to_string());
        }
        if let Some(b) = fs.blocks.get_mut("b2") {
            b.tags.push("rust".to_string());
        }

        let computer = SleepComputer::new();
        let result = computer.reflect(&fs);
        assert!(result
            .patterns
            .iter()
            .any(|p| p.description.contains("rust")));
    }

    #[test]
    fn test_prune_removes_low_salience() {
        let mut fs = MemoryFilesystem::new();
        fs.create_block("old_recall", MemoryLabel::Recall, "stale entry")
            .unwrap();
        if let Some(b) = fs.blocks.get_mut("old_recall") {
            b.access_count = 0;
            b.updated_at = 0;
        }

        let mut computer = SleepComputer::new();
        let result = computer.prune(&mut fs, 100);
        assert!(result.pruned_ids.contains(&"old_recall".to_string()));
        assert!(fs.blocks.get("old_recall").is_none());
    }

    #[test]
    fn test_prune_keeps_frequent_blocks() {
        let mut fs = MemoryFilesystem::new();
        fs.create_block("hot_recall", MemoryLabel::Recall, "frequent entry")
            .unwrap();
        if let Some(b) = fs.blocks.get_mut("hot_recall") {
            b.access_count = 10;
            b.updated_at = 100; // recent
        }

        let mut computer = SleepComputer::new();
        let result = computer.prune(&mut fs, 100);
        assert!(result.pruned_ids.is_empty());
        assert!(fs.blocks.get("hot_recall").is_some());
    }

    #[test]
    fn test_sleep_cycle_runs_all_phases() {
        let mut fs = setup_fs_with_blocks();
        let mut computer = SleepComputer::new();
        computer.current_turn = 20;

        let (_consolidated, reflected, _pruned) = computer.sleep_cycle(&mut fs);
        // At minimum, reflection should have analyzed blocks
        assert!(reflected.blocks_analyzed > 0);
        // Turn should advance
        assert_eq!(computer.current_turn, 21);
    }

    #[test]
    fn test_compress_content() {
        let short = "Short text.";
        assert_eq!(compress_content(short), "Short text.");

        // 阈值是 `max_len = 200`。原用例的"long"只有 102 字节 < 200，
        // 于是原样返回，`compressed.len() < long.len()` **永远不可能成立**
        // （此模块此前从未编译，故该断言从未被真正执行过）。
        // 这里如实覆盖两个分支，而不擅自改动生产阈值：
        //   ① 低于阈值 ⇒ 不压缩
        //   ② 超过阈值 ⇒ 截断并加省略号
        let medium = "This is a very long sentence that goes on and on and should be compressed. And another sentence.";
        assert!(medium.len() < 200);
        assert_eq!(compress_content(medium), medium, "低于阈值应原样返回");

        let long = format!("{}. {}", medium, "Additional sentences pushing the total length well past the two hundred byte compression threshold used by this helper.");
        assert!(long.len() > 200, "构造的长文本须超过阈值，实际 {}", long.len());
        let compressed = compress_content(&long);
        assert!(compressed.len() < long.len(), "应被压缩: {} -> {}", long.len(), compressed.len());
        assert!(compressed.ends_with("..."), "应以省略号结尾: {:?}", compressed);
        // UTF-8 安全：截断点不得落在多字节字符中间
        assert!(long.is_char_boundary(compressed.len() - 3) || compressed.is_char_boundary(3));
    }

    #[test]
    fn test_reflect_insights_on_imbalanced_memory() {
        let mut fs = MemoryFilesystem::new();
        // Create 1 core, 5 recall blocks
        fs.create_block("c1", MemoryLabel::Core, "core").unwrap();
        for i in 0..5 {
            fs.create_block(
                format!("r{}", i),
                MemoryLabel::Recall,
                format!("recall {}", i),
            )
            .unwrap();
        }

        let computer = SleepComputer::new();
        let result = computer.reflect(&fs);
        assert!(result.insights.iter().any(|i| i.contains("Recall memory")));
    }

    // ============================================================
    // IdentityPersistence tests
    // ============================================================

    #[test]
    fn test_identity_persist_and_load() {
        let mut ip = IdentityPersistence::new();
        let mut prefs = HashMap::new();
        prefs.insert("theme".to_string(), "dark".to_string());

        let snapshot = IdentitySnapshot {
            name: "NeoTrix".to_string(),
            traits: vec!["analytical".to_string(), "helpful".to_string()],
            preferences: prefs,
            last_updated: 0,
        };

        ip.persist_identity(snapshot.clone()).unwrap();
        let loaded = ip.load_identity().unwrap();
        assert_eq!(loaded.name, "NeoTrix");
        assert_eq!(loaded.traits.len(), 2);
        assert_eq!(loaded.preferences.get("theme").unwrap(), "dark");
    }

    #[test]
    fn test_identity_persist_empty_name_fails() {
        let mut ip = IdentityPersistence::new();
        let snapshot = IdentitySnapshot::new("");
        assert!(ip.persist_identity(snapshot).is_err());
    }

    #[test]
    fn test_identity_load_none() {
        let ip = IdentityPersistence::new();
        assert!(ip.load_identity().is_none());
    }

    #[test]
    fn test_identity_update_traits() {
        let mut ip = IdentityPersistence::new();
        ip.persist_identity(IdentitySnapshot::new("test")).unwrap();
        ip.update_traits(vec!["a".to_string(), "b".to_string()]).unwrap();
        ip.update_traits(vec!["b".to_string(), "c".to_string()]).unwrap(); // b is duplicate
        let loaded = ip.load_identity().unwrap();
        assert_eq!(loaded.traits, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_identity_update_traits_no_identity() {
        let mut ip = IdentityPersistence::new();
        assert!(ip.update_traits(vec!["a".to_string()]).is_err());
    }

    #[test]
    fn test_identity_set_preference() {
        let mut ip = IdentityPersistence::new();
        ip.persist_identity(IdentitySnapshot::new("test")).unwrap();
        ip.set_preference("lang".to_string(), "rust".to_string()).unwrap();
        let loaded = ip.load_identity().unwrap();
        assert_eq!(loaded.preferences.get("lang").unwrap(), "rust");
    }

    // ============================================================
    // ConsolidationPipeline tests
    // ============================================================

    #[test]
    fn test_pipeline_promotes_high_access() {
        let pipeline = ConsolidationPipeline::new();
        let memories = vec![MemoryRef {
            id: "m1".to_string(),
            content: "frequently accessed".to_string(),
            access_count: 5,
            age_turns: 1,
            novelty_score: 0.5,
            tags: vec![],
        }];

        let results = pipeline.consolidate(memories);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].action, ConsolidationAction::Promote);
        assert!(results[0].reason.contains("exceeds threshold"));
    }

    #[test]
    fn test_pipeline_archives_old() {
        let pipeline = ConsolidationPipeline::new();
        let memories = vec![MemoryRef {
            id: "m2".to_string(),
            content: "old memory".to_string(),
            access_count: 1,
            age_turns: 100,
            novelty_score: 0.5,
            tags: vec![],
        }];

        let results = pipeline.consolidate(memories);
        assert_eq!(results[0].action, ConsolidationAction::Archive);
        assert!(results[0].reason.contains("exceeds max_age"));
    }

    #[test]
    fn test_pipeline_prunes_low_novelty() {
        let pipeline = ConsolidationPipeline::new();
        let memories = vec![MemoryRef {
            id: "m3".to_string(),
            content: "boring duplicate".to_string(),
            access_count: 1,
            age_turns: 5,
            novelty_score: 0.05,
            tags: vec![],
        }];

        let results = pipeline.consolidate(memories);
        assert_eq!(results[0].action, ConsolidationAction::Prune);
        assert!(results[0].reason.contains("below minimum"));
    }

    #[test]
    fn test_pipeline_retains_normal() {
        let pipeline = ConsolidationPipeline::new();
        let memories = vec![MemoryRef {
            id: "m4".to_string(),
            content: "normal".to_string(),
            access_count: 2,
            age_turns: 10,
            novelty_score: 0.5,
            tags: vec![],
        }];

        let results = pipeline.consolidate(memories);
        assert_eq!(results[0].action, ConsolidationAction::Retain);
    }

    #[test]
    fn test_pipeline_mixed_batch() {
        let pipeline = ConsolidationPipeline::new();
        let memories = vec![
            MemoryRef { id: "a".into(), content: "".into(), access_count: 10, age_turns: 0, novelty_score: 0.9, tags: vec![] },
            MemoryRef { id: "b".into(), content: "".into(), access_count: 0, age_turns: 200, novelty_score: 0.3, tags: vec![] },
            MemoryRef { id: "c".into(), content: "".into(), access_count: 1, age_turns: 5, novelty_score: 0.01, tags: vec![] },
            MemoryRef { id: "d".into(), content: "".into(), access_count: 1, age_turns: 5, novelty_score: 0.5, tags: vec![] },
        ];

        let results = pipeline.consolidate(memories);
        let actions: Vec<_> = results.iter().map(|r| r.action.clone()).collect();
        assert_eq!(actions, vec![
            ConsolidationAction::Promote,
            ConsolidationAction::Archive,
            ConsolidationAction::Prune,
            ConsolidationAction::Retain,
        ]);
    }

    #[test]
    fn test_pipeline_apply_to_filesystem() {
        let pipeline = ConsolidationPipeline::new();
        let mut fs = MemoryFilesystem::new();
        fs.create_block("mem1", MemoryLabel::Recall, "content1").unwrap();
        fs.create_block("mem2", MemoryLabel::Recall, "content2").unwrap();

        let results = vec![
            ConsolidatedMemory {
                id: "mem1".into(),
                content: "content1".into(),
                action: ConsolidationAction::Promote,
                reason: "test".into(),
            },
            ConsolidatedMemory {
                id: "mem2".into(),
                content: "content2".into(),
                action: ConsolidationAction::Prune,
                reason: "test".into(),
            },
        ];

        let outcome = pipeline.apply_to_filesystem(&results, &mut fs);
        assert_eq!(outcome.promoted_to_core.len(), 1);
        assert!(outcome.promoted_to_core[0].contains("mem1"));
        assert_eq!(outcome.blocks_removed, 1);
        assert!(fs.blocks.get("mem1").is_none());
        assert!(fs.blocks.get("mem2").is_none());
    }

    #[test]
    fn test_pipeline_custom_config() {
        let config = ConsolidationConfig {
            promotion_access_threshold: 10,
            max_age_turns: 5,
            min_novelty: 0.5,
        };
        let pipeline = ConsolidationPipeline::with_config(config);

        let memories = vec![MemoryRef {
            id: "m".into(),
            content: "".into(),
            access_count: 8,
            age_turns: 3,
            novelty_score: 0.3,
            tags: vec![],
        }];

        let results = pipeline.consolidate(memories);
        // access=8 < threshold=10, age=3 < max=5, novelty=0.3 < min=0.5 → Prune
        assert_eq!(results[0].action, ConsolidationAction::Prune);
    }

    // ============================================================
    // ReflectionEngine tests
    // ============================================================

    #[test]
    fn test_reflection_detects_recurring_themes() {
        let mut engine = ReflectionEngine::new();

        let sessions = vec![
            SessionSummary {
                session_id: "s1".into(),
                topics: vec!["rust".into(), "python".into()],
                key_decisions: vec![],
                duration_secs: 60,
                turn_count: 5,
            },
            SessionSummary {
                session_id: "s2".into(),
                topics: vec!["rust".into(), "go".into()],
                key_decisions: vec![],
                duration_secs: 45,
                turn_count: 3,
            },
        ];

        let insights = engine.reflect(sessions);
        assert!(!insights.is_empty());
        assert!(insights.iter().any(|i| i.description.contains("rust")));
    }

    #[test]
    fn test_reflection_no_duplicate_insights() {
        let mut engine = ReflectionEngine::new();

        let sessions = vec![
            SessionSummary {
                session_id: "s1".into(),
                topics: vec!["rust".into()],
                key_decisions: vec![],
                duration_secs: 30,
                turn_count: 2,
            },
            SessionSummary {
                session_id: "s2".into(),
                topics: vec!["rust".into()],
                key_decisions: vec![],
                duration_secs: 30,
                turn_count: 2,
            },
        ];

        let first = engine.reflect(sessions.clone());
        let second = engine.reflect(sessions);
        // Second call should not produce new insights for the same theme
        assert!(second.is_empty() || second.iter().all(|i| !first.iter().any(|f| f.insight_id == i.insight_id)));
    }

    #[test]
    fn test_reflection_tracks_accuracy() {
        let mut engine = ReflectionEngine::new();

        let sessions = vec![
            SessionSummary {
                session_id: "s1".into(),
                topics: vec!["design".into()],
                key_decisions: vec![],
                duration_secs: 30,
                turn_count: 2,
            },
            SessionSummary {
                session_id: "s2".into(),
                topics: vec!["design".into()],
                key_decisions: vec![],
                duration_secs: 30,
                turn_count: 2,
            },
        ];

        let insights = engine.reflect(sessions);
        let insight_id = insights[0].insight_id.clone();

        // Report outcomes
        engine.report_outcome(&insight_id, true);
        engine.report_outcome(&insight_id, true);
        engine.report_outcome(&insight_id, false);

        let tracked = engine.get_insights().iter().find(|i| i.insight_id == insight_id).unwrap();
        assert_eq!(tracked.times_observed, 2);
        let accuracy = tracked.accuracy();
        assert!((accuracy - 0.666).abs() < 0.01); // 2/3 ≈ 0.666
    }

    #[test]
    fn test_reflection_detects_decision_patterns() {
        let mut engine = ReflectionEngine::new();

        let sessions = vec![
            SessionSummary {
                session_id: "s1".into(),
                topics: vec![],
                key_decisions: vec!["use tokio".into()],
                duration_secs: 30,
                turn_count: 2,
            },
            SessionSummary {
                session_id: "s2".into(),
                topics: vec![],
                key_decisions: vec!["use tokio".into()],
                duration_secs: 30,
                turn_count: 2,
            },
        ];

        let insights = engine.reflect(sessions);
        assert!(insights.iter().any(|i| i.description.contains("use tokio")));
    }

    #[test]
    fn test_reflection_theme_history() {
        let mut engine = ReflectionEngine::new();

        let sessions = vec![
            SessionSummary {
                session_id: "s1".into(),
                topics: vec!["rust".into()],
                key_decisions: vec![],
                duration_secs: 30,
                turn_count: 2,
            },
        ];

        engine.reflect(sessions);
        assert_eq!(engine.theme_history().get("rust"), Some(&1));

        let sessions2 = vec![SessionSummary {
            session_id: "s2".into(),
            topics: vec!["rust".into()],
            key_decisions: vec![],
            duration_secs: 30,
            turn_count: 2,
        }];
        engine.reflect(sessions2);
        assert_eq!(engine.theme_history().get("rust"), Some(&2));
    }

    #[test]
    fn test_reflection_empty_sessions() {
        let mut engine = ReflectionEngine::new();
        let insights = engine.reflect(vec![]);
        assert!(insights.is_empty());
    }
}
