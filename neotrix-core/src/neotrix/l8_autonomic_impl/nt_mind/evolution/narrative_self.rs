//! 叙事自我 — 连贯人生故事的核心表示（P2 叙事自我）。
//!
//! 核心能力：
//! - 维护连贯的生命线索引（AutobiographicalIndex + 章节序列）
//! - 关键转折点标记与主题抽取
//! - 自我叙事摘要生成：起源、关键转折、核心价值观、当前主题
//! - 自然语言查询：`query("最遗憾的决定")` → 返回带因果链的记忆
//! - 与 SleepEngine/NarrativeIntegrator 协同：离线期自动更新叙事模型

use crate::core::nt_core_kb_primitives::{now, schema_initialize, open_raw_conn};
use crate::neotrix::l8_autonomic_impl::nt_mind::evolution::autobiographical_index::{AutobiographicalIndex, AutobiographicalEntry, NarrativeChapter, NarrativeArcType, IndexConfig, QueryResult};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{Arc, RwLock};

/// NarrativeSelf namespace — KB kv_store 命名空间。
pub const NS_NARRATIVE_SELF: &str = "narrative_self";

/// 叙事自我配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeConfig {
    /// 自动更新间隔（秒）
    pub auto_update_interval_secs: u64,
    /// 最大保留章节数
    pub max_chapters: usize,
    /// 摘要最大长度
    pub max_summary_length: usize,
    /// 关键转折点最小重要性
    pub min_turning_point_importance: f64,
}

impl Default for NarrativeConfig {
    fn default() -> Self {
        Self {
            auto_update_interval_secs: 3600, // 1 小时
            max_chapters: 1000,
            max_summary_length: 2000,
            min_turning_point_importance: 0.7,
        }
    }
}

/// 叙事自我运行时 — 统一管理自传体索引 + 章节序列 + 派生模型。
#[derive(Clone)]
pub struct NarrativeSelf {
    index: Arc<AutobiographicalIndex>,
    #[allow(dead_code)]
    config: NarrativeConfig,
    #[allow(dead_code)]
    last_update: Arc<RwLock<i64>>,
    derived_models: Arc<RwLock<DerivedModels>>,
}

/// 派生模型 — 从原始记忆派生的高层表征。
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
struct DerivedModels {
    /// 关键转折点（时间戳 -> 描述）
    turning_points: BTreeMap<i64, TurningPoint>,
    /// 核心主题随时间演化
    theme_evolution: Vec<ThemeEvolutionSnapshot>,
    /// 核心价值观轨迹
    value_trajectory: Vec<ValueTrajectorySnapshot>,
    /// 当前主动主题
    active_themes: HashSet<String>,
    /// 最后更新时间
    last_updated: i64,
}

/// 关键转折点。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurningPoint {
    pub timestamp: i64,
    pub title: String,
    pub description: String,
    pub arc_type: NarrativeArcType,
    pub importance: f64,
    pub value_shift: HashMap<String, f64>, // 价值观权重变化
    pub related_entry_ids: Vec<String>,
}

/// 主题演化快照。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeEvolutionSnapshot {
    pub timestamp: i64,
    pub dominant_themes: Vec<(String, f64)>, // (主题, 强度)
    pub theme_relations: HashMap<String, Vec<String>>, // 主题共现关系
}

/// 价值观轨迹快照。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueTrajectorySnapshot {
    pub timestamp: i64,
    pub values: HashMap<String, f64>, // value_id -> weight
    pub dominant_value: String,
}

/// 叙事自我统计信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeStats {
    pub total_chapters: usize,
    pub total_turning_points: usize,
    pub dominant_themes: Vec<(String, f64)>,
    pub core_values: Vec<(String, f64)>,
    pub narrative_coherence: f64,
    pub last_updated: i64,
}

impl NarrativeSelf {
    /// 从 KB 初始化叙事自我。
    pub fn from_kb() -> Result<Self, String> {
        // 生产: 真实 KB; 打开/建表/读入任一环节失败 (含并发 busy) → 内存库兜底,
        // 意识体启动不被存储层阻塞, 稍后由后台循环重新挂载持久层
        let conn = match std::env::var("HOME").ok()
            .map(|h| rusqlite::Connection::open(std::path::PathBuf::from(h).join(".neotrix").join("knowledge.db")))
            .transpose()
        {
            Ok(Some(c)) => match schema_initialize(&c) {
                Ok(()) => c,
                Err(_) => rusqlite::Connection::open_in_memory().expect("mem conn"),
            },
            _ => rusqlite::Connection::open_in_memory().expect("mem conn"),
        };

        let index = Arc::new(AutobiographicalIndex::new(IndexConfig::default()));
        let _ = index.load_from_kb(&conn);
        let _ = index.load_chapters(&conn);

        let self_ = Self {
            index,
            config: NarrativeConfig::default(),
            last_update: Arc::new(RwLock::new(now())),
            derived_models: Arc::new(RwLock::new(DerivedModels::default())),
        };

        // 重建派生模型 (纯内存; 失败则派生模型为空壳但不阻塞)
        let _ = self_.rebuild_derived_models();
        Ok(self_)
    }

    /// 重建派生模型（启动时/重大更新后）。
    fn rebuild_derived_models(&self) -> Result<(), String> {
        let chapters = self.index.chapter_index.read().unwrap().values().cloned().collect::<Vec<_>>();

        // 1. 提取关键转折点：章节弧类型变化 + 高重要性条目
        let mut turning_points = BTreeMap::new();
        for ch in &chapters {
            if ch.arc_type != NarrativeArcType::Ongoing && ch.arc_type != NarrativeArcType::Resolution {
                let tp = TurningPoint {
                    timestamp: ch.time_range.0,
                    title: ch.title.clone(),
                    description: ch.summary.clone(),
                    arc_type: ch.arc_type.clone(),
                    importance: ch.entry_ids.len() as f64 / 10.0, // 简化
                    value_shift: HashMap::new(),
                    related_entry_ids: ch.entry_ids.clone(),
                };
                turning_points.insert(ch.time_range.0, tp);
            }
        }

        // 高重要性条目作为潜在转折点
        let mut entries = self.index.all_entries();
        entries.sort_by(|a,b| b.importance.partial_cmp(&a.importance).unwrap());
        for entry in entries.iter().take(20) {
            if entry.importance >= 0.8 {
                let tp = TurningPoint {
                    timestamp: entry.timestamp,
                    title: entry.title.clone(),
                    description: entry.summary.clone(),
                    arc_type: NarrativeArcType::Challenge,
                    importance: entry.importance,
                    value_shift: entry.value_ids.iter().map(|v| (v.clone(), 1.0)).collect(),
                    related_entry_ids: vec![entry.id.clone()],
                };
                turning_points.insert(entry.timestamp, tp);
            }
        }

        // 2. 主题演化：按时间窗口聚合主题
        let theme_evolution = self.build_theme_evolution(&entries)?;

        // 3. 价值观轨迹：从 ValueCompass 历史重建（简化：从条目 value_ids 推断）
        let value_trajectory = self.build_value_trajectory(&entries)?;

        // 4. 活跃主题
        let mut active_themes = HashSet::new();
        for e in entries.iter().rev().take(50) {
            for v in &e.value_ids { active_themes.insert(v.clone()); }
        }

        let models = DerivedModels {
            turning_points,
            theme_evolution,
            value_trajectory,
            active_themes,
            last_updated: now(),
        };

        *self.derived_models.write().unwrap() = models;
        Ok(())
    }

    fn build_theme_evolution(&self, entries: &[AutobiographicalEntry]) -> Result<Vec<ThemeEvolutionSnapshot>, String> {
        if entries.is_empty() { return Ok(Vec::new()); }

        // 按周分桶
        let mut buckets: HashMap<i64, HashMap<String, usize>> = HashMap::new();
        for e in entries {
            let week = e.timestamp / (7 * 86400);
            let bucket = buckets.entry(week).or_default();
            for v in &e.value_ids { *bucket.entry(v.clone()).or_insert(0) += 1; }
            for g in &e.goal_ids { *bucket.entry(format!("goal:{}", g)).or_insert(0) += 1; }
            // 从内容提取关键词
            for w in e.content.split(|c: char| !c.is_alphanumeric()) {
                if w.len() > 4 {
                    *buckets.entry(week).or_default().entry(w.to_lowercase()).or_insert(0) += 1;
                }
            }
        }

        let mut snapshots = Vec::new();
        for (week, counts) in buckets {
            let mut sorted: Vec<_> = counts.into_iter().collect();
            sorted.sort_by(|a,b| b.1.cmp(&a.1));
            let dominant: Vec<_> = sorted.into_iter().take(5).collect();
            snapshots.push(ThemeEvolutionSnapshot {
                timestamp: week * 7 * 86400,
                dominant_themes: dominant.iter().map(|(k,v)| (k.clone(), *v as f64)).collect(),
                theme_relations: HashMap::new(),
            });
        }
        snapshots.sort_by_key(|s| s.timestamp);
        Ok(snapshots)
    }

    fn build_value_trajectory(&self, entries: &[AutobiographicalEntry]) -> Result<Vec<ValueTrajectorySnapshot>, String> {
        // 简化：按月聚合价值观频次
        let mut buckets: HashMap<i64, HashMap<String, usize>> = HashMap::new();
        for e in entries {
            let month = e.timestamp / (30 * 86400);
            let bucket = buckets.entry(month).or_default();
            for v in &e.value_ids {
                *bucket.entry(v.clone()).or_insert(0) += 1;
            }
        }

        let mut snapshots = Vec::new();
        for (month, counts) in buckets {
            let mut sorted: Vec<_> = counts.into_iter().collect();
            sorted.sort_by(|a,b| b.1.cmp(&a.1));
            let values: HashMap<_, _> = sorted.into_iter().map(|(k,v)| (k, v as f64)).collect();
            let dominant = values.keys().next().cloned().unwrap_or_default();
            snapshots.push(ValueTrajectorySnapshot {
                timestamp: month * 30 * 86400,
                values,
                dominant_value: dominant,
            });
        }
        snapshots.sort_by_key(|s| s.timestamp);
        Ok(snapshots)
    }

    /// 自然语言查询 → 返回匹配的记忆 + 因果链 + 叙事上下文。
    pub fn query(&self, query: &str, limit: usize) -> Vec<QueryResult> {
        let results = self.index.query(query, limit);
        // 增加叙事上下文：该记忆所在的章节
        let chapters = self.index.chapter_index.read().unwrap();
        let chapter_map: HashMap<_, _> = chapters.values()
            .flat_map(|ch| ch.entry_ids.iter().map(move |id| (id.clone(), ch.id.clone())))
            .collect();

        results.into_iter().map(|mut r| {
            if let Some(ch_id) = chapter_map.get(&r.entry.id) {
                r.entry.metadata.insert("chapter_id".into(), ch_id.clone());
            }
            r
        }).collect()
    }

    /// 生成自传体摘要（用于自我认知/对外表达）。
    pub fn autobiographical_summary(&self) -> String {
        let models = self.derived_models.read().unwrap();
        let stats = self.index.stats();

        let mut parts = Vec::new();
        parts.push(format!("记忆总览：{} 条关键记忆，{} 个叙事章节。", stats.total_entries, stats.total_chapters));

        // 起源
        if let Some((_, first)) = models.turning_points.iter().next() {
            parts.push(format!("起源：{} — {}", first.title, first.description));
        }

        // 关键转折点（最多 3 个）
        let turning: Vec<_> = models.turning_points.values().take(3).collect();
        if !turning.is_empty() {
            let desc = turning.iter().map(|tp| format!("{} ({:?})", tp.title, tp.arc_type)).collect::<Vec<_>>().join(" → ");
            parts.push(format!("关键转折：{}", desc));
        }

        // 核心价值观
        let core_values: String = models.value_trajectory.last()
            .map(|v| {
                let mut vs: Vec<_> = v.values.iter().filter(|(_, w)| **w > 2.0).map(|(k, _)| k.clone()).collect();
                vs.sort();
                vs.join("、")
            })
            .unwrap_or_default();
        if !core_values.is_empty() {
            parts.push(format!("核心驱动：{}", core_values));
        }

        // 当前主题
        if !models.active_themes.is_empty() {
            let themes: Vec<_> = models.active_themes.iter().take(5).cloned().collect();
            parts.push(format!("当前关注：{}", themes.join("、")));
        }

        // 叙事连贯性
        parts.push(format!("叙事连贯性：{:.1}%", self.narrative_coherence() * 100.0));

        parts.join("\n")
    }

    /// 叙事连贯性评分 [0,1]。
    pub fn narrative_coherence(&self) -> f64 {
        let chapters = self.index.chapter_index.read().unwrap().values().cloned().collect::<Vec<_>>();
        if chapters.is_empty() { return 0.0; }
        let coherent = chapters.iter().filter(|c| c.arc_type != NarrativeArcType::Ongoing).count();
        coherent as f64 / chapters.len() as f64
    }

    /// 记录新体验（行动执行后调用）。
    pub fn record_experience(&self, action: &str, outcome: &str, valence: f64, importance: f64, value_ids: Vec<String>, goal_ids: Vec<String>, session_id: &str) -> Result<(), String> {
        let entry = AutobiographicalEntry {
            id: format!("exp_{}", now()),
            timestamp: now(),
            title: action.chars().take(20).collect::<String>(),
            summary: outcome.chars().take(100).collect::<String>(),
            content: format!("行动：{}\n结果：{}", action, outcome),
            causal_chain: vec![],
            valence,
            importance,
            value_ids,
            goal_ids,
            metadata: BTreeMap::from([("session_id".into(), session_id.into())]),
        };

        if let Some(conn) = open_raw_conn() {
            let _ = schema_initialize(&conn);
            let _ = self.index.add_entry(&conn, entry.clone());
        }
        let _ = entry;

        // 异步更新派生模型（简化：同步重建）
        self.rebuild_derived_models().map_err(|e| e.to_string())
    }

    /// 获取统计信息。
    pub fn stats(&self) -> NarrativeStats {
        let models = self.derived_models.read().unwrap();
        let stats = self.index.stats();
        let mut core_values: Vec<_> = models.value_trajectory.last()
            .map(|v| v.values.iter().map(|(k,w)| (k.clone(), *w)).collect::<Vec<_>>())
            .unwrap_or_default();
        core_values.sort_by(|a,b| b.1.partial_cmp(&a.1).unwrap());
        let dominant_themes: Vec<_> = models.theme_evolution.last()
            .map(|s| s.dominant_themes.clone())
            .unwrap_or_default();
        let core_vals: Vec<_> = core_values.into_iter().take(5).collect();

        NarrativeStats {
            total_chapters: stats.total_chapters,
            total_turning_points: models.turning_points.len(),
            dominant_themes,
            core_values: core_vals,
            narrative_coherence: self.narrative_coherence(),
            last_updated: models.last_updated,
        }
    }

    /// 获取指定时间范围的叙事片段。
    pub fn get_narrative_slice(&self, start: i64, end: i64) -> Vec<NarrativeChapter> {
        let chapters = self.index.chapter_index.read().unwrap();
        chapters.values()
            .filter(|c| c.time_range.0 >= start && c.time_range.1 <= end)
            .cloned()
            .collect()
    }

    /// 强制更新派生模型（外部触发）。
    pub fn force_update(&self) -> Result<(), String> {
        self.rebuild_derived_models()
    }
}

/// 便捷函数：从 KB 初始化 NarrativeSelf。
pub fn initialize_narrative_self() -> Result<NarrativeSelf, String> {
    NarrativeSelf::from_kb()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use crate::core::nt_core_kb_primitives::schema_initialize;
    
    fn mem_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        schema_initialize(&conn).unwrap();
        conn
    }

    #[test]
    fn test_narrative_self_creation() {
        let ns = NarrativeSelf::from_kb();
        assert!(ns.is_ok());
    }

    #[test]
    fn test_autobiographical_summary() {
        let ns = NarrativeSelf::from_kb().unwrap();
        let summary = ns.autobiographical_summary();
        println!("Summary: {}", summary);
        assert!(!summary.is_empty());
    }

    #[test]
    fn test_query_with_narrative_context() {
        let ns = NarrativeSelf::from_kb().unwrap();
        let results = ns.query("学习", 5);
        for r in results {
            println!("Query result: {} - {:?}", r.entry.title, r.entry.metadata.get("chapter_id"));
        }
    }

    #[test]
    fn test_record_experience() {
        let ns = NarrativeSelf::from_kb().unwrap();
        ns.record_experience(
            "学习新概念",
            "理解了所有权机制",
            0.8, 0.9,
            vec!["growth".into(), "truth_seeking".into()],
            vec!["learn_rust".into()],
            "test_session"
        ).unwrap();
    }
}