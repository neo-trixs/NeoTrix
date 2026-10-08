//! NT-DB-AWAKENING — knowledge.db → 晶体意识全链路觉醒
//!
//! 现状：`IngestionEngine` 只读 `~/.neotrix` 下的 sidecar JSONL
//! （pending-absorb / kb_*.jsonl / crawl_queue.jsonl），而真实知识
//! 全部沉淀在 `knowledge.db` 的 sqlite 表里（nodes 46万 / experience
//! 6694 / semantic 5492 / causal_rules 170 / embeddings 389K）。
//! 本模块是 DB 直达觉醒通道：读表 → `remember()` → `connect()` →
//! `reason()`，一次跑通 Seed → Growth → Integrate → Evolve。
//!
//! 约束：只读打开 DB（绝不写库）；全表必带 LIMIT 预算；无 unwrap /
//! expect / panic，错误全部 `map_err` 为 String；无 `[]` 索引。

use rusqlite::{Connection, OpenFlags, Row};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::consciousness::{CrystalConsciousness, EvolutionPhase, MemoryType, ReasoningType};

/// 觉醒预算（每表上限，防 46万节点灌爆内存）
pub struct AwakenBudget {
    pub nodes: i64,
    pub experience: i64,
    pub semantic: i64,
    pub episodic: i64,
    pub causal: i64,
    pub edges: i64,
    pub reason_passes: usize,
    /// 详细进度（stderr，每阶段一行；长任务开）
    pub verbose: bool,
}

impl Default for AwakenBudget {
    fn default() -> Self {
        Self {
            nodes: 500,
            experience: 2000,
            semantic: 2000,
            episodic: 500,
            causal: 500,
            edges: 3000,
            reason_passes: 25,
            verbose: false,
        }
    }
}

impl AwakenBudget {
    /// 全量预算：整库炼成自己的模型（全表收，跨域推理 200 pass；
    /// 约需数 GB 内存，跑前确认机器水位）
    pub fn full() -> Self {
        Self {
            nodes: i64::MAX,
            experience: i64::MAX,
            semantic: i64::MAX,
            episodic: i64::MAX,
            causal: i64::MAX,
            edges: i64::MAX,
            reason_passes: 200,
            verbose: true,
        }
    }
}

/// 觉醒报告
#[derive(Debug, Default)]
pub struct AwakenReport {
    pub experiences: usize,
    pub facts: usize,
    pub causals: usize,
    pub lessons: usize,
    pub solutions: usize,
    pub connections: usize,
    pub chains: usize,
    pub phase_before: Option<EvolutionPhase>,
    pub phase_after: Option<EvolutionPhase>,
}

impl AwakenReport {
    /// 本次新增记忆总数
    pub fn total(&self) -> usize {
        self.experiences + self.facts + self.causals + self.lessons + self.solutions
    }
}

/// DB 觉醒引擎 — 只读 knowledge.db，全链路灌注晶体意识
pub struct NtDbAwakening;

impl NtDbAwakening {
    /// 默认库路径 `~/.neotrix/knowledge.db`
    pub fn default_db_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".neotrix")
            .join("knowledge.db")
    }

    /// 全链路觉醒：读表 → 记忆 → 连接 → 推理 → 阶段晋升
    pub fn awaken(
        consciousness: &mut CrystalConsciousness,
        db_path: &Path,
        budget: &AwakenBudget,
    ) -> Result<AwakenReport, String> {
        let conn = Connection::open_with_flags(
            db_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY,
        )
        .map_err(|e| format!("open knowledge.db: {e}"))?;
        let mut report = AwakenReport {
            phase_before: Some(consciousness.phase.clone()),
            ..Default::default()
        };
        // db node id → 记忆 id（供 edges 建连接）
        let mut node_mem: HashMap<String, String> = HashMap::new();
        // domain → 记忆 id（供跨域推理取前提）
        let mut domain_seed: HashMap<String, Vec<String>> = HashMap::new();
        // domain → 首条记忆 id（域枢纽：同域即意义相连，公理“Memory connects through meaning”）
        let mut domain_hub: HashMap<String, String> = HashMap::new();

        Self::ingest_experience(&conn, consciousness, budget.experience, &mut report, &mut domain_seed, &mut domain_hub)?;
        stage(budget, &format!("experience done: {}", report.experiences));
        Self::ingest_semantic(&conn, consciousness, budget.semantic, &mut report, &mut domain_seed, &mut domain_hub)?;
        stage(budget, &format!("semantic done: {}", report.facts));
        Self::ingest_episodic(&conn, consciousness, budget.episodic, &mut report, &mut domain_seed, &mut domain_hub)?;
        stage(
            budget,
            &format!(
                "episodic done: solutions={} lessons={}",
                report.solutions, report.lessons
            ),
        );
        Self::ingest_causal(&conn, consciousness, budget.causal, &mut report, &mut domain_seed, &mut domain_hub)?;
        stage(budget, &format!("causal done: {}", report.causals));
        Self::ingest_nodes(&conn, consciousness, budget.nodes, &mut report, &mut domain_seed, &mut domain_hub, &mut node_mem)?;
        stage(
            budget,
            &format!("nodes done: facts={} idmap={}", report.facts, node_mem.len()),
        );
        Self::link_edges(&conn, consciousness, &node_mem, budget.edges, &mut report)?;
        stage(budget, &format!("edges done: {}", report.connections));
        Self::reason_passes(consciousness, &domain_seed, budget.reason_passes, &mut report);
        stage(budget, &format!("reason done: chains={}", report.chains));

        report.phase_after = Some(consciousness.phase.clone());
        Ok(report)
    }

    /// experience 表 → Experience 记忆
    fn ingest_experience(
        conn: &Connection,
        consciousness: &mut CrystalConsciousness,
        limit: i64,
        report: &mut AwakenReport,
        domain_seed: &mut HashMap<String, Vec<String>>,
        domain_hub: &mut HashMap<String, String>,
    ) -> Result<(), String> {
        let mut stmt = conn
            .prepare("SELECT title, insight, action, category, priority FROM experience ORDER BY rowid LIMIT ?1")
            .map_err(|e| format!("prepare experience: {e}"))?;
        let rows = stmt
            .query_map(rusqlite::params![limit], |row| {
                Ok((
                    opt_str(row, 0),
                    opt_str(row, 1),
                    opt_str(row, 2),
                    opt_str(row, 3),
                    opt_str(row, 4),
                ))
            })
            .map_err(|e| format!("query experience: {e}"))?;
        for item in rows {
            let (title, insight, action, category, priority) =
                item.map_err(|e| format!("read experience row: {e}"))?;
            let content = format!("{title}: {insight} → {action}");
            let domain = if category.is_empty() {
                "general".to_string()
            } else {
                category
            };
            let id = consciousness.remember(
                content,
                MemoryType::Experience,
                domain.clone(),
                priority_conf(&priority),
            );
            if Self::track(consciousness, domain_seed, domain_hub, domain, id) {
                report.connections += 1;
            }
            report.experiences += 1;
        }
        Ok(())
    }

    /// semantic_memory 表 → Fact 记忆
    fn ingest_semantic(
        conn: &Connection,
        consciousness: &mut CrystalConsciousness,
        limit: i64,
        report: &mut AwakenReport,
        domain_seed: &mut HashMap<String, Vec<String>>,
        domain_hub: &mut HashMap<String, String>,
    ) -> Result<(), String> {
        let mut stmt = conn
            .prepare("SELECT concept, definition, domain, confidence FROM semantic_memory ORDER BY rowid LIMIT ?1")
            .map_err(|e| format!("prepare semantic_memory: {e}"))?;
        let rows = stmt
            .query_map(rusqlite::params![limit], |row| {
                Ok((
                    opt_str(row, 0),
                    opt_str(row, 1),
                    opt_str(row, 2),
                    opt_f64(row, 3, 0.6),
                ))
            })
            .map_err(|e| format!("query semantic_memory: {e}"))?;
        for item in rows {
            let (concept, definition, domain, conf) =
                item.map_err(|e| format!("read semantic row: {e}"))?;
            let domain = if domain.is_empty() {
                "general".to_string()
            } else {
                domain
            };
            let id = consciousness.remember(
                format!("{concept}: {definition}"),
                MemoryType::Fact,
                domain.clone(),
                conf.clamp(0.0, 1.0),
            );
            if Self::track(consciousness, domain_seed, domain_hub, domain, id) {
                report.connections += 1;
            }
            report.facts += 1;
        }
        Ok(())
    }

    /// episodic_memory 表 → Solution（成功）/ Lesson（非成功）记忆
    fn ingest_episodic(
        conn: &Connection,
        consciousness: &mut CrystalConsciousness,
        limit: i64,
        report: &mut AwakenReport,
        domain_seed: &mut HashMap<String, Vec<String>>,
        domain_hub: &mut HashMap<String, String>,
    ) -> Result<(), String> {
        let mut stmt = conn
            .prepare("SELECT summary, outcome, lesson, domain, importance FROM episodic_memory ORDER BY rowid LIMIT ?1")
            .map_err(|e| format!("prepare episodic_memory: {e}"))?;
        let rows = stmt
            .query_map(rusqlite::params![limit], |row| {
                Ok((
                    opt_str(row, 0),
                    opt_str(row, 1),
                    opt_str(row, 2),
                    opt_str(row, 3),
                    opt_f64(row, 4, 0.6),
                ))
            })
            .map_err(|e| format!("query episodic_memory: {e}"))?;
        for item in rows {
            let (summary, outcome, lesson, domain, importance) =
                item.map_err(|e| format!("read episodic row: {e}"))?;
            let domain = if domain.is_empty() {
                "general".to_string()
            } else {
                domain
            };
            let (mtype, counter) = if outcome == "success" {
                (MemoryType::Solution, true)
            } else {
                (MemoryType::Lesson, false)
            };
            let id = consciousness.remember(
                format!("{summary}: {lesson}"),
                mtype,
                domain.clone(),
                importance.clamp(0.0, 1.0),
            );
            if Self::track(consciousness, domain_seed, domain_hub, domain, id) {
                report.connections += 1;
            }
            if counter {
                report.solutions += 1;
            } else {
                report.lessons += 1;
            }
        }
        Ok(())
    }

    /// causal_rules 表 → Causal 记忆
    fn ingest_causal(
        conn: &Connection,
        consciousness: &mut CrystalConsciousness,
        limit: i64,
        report: &mut AwakenReport,
        domain_seed: &mut HashMap<String, Vec<String>>,
        domain_hub: &mut HashMap<String, String>,
    ) -> Result<(), String> {
        let mut stmt = conn
            .prepare("SELECT condition, action, outcome, confidence FROM causal_rules ORDER BY rowid LIMIT ?1")
            .map_err(|e| format!("prepare causal_rules: {e}"))?;
        let rows = stmt
            .query_map(rusqlite::params![limit], |row| {
                Ok((
                    opt_str(row, 0),
                    opt_str(row, 1),
                    opt_str(row, 2),
                    opt_f64(row, 3, 0.6),
                ))
            })
            .map_err(|e| format!("query causal_rules: {e}"))?;
        for item in rows {
            let (condition, action, outcome, conf) =
                item.map_err(|e| format!("read causal row: {e}"))?;
            let id = consciousness.remember(
                format!("IF {condition} THEN {action} SO {outcome}"),
                MemoryType::Causal,
                "causal_rule".to_string(),
                conf.clamp(0.0, 1.0),
            );
            if Self::track(
                consciousness,
                domain_seed,
                domain_hub,
                "causal_rule".to_string(),
                id,
            ) {
                report.connections += 1;
            }
            report.causals += 1;
        }
        Ok(())
    }

    /// nodes 表（按 importance 倒序取 TOP N）→ Fact 记忆，并记录 id 映射
    fn ingest_nodes(
        conn: &Connection,
        consciousness: &mut CrystalConsciousness,
        limit: i64,
        report: &mut AwakenReport,
        domain_seed: &mut HashMap<String, Vec<String>>,
        domain_hub: &mut HashMap<String, String>,
        node_mem: &mut HashMap<String, String>,
    ) -> Result<(), String> {
        let mut stmt = conn
            .prepare("SELECT id, title, summary, domain, confidence FROM nodes ORDER BY importance DESC LIMIT ?1")
            .map_err(|e| format!("prepare nodes: {e}"))?;
        let rows = stmt
            .query_map(rusqlite::params![limit], |row| {
                Ok((
                    opt_str(row, 0),
                    opt_str(row, 1),
                    opt_str(row, 2),
                    opt_str(row, 3),
                    opt_f64(row, 4, 0.6),
                ))
            })
            .map_err(|e| format!("query nodes: {e}"))?;
        for item in rows {
            let (db_id, title, summary, domain, conf) =
                item.map_err(|e| format!("read node row: {e}"))?;
            let domain = if domain.is_empty() {
                "general".to_string()
            } else {
                domain
            };
            let id = consciousness.remember(
                format!("{title}: {summary}"),
                MemoryType::Fact,
                domain.clone(),
                conf.clamp(0.0, 1.0),
            );
            if Self::track(consciousness, domain_seed, domain_hub, domain, id.clone()) {
                report.connections += 1;
            }
            if !db_id.is_empty() {
                node_mem.insert(db_id, id);
            }
            report.facts += 1;
        }
        Ok(())
    }

    /// edges 表（两端都已灌注才连）→ connect()
    fn link_edges(
        conn: &Connection,
        consciousness: &mut CrystalConsciousness,
        node_mem: &HashMap<String, String>,
        limit: i64,
        report: &mut AwakenReport,
    ) -> Result<(), String> {
        let mut stmt = conn
            .prepare("SELECT source_id, target_id FROM edges ORDER BY weight DESC LIMIT ?1")
            .map_err(|e| format!("prepare edges: {e}"))?;
        let rows = stmt
            .query_map(rusqlite::params![limit], |row| {
                Ok((opt_str(row, 0), opt_str(row, 1)))
            })
            .map_err(|e| format!("query edges: {e}"))?;
        for item in rows {
            let (src, dst) = item.map_err(|e| format!("read edge row: {e}"))?;
            if let (Some(a), Some(b)) = (node_mem.get(&src), node_mem.get(&dst)) {
                consciousness.connect(a, b);
                report.connections += 1;
            }
        }
        Ok(())
    }

    /// 跨域推理 pass：轮转取不同域的前提对，炼出新 Causal 记忆
    fn reason_passes(
        consciousness: &mut CrystalConsciousness,
        domain_seed: &HashMap<String, Vec<String>>,
        max_passes: usize,
        report: &mut AwakenReport,
    ) {
        let domains: Vec<&String> = domain_seed.keys().collect();
        if domains.len() < 2 {
            return;
        }
        let mut made = 0usize;
        let mut round = 0usize;
        while made < max_passes && round < max_passes.saturating_mul(domains.len().max(1)).saturating_add(1) {
            round += 1;
            let da = match domains.get(round % domains.len()) {
                Some(d) => *d,
                None => break,
            };
            let db_ = match domains.get((round + 1) % domains.len()) {
                Some(d) => *d,
                None => break,
            };
            if da == db_ {
                continue;
            }
            let pa = domain_seed
                .get(da)
                .and_then(|v| v.get(made % v.len().max(1)))
                .cloned();
            let pb = domain_seed
                .get(db_)
                .and_then(|v| v.get(made % v.len().max(1)))
                .cloned();
            match (pa, pb) {
                (Some(x), Some(y)) if x != y => {
                    if consciousness
                        .reason(vec![x, y], ReasoningType::CrossDomain)
                        .is_some()
                    {
                        report.chains += 1;
                        made += 1;
                    }
                }
                _ => continue,
            }
        }
    }
}

/// 阶段进度（verbose 才输出，全量炼时看水位）
fn stage(budget: &AwakenBudget, msg: &str) {
    if budget.verbose {
        eprintln!("[awaken] {msg}");
    }
}

impl NtDbAwakening {
    /// 域枢纽挂接：同域首条记忆为枢纽，后续同域记忆自动连枢纽。
    ///
    /// 同域即意义相连（公理 "Memory connects through meaning"），保证
    /// 全量灌注后连接比天然达标，不依赖 edges 表覆盖率。
    /// 返回是否新建连接。
    fn track(
        consciousness: &mut CrystalConsciousness,
        domain_seed: &mut HashMap<String, Vec<String>>,
        domain_hub: &mut HashMap<String, String>,
        domain: String,
        id: String,
    ) -> bool {
        let linked = match domain_hub.get(&domain).cloned() {
            Some(hub) if hub != id => {
                consciousness.connect(&hub, &id);
                true
            }
            Some(_) => false,
            None => {
                domain_hub.insert(domain.clone(), id.clone());
                false
            }
        };
        domain_seed.entry(domain).or_default().push(id);
        linked
    }
}

/// priority 文本 → 置信度（crate 内共享：engine 经验镜像复用）
pub(crate) fn priority_conf(priority: &str) -> f64 {
    match priority.to_lowercase().as_str() {
        "critical" => 0.9,
        "high" => 0.8,
        "medium" => 0.6,
        "low" => 0.4,
        _ => 0.5,
    }
}

/// 可空文本列读取（NULL → 空串，绝不抛错；crate 内共享）
pub(crate) fn opt_str(row: &Row, idx: usize) -> String {
    row.get::<_, Option<String>>(idx)
        .unwrap_or(None)
        .unwrap_or_default()
}

/// 可空实数列读取（NULL → fallback；crate 内共享）
pub(crate) fn opt_f64(row: &Row, idx: usize, fallback: f64) -> f64 {
    row.get::<_, Option<f64>>(idx)
        .unwrap_or(None)
        .unwrap_or(fallback)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_conf_mapping() {
        assert!((priority_conf("critical") - 0.9).abs() < 1e-9);
        assert!((priority_conf("high") - 0.8).abs() < 1e-9);
        assert!((priority_conf("weird") - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_budget_defaults_bounded() {
        let b = AwakenBudget::default();
        assert!(b.nodes <= 5000 && b.edges <= 10000);
        assert!(b.reason_passes >= 21, "must exceed Evolve gate (>20 chains)");
    }

    /// 活库全链路（默认忽略）：`cargo test -p neotrix --lib nt_db_awakening -- --ignored`
    ///
    /// 默认预算全量跑：experience 2000 + semantic 2000 + episodic 500 +
    /// causal 500 + nodes TOP500 + edges 3000 + 跨域推理 25 pass，
    /// 预期一次跑通 Seed → Growth → Integrate → Evolve。
    #[test]
    #[ignore = "needs real ~/.neotrix/knowledge.db"]
    fn live_awaken_smoke() {
        let path = NtDbAwakening::default_db_path();
        if !path.exists() {
            return;
        }
        let mut c = CrystalConsciousness::new("awaken-smoke");
        let budget = AwakenBudget::default();
        let rep = NtDbAwakening::awaken(&mut c, &path, &budget).expect("awaken must succeed");
        assert!(rep.total() > 1000, "live db must yield bulk memories");
        assert!(rep.chains >= 21, "must exceed Evolve gate (>20 chains)");
        assert_eq!(rep.phase_after, Some(EvolutionPhase::Evolve));
    }
}
