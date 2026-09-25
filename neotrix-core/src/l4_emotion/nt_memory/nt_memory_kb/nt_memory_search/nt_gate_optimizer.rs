use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::super::nt_memory_types::*;
use super::nt_pure_fns::decay_factor;

// ── Gate + optimizer (moved from nt_memory_search.rs, pure move) ──

// ═══════════════════════════════════════════════════════════════════
// P0: CraniMEM Gating Mechanism
// ═══════════════════════════════════════════════════════════════════
// Goal-conditioned input filtering for GWT salience.
// Filters sensory events based on current active goals, preventing
// irrelevant stimuli from consuming attention bandwidth.

/// Goal-conditioned gating for sensory input.
/// Each goal has associated keywords/topics; events are scored by
/// relevance to active goals before entering GWT salience computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CraniMEMGate {
    /// Active goals with associated relevance keywords
    pub active_goals: Vec<GoalContext>,
    /// Minimum relevance threshold to pass the gate (0.0 = all pass)
    pub threshold: f64,
    /// Emergency override: when true, all events pass (e.g. system alerts)
    pub emergency_override: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalContext {
    pub goal_id: String,
    pub keywords: Vec<String>,
    pub weight: f64,
}

impl Default for CraniMEMGate {
    fn default() -> Self {
        Self {
            active_goals: Vec::new(),
            threshold: 0.1,
            emergency_override: false,
        }
    }
}

impl CraniMEMGate {
    /// Compute gating score for a sensory event description.
    /// Returns (passed, score) — score in [0.0, 1.0], passed = score >= threshold.
    pub fn gate_event(&self, event_description: &str) -> (bool, f64) {
        if self.emergency_override || self.active_goals.is_empty() {
            return (true, 1.0);
        }
        let desc_lower = event_description.to_lowercase();
        let max_relevance: f64 = self
            .active_goals
            .iter()
            .map(|g| {
                let keyword_hits = g
                    .keywords
                    .iter()
                    .filter(|kw| desc_lower.contains(&kw.to_lowercase()))
                    .count();
                if keyword_hits == 0 {
                    0.0
                } else {
                    g.weight * (keyword_hits as f64 / g.keywords.len() as f64).min(1.0)
                }
            })
            .fold(0.0, f64::max);
        (max_relevance >= self.threshold, max_relevance)
    }

    /// Filter a batch of sensory events, returning only those that pass the gate.
    pub fn filter_events(&self, events: Vec<(String, f64)>) -> Vec<(String, f64)> {
        events
            .into_iter()
            .filter(|(desc, _)| self.gate_event(desc).0)
            .collect()
    }

    /// Update active goals (called by task dispatcher / goal loop).
    pub fn set_goals(&mut self, goals: Vec<GoalContext>) {
        self.active_goals = goals;
    }

    /// Set emergency override (system alerts bypass gating).
    pub fn set_emergency(&mut self, active: bool) {
        self.emergency_override = active;
    }
}

// ── FTS5 Optimization Configuration (absorbed from ZSTD+FTS5 180,000× pattern 2026) ──
pub struct Fts5OptimizerConfig {
    /// cache_size in KB (negative = KB, positive = pages). Default: -256000 (256MB)
    pub cache_size: i64,
    /// mmap_size for memory-mapped I/O. Default: 268435456 (256MB)
    pub mmap_size: i64,
    /// page_size (512-65536, power of 2). Default: 4096
    pub page_size: i64,
    /// synchronous mode. Default: "NORMAL"
    pub synchronous: &'static str,
    /// journal mode. Default: "WAL"
    pub journal_mode: &'static str,
    /// busy_timeout in ms. Default: 5000
    pub busy_timeout: i64,
    /// Auto-ANALYZE after bulk inserts for query planner optimization
    pub auto_analyze: bool,
}

impl Default for Fts5OptimizerConfig {
    fn default() -> Self {
        Self {
            cache_size: -256000,
            mmap_size: 268435456,
            page_size: 4096,
            synchronous: "NORMAL",
            journal_mode: "WAL",
            busy_timeout: 5000,
            auto_analyze: true,
        }
    }
}

impl Fts5OptimizerConfig {
    pub(crate) fn _apply_pragmas(&self, conn: &rusqlite::Connection) -> rusqlite::Result<()> {
        conn.pragma_update(None, "cache_size", self.cache_size)?;
        conn.pragma_update(None, "mmap_size", self.mmap_size)?;
        conn.pragma_update(None, "page_size", self.page_size)?;
        conn.pragma_update(None, "synchronous", self.synchronous)?;
        conn.pragma_update(None, "journal_mode", self.journal_mode)?;
        conn.pragma_update(None, "busy_timeout", self.busy_timeout)?;
        if self.auto_analyze {
            conn.execute_batch("ANALYZE;").ok();
        }
        Ok(())
    }

    /// Contentless FTS5 table schema: stores only the inverted index, not the full text.
    /// Requires a secondary `nodes` table join for full text retrieval.
    /// Trade-off: halves FTS storage at cost of one extra lookup per result.
    pub const CONTENTLESS_FTS_SCHEMA: &'static str =
        "CREATE VIRTUAL TABLE IF NOT EXISTS nodes_fts USING fts5(
            title, summary, content,
            content='nodes',
            content_rowid='rowid',
            tokenize='unicode61 remove_diacritics=2'
        );";

    /// Triggers to keep FTS index in sync with nodes table changes
    pub(crate) const _FTS_SYNC_TRIGGERS: &'static str = r#"
        CREATE TRIGGER IF NOT EXISTS nodes_ai AFTER INSERT ON nodes BEGIN
            INSERT INTO nodes_fts(rowid, title, summary, content)
            VALUES (new.rowid, new.title, new.summary, new.content);
        END;
        CREATE TRIGGER IF NOT EXISTS nodes_ad AFTER DELETE ON nodes BEGIN
            INSERT INTO nodes_fts(nodes_fts, rowid, title, summary, content)
            VALUES ('delete', old.rowid, old.title, old.summary, old.content);
        END;
        CREATE TRIGGER IF NOT EXISTS nodes_au AFTER UPDATE ON nodes BEGIN
            INSERT INTO nodes_fts(nodes_fts, rowid, title, summary, content)
            VALUES ('delete', old.rowid, old.title, old.summary, old.content);
            INSERT INTO nodes_fts(rowid, title, summary, content)
            VALUES (new.rowid, new.title, new.summary, new.content);
        END;
    "#;

    pub(crate) fn _rebuild_fts(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
        conn.execute_batch("INSERT INTO nodes_fts(nodes_fts) VALUES('rebuild');")
    }
}

// ═══════════════════════════════════════════════════════════════════
// RetrievalEvolver — 检索自进化 (SimpleMem EvolveMem absorb, G4)
// ═══════════════════════════════════════════════════════════════════
// EvolveMem 闭环: Evaluate(记录每次检索质量) → Diagnose(定位低效查询类)
// → Propose(提出调参建议) → Guard(单调性门: 仅当新窗口均值优于 committed
// 基线才提交, 否则回滚)。提交的 tuning 持久化 (kv_store) 并影响后续召回深度
// (VSA 扩召 top_k), 使检索机制自身随使用自评自调。

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalEvalPoint {
    pub query: String,
    pub results_len: usize,
    pub mean_score: f64,
    pub ts: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RetrievalTuning {
    /// VSA 扩召 top_k 的召回加成 (self-evolved), clamp 到 [-2, +4]
    pub boost: f64,
    pub committed_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnosis {
    pub class: String,
    pub sample_count: usize,
    pub mean_score: f64,
    pub degraded: bool,
}

#[derive(Debug)]
pub struct RetrievalEvolver {
    window: Vec<RetrievalEvalPoint>,
    max_window: usize,
    pub tuning: RetrievalTuning,
    baseline_mean: Option<f64>,
}

impl Default for RetrievalEvolver {
    fn default() -> Self {
        Self::new()
    }
}

impl RetrievalEvolver {
    pub fn new() -> Self {
        Self {
            window: Vec::new(),
            max_window: 128,
            tuning: RetrievalTuning::default(),
            baseline_mean: None,
        }
    }

    pub fn window_len(&self) -> usize {
        self.window.len()
    }

    pub fn window_mean(&self) -> Option<f64> {
        if self.window.is_empty() {
            return None;
        }
        Some(self.window.iter().map(|p| p.mean_score).sum::<f64>() / self.window.len() as f64)
    }

    /// Evaluate: 记录一次检索质量 (每次 production search 调用)
    pub fn evaluate(&mut self, query: &str, results_len: usize, mean_score: f64) {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        if self.window.len() >= self.max_window {
            self.window.remove(0);
        }
        self.window.push(RetrievalEvalPoint {
            query: query.to_string(),
            results_len,
            mean_score,
            ts,
        });
    }

    /// Diagnose: 按查询类聚合并报告低效类。类: long(>5 token) / short / empty(0结果)
    pub fn diagnose(&self) -> Vec<Diagnosis> {
        let overall = self.window_mean().unwrap_or(0.0);
        let mut classes: HashMap<&str, Vec<&RetrievalEvalPoint>> = HashMap::new();
        for p in &self.window {
            if p.results_len == 0 {
                classes.entry("empty").or_default().push(p);
            } else if p.query.split_whitespace().count() > 5 {
                classes.entry("long").or_default().push(p);
            } else {
                classes.entry("short").or_default().push(p);
            }
        }
        classes
            .into_iter()
            .map(|(class, pts)| {
                let mean = pts.iter().map(|p| p.mean_score).sum::<f64>() / pts.len() as f64;
                Diagnosis {
                    class: class.to_string(),
                    sample_count: pts.len(),
                    mean_score: mean,
                    degraded: mean < overall * 0.9 && !pts.is_empty(),
                }
            })
            .collect()
    }

    /// Propose: 基于诊断提出调参建议 — 低效类占比高则建议提升召回加成
    pub fn propose(&self) -> Option<RetrievalTuning> {
        let diagnoses = self.diagnose();
        if self.window.is_empty() {
            return None;
        }
        let degraded_share = diagnoses
            .iter()
            .filter(|d| d.degraded)
            .map(|d| d.sample_count)
            .sum::<usize>() as f64
            / self.window.len() as f64;
        let empty_share = diagnoses
            .iter()
            .find(|d| d.class == "empty")
            .map(|d| d.sample_count as f64 / self.window.len() as f64)
            .unwrap_or(0.0);
        let new_boost = if empty_share > 0.3 || degraded_share > 0.5 {
            (self.tuning.boost + 0.5).min(4.0)
        } else if degraded_share < 0.15 && self.tuning.boost > 0.0 {
            // 过度激进 → 适当回退 (防过度召回噪声)
            (self.tuning.boost - 0.5).max(-2.0)
        } else {
            self.tuning.boost
        };
        if (new_boost - self.tuning.boost).abs() < 1e-9 {
            return None;
        }
        Some(RetrievalTuning {
            boost: new_boost,
            committed_at: 0,
        })
    }

    /// Guard: 单调性门 — 仅当提交后窗口均值 ≥ committed 基线才接受调参,
    /// 否则拒绝 (保留原 tuning)。返回是否提交。
    pub fn guard(&mut self, proposal: &RetrievalTuning) -> bool {
        let Some(current_mean) = self.window_mean() else {
            return false;
        };
        let baseline = self.baseline_mean.unwrap_or(current_mean);
        if current_mean >= baseline {
            self.tuning = proposal.clone();
            self.tuning.committed_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            self.baseline_mean = Some(current_mean);
            true
        } else {
            false
        }
    }

    /// 自进化主循环: Evaluate 已由外部调用; 每 max_window 次评估执行一次
    /// Diagnose→Propose→Guard 并返回是否提交了调参。
    pub fn evolve_if_due(&mut self) -> Option<RetrievalTuning> {
        if self.window.len() < self.max_window {
            return None;
        }
        let proposal = self.propose()?;
        if self.guard(&proposal) {
            Some(self.tuning.clone())
        } else {
            None
        }
    }

    /// 当前生效的召回加成 (clamp 供外部使用)
    pub fn recall_boost(&self) -> f64 {
        self.tuning.boost.clamp(-2.0, 4.0)
    }
}

/// 检索精度门控 (arXiv:2608.14036 "Demystifying Agent Skills" absorbed 2026-08-18):
/// 技能/记忆候选池从 5 条增长到 100 条时, actual-use precision 从 29.6% 崩到 3.3% —
/// 检索是独立于技能质量的瓶颈。池规模越大, 低分结果被实际使用的概率越低,
/// 因此按池规模收紧分数阈值, 丢弃明显低质候选, 抑制 precision 崩塌。
pub fn precision_gate(results: Vec<SearchResult>, pool_size: usize) -> Vec<SearchResult> {
    if results.is_empty() {
        return results;
    }
    // 池规模阈值: 池越大门槛越高 (对数标度)。pool 5→100 → precision 29.6%→3.3%。
    // 使用 sqrt(log2(pool+1)) 使阈值在常见池规模 (10-500) 内单调收紧且不会全杀。
    let log2_pool = (pool_size as f64 + 1.0).log2();
    let threshold_ratio = (0.25 + 0.10 * log2_pool).min(0.9);
    let top = results.iter().map(|r| r.score).fold(f64::MIN, f64::max);
    if top <= 0.0 {
        return results;
    }
    let cutoff = top * threshold_ratio;
    let retained: Vec<SearchResult> = results.into_iter().filter(|r| r.score >= cutoff).collect();
    // 保底: 小池/无低分时不得清空 (precision gate 是软化, 不是硬截断)
    if retained.is_empty() {
        vec![]
    } else {
        retained
    }
}

/// 陈旧信号标注 (codegraph staleness-signaling absorbed 2026-08-19):
/// 填充 SearchResult.signals 预留槽 `[f64;4]` — 该槽此前从未被写入。
/// 语义 (对齐 codegraph "索引永不过期" + agentmemory 7 天半衰期):
///   signals[0] = stale_ratio   1.0=全新, →0.0=陈旧 (decay_factor 语义)
///   signals[1] = age_days      节点距上次更新天数
///   signals[2] = decay         <0.5 表示超过 1 个半衰期
///   signals[3] = stale_banner  age > 30 天 → 1.0 (消费方据此显示 ⚠️ stale 横幅)
/// 检索结果信封从此携带 trust 级别 — agent 知道每个结果是新鲜的还是陈旧的。
pub fn staleness_signal(results: Vec<SearchResult>) -> Vec<SearchResult> {
    const HALF_LIFE_SECS: i64 = 7 * 24 * 3600; // 7 天
    const STALE_DAYS: i64 = 30;
    let now = chrono::Utc::now().timestamp();
    results
        .into_iter()
        .map(|mut r| {
            let age = now.saturating_sub(r.node.updated_at).max(0);
            let age_days = age / 86400;
            let decay = decay_factor(age, HALF_LIFE_SECS);
            let stale_banner = if age_days > STALE_DAYS { 1.0 } else { 0.0 };
            r.signals = Some([decay, age_days as f64, decay, stale_banner]);
            r
        })
        .collect()
}
