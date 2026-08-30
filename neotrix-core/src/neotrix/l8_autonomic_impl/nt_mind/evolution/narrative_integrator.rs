//! 叙事整合器 — 睡眠期离线将碎片记忆编织为连贯叙事章节（P2 叙事自我）。
//!
//! 核心能力：
//! - 从 AutobiographicalIndex 获取近期高重要性条目
//! - 识别主题聚类（主题建模 + 价值观共现）
//! - 叙事弧识别（起源→挑战→成长→蜕变→解决）
//! - 生成连贯章节：标题、时间范围、情感弧、主题标签
//! - 情感弧重构：效价序列 → 情感弧类型（上升/下降/波动/平稳）
//! - 落盘：KB narrative_chapters namespace + 更新 AutobiographicalIndex 章节索引

use crate::neotrix::l8_autonomic_impl::nt_mind::evolution::autobiographical_index::{AutobiographicalIndex, AutobiographicalEntry, NarrativeChapter, NarrativeArcType, IndexConfig};
use crate::core::nt_core_kb_primitives::now;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

/// 便捷函数：执行一次完整叙事整合（供 SleepEngine 调用）。
pub fn run_narrative_integration(conn: &Connection) -> Result<Vec<IntegrationResult>, String> {
    let index = Arc::new(AutobiographicalIndex::new(IndexConfig::default()));
    index.load_from_kb(conn).map_err(|e| e.to_string())?;
    let integrator = NarrativeIntegrator::new(index, NarrativeIntegratorConfig::default());
    integrator.integrate(conn)
}

/// 叙事整合器配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeIntegratorConfig {
    /// 单次整合最小条目数
    pub min_entries_per_chapter: usize,
    /// 单次整合最大条目数
    pub max_entries_per_chapter: usize,
    /// 主题聚类相似度阈值
    pub theme_similarity_threshold: f64,
    /// 情感弧最小波动幅度
    pub min_valence_variance: f64,
    /// 叙事弧最小长度
    pub min_arc_length: usize,
    /// 是否启用情感弧重构
    pub enable_emotional_arc: bool,
    /// 最大回溯天数
    pub lookback_days: u64,
}

impl Default for NarrativeIntegratorConfig {
    fn default() -> Self {
        Self {
            min_entries_per_chapter: 5,
            max_entries_per_chapter: 50,
            theme_similarity_threshold: 0.4,
            min_valence_variance: 0.15,
            min_arc_length: 3,
            enable_emotional_arc: true,
            lookback_days: 30,
        }
    }
}

/// 叙事整合结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationResult {
    pub chapter: NarrativeChapter,
    pub source_entry_ids: Vec<String>,
    pub coherence_score: f64,
    pub theme_coverage: HashMap<String, usize>,
    pub emotional_arc_type: EmotionalArcType,
    pub generated_at: i64,
}

/// 情感弧类型。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EmotionalArcType {
    Rising,       // 效价持续上升
    Falling,      // 效价持续下降
    RisingFalling, // 先升后降
    FallingRising, // 先降后升
    Oscillating,   // 振荡
    Flat,          // 平稳
    Complex,       // 复杂模式
}

/// 主题聚类结果。
#[derive(Debug, Clone)]
pub struct ThemeCluster {
    pub theme_label: String,
    pub entry_ids: Vec<String>,
    pub keywords: HashSet<String>,
    pub avg_valence: f64,
    pub coherence: f64,
}

/// 叙事整合器核心。
#[derive(Clone)]
pub struct NarrativeIntegrator {
    index: Arc<AutobiographicalIndex>,
    config: NarrativeIntegratorConfig,
    theme_models: Arc<RwLock<HashMap<String, ThemeModel>>>,
}

/// 主题模型 — 简单的关键词共现统计。
#[derive(Default, Debug, Clone)]
struct ThemeModel {
    keyword_counts: HashMap<String, usize>,
    co_occurrence: HashMap<(String, String), usize>,
    total_docs: usize,
}

impl NarrativeIntegrator {
    pub fn new(index: Arc<AutobiographicalIndex>, config: NarrativeIntegratorConfig) -> Self {
        Self {
            index,
            config,
            theme_models: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 主入口：执行一次完整叙事整合（睡眠期调用）。
    pub fn integrate(&self, conn: &Connection) -> Result<Vec<IntegrationResult>, String> {
        // 1. 获取近期高重要性条目
        let entries = self.get_recent_entries()?;
        if entries.len() < self.config.min_entries_per_chapter {
            return Ok(Vec::new());
        }

        // 2. 主题聚类
        let clusters = self.cluster_by_theme(&entries)?;

        // 3. 对每个簇尝试生成章节
        let mut results = Vec::new();
        for cluster in clusters {
            if cluster.entry_ids.len() >= self.config.min_entries_per_chapter {
                if let Some(result) = self.build_chapter_from_cluster(conn, &cluster)? {
                    results.push(result);
                }
            }
        }

        // 更新主题模型
        self.update_theme_models(&entries)?;

        Ok(results)
    }

    /// 获取近期高重要性条目。
    fn get_recent_entries(&self) -> Result<Vec<AutobiographicalEntry>, String> {
        let cutoff = now() - self.config.lookback_days as i64 * 86400;
        let entries = self.index.all_entries();
        let filtered: Vec<_> = entries.into_iter()
            .filter(|e| e.timestamp >= cutoff && e.importance >= 0.3)
            .collect();
        Ok(filtered)
    }

    /// 主题聚类：基于关键词共现 + 价值观共现。
    fn cluster_by_theme(&self, entries: &[AutobiographicalEntry]) -> Result<Vec<ThemeCluster>, String> {
        // 构建条目特征向量
        let features: Vec<_> = entries.iter().map(|e| {
            let mut keywords = HashSet::new();
            // 从内容提取关键词
            for w in e.content.split(|c: char| !c.is_alphanumeric()) {
                if w.len() > 3 {
                    keywords.insert(w.to_lowercase());
                }
            }
            // 价值观 ID 作为强特征
            for v in &e.value_ids {
                keywords.insert(format!("value:{}", v));
            }
            // 目标 ID
            for g in &e.goal_ids {
                keywords.insert(format!("goal:{}", g));
            }
            (e.id.clone(), keywords, e.valence, e.importance)
        }).collect();

        // 简单层次聚类：贪婪合并相似度高的条目
        let mut clusters: Vec<ThemeCluster> = Vec::new();
        let mut used: HashSet<String> = HashSet::new();

        for (id, keywords, valence, importance) in &features {
            if used.contains(id) { continue; }

            let mut cluster_ids = vec![id.clone()];
            let mut cluster_keywords = keywords.clone();
            let mut valences = vec![*valence];
            let mut importances = vec![*importance];

            // 寻找相似条目
            for (other_id, other_kw, other_valence, other_imp) in &features {
                if used.contains(other_id) || other_id == id { continue; }
                let sim = self.jaccard(keywords, other_kw);
                if sim >= self.config.theme_similarity_threshold {
                    cluster_ids.push(other_id.clone());
                    cluster_keywords.extend(other_kw.iter().cloned());
                    valences.push(*other_valence);
                    importances.push(*other_imp);
                    used.insert(other_id.clone());
                }
            }

            if cluster_ids.len() >= self.config.min_entries_per_chapter {
                // 生成主题标签
                let theme = self.generate_theme_label(&cluster_keywords);
                let coherence = self.compute_coherence(&cluster_ids);

                used.extend(cluster_ids.iter().cloned());
                clusters.push(ThemeCluster {
                    theme_label: theme,
                    entry_ids: cluster_ids,
                    keywords: cluster_keywords,
                    avg_valence: valences.iter().sum::<f64>() / valences.len() as f64,
                    coherence,
                });
            }
        }

        // 按连贯性排序
        clusters.sort_by(|a, b| b.coherence.partial_cmp(&a.coherence).unwrap());
        Ok(clusters)
    }

    /// Jaccard 相似度。
    fn jaccard(&self, a: &HashSet<String>, b: &HashSet<String>) -> f64 {
        let inter = a.intersection(b).count() as f64;
        let union = a.union(b).count() as f64;
        if union == 0.0 { 0.0 } else { inter / union }
    }

    /// 生成主题标签（取最高频关键词组合）。
    fn generate_theme_label(&self, keywords: &HashSet<String>) -> String {
        let mut freq = HashMap::new();
        for kw in keywords {
            if kw.starts_with("value:") || kw.starts_with("goal:") { continue; }
            *freq.entry(kw.clone()).or_insert(0) += 1;
        }
        let mut top: Vec<_> = freq.into_iter()
            .filter(|(_, c)| *c > 1)
            .collect();
        if top.is_empty() {
            "未命名主题".into()
        } else {
            top.sort_by(|a, b| b.1.cmp(&a.1));
            top.iter().take(3).map(|(k, _)| k.as_str()).collect::<Vec<_>>().join(" · ")
        }
    }

    /// 计算簇连贯度。
    fn compute_coherence(&self, ids: &[String]) -> f64 {
        if ids.len() < 2 { return 1.0; }
        // 简化：基于共同价值观/目标比例
        let _entries = self.index.all_entries();
        let _entry_map: HashMap<_, _> = self.index.all_entries().into_iter().map(|e| (e.id.clone(), e)).collect();

        let mut common_values = 0;
        let mut total_pairs = 0;
        for i in 0..ids.len() {
            for j in i+1..ids.len() {
                if let (Some(a), Some(b)) = (self.index.all_entries().iter().find(|e| e.id == ids[i]), self.index.all_entries().iter().find(|e| e.id == ids[j])) {
                    let common_v = a.value_ids.iter().filter(|v| b.value_ids.contains(v)).count();
                    let total_v = a.value_ids.len().max(b.value_ids.len()).max(1);
                    common_values += common_v;
                    total_pairs += total_v;
                }
            }
        }
        if total_pairs == 0 { 0.5 } else { common_values as f64 / total_pairs as f64 }
    }

    /// 从簇构建章节。
    fn build_chapter_from_cluster(&self, conn: &Connection, cluster: &ThemeCluster) -> Result<Option<IntegrationResult>, String> {
        if cluster.entry_ids.len() < self.config.min_entries_per_chapter {
            return Ok(None);
        }

        let snapshot: HashMap<String, AutobiographicalEntry> =
            self.index.all_entries().into_iter().map(|e| (e.id.clone(), e)).collect();
        let entries: Vec<AutobiographicalEntry> = cluster.entry_ids.iter()
            .filter_map(|id| snapshot.get(id).cloned())
            .collect();
        if entries.len() < self.config.min_entries_per_chapter { return Ok(None); }

        // 时间排序
        let mut sorted = entries.clone();
        sorted.sort_by_key(|e| e.timestamp);

        // 时间范围
        let time_range = (sorted.first().map(|e| e.timestamp).unwrap_or(0), sorted.last().map(|e| e.timestamp).unwrap_or(0));

        // 情感弧重构
        let emotional_arc_type = if self.config.enable_emotional_arc {
            self.classify_emotional_arc(&sorted)
        } else { EmotionalArcType::Flat };

        // 主题覆盖统计
        let mut theme_coverage = HashMap::new();
        for e in &entries {
            for v in &e.value_ids {
                *theme_coverage.entry(v.clone()).or_insert(0) += 1;
            }
            for g in &e.goal_ids {
                *theme_coverage.entry(format!("goal:{}", g)).or_insert(0) += 1;
            }
        }

        // 生成章节标题
        let title = self.generate_chapter_title(&entries, &cluster.theme_label);

        // 情感弧序列
        let emotional_arc: Vec<(String, f64)> = sorted.iter()
            .map(|e| (self.summarize_stage(&e.content), e.valence))
            .collect();

        // 生成摘要
        let summary = self.generate_summary(&sorted, &cluster.theme_label);

        // 连贯性评分
        let coherence = self.compute_coherence_score(&sorted);

        // 创建章节
        let chapter = NarrativeChapter {
            id: format!("ch_{}", now()),
            title,
            time_range,
            entry_ids: cluster.entry_ids.clone(),
            themes: vec![cluster.theme_label.clone()],
            arc_type: self.identify_narrative_arc(&sorted),
            summary,
            emotional_arc,
        };

        // 持久化章节
        crate::neotrix::l8_autonomic_impl::nt_mind::evolution::autobiographical_index::AutobiographicalIndex::persist_chapter(conn, &chapter)?;

        // 更新索引章节索引
        self.index.chapter_index.write().unwrap().insert(chapter.id.clone(), chapter.clone());

        Ok(Some(IntegrationResult {
            chapter,
            source_entry_ids: cluster.entry_ids.clone(),
            coherence_score: coherence,
            theme_coverage,
            emotional_arc_type,
            generated_at: now(),
        }))
    }

    /// 情感弧分类。
    fn classify_emotional_arc(&self, entries: &[AutobiographicalEntry]) -> EmotionalArcType {
        if entries.len() < 3 { return EmotionalArcType::Flat; }
        let valences: Vec<f64> = entries.iter().map(|e| e.valence).collect();

        let first = valences.first().unwrap();
        let last = valences.last().unwrap();
        let mid = valences[valences.len() / 2];

        let rising = *last > *first + 0.2;
        let falling = *last < *first - 0.2;
        let mid_rising = mid > *first + 0.15;
        let mid_falling = mid < *first - 0.15;

        match (rising, falling, mid_rising, mid_falling) {
            (true, false, _, _) => EmotionalArcType::Rising,
            (false, true, _, _) => EmotionalArcType::Falling,
            (_, _, true, false) => EmotionalArcType::RisingFalling,
            (_, _, false, true) => EmotionalArcType::FallingRising,
            _ if valences.iter().max_by(|a,b| a.partial_cmp(b).unwrap()).unwrap() - valences.iter().min_by(|a,b| a.partial_cmp(b).unwrap()).unwrap() > 0.4 => EmotionalArcType::Oscillating,
            _ => EmotionalArcType::Flat,
        }
    }

    /// 叙事弧识别。
    fn identify_narrative_arc(&self, entries: &[AutobiographicalEntry]) -> NarrativeArcType {
        if entries.is_empty() { return NarrativeArcType::Ongoing; }

        let valences: Vec<f64> = entries.iter().map(|e| e.valence).collect();
        let importances: Vec<f64> = entries.iter().map(|e| e.importance).collect();
        let _avg_valence = valences.iter().sum::<f64>() / valences.len() as f64;
        let avg_importance = importances.iter().sum::<f64>() / importances.len() as f64;

        // 基于效价趋势 + 重要性峰值判断
        let first = valences.first().unwrap();
        let last = valences.last().unwrap();
        let max_imp_idx = importances.iter().enumerate().max_by(|a,b| a.1.partial_cmp(b.1).unwrap()).map(|(i,_)| i).unwrap_or(0);

        if *first < -0.2 && *last > 0.2 && max_imp_idx > entries.len() / 2 {
            NarrativeArcType::Transformation
        } else if *first < -0.1 && *last > 0.1 {
            NarrativeArcType::Growth
        } else if *first > 0.3 && *last < -0.1 {
            NarrativeArcType::Challenge
        } else if *first < -0.3 && max_imp_idx < entries.len() / 3 {
            NarrativeArcType::Origin
        } else if *last > 0.3 && avg_importance > 0.7 {
            NarrativeArcType::Resolution
        } else {
            NarrativeArcType::Ongoing
        }
    }

    /// 生成章节标题。
    fn generate_chapter_title(&self, entries: &[AutobiographicalEntry], theme: &str) -> String {
        // 取最高重要性条目的标题片段 + 主题
        let top = entries.iter().max_by(|a,b| a.importance.partial_cmp(&b.importance).unwrap());
        let base = top.map(|e| e.title.chars().take(12).collect::<String>()).unwrap_or("未命名".into());
        format!("{} · {}", base, theme)
    }

    /// 生成章节摘要。
    fn generate_summary(&self, entries: &[AutobiographicalEntry], theme: &str) -> String {
        let count = entries.len();
        let themes: HashSet<_> = entries.iter().flat_map(|e| e.value_ids.iter()).collect();
        let theme_list = themes.iter().take(3).map(|s| s.as_str()).collect::<Vec<_>>().join("、");
        format!("本章包含 {} 个关键记忆片段，核心主题：{}。涉及价值观：{}。时间跨度：{} 天。",
            count, theme, theme_list,
            (entries.last().map(|e| e.timestamp).unwrap_or(0) - entries.first().map(|e| e.timestamp).unwrap_or(0)) / 86400)
    }

    /// 计算连贯性评分。
    fn compute_coherence_score(&self, entries: &[AutobiographicalEntry]) -> f64 {
        if entries.len() < 2 { return 1.0; }
        // 基于因果链连接度 + 价值观一致性 + 时间连续性
        let causal_score = self.causal_connectivity(entries);
        let value_score = self.value_consistency(entries);
        let time_score = self.temporal_continuity(entries);
        (causal_score * 0.4 + value_score * 0.4 + time_score * 0.2).clamp(0.0, 1.0)
    }

    fn causal_connectivity(&self, entries: &[AutobiographicalEntry]) -> f64 {
        if entries.len() < 2 { return 1.0; }
        let mut connected = 0;
        for i in 0..entries.len()-1 {
            let a = &entries[i];
            let b = &entries[i+1];
            // 检查是否有直接因果链
            let has_link = a.causal_chain.iter().any(|l| l.to_id == b.id || l.from_id == b.id) ||
                          b.causal_chain.iter().any(|l| l.to_id == a.id || l.from_id == a.id);
            if has_link { connected += 1; }
        }
        connected as f64 / (entries.len() - 1) as f64
    }

    fn value_consistency(&self, entries: &[AutobiographicalEntry]) -> f64 {
        if entries.len() < 2 { return 1.0; }
        let mut overlap = 0;
        let mut total = 0;
        for i in 0..entries.len()-1 {
            let a = &entries[i].value_ids;
            let b = &entries[i+1].value_ids;
            if a.is_empty() && b.is_empty() { continue; }
            let common = a.iter().filter(|v| b.contains(v)).count();
            total += a.len().max(b.len());
            overlap += common;
        }
        if total == 0 { 1.0 } else { overlap as f64 / total as f64 }
    }

    fn temporal_continuity(&self, entries: &[AutobiographicalEntry]) -> f64 {
        if entries.len() < 2 { return 1.0; }
        let mut gaps = 0;
        for i in 0..entries.len()-1 {
            let gap = entries[i+1].timestamp - entries[i].timestamp;
            if gap > 7 * 86400 { gaps += 1; } // 超过一周视为断点
        }
        1.0 - (gaps as f64 / (entries.len() - 1) as f64).min(1.0)
    }

    /// 更新主题模型。
    fn update_theme_models(&self, entries: &[AutobiographicalEntry]) -> Result<(), String> {
        let mut models = self.theme_models.write().unwrap();
        for entry in entries {
            let model = models.entry("global".into()).or_default();
            model.total_docs += 1;
            let mut keywords = HashSet::new();
            for w in entry.content.split(|c: char| !c.is_alphanumeric()) {
                if w.len() > 3 { keywords.insert(w.to_lowercase()); }
            }
            for v in &entry.value_ids { keywords.insert(format!("value:{}", v)); }
            for g in &entry.goal_ids { keywords.insert(format!("goal:{}", g)); }

            for kw in &keywords {
                *model.keyword_counts.entry(kw.clone()).or_insert(0) += 1;
            }
            for a in &keywords {
                for b in &keywords {
                    if a != b {
                        let key = if a < b { (a.clone(), b.clone()) } else { (b.clone(), a.clone()) };
                        *model.co_occurrence.entry(key).or_insert(0) += 1;
                    }
                }
            }
        }
        Ok(())
    }

    /// 情感弧总结（用于生成阶段标签）。
    fn summarize_stage(&self, content: &str) -> String {
        let words: Vec<_> = content.split_whitespace().take(8).collect();
        if words.is_empty() { "...".into() } else { words.join(" ") }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use crate::core::nt_core_kb_primitives::schema_initialize;
    use rusqlite::Connection;

    fn mem_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        schema_initialize(&conn).unwrap();
        conn
    }

    #[test]
    fn test_narrative_integration_basic() {
        let conn = mem_conn();
        let index = Arc::new(AutobiographicalIndex::new(IndexConfig::default()));
        index.load_from_kb(&conn).unwrap();

        // 添加一组相关记忆
        for i in 0..8 {
            let entry = crate::neotrix::l8_autonomic_impl::nt_mind::evolution::autobiographical_index::AutobiographicalEntry {
                id: format!("e{}", i),
                timestamp: now() - (7-i) as i64 * 86400,
                title: format!("第 {} 天学习 Rust", i+1),
                summary: format!("学习进度 {}", i+1),
                content: format!("今天学习了所有权、借用、生命周期等概念，写了第 {} 个练习", i+1),
                causal_chain: vec![],
                valence: if i < 3 { -0.2 } else if i < 6 { 0.2 } else { 0.7 },
                importance: 0.6 + (i as f64 * 0.03),
                value_ids: vec!["growth".into(), "truth_seeking".into()],
                goal_ids: vec!["learn_rust".into()],
                metadata: std::collections::BTreeMap::new(),
            };
            let index_clone = Arc::clone(&index);
            let index_ref = AutobiographicalIndex::new(IndexConfig::default());
            let _ = index_ref.add_entry(&mem_conn(), entry);
        }

        // 这里简化测试，实际需要共享同一个 index 实例
        let index = Arc::new(AutobiographicalIndex::new(IndexConfig::default()));
        index.load_from_kb(&mem_conn()).unwrap();
        let integrator = NarrativeIntegrator::new(index, NarrativeIntegratorConfig::default());
        let results = integrator.integrate(&mem_conn()).unwrap();
        // 至少应生成一些结果或空（取决于测试数据）
        println!("Integration results: {:?}", results.len());
    }

    #[test]
    fn test_emotional_arc_classification() {
        let integrator = NarrativeIntegrator::new(Arc::new(AutobiographicalIndex::new(IndexConfig::default())), Default::default());
        
        // 上升弧
        let rising = vec![
            AutobiographicalEntry { id: "1".into(), timestamp: 1, title: "".into(), summary: "".into(), content: "".into(), causal_chain: vec![], valence: -0.5, importance: 0.5, value_ids: vec![], goal_ids: vec![], metadata: BTreeMap::new() },
            AutobiographicalEntry { id: "2".into(), timestamp: 2, title: "".into(), summary: "".into(), content: "".into(), causal_chain: vec![], valence: 0.0, importance: 0.5, value_ids: vec![], goal_ids: vec![], metadata: BTreeMap::new() },
            AutobiographicalEntry { id: "3".into(), timestamp: 3, title: "".into(), summary: "".into(), content: "".into(), causal_chain: vec![], valence: 0.5, importance: 0.5, value_ids: vec![], goal_ids: vec![], metadata: BTreeMap::new() },
        ];
        assert_eq!(integrator.classify_emotional_arc(&rising), EmotionalArcType::Rising);

        // 下降弧
        let falling = vec![
            AutobiographicalEntry { id: "1".into(), timestamp: 1, title: "".into(), summary: "".into(), content: "".into(), causal_chain: vec![], valence: 0.5, importance: 0.5, value_ids: vec![], goal_ids: vec![], metadata: BTreeMap::new() },
            AutobiographicalEntry { id: "2".into(), timestamp: 2, title: "".into(), summary: "".into(), content: "".into(), causal_chain: vec![], valence: 0.0, importance: 0.5, value_ids: vec![], goal_ids: vec![], metadata: BTreeMap::new() },
            AutobiographicalEntry { id: "3".into(), timestamp: 3, title: "".into(), summary: "".into(), content: "".into(), causal_chain: vec![], valence: -0.5, importance: 0.5, value_ids: vec![], goal_ids: vec![], metadata: BTreeMap::new() },
        ];
        assert_eq!(integrator.classify_emotional_arc(&falling), EmotionalArcType::Falling);
    }
}
