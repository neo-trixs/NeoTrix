//! GWT 注意力路由层 — 检索意图分类 (B1/B2 瓶颈修复)
//!
//! 替换 `nt_memory_adaptive_rag::heuristic_classify` 的纯词法计数:
//! 用 5 通道共振打分 (GWT 广播思想) 替代单一规则分支。每个通道是一组
//! 轻量语义特征, 加权求和后取 argmax。不触发 LLM 调用 (保持廉价)。
//!
//! 设计来源: KB-AGENTIC-RAG-EVOLUTION.md §3.1
//! - Fast      → FTS+BM25 快速通道 (事实性/定义类查询)
//! - Vector    → 语义向量通道 (语义问答)
//! - Graph     → 图通道 (关系/多跳推理)
//! - AgentLoop → agent 循环 (复杂分析, C4 接入 E8 状态机)
//! - Decompose → 查询分解 (对比/综述, C2 接入 map-reduce)

use std::num::NonZeroUsize;
use std::sync::RwLock;

/// 检索通道枚举 — 对应目标架构 5 通道
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetrievalChannel {
    Fast,
    Vector,
    Graph,
    AgentLoop,
    Decompose,
}

impl RetrievalChannel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Fast => "fast",
            Self::Vector => "vector",
            Self::Graph => "graph",
            Self::AgentLoop => "agent_loop",
            Self::Decompose => "decompose",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "fast" => Self::Fast,
            "vector" => Self::Vector,
            "graph" => Self::Graph,
            "agent_loop" => Self::AgentLoop,
            _ => Self::Decompose,
        }
    }
}

/// 意图路由结果
#[derive(Debug, Clone, PartialEq)]
pub struct QueryIntent {
    pub channel: RetrievalChannel,
    pub confidence: f64,
    /// 各通道共振得分 (GWT 广播记录, 可审计)
    pub resonance: [f64; 5],
}

/// GWT 路由配置 — 各特征信号权重
pub struct GwtRouterConfig {
    /// 通道先验权重 (log 概率, 默认向量通道主导)
    pub channel_priors: [f64; 5],
    /// 实体密度信号权重
    pub entity_weight: f64,
    /// 关系动词信号权重
    pub relation_weight: f64,
    /// 对比/分解信号权重
    pub compare_weight: f64,
    /// 多跳/因果信号权重
    pub multi_hop_weight: f64,
    /// 事实问句信号权重
    pub factual_weight: f64,
}

impl Default for GwtRouterConfig {
    fn default() -> Self {
        Self {
            channel_priors: [0.30, 0.30, 0.20, 0.10, 0.10],
            entity_weight: 0.25,
            relation_weight: 0.35,
            compare_weight: 0.30,
            multi_hop_weight: 0.30,
            factual_weight: 0.20,
        }
    }
}

/// GWT 路由器 — 纯内存, 无 DB 依赖, 可独立测试
pub struct GwtRouter {
    pub config: GwtRouterConfig,
    cache: RwLock<lru::LruCache<String, QueryIntent>>,
}

/// 查询特征 (轻量语义信号)
#[derive(Debug, Clone, Default)]
pub struct QueryFeatures {
    /// 实体/专有名词数
    pub entity_count: usize,
    /// 关系动词命中 (影响/依赖/related/between...)
    pub has_relation: bool,
    /// 对比标记命中 (对比/比较/difference/pros vs cons)
    pub has_compare: bool,
    /// 多跳/因果标记命中 (导致/引发/then/subsequently)
    pub has_multi_hop: bool,
    /// 纯事实问句 (what is/定义/谁/什么是)
    pub is_factual: bool,
    /// 查询长度 (字符数)
    pub char_len: usize,
}

impl GwtRouter {
    pub fn new(config: GwtRouterConfig) -> Self {
        Self {
            config,
            cache: RwLock::new(lru::LruCache::new(
                NonZeroUsize::new(200).expect("non-zero cache capacity"),
            )),
        }
    }

    /// 意图路由 — 特征提取 → 5 通道共振打分 → argmax
    pub fn route(&self, query: &str) -> QueryIntent {
        if let Ok(mut cache) = self.cache.write() {
            if let Some(cached) = cache.get(query) {
                return cached.clone();
            }
        }

        let f = extract_features(query);
        let intent = self.score_channels(query, &f);

        if let Ok(mut cache) = self.cache.write() {
            cache.put(query.to_string(), intent.clone());
        }
        intent
    }

    /// 5 通道共振打分
    fn score_channels(&self, _query: &str, f: &QueryFeatures) -> QueryIntent {
        let cfg = &self.config;
        let mut scores = [0f64; 5];

        // 基础先验
        for i in 0..5 {
            scores[i] += cfg.channel_priors[i];
        }

        // 实体密度: 实体多 → 复杂通道 (AgentLoop/Decompose), Fast 压低
        if f.entity_count >= 4 {
            if f.has_compare {
                // 多实体显式对比 → 分解 (map-reduce 子查询)
                scores[RetrievalChannel::Decompose as usize] +=
                    cfg.entity_weight * 1.3 + cfg.compare_weight * 0.6;
            } else if f.has_multi_hop || f.has_relation {
                // 多实体 + 因果/关系链 → agent 循环 (每跳定向推进)
                scores[RetrievalChannel::AgentLoop as usize] += cfg.entity_weight * 1.4;
            } else {
                // 多实体无结构 → agent 循环兜底
                scores[RetrievalChannel::AgentLoop as usize] += cfg.entity_weight * 1.2;
                scores[RetrievalChannel::Decompose as usize] += cfg.entity_weight;
            }
            scores[RetrievalChannel::Fast as usize] -= cfg.entity_weight * 0.8;
        } else if f.entity_count >= 2 {
            scores[RetrievalChannel::AgentLoop as usize] += cfg.entity_weight * 0.5;
            scores[RetrievalChannel::Graph as usize] += cfg.entity_weight * 0.4;
        } else {
            scores[RetrievalChannel::Fast as usize] += cfg.entity_weight * 0.5;
        }

        // 关系 + 多跳 联合命中 → 因果链, 归入 AgentLoop (链式推进由 agent 控制)
        if f.has_relation && f.has_multi_hop {
            scores[RetrievalChannel::AgentLoop as usize] +=
                (cfg.relation_weight + cfg.multi_hop_weight) * 1.2;
            scores[RetrievalChannel::Fast as usize] -= cfg.relation_weight * 0.5;
        } else if f.has_relation {
            // 纯关系查询 → Graph 通道
            scores[RetrievalChannel::Graph as usize] += cfg.relation_weight;
            scores[RetrievalChannel::Fast as usize] -= cfg.relation_weight * 0.5;
        } else if f.has_multi_hop {
            // 纯多跳 → AgentLoop
            scores[RetrievalChannel::AgentLoop as usize] += cfg.multi_hop_weight * 1.2;
        }

        // 对比标记: → Decompose 通道 (显式分解子查询)
        if f.has_compare {
            scores[RetrievalChannel::Decompose as usize] += cfg.compare_weight * 1.6;
            scores[RetrievalChannel::Fast as usize] -= cfg.compare_weight * 0.6;
        }

        // 纯事实问句: → Fast 通道 (轻量快速回答)
        if f.is_factual {
            scores[RetrievalChannel::Fast as usize] += cfg.factual_weight * 1.2;
            scores[RetrievalChannel::AgentLoop as usize] -= cfg.factual_weight * 0.5;
        }

        // 长查询 (>80 字符) 且非事实 → 抬升 Vector (语义召回面大)
        if f.char_len > 80 && !f.is_factual {
            scores[RetrievalChannel::Vector as usize] += 0.15;
        }

        // argmax + 置信度 (softmax 风格: 冠军-亚军差距)
        let mut channel = RetrievalChannel::Fast;
        let mut best = f64::MIN;
        for (i, s) in scores.iter().enumerate() {
            if *s > best {
                best = *s;
                channel = match i {
                    0 => RetrievalChannel::Fast,
                    1 => RetrievalChannel::Vector,
                    2 => RetrievalChannel::Graph,
                    3 => RetrievalChannel::AgentLoop,
                    _ => RetrievalChannel::Decompose,
                };
            }
        }
        let mut second = f64::MIN;
        for s in scores.iter() {
            if (*s - best).abs() > 1e-9 && *s > second {
                second = *s;
            }
        }
        let margin = (best - second).abs();
        let confidence = (0.5 + margin * 2.0).clamp(0.5, 0.98);

        QueryIntent { channel, confidence, resonance: scores }
    }
}

/// 特征提取 — 中英文双语感知 (Unicode 处理, 非 ASCII 计数)
pub fn extract_features(query: &str) -> QueryFeatures {
    let q = query.trim().to_lowercase();
    let mut f = QueryFeatures {
        char_len: q.chars().count(),
        ..Default::default()
    };

    // 实体计数: 大写专名 (英文, HashSet 去重防重复计数) + 中文书名号/引号内词
    let mut entities: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for w in q.split_whitespace() {
        let is_named = w.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
            && w.len() > 2
            && !["the", "this", "that", "what", "why", "how", "when", "where", "which"]
                .contains(&w);
        if is_named {
            entities.insert(w);
        }
    }
    // 中文实体: 顿号/书名号分隔的 2+ 字片段 (去重)
    q.split(|c: char| c == '《' || c == '》' || c == '、' || c == '，')
        .filter(|seg| seg.chars().count() >= 2 && !seg.contains(' '))
        .for_each(|seg| {
            entities.insert(seg);
        });
    f.entity_count = entities.len();

    // 关系动词 (中英)
    f.has_relation = [
        "影响", "依赖", "关系", "相关", "属于", "导致", "between", "related", "depends",
        "influences", "associated", "relation",
    ]
    .iter()
    .any(|kw| q.contains(kw));

    // 对比标记
    f.has_compare = [
        "对比", "比较", "区别", "差异", "versus", "difference", "compare", "pros", "cons",
        "vs ",
    ]
    .iter()
    .any(|kw| q.contains(kw));

    // 多跳/因果标记
    f.has_multi_hop = [
        "然后", "之后", "逐步", "进而", "最终", "then", "subsequently", "after that",
        "therefore", "consequently",
    ]
    .iter()
    .any(|kw| q.contains(kw));

    // 纯事实问句
    f.is_factual = [
        "什么是", "谁", "定义", "介绍", "what is", "what are", "who is", "define", "explain ",
    ]
    .iter()
    .any(|kw| q.contains(kw))
        && !f.has_relation
        && !f.has_compare;

    f
}

#[cfg(test)]
mod tests {
    use super::*;

    fn router() -> GwtRouter {
        GwtRouter::new(GwtRouterConfig::default())
    }

    #[test]
    fn test_factual_query_routes_fast() {
        let intent = router().route("什么是 RAG 检索增强生成?");
        assert_eq!(intent.channel, RetrievalChannel::Fast,
            "事实问句应路由 Fast, 实际 {:?} resonance={:?}", intent.channel, intent.resonance);
        assert!(intent.confidence >= 0.5);
    }

    #[test]
    fn test_relation_query_routes_graph() {
        let intent = router().route("E8 hexagram 如何影响 GWT 注意力路由的关系?");
        assert_eq!(intent.channel, RetrievalChannel::Graph,
            "关系查询应路由 Graph, 实际 {:?}", intent.channel);
    }

    #[test]
    fn test_compare_query_routes_decompose() {
        let intent = router().route("对比 Naive RAG 与 Agentic RAG 的差异和优缺点");
        assert_eq!(intent.channel, RetrievalChannel::Decompose,
            "对比查询应路由 Decompose, 实际 {:?}", intent.channel);
    }

    #[test]
    fn test_multi_hop_query_routes_agentloop() {
        let intent = router().route(
            "SEAL pipeline 进化 然后 逐步影响 ConsciousnessTree 最终 导致 能力网 重构"
        );
        assert_eq!(intent.channel, RetrievalChannel::AgentLoop,
            "多跳因果查询应路由 AgentLoop, 实际 {:?}", intent.channel);
    }

    #[test]
    fn test_high_entity_compare_routes_decompose() {
        let intent = router().route(
            "Compare SEAL self-iteration with PRM reward model and E8 hex state transitions and HyperCube"
        );
        assert_eq!(intent.channel, RetrievalChannel::Decompose,
            "4+ 实体显式对比应路由 Decompose (map-reduce), 实际 {:?}", intent.channel);
    }

    #[test]
    fn test_high_entity_causal_routes_agentloop() {
        let intent = router().route(
            "SEAL 的 E8 和 HyperCube 与 GWT 的关系 然后 进化为 ConsciousnessTree 最终 重构 能力网"
        );
        assert_eq!(intent.channel, RetrievalChannel::AgentLoop,
            "4+ 实体因果链应路由 AgentLoop, 实际 {:?}", intent.channel);
    }

    #[test]
    fn test_english_factual_routes_fast() {
        let intent = router().route("What is the definition of vector symbolic architecture?");
        assert_eq!(intent.channel, RetrievalChannel::Fast);
    }

    #[test]
    fn test_cache_hit() {
        let r = router();
        let a = r.route("什么是 RAG?");
        let b = r.route("什么是 RAG?");
        assert_eq!(a, b, "相同查询应命中缓存返回一致结果");
    }

    #[test]
    fn test_extract_features_bilingual() {
        let f = extract_features("《史记》和《资治通鉴》的对比 以及 对后世的差异影响");
        assert!(f.has_compare, "中文对比标记未检出");
        assert!(f.has_relation, "中文关系词未检出");
        assert!(f.entity_count >= 1, "中文书名号实体未检出: {}", f.entity_count);
    }

    #[test]
    fn test_channel_roundtrip() {
        for s in ["fast", "vector", "graph", "agent_loop", "decompose"] {
            assert_eq!(RetrievalChannel::from_str(s).as_str(), s);
        }
        assert_eq!(RetrievalChannel::from_str("unknown"), RetrievalChannel::Decompose);
    }
}
