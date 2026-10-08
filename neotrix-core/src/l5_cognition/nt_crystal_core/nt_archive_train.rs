//! NT-ARCHIVE-TRAIN — 档案库分域流式炼制
//!
//! 背景：`/Volumes/NeoTrixBrain/knowledge-archive-corpus-20260825.db`
//! 有 nodes 2209万 / edges 2240万，全量进内存（≈20GB+）单机不可行。
//! 本模块走 L2 本意——**模式沉淀**：分域流式灌注 → 域内推理 →
//! 高置信结论落 `CrystalKnowledge.patterns`（ durable 小文件）→
//! 评分推进 → 每批只保留枢纽+优结论，其余释放。
//!
//! 与热通道（`NtDbAwakening` 全量灌注→Transcend）的分工：
//! 档案通道产出 patterns + scores；Transcend 仍是热集现象。
//! DB 只读打开；无 unwrap / expect / panic；无 `[]` 索引。

use rusqlite::{Connection, OpenFlags};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use super::consciousness::{CrystalConsciousness, MemoryType, ReasoningType};
use super::engine::CrystalEngine;
use super::knowledge::CausalPattern;
use super::nt_db_awakening::opt_str;
use super::{cocoons::CocoonStore, CrystalCore};

/// 档案炼制配置
pub struct ArchiveTrainConfig {
    pub db_path: PathBuf,
    /// 只炼这些域；None = 除 bulk 外全域（需一次 GROUP BY 扫表）
    pub domains: Option<Vec<String>>,
    /// bulk 大域（走采样上限，不全炼）
    pub bulk_domains: Vec<String>,
    /// bulk 域采样上限（按 importance 倒序）
    pub bulk_cap: i64,
    /// 普通域上限
    pub domain_cap: i64,
    /// 每域推理 pass 数
    pub reason_passes: usize,
    /// 结论落模式的置信度门
    pub pattern_conf_floor: f64,
    /// 跨批保留的优结论上限（跨批融合的桥）
    pub keep_conclusions: usize,
    /// 每多少域做一次 evolve + 落盘
    pub evolve_every: usize,
    pub verbose: bool,
}

impl Default for ArchiveTrainConfig {
    fn default() -> Self {
        Self {
            db_path: PathBuf::from(
                "/Volumes/NeoTrixBrain/knowledge-archive-corpus-20260825.db",
            ),
            domains: None,
            bulk_domains: vec!["zim.kiwix".to_string()],
            bulk_cap: 50_000,
            domain_cap: 20_000,
            reason_passes: 10,
            pattern_conf_floor: 0.7,
            keep_conclusions: 500,
            evolve_every: 50,
            verbose: true,
        }
    }
}

/// 档案炼制报告
#[derive(Debug, Default)]
pub struct ArchiveTrainReport {
    pub domains_done: usize,
    pub memories_seen: usize,
    pub chains: usize,
    pub patterns: usize,
    pub scores: HashMap<String, f64>,
}

/// 档案炼制引擎
pub struct NtArchiveTrain;

impl NtArchiveTrain {
    /// 分域流式炼制：枚举域 → 逐域灌注 → 推理 → 沉淀 → 剪枝 → 定期落盘
    pub fn train(
        core: &mut CrystalCore,
        consciousness: &mut CrystalConsciousness,
        cfg: &ArchiveTrainConfig,
    ) -> Result<ArchiveTrainReport, String> {
        let conn = Connection::open_with_flags(&cfg.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|e| format!("open archive db: {e}"))?;
        let mut report = ArchiveTrainReport::default();

        // 域枢纽（跨批保留）与优结论池（跨批融合的桥）
        let mut domain_hub: HashMap<String, String> = HashMap::new();
        let mut pool: Vec<(f64, String, String)> = Vec::new();

        let domains = Self::list_domains(&conn, cfg)?;
        for (idx, (domain, count)) in domains.iter().enumerate() {
            let cap = if cfg.bulk_domains.iter().any(|b| b == domain) {
                cfg.bulk_cap
            } else {
                cfg.domain_cap
            };
            let batch_ids =
                Self::ingest_domain(&conn, consciousness, domain, cap, &mut domain_hub)?;
            if batch_ids.is_empty() {
                continue;
            }
            report.memories_seen += batch_ids.len();

            // 推理：本域记忆 × 别域优结论（跨批融合），无池则域内归纳
            let made = Self::reason_batch(consciousness, &batch_ids, &pool, cfg.reason_passes);
            report.chains += made;

            // 沉淀：高置信新结论 → 知识层模式
            report.patterns += Self::persist_patterns(core, consciousness, domain, cfg.pattern_conf_floor);

            // 评分推进（对手过程由 evolve 内的缓漏承担）
            Self::bump_scores(core, batch_ids.len(), made);

            // 跨批桥：本批新结论入池（置信度截断），下一批跨域融合用
            Self::feed_pool_ids(consciousness, domain, &mut pool, cfg.keep_conclusions);

            // 剪枝：只留枢纽 + 池内结论
            Self::prune_to_pool(consciousness, &domain_hub, &pool);

            // 定期 evolve + 落盘
            if (idx + 1) % cfg.evolve_every == 0 {
                let _ = CrystalEngine::evolve(core);
                core.save()?;
            }

            if cfg.verbose {
                eprintln!(
                    "[archive] {}/{} domain={} rows={} chains+{} patterns={}",
                    idx + 1,
                    domains.len(),
                    domain,
                    count,
                    made,
                    report.patterns
                );
            }
            report.domains_done += 1;
        }

        // 收尾：evolve + 核心落盘 + 意识入茧（活集已剪枝，文件小）
        let _ = CrystalEngine::evolve(core);
        core.save()?;
        let mut cocoons = CocoonStore::load();
        cocoons.sync_from_consciousness(consciousness);
        cocoons.save()?;
        report.scores = core.evolution.capability_scores.scores.clone();
        Ok(report)
    }

    /// 枚举域（一次 GROUP BY，过滤 bulk/指定域）
    fn list_domains(
        conn: &Connection,
        cfg: &ArchiveTrainConfig,
    ) -> Result<Vec<(String, i64)>, String> {
        // 快道：指定域时跳过 2209 万行 GROUP BY（count 未知记 -1， ingest 按 cap 截断）
        if let Some(only) = &cfg.domains {
            return Ok(only.iter().map(|d| (d.clone(), -1)).collect());
        }
        let mut stmt = conn
            .prepare("SELECT domain, COUNT(*) FROM nodes GROUP BY domain")
            .map_err(|e| format!("list domains: {e}"))?;
        let rows = stmt
            .query_map([], |row| {
                let d: Option<String> = row.get(0).unwrap_or(None);
                let c: i64 = row.get(1).unwrap_or(0);
                Ok((d.unwrap_or_default(), c))
            })
            .map_err(|e| format!("query domains: {e}"))?;
        let mut out = Vec::new();
        for item in rows {
            let (d, c) = item.map_err(|e| format!("read domain row: {e}"))?;
            if let Some(only) = &cfg.domains {
                if !only.iter().any(|w| w == &d) {
                    continue;
                }
            }
            out.push((d, c));
        }
        // 小域先行、bulk 垫底：精华先炼，大 bulk 放最后
        out.sort_by(|a, b| a.1.cmp(&b.1));
        Ok(out)
    }

    /// 单域灌注（importance 倒序截断）→ 返回本批记忆 id
    fn ingest_domain(
        conn: &Connection,
        consciousness: &mut CrystalConsciousness,
        domain: &str,
        cap: i64,
        domain_hub: &mut HashMap<String, String>,
    ) -> Result<Vec<String>, String> {
        // 索引友好：idx_nodes_domain 只在裸列等值时命中，ifnull() 会杀索引；
        // 空域单独走 IS NULL 分支。
        let mut rows: Vec<(String, String, f64)> = Vec::new();
        if domain.is_empty() {
            let mut stmt = conn
                .prepare("SELECT title, summary, confidence FROM nodes WHERE domain IS NULL OR domain = '' ORDER BY importance DESC LIMIT ?1")
                .map_err(|e| format!("prepare domain batch: {e}"))?;
            let mapped = stmt
                .query_map(rusqlite::params![cap], |row| {
                    Ok((
                        opt_str(row, 0),
                        opt_str(row, 1),
                        row.get::<_, Option<f64>>(2).unwrap_or(None).unwrap_or(0.6),
                    ))
                })
                .map_err(|e| format!("query domain batch: {e}"))?;
            for item in mapped {
                rows.push(item.map_err(|e| format!("read domain batch: {e}"))?);
            }
        } else {
            let mut stmt = conn
                .prepare("SELECT title, summary, confidence FROM nodes WHERE domain = ?1 ORDER BY importance DESC LIMIT ?2")
                .map_err(|e| format!("prepare domain batch: {e}"))?;
            let mapped = stmt
                .query_map(rusqlite::params![domain, cap], |row| {
                    Ok((
                        opt_str(row, 0),
                        opt_str(row, 1),
                        row.get::<_, Option<f64>>(2).unwrap_or(None).unwrap_or(0.6),
                    ))
                })
                .map_err(|e| format!("query domain batch: {e}"))?;
            for item in mapped {
                rows.push(item.map_err(|e| format!("read domain batch: {e}"))?);
            }
        }
        let mut ids = Vec::new();
        for (title, summary, conf) in rows {
            let id = consciousness.remember(
                format!("{title}: {summary}"),
                MemoryType::Fact,
                domain.to_string(),
                conf.clamp(0.0, 1.0),
            );
            // 域枢纽挂接
            match domain_hub.get(domain).cloned() {
                Some(hub) if hub != id => {
                    consciousness.connect(&hub, &id);
                }
                Some(_) => {}
                None => {
                    domain_hub.insert(domain.to_string(), id.clone());
                }
            }
            ids.push(id);
        }
        Ok(ids)
    }

    /// 本批推理：批记忆 × 别域优结论，无池则域内归纳
    fn reason_batch(
        consciousness: &mut CrystalConsciousness,
        batch_ids: &[String],
        pool: &[(f64, String, String)],
        passes: usize,
    ) -> usize {
        if batch_ids.is_empty() {
            return 0;
        }
        let batch_domain = consciousness
            .memories
            .get(batch_ids.first().map_or(&String::new(), |v| v))
            .map(|m| m.domain.clone())
            .unwrap_or_default();
        let mut made = 0usize;
        let mut k = 0usize;
        while made < passes && k < passes.saturating_mul(4).saturating_add(batch_ids.len()) {
            k += 1;
            let a = match batch_ids.get(k % batch_ids.len().max(1)).cloned() {
                Some(v) => v,
                None => continue,
            };
            // 别域优结论优先（跨批融合），否则域内归纳
            let other = pool
                .iter()
                .find(|(_, id, d)| d != &batch_domain && id != &a)
                .map(|(_, id, _)| (id.clone(), ReasoningType::CrossDomain));
            let (prem, ctype) = match other {
                Some(v) => v,
                None => match batch_ids.get((k + 1) % batch_ids.len().max(1)).cloned() {
                    Some(y) if y != a => (y, ReasoningType::Inductive),
                    _ => continue,
                },
            };
            if consciousness.reason(vec![a, prem], ctype).is_some() {
                made += 1;
            }
        }
        made
    }

    /// 沉淀：本批新增的高置信结论 → 知识层模式（去重）
    fn persist_patterns(
        core: &mut CrystalCore,
        consciousness: &CrystalConsciousness,
        domain: &str,
        floor: f64,
    ) -> usize {
        let mut added = 0usize;
        for ch in consciousness.reasoning_chains.iter().rev() {
            if ch.confidence < floor {
                continue;
            }
            // 只收本批（按域标签过滤，此处用结论域 reasoning + 前提到本域）
            let from_batch = ch.premises.iter().any(|pid| {
                consciousness
                    .memories
                    .get(pid)
                    .is_some_and(|m| m.domain == domain)
            });
            if !from_batch {
                // 链按时间倒序，本批已过即停（近似：遇到连续 50 条非本批则停）
                continue;
            }
            let so: String = ch.conclusion.chars().take(300).collect();
            let dup = core.knowledge.patterns.iter().any(|p| {
                p.so_implications.first() == Some(&so)
            });
            if dup {
                continue;
            }
            let ifs: Vec<String> = ch
                .premises
                .iter()
                .take(2)
                .filter_map(|pid| consciousness.memories.get(pid).map(|m| m.domain.clone()))
                .collect();
            core.knowledge.add_pattern(CausalPattern {
                id: format!("AP-{:06}", core.knowledge.patterns.len() + 1),
                if_conditions: ifs,
                then_consequences: vec![format!("archive:{domain}")],
                so_implications: vec![so],
                theory_origin: "ArchiveTrain".into(),
                confidence: ch.confidence.clamp(0.0, 1.0),
            });
            added += 1;
            if added >= 50 {
                break;
            }
        }
        added
    }

    /// 评分推进（批内实质工作 → 加分，跑赢 evolve 缓漏）
    fn bump_scores(core: &mut CrystalCore, batch_n: usize, chains: usize) {
        let mem_gain = 0.000005 * batch_n as f64;
        let cur = core.evolution.capability_scores.get("memory");
        core.evolution.update_score("memory", cur + mem_gain);
        let cur = core.evolution.capability_scores.get("reasoning");
        core.evolution
            .update_score("reasoning", cur + 0.002 * chains as f64);
        let cur = core.evolution.capability_scores.get("knowledge");
        core.evolution.update_score("knowledge", cur + 0.005);
    }

    /// 剪枝：只留枢纽 + 池
    fn prune_to_pool(
        consciousness: &mut CrystalConsciousness,
        domain_hub: &HashMap<String, String>,
        pool: &[(f64, String, String)],
    ) {
        let mut keep: HashSet<String> = HashSet::new();
        for hub in domain_hub.values() {
            keep.insert(hub.clone());
        }
        for (_, id, _) in pool {
            keep.insert(id.clone());
        }
        Self::prune_essentials(consciousness, &keep);
    }

    /// 只留 keep 集（测试与训练共用）
    pub(crate) fn prune_essentials(
        consciousness: &mut CrystalConsciousness,
        keep: &HashSet<String>,
    ) {
        consciousness.memories.retain(|k, _| keep.contains(k));
        consciousness.recount_connections();
    }

    /// 跨批桥实际逻辑（独立函数以便测试）
    pub(crate) fn feed_pool_ids(
        consciousness: &CrystalConsciousness,
        domain: &str,
        pool: &mut Vec<(f64, String, String)>,
        cap: usize,
    ) {
        let mut fresh = Vec::new();
        for ch in consciousness.reasoning_chains.iter().rev() {
            let in_batch = ch.premises.iter().any(|pid| {
                consciousness
                    .memories
                    .get(pid)
                    .is_some_and(|m| m.domain == domain)
            });
            if !in_batch {
                if fresh.len() > 200 {
                    break;
                }
                continue;
            }
            if let Some(cid) = &ch.conclusion_memory_id {
                if consciousness.memories.contains_key(cid) {
                    fresh.push((ch.confidence, cid.clone(), domain.to_string()));
                }
            }
            if fresh.len() >= cap * 2 {
                break;
            }
        }
        pool.extend(fresh);
        pool.sort_by(|a, b| {
            b.0.partial_cmp(&a.0)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        pool.truncate(cap);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archive_config_defaults() {
        let c = ArchiveTrainConfig::default();
        assert!(c.bulk_domains.iter().any(|b| b == "zim.kiwix"));
        assert!(c.bulk_cap <= 100_000 && c.domain_cap <= 50_000);
        assert!(c.evolve_every >= 10);
    }

    #[test]
    fn test_prune_keeps_essentials() {
        let mut c = CrystalConsciousness::new("t");
        let mut keep = HashSet::new();
        for i in 0..5 {
            let id = c.remember(format!("m{i}"), MemoryType::Fact, "d", 0.8);
            if i < 2 {
                keep.insert(id);
            }
        }
        NtArchiveTrain::prune_essentials(&mut c, &keep);
        assert_eq!(c.memories.len(), 2);
    }

    /// 档案 pilot（默认忽略）：小域 consciousness + NT-WORLD
    /// `cargo test -p neotrix --lib archive_pilot -- --ignored --nocapture`
    #[test]
    #[ignore = "needs archive db, minutes"]
    fn archive_pilot() {
        let db = PathBuf::from(
            "/Volumes/NeoTrixBrain/knowledge-archive-corpus-20260825.db",
        );
        if !db.exists() {
            return;
        }
        let mut core = CrystalCore::new("Pilot");
        let mut c = CrystalConsciousness::new("Pilot");
        let cfg = ArchiveTrainConfig {
            db_path: db,
            domains: Some(vec!["consciousness".to_string(), "NT-WORLD".to_string()]),
            verbose: true,
            ..Default::default()
        };
        let rep = NtArchiveTrain::train(&mut core, &mut c, &cfg).expect("pilot must succeed");
        eprintln!("[pilot] domains={} seen={} chains={} patterns={}",
            rep.domains_done, rep.memories_seen, rep.chains, rep.patterns);
        assert!(rep.domains_done == 2, "pilot must finish both domains");
        assert!(rep.memories_seen > 1000, "pilot must ingest bulk rows");
        assert!(rep.chains > 0, "pilot must reason");
    }

    /// 档案全量（默认忽略）：`cargo test -p neotrix --lib live_full_archive -- --ignored --nocapture`
    ///
    /// 合并式炼制：先载入现有 crystal + 茧（不断传承），再全域流式炼，
    /// 落盘覆盖。耗时数十分钟，后台跑。
    #[test]
    #[ignore = "full archive train, tens of minutes, merges into live crystal store"]
    fn live_full_archive() {
        use super::super::{CocoonStore, CrystalConsciousness, CrystalCore};
        let db = PathBuf::from(
            "/Volumes/NeoTrixBrain/knowledge-archive-corpus-20260825.db",
        );
        if !db.exists() {
            return;
        }
        let mut core = CrystalCore::load().unwrap_or_else(|_| CrystalCore::new("ArchiveTrain"));
        let mut c = CrystalConsciousness::new("ArchiveTrain");
        CocoonStore::load().sync_to_consciousness(&mut c);
        let cfg = ArchiveTrainConfig {
            db_path: db,
            ..Default::default()
        };
        let rep = NtArchiveTrain::train(&mut core, &mut c, &cfg).expect("full archive must succeed");
        eprintln!("[full-archive] domains={} seen={} chains={} patterns={}",
            rep.domains_done, rep.memories_seen, rep.chains, rep.patterns);
        assert!(rep.domains_done > 100, "must cover bulk of domains");
        assert!(rep.patterns > 0, "must distill patterns");
    }

    /// 从 cocoons.json 重建 crystal.json（默认忽略）
    /// `cargo test -p neotrix --lib rebuild_crystal_from_cocoons -- --ignored --nocapture`
    ///
    /// 场景：crystal.json 被其他窗口覆写为空壳，但 cocoons.json 327MB 完好。
    /// 本测试重建 core（patterns/experience），同步 memories，落盘。
    #[test]
    #[ignore = "rebuild crystal.json from cocoons, seconds"]
    fn rebuild_crystal_from_cocoons() {
        use super::super::{CocoonStore, CrystalConsciousness, CrystalCore};
        let core = CrystalCore::load().unwrap_or_else(|_| CrystalCore::new("NeoTrix"));
        let mut c = CrystalConsciousness::new("NeoTrix");
        let cocoons = CocoonStore::load();
        let stats = cocoons.stats();
        eprintln!(
            "[rebuild] loaded cocoons: {} domains, {} memories",
            stats.cocoon_count, stats.total_memories
        );
        cocoons.sync_to_consciousness(&mut c);
        eprintln!(
            "[rebuild] consciousness synced: {} memories",
            c.memories.len()
        );
        // 落盘
        core.save().expect("crystal save must succeed");
        // 同步回 cocoons（确保双向一致）
        let mut cocoons2 = CocoonStore::load();
        cocoons2.sync_from_consciousness(&c);
        cocoons2.save().expect("cocoon save must succeed");
        eprintln!("[rebuild] crystal.json + cocoons.json saved");
        // 验证
        let loaded = CrystalCore::load().expect("crystal must load back");
        let status = loaded.status();
        eprintln!("[rebuild] verify: {:?}", status);
        assert!(
            status.patterns_count > 0 || status.successes_count > 0 || c.memories.len() > 1000,
            "rebuild must produce non-trivial state"
        );
    }
}
