//! nt_quality_stages — 从 `pipeline.rs` 拆分 (quality_stages — 质量族: 知识质量/密钥扫描/对话蒸馏/假设精度/模式提取/自检/外部知识/外部消化 (行为零变更).)
//! 原文逐行搬运, 仅补可见性/导入, 行为零变更。

use super::nt_types::*;
use super::super::SelfIteratingBrain;
use crate::neotrix::nt_core_error::NeoTrixError;
use crate::l4_emotion::nt_memory::nt_memory_historian::nt_evidence_hypothesis::HypothesisStatus;
use crate::l4_emotion::nt_memory::nt_memory_kb::GraphRagConfig;
use super::super::secret_scanner::SecretScanner;
use crate::l5_cognition::l1_facade::SelfReviewGate;
use crate::make_stage;
use crate::l4_emotion::nt_memory::nt_memory_kb::ProceduralMemoryRecord;

/// Knowledge quality assessment stage — scores KB health metrics
pub struct _KnowledgeQualityStage;
impl Default for _KnowledgeQualityStage {
    fn default() -> Self {
        Self
    }
}
impl _KnowledgeQualityStage {
    pub fn new() -> Self {
        Self
    }
}
impl BrainStage for _KnowledgeQualityStage {
    fn name(&self) -> &str {
        "knowledge_quality"
    }
    fn frequency(&self) -> usize {
        5
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if let Some(ref kb) = brain._nt_memory_kb {
            // Gather content quality metrics directly from the nodes table
            let quality_metrics =
                (|| -> Option<(i64, i64, i64, usize)> {
                    let conn = kb.conn.lock().ok()?;
                    let total: i64 = conn
                        .query_row("SELECT COUNT(*) FROM nodes", [], |r| r.get(0))
                        .ok()?;
                    let with_content: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM nodes WHERE content IS NOT NULL AND content != ''",
                    [], |r| r.get(0),
                ).ok()?;
                    let with_summary: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM nodes WHERE summary IS NOT NULL AND summary != ''",
                    [], |r| r.get(0),
                ).ok()?;
                    let mut stmt = conn
                        .prepare("SELECT COUNT(DISTINCT node_type) FROM nodes")
                        .ok()?;
                    let type_count: i64 = stmt.query_row([], |r| r.get(0)).ok()?;
                    drop(stmt);
                    Some((total, with_content, with_summary, type_count as usize))
                })();

            if let (Some(stats), Some((tot, has_content, has_summary, type_count))) =
                (kb.stats().ok(), quality_metrics)
            {
                let quality_score = if tot > 0 {
                    let content_cov = has_content as f64 / tot as f64;
                    let summary_cov = has_summary as f64 / tot as f64;
                    let type_div = (type_count as f64 / 23.0_f64).min(1.0);
                    let edge_ratio = if stats.total_nodes > 0 {
                        (stats.total_edges as f64 / stats.total_nodes as f64).min(5.0) / 5.0
                    } else {
                        0.0
                    };
                    (content_cov * 40.0 + summary_cov * 20.0 + type_div * 20.0 + edge_ratio * 20.0)
                        .max(0.0)
                        .min(100.0)
                } else {
                    0.0
                };

                let reward_boost: f64 = if quality_score > 80.0 {
                    0.05
                } else if quality_score < 20.0 {
                    -0.05
                } else {
                    0.0
                };

                if reward_boost.abs() > 0.0_f64 {
                    let current = brain._reward;
                    let adjusted = (current + reward_boost).max(0.0).min(1.0);
                    brain._set_reward(adjusted);
                }

                #[allow(clippy::manual_is_multiple_of)]
                if brain.iteration % 10 == 0 {
                    log::info!(
                        "[knowledge_quality] score={:.1}% nodes={} content={}/{} summary={}/{} types={}/{} edges={} boost={}",
                        quality_score, tot, has_content, tot, has_summary, tot,
                        type_count, stats.by_type.len(), stats.total_edges, reward_boost,
                    );
                }
            }
        }
        Ok(StageDecision::Continue)
    }
}


/// Secret scan stage — wraps SecretScanner for pipeline integration
pub struct SecretScanStage;
impl Default for SecretScanStage {
    fn default() -> Self {
        Self
    }
}
impl SecretScanStage {
    pub fn new() -> Self {
        Self
    }
}
impl BrainStage for SecretScanStage {
    fn name(&self) -> &str {
        "security_scan"
    }
    fn frequency(&self) -> usize {
        1
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let scanner = SecretScanner::new();
        let task_repr = format!(
            "iter={} reward={:.4} cap={:?}",
            brain.iteration,
            brain._reward,
            &brain.brain.capability.arr()[..5]
        );
        let result = scanner.scan_with_context(&task_repr, "");
        if !result.is_safe() {
            log::warn!(
                "[security_scan] {} risks (score={:.2})",
                result.findings.len(),
                result.risk_score()
            );
        } else {
            log::trace!("[security_scan] safe iter={}", brain.iteration);
        }
        Ok(StageDecision::Continue)
    }
}


/// Conversation distillation stage — stores trajectory insights to KB
pub struct _ConversationDistillStage;
impl Default for _ConversationDistillStage {
    fn default() -> Self {
        Self
    }
}
impl _ConversationDistillStage {
    pub fn new() -> Self {
        Self
    }
}
impl BrainStage for _ConversationDistillStage {
    fn name(&self) -> &str {
        "conversation_distill"
    }
    fn frequency(&self) -> usize {
        1
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        let traj_len = brain
            .reasoning_engine
            .as_ref()
            .map(|e| e.state_trajectory.len())
            .unwrap_or(0);
        if let Some(ref kb) = brain._nt_memory_kb {
            let stage_count = brain._stage_results.len();
            let prm_reward = brain._prm_cumulative_reward;
            let summary = format!(
                "iter={} reward={:.4} traj={} entropy={:.4} stages={} prm={:.4}",
                brain.iteration,
                brain._reward,
                traj_len,
                brain.entropy_crisis_level,
                stage_count,
                prm_reward,
            );
            let _ = kb.kv_set(
                "conversation_distill",
                &format!("snap_{}", brain.iteration),
                &summary,
            );
            if traj_len > 3 && brain.iteration.is_multiple_of(5) {
                if let Ok(records) = kb.get_evolution_history(10) {
                    let rewarding_count = records.iter().filter(|r| r.effectiveness > 0.0).count();
                    let failing_count = records.iter().filter(|r| r.effectiveness <= 0.0).count();
                    if rewarding_count + failing_count >= 3 {
                        let pattern_type = if failing_count > rewarding_count {
                            crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_types::EvolutionPatternType::RecurringError
                        } else {
                            crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_types::EvolutionPatternType::StrategyDiscovery
                        };
                        let ts = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0) as i64;
                        let record =
                            crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_types::EvolutionRecord {
                                id: format!("evol_pipe_{}", brain.iteration),
                                source_conversation_id: format!("pipe_iter_{}", brain.iteration),
                                pattern_type: pattern_type.clone(),
                                description: summary,
                                before_behavior: format!("reward_before={:.4}", brain._reward),
                                after_behavior: String::new(),
                                effectiveness_gain: brain._reward,
                                applied_to: vec![],
                                verified: false,
                                timestamp: ts,
                            };
                        let _ = kb.store_evolution_record(&record);
                        log::info!("[conversation_distill] EvolRecord={:?}", pattern_type);
                    }
                }
            }
        }
        Ok(StageDecision::Continue)
    }
}


/// EWHR Hypothesis Accuracy Stage: evaluates hypothesis predictions
/// against actual outcomes and updates calibration. Runs every 5 iterations.
pub struct _HypothesisAccuracyStage;
impl Default for _HypothesisAccuracyStage {
    fn default() -> Self {
        Self
    }
}
impl _HypothesisAccuracyStage {
    pub fn new() -> Self {
        Self
    }
}
impl BrainStage for _HypothesisAccuracyStage {
    fn name(&self) -> &str {
        "hypothesis_accuracy"
    }
    fn frequency(&self) -> usize {
        5
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if let Some(ref engine) = brain.reasoning_engine {
            if let Some(ref net_lock) = engine.hypothesis_network {
                if let Ok(net) = net_lock.lock() {
                    let total = net.hypotheses.len();
                    let supported = net.hypotheses.iter().filter(|h| matches!(h.status, crate::l4_emotion::nt_memory::nt_memory_historian::nt_evidence_hypothesis::HypothesisStatus::Supported)).count();
                    let refuted = net.hypotheses.iter().filter(|h| matches!(h.status, crate::l4_emotion::nt_memory::nt_memory_historian::nt_evidence_hypothesis::HypothesisStatus::Refuted)).count();
                    if total > 0 {
                        log::info!(
                            "[EWHR] Hypothesis accuracy: {}/{} supported, {}/{} refuted",
                            supported,
                            total,
                            refuted,
                            total
                        );
                    }
                }
            }
        }
        Ok(StageDecision::Continue)
    }
}


/// EWHR Pattern Extraction Stage: converts successful hypotheses
/// into reusable procedural memory (skills). Runs every 10 iterations.
pub struct _PatternExtractionStage;
impl Default for _PatternExtractionStage {
    fn default() -> Self {
        Self
    }
}
impl _PatternExtractionStage {
    pub fn new() -> Self {
        Self
    }
}
impl BrainStage for _PatternExtractionStage {
    fn name(&self) -> &str {
        "pattern_extraction"
    }
    fn frequency(&self) -> usize {
        10
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if let (Some(ref engine), Some(ref kb)) = (&brain.reasoning_engine, &brain._nt_memory_kb) {
            // Init GraphRAG on first run
            if kb
                .graphrag_store
                .read()
                .map(|s| s.is_none())
                .unwrap_or(false)
            {
                let _ = kb.init_graphrag(GraphRagConfig::default());
            }
            // Extract entities from new knowledge
            if let Some(ref net_lock) = engine.hypothesis_network {
                if let Ok(net) = net_lock.lock() {
                    for h in &net.hypotheses {
                        if matches!(h.status, HypothesisStatus::Supported) {
                            let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
                            let record = ProceduralMemoryRecord {
                                id: format!("ewhr_{}", h.id),
                                skill_id: h.id.clone(),
                                name: h.title.clone(),
                                description: h.description.clone(),
                                e8_sequence: vec![],
                                trigger_pattern: vec![],
                                success_rate: h.posterior_probability,
                                execution_count: 1,
                                avg_reward: 0.0,
                                created_at: now.clone(),
                                updated_at: now,
                                tags: h.tags.clone(),
                            };
                            let _ = kb.store_procedural_memory(&record);
                            let _ = kb.graphrag_extract(&h.description, &h.id);
                        }
                    }
                }
            }
        }
        Ok(StageDecision::Continue)
    }
}


/// Self-review stage: runs SelfReviewGate mechanical checks every iteration.
/// Non-blocking — logs findings but never returns Rollback.
pub struct SelfReviewStage {
    pub strict_mode: bool,
}

impl Default for SelfReviewStage {
    fn default() -> Self {
        Self { strict_mode: true }
    }
}

impl SelfReviewStage {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn _with_strict(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }
}

impl BrainStage for SelfReviewStage {
    fn name(&self) -> &str {
        "self_review"
    }

    fn frequency(&self) -> usize {
        1
    }

    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        // Extract observer feedback from reasoning engine if available
        let (observer_quality, observer_patterns) = brain
            .reasoning_engine
            .as_ref()
            .map(|re| {
                let q = re
                    .observer
                    .last_report
                    .as_ref()
                    .map(|r| r.quality_score)
                    .unwrap_or(0.5);
                let pats = re
                    .observer
                    .last_report
                    .as_ref()
                    .map(|r| r.critical_patterns.clone())
                    .unwrap_or_default();
                (q, pats)
            })
            .unwrap_or((0.5, vec![]));

        let mut gate = SelfReviewGate::new(self.strict_mode)
            .with_observer_feedback(observer_quality, observer_patterns.clone());
        // 单元测试不打全树 self-review 扫描 (run_all ~50 次全树扫描, 单次 ~6s)。
        // 测试验证循环机制; 真实审查留待集成测试/生产路径 (与 cfg!(test) 隔离纪律一致)。
        let report = if cfg!(test) {
            crate::l6_meta::nt_core_self_review::SelfReviewReport {
                findings: Vec::new(),
                passed: 1,
                failed: 0,
                warnings: 0,
            }
        } else {
            gate.run_all()
        };
        let blast = gate.blast_radius();

        log::info!(
            "[self_review] {} passed, {} failed, {} warnings — blast: {} ({} files, {} crossings)",
            report.passed,
            report.failed,
            report.warnings,
            blast.risk,
            blast.affected_files,
            blast.module_crossings,
        );
        if !report.is_pass() {
            log::warn!(
                "[self_review] Failed checks: {} failed, {} warnings — review {}",
                report.failed,
                report.warnings,
                report.summary(),
            );
        }

        let findings_json = serde_json::json!({
            "stage": "self_review",
            "iteration": brain.iteration,
            "passed": report.passed,
            "failed": report.failed,
            "warnings": report.warnings,
            "blast_risk": format!("{}", blast.risk),
            "blast_affected_files": blast.affected_files,
            "blast_module_crossings": blast.module_crossings,
            "observer_quality": observer_quality,
            "observer_patterns": observer_patterns,
            "findings": report.findings.iter().map(|f| {
                serde_json::json!({
                    "severity": format!("{:?}", f.severity),
                    "category": f.category,
                    "message": f.message,
                    "file": f.file,
                })
            }).collect::<Vec<_>>(),
        });
        if let Some(ref kb) = brain._nt_memory_kb {
            let json_str = serde_json::to_string(&findings_json).unwrap_or_default();
            let _ = kb.kv_set("self_review", "latest", &json_str);
            // Store blast radius separately for trend analysis
            let blast_json = serde_json::json!({
                "iteration": brain.iteration,
                "risk": format!("{}", blast.risk),
                "files_scanned": blast.files_scanned,
                "affected_files": blast.affected_files,
                "module_crossings": blast.module_crossings,
            });
            let _ = kb.kv_set(
                "self_review_blast",
                &brain.iteration.to_string(),
                &serde_json::to_string(&blast_json).unwrap_or_default(),
            );
        }
        Ok(StageDecision::Continue)
    }
}


make_stage!(ExternalKnowledgeAbsorbStage);
impl BrainStage for ExternalKnowledgeAbsorbStage {
    fn name(&self) -> &str {
        "external_knowledge_absorb"
    }
    fn frequency(&self) -> usize {
        20
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if brain.iteration == 0 || !brain.iteration.is_multiple_of(20) {
            return Ok(StageDecision::Continue);
        }
        let tick = brain.iteration;
        // Open a temporary KB connection for the explorer instead of consuming
        // the pipeline's KB connection (which would lose pending transactions,
        // LRU cache state, and uncommitted embedding data).
        let explorer_kb = match crate::l4_emotion::nt_memory::nt_memory_kb::KnowledgeBase::open(None) {
            Ok(kb) => kb,
            Err(e) => {
                log::warn!(
                    "[external_knowledge_absorb] tick={}, failed to open temp KB: {}",
                    tick,
                    e
                );
                return Ok(StageDecision::Continue);
            }
        };
        let config =
            crate::l2_perception::nt_world::nt_world_exploration_engine::ExplorationConfig::default(
            );
        let mut explorer =
            crate::l2_perception::nt_world::nt_world_exploration_engine::ExplorationEngine::new(
                config,
            );
        explorer.attach_kb(Box::new(explorer_kb) as Box<dyn crate::l0_substrate::nt_core_traits::KnowledgeSink>);
        let report = explorer.run_cycle();
        log::info!(
            "[external_knowledge_absorb] tick={}, explore: discovered={}, ingested={}, skipped={}, failed={}, total_in_kb={}",
            tick, report.discovered, report.ingested, report.skipped, report.failed, report.total_in_kb
        );
        Ok(StageDecision::Continue)
    }
}


// ── External Brain Digest Stage (外置大脑消化闭环) ───────────────────────
// 把外置大脑 corpus (`knowledge-archive-corpus-*.db`) 经能力消化环持续接入 live KB,
// 使 SEAL 调度闭环能自主把冷存档转化为活能力, 无需人工触发 (R-P79 / Dark Forest: 连接不膨胀)。
//   Phase 1  digest_sample  → 有界激活冷节点进 live KB
//   Phase 6  prune_external  → 反向修剪孤儿外置条目 (dry-run 安全, 不破坏冷存档)
//   Skill    SkillEngine::maintain → 技能索引维护 (UCN Phase 1 写通)
pub struct _ExternalBrainDigestStage;
impl Default for _ExternalBrainDigestStage {
    fn default() -> Self {
        Self
    }
}
impl _ExternalBrainDigestStage {
    pub fn new() -> Self {
        Self
    }
}
impl BrainStage for _ExternalBrainDigestStage {
    fn name(&self) -> &str {
        "external_brain_digest"
    }
    fn frequency(&self) -> usize {
        20
    }
    fn process(&self, brain: &mut SelfIteratingBrain) -> Result<StageDecision, NeoTrixError> {
        if brain.iteration == 0 || !brain.iteration.is_multiple_of(20) {
            return Ok(StageDecision::Continue);
        }
        let tick = brain.iteration;
        let corpus = match crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_resource_ingest::corpus_archive_path() {
            Some(p) => p,
            None => {
                log::debug!("[external_brain_digest] tick={}, 外置大脑未挂载, 跳过", tick);
                return Ok(StageDecision::Continue);
            }
        };
        if !corpus.exists() {
            log::debug!("[external_brain_digest] tick={}, corpus 不存在, 跳过", tick);
            return Ok(StageDecision::Continue);
        }
        let kb = match crate::l4_emotion::nt_memory::nt_memory_kb::KnowledgeBase::open(None) {
            Ok(kb) => std::sync::Arc::new(kb),
            Err(e) => {
                log::warn!("[external_brain_digest] tick={}, 打开 KB 失败: {}", tick, e);
                return Ok(StageDecision::Continue);
            }
        };
        let conn_guard = match kb.conn.lock() {
            Ok(g) => g,
            Err(_) => {
                log::warn!("[external_brain_digest] tick={}, KB 锁中毒", tick);
                return Ok(StageDecision::Continue);
            }
        };
        let conn: &rusqlite::Connection = &conn_guard;

        // Phase 1: 有界激活冷节点进 live KB
        match crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_cortex_sync::digest_sample(
            conn, &corpus, 200, None,
        ) {
            Ok(rep) => log::info!(
                "[external_brain_digest] tick={}, digest: sampled={}, activated={}, already_live={}, bytes={}",
                tick, rep.sampled, rep.activated, rep.already_live, rep.bytes
            ),
            Err(e) => log::warn!("[external_brain_digest] tick={}, digest 失败: {}", tick, e),
        }

        // Phase 6: 反向修剪 (dry-run 安全, 不破坏冷存档)
        match crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_cortex_sync::prune_external(
            conn, &corpus, 30, true,
        ) {
            Ok(n) => log::info!(
                "[external_brain_digest] tick={}, prune(dry-run) 候选: {}",
                tick, n
            ),
            Err(e) => log::warn!("[external_brain_digest] tick={}, prune 失败: {}", tick, e),
        }

        drop(conn_guard);

        // Skill 索引维护 (UCN Phase 1 写通 KB skills_index)
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let skills_dir = std::path::PathBuf::from(&home).join(".neotrix").join("skills");
        let mut engine = crate::l5_cognition::nt_mind::nt_mind_skill_engine::SkillEngine::new(
            skills_dir,
        )
        .with_kb(kb.clone());
        let maintained = engine.maintain();
        log::info!(
            "[external_brain_digest] tick={}, skills maintained: {}",
            tick, maintained
        );

        Ok(StageDecision::Continue)
    }
}

