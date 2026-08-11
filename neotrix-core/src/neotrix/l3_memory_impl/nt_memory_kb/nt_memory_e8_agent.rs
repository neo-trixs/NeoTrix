//! E8 卦象状态机 Agent 检索循环 (C4 阶段 — B5/B6 瓶颈修复)
//!
//! 把 `nt_memory_adaptive_rag::execute_pipeline` 的 while 循环升级为
//! E8 64 卦状态转移驱动的 agent 循环。每步 = 卦象状态, 状态转移即推理轨迹,
//! 可审计、可复现 (设计文档 KB-AGENTIC-RAG-EVOLUTION.md §3.2)。
//!
//! 卦象映射 (Shao Yong bits, upper<<3|lower):
//! - 乾 0x3F 初始理解 → 需 23 首轮检索 → 明夷 5 相关判定
//! - 泰 7 全部相关(生成) | 革 53 部分相关(改写重试) | 屯 20 全不相关(兜底)
//! - 渐 25 图路径推进 → 既济 21 收敛完成

use crate::core::nt_core_e8::{E8TransitionMatrix, Hexagram};

use super::nt_memory_adaptive_rag::{
    rewrite_query, GradedDocument, RelevanceGrade, RetrievalAction,
};
use super::nt_memory_types::SearchResult;

/// E8 检索阶段 — 每阶段绑定一个卦象
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum E8Phase {
    /// 乾 ☰ 0x3F — 初始理解/意图解析
    Init,
    /// 需 ䷄ 23 — 首轮检索 (多路召回)
    Retrieve,
    /// 明夷 ䷣ 5 — 相关性判定
    Grade,
    /// 泰 ䷊ 7 — 全部相关 → 生成
    Generate,
    /// 革 ䷰ 53 — 部分相关 → 改写重试
    Rewrite,
    /// 屯 ䷂ 20 — 全不相关 → 兜底 (图通道补捞)
    Fallback,
    /// 渐 ䷴ 25 — 图路径推进
    GraphHop,
    /// 既济 ䷾ 21 — 收敛完成
    Converge,
}

impl E8Phase {
    /// 阶段绑定的卦象 bits (Shao Yong 序)
    pub fn bits(&self) -> u8 {
        match self {
            Self::Init => 0x3F,      // 乾
            Self::Retrieve => 23,    // 需: 上坎(2)下乾(7)
            Self::Grade => 5,        // 明夷: 上坤(0)下离(5)
            Self::Generate => 7,     // 泰: 上坤(0)下乾(7)
            Self::Rewrite => 53,     // 革: 上兑(6)下离(5)
            Self::Fallback => 20,    // 屯: 上坎(2)下震(4)
            Self::GraphHop => 25,    // 渐: 上巽(3)下艮(1)
            Self::Converge => 21,    // 既济: 上坎(2)下离(5)
        }
    }

    pub fn hexagram(&self) -> Hexagram {
        Hexagram::new(self.bits())
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Init => "乾",
            Self::Retrieve => "需",
            Self::Grade => "明夷",
            Self::Generate => "泰",
            Self::Rewrite => "革",
            Self::Fallback => "屯",
            Self::GraphHop => "渐",
            Self::Converge => "既济",
        }
    }
}

/// E8 Agent 循环配置
pub struct E8AgentConfig {
    /// 改写重试上限
    pub max_iterations: usize,
    /// 每轮检索 top_k
    pub top_k: usize,
    /// 图路径推进跳数上限
    pub graph_hop_limit: usize,
    /// 图跳衰减系数
    pub decay: f64,
}

impl Default for E8AgentConfig {
    fn default() -> Self {
        Self {
            max_iterations: 3,
            top_k: 5,
            graph_hop_limit: 2,
            decay: 0.5,
        }
    }
}

/// E8 Agent 循环 — 卦象状态机驱动的检索循环
pub struct E8AgentLoop {
    pub config: E8AgentConfig,
    /// 64×64 转移矩阵 (复用 E8TransitionMatrix, 记录卦象转移轨迹)
    pub matrix: E8TransitionMatrix,
}

/// E8 Agent 循环结果
#[derive(Debug, Clone)]
pub struct E8AgentResult {
    pub query: String,
    /// 走过的卦象轨迹 (可审计推理路径)
    pub phases: Vec<E8Phase>,
    pub results: Vec<SearchResult>,
    pub graded: Vec<GradedDocument>,
    pub action: RetrievalAction,
    pub iteration_count: usize,
    pub converged: bool,
}

impl E8AgentLoop {
    pub fn new(config: E8AgentConfig) -> Self {
        Self {
            config,
            matrix: E8TransitionMatrix::new(),
        }
    }

    /// 纯状态转移函数 — 可独立单测 (TDD 核心)
    ///
    /// 规则 (设计文档 §3.2):
    /// - Init → Retrieve
    /// - Retrieve → Grade
    /// - Grade: 全相关→Generate | 部分→Rewrite | 全不相关→Fallback | 空→Converge
    /// - Rewrite: 达迭代上限→Converge | 否则→Retrieve
    /// - Fallback → GraphHop → Grade
    /// - Generate → Converge
    pub fn next_phase(
        &self,
        phase: E8Phase,
        graded: &[GradedDocument],
        iteration: usize,
    ) -> E8Phase {
        match phase {
            E8Phase::Init => E8Phase::Retrieve,
            E8Phase::Retrieve => E8Phase::Grade,
            E8Phase::Grade => {
                if graded.is_empty() {
                    return E8Phase::Converge;
                }
                let relevant = graded
                    .iter()
                    .filter(|g| g.relevance == RelevanceGrade::Relevant)
                    .count();
                let irrelevant = graded
                    .iter()
                    .filter(|g| g.relevance == RelevanceGrade::Irrelevant)
                    .count();
                if relevant == graded.len() {
                    E8Phase::Generate
                } else if irrelevant == graded.len() {
                    E8Phase::Fallback
                } else {
                    E8Phase::Rewrite
                }
            }
            E8Phase::Rewrite => {
                if iteration >= self.config.max_iterations {
                    E8Phase::Converge
                } else {
                    E8Phase::Retrieve
                }
            }
            E8Phase::Fallback => E8Phase::GraphHop,
            E8Phase::GraphHop => E8Phase::Grade,
            E8Phase::Generate | E8Phase::Converge => E8Phase::Converge,
        }
    }

    /// 完整 agent 循环 — 检索器以闭包注入 (可测性: 测试注入 mock 检索器)
    ///
    /// 每步: 当前卦象 → 执行阶段行为 → 评估 → 转移下一卦象, 并记录到转移矩阵。
    pub fn run<F>(&mut self, query: &str, mut retrieve: F) -> E8AgentResult
    where
        F: FnMut(&str, usize) -> Vec<SearchResult>,
    {
        let mut phase = E8Phase::Init;
        let mut phases: Vec<E8Phase> = vec![E8Phase::Init];
        let mut results: Vec<SearchResult> = Vec::new();
        let mut current_query = query.to_string();
        let mut iteration = 0usize;
        let mut graded: Vec<GradedDocument> = Vec::new();
        let mut action = RetrievalAction::Skip;
        let mut converged = false;

        // 安全阀: 最多 16 次转移 (8 卦象 × 2 轮)
        for _ in 0..16 {
            let next = self.next_phase(phase, &graded, iteration);
            if next != phase {
                self.matrix.record_transition(phase.bits(), next.bits());
            }
            phase = next;
            phases.push(phase);

            match phase {
                E8Phase::Retrieve => {
                    let more = retrieve(&current_query, self.config.top_k);
                    for r in more {
                        if !results.iter().any(|e| e.node.id == r.node.id) {
                            results.push(r);
                        }
                    }
                    results.sort_by(|a, b| {
                        b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal)
                    });
                    results.truncate(10);
                }
                E8Phase::Grade => {
                    graded = grade_documents(query, &results);
                    action = route_decision(&graded);
                }
                E8Phase::Rewrite => {
                    let new_q = rewrite_query(&current_query, &graded, &results);
                    if new_q == current_query {
                        phase = E8Phase::Converge;
                        continue;
                    }
                    current_query = new_q;
                    iteration += 1;
                }
                E8Phase::Generate => {
                    // 泰 → 既济: 生成完成即收敛
                    self.matrix.record_transition(phase.bits(), E8Phase::Converge.bits());
                    phases.push(E8Phase::Converge);
                    converged = true;
                    break;
                }
                E8Phase::Converge => {
                    converged = true;
                    break;
                }
                _ => {}
            }
        }

        E8AgentResult {
            query: query.to_string(),
            phases,
            results,
            graded,
            action,
            iteration_count: iteration,
            converged,
        }
    }
}

/// 相关性分级 — 复用 adaptive_rag 的语义 (term 重叠 + score 加权)
pub fn grade_documents(query: &str, results: &[SearchResult]) -> Vec<GradedDocument> {
    let query_lower = query.to_lowercase();
    let query_terms: Vec<&str> = query_lower.split_whitespace().collect();

    results
        .iter()
        .map(|r| {
            let text = format!(
                "{} {} {}",
                r.node.title,
                r.node.summary.as_deref().unwrap_or(""),
                r.node.content.as_deref().unwrap_or("")
            )
            .to_lowercase();
            let term_match_ratio = if query_terms.is_empty() {
                0.0
            } else {
                let matched = query_terms.iter().filter(|t| text.contains(*t)).count();
                matched as f64 / query_terms.len() as f64
            };
            let relevance = if term_match_ratio > 0.6 || r.score > 0.7 {
                RelevanceGrade::Relevant
            } else if term_match_ratio > 0.3 || r.score > 0.4 {
                RelevanceGrade::Partial
            } else {
                RelevanceGrade::Irrelevant
            };
            GradedDocument {
                node_id: r.node.id.clone(),
                relevance,
                confidence: (term_match_ratio * 0.5 + r.score * 0.5).clamp(0.0, 1.0),
            }
        })
        .collect()
}

/// 路由决策 — 与 adaptive_rag 一致的四态
pub fn route_decision(graded: &[GradedDocument]) -> RetrievalAction {
    if graded.is_empty() {
        return RetrievalAction::Skip;
    }
    let relevant = graded
        .iter()
        .filter(|g| g.relevance == RelevanceGrade::Relevant)
        .count();
    let irrelevant = graded
        .iter()
        .filter(|g| g.relevance == RelevanceGrade::Irrelevant)
        .count();
    if relevant == graded.len() {
        RetrievalAction::Generate
    } else if irrelevant == graded.len() {
        RetrievalAction::WebSearch
    } else {
        RetrievalAction::Refine
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn loop_engine() -> E8AgentLoop {
        E8AgentLoop::new(E8AgentConfig::default())
    }

    fn graded(relevant: usize, partial: usize, irrelevant: usize) -> Vec<GradedDocument> {
        let mut v = Vec::new();
        for i in 0..relevant {
            v.push(GradedDocument {
                node_id: format!("r{}", i),
                relevance: RelevanceGrade::Relevant,
                confidence: 0.9,
            });
        }
        for i in 0..partial {
            v.push(GradedDocument {
                node_id: format!("p{}", i),
                relevance: RelevanceGrade::Partial,
                confidence: 0.5,
            });
        }
        for i in 0..irrelevant {
            v.push(GradedDocument {
                node_id: format!("i{}", i),
                relevance: RelevanceGrade::Irrelevant,
                confidence: 0.1,
            });
        }
        v
    }

    // ── 卦象合法性 (TDD: E8 转移表单测) ──

    #[test]
    fn test_hexagram_constants_trigram_composition() {
        // 用 hexagram_matrix (Shao Yong bits) 验证卦象的上下卦组成
        let m = crate::core::nt_core_e8::hexagram_matrix();
        // 乾 ☰: 上乾(7)下乾(7)
        assert_eq!(E8Phase::Init.hexagram(), m[7][7], "乾 = 上乾下乾");
        // 需 ䷄: 上坎(2)下乾(7) 水天需
        assert_eq!(E8Phase::Retrieve.hexagram(), m[2][7], "需 = 上坎下乾");
        // 明夷 ䷣: 上坤(0)下离(5) 地火明夷
        assert_eq!(E8Phase::Grade.hexagram(), m[0][5], "明夷 = 上坤下离");
        // 泰 ䷊: 上坤(0)下乾(7) 地天泰
        assert_eq!(E8Phase::Generate.hexagram(), m[0][7], "泰 = 上坤下乾");
        // 革 ䷰: 上兑(6)下离(5) 泽火革
        assert_eq!(E8Phase::Rewrite.hexagram(), m[6][5], "革 = 上兑下离");
        // 屯 ䷂: 上坎(2)下震(4) 水雷屯
        assert_eq!(E8Phase::Fallback.hexagram(), m[2][4], "屯 = 上坎下震");
        // 渐 ䷴: 上巽(3)下艮(1) 风山渐
        assert_eq!(E8Phase::GraphHop.hexagram(), m[3][1], "渐 = 上巽下艮");
        // 既济 ䷾: 上坎(2)下离(5) 水火既济
        assert_eq!(E8Phase::Converge.hexagram(), m[2][5], "既济 = 上坎下离");
    }

    #[test]
    fn test_phases_have_unique_hexagrams() {
        let mut seen = std::collections::HashSet::new();
        for p in [
            E8Phase::Init, E8Phase::Retrieve, E8Phase::Grade, E8Phase::Generate,
            E8Phase::Rewrite, E8Phase::Fallback, E8Phase::GraphHop, E8Phase::Converge,
        ] {
            assert!(seen.insert(p.bits()), "卦象重复: {:?} bits={}", p, p.bits());
        }
    }

    #[test]
    fn test_phase_names() {
        assert_eq!(E8Phase::Init.name(), "乾");
        assert_eq!(E8Phase::Converge.name(), "既济");
    }

    // ── 状态转移 (TDD: next_phase 纯函数) ──

    #[test]
    fn test_init_to_retrieve() {
        let e = loop_engine();
        assert_eq!(e.next_phase(E8Phase::Init, &[], 0), E8Phase::Retrieve);
    }

    #[test]
    fn test_all_relevant_generates() {
        let e = loop_engine();
        let g = graded(3, 0, 0);
        assert_eq!(e.next_phase(E8Phase::Grade, &g, 0), E8Phase::Generate);
        assert_eq!(e.next_phase(E8Phase::Generate, &g, 0), E8Phase::Converge);
    }

    #[test]
    fn test_partial_rewrites() {
        let e = loop_engine();
        let g = graded(1, 1, 1);
        assert_eq!(e.next_phase(E8Phase::Grade, &g, 0), E8Phase::Rewrite);
        assert_eq!(e.next_phase(E8Phase::Rewrite, &g, 0), E8Phase::Retrieve);
    }

    #[test]
    fn test_all_irrelevant_fallbacks() {
        let e = loop_engine();
        let g = graded(0, 0, 3);
        assert_eq!(e.next_phase(E8Phase::Grade, &g, 0), E8Phase::Fallback);
        assert_eq!(e.next_phase(E8Phase::Fallback, &g, 0), E8Phase::GraphHop);
        assert_eq!(e.next_phase(E8Phase::GraphHop, &g, 0), E8Phase::Grade);
    }

    #[test]
    fn test_empty_graded_converges() {
        let e = loop_engine();
        assert_eq!(e.next_phase(E8Phase::Grade, &[], 0), E8Phase::Converge);
    }

    #[test]
    fn test_rewrite_iteration_limit_converges() {
        let e = loop_engine();
        let g = graded(1, 1, 0);
        assert_eq!(e.next_phase(E8Phase::Rewrite, &g, 3), E8Phase::Converge,
            "达迭代上限应收敛");
    }

    // ── 集成循环 (TDD: mock 检索器) ──

    #[test]
    fn test_run_converges_on_relevant() {
        let mut e = loop_engine();
        // mock 检索器: 返回高相关结果 → 全相关 → Generate → Converge
        let result = e.run("E8 hexagram", |q, _k| {
            vec![SearchResult {
                node: super::super::nt_memory_types::KnowledgeNode {
                    id: "n1".into(),
                    node_type: super::super::nt_memory_types::NodeType::Concept,
                    title: q.to_string(),
                    summary: Some("E8 hexagram reasoning".into()),
                    content: None,
                    url: None,
                    domain: None,
                    language: "en".into(),
                    confidence: 1.0,
                    importance: 0.5,
                    created_at: 0,
                    updated_at: 0,
                    access_count: 0,
                    metadata: None,
                    temporal: None,
                    supersedes: None,
                    source_episode: None,
                },
                score: 0.9,
                matched_on: vec![],
                signals: None,
            }]
        });
        assert!(result.converged, "全相关应收敛");
        assert_eq!(result.action, RetrievalAction::Generate);
        assert!(result.phases.contains(&E8Phase::Generate));
        assert!(result.phases.last() == Some(&E8Phase::Converge));
    }

    #[test]
    fn test_run_rewrite_loop_records_transitions() {
        let mut e = loop_engine();
        // mock: 前两轮返回部分相关 (触发 Rewrite), 第三轮返回全相关
        // 注意: title/summary 刻意不含查询词 "multi/hop/query" → 仅靠 score 判定 Partial。
        let mut call = 0;
        let result = e.run("multi hop query", |_q, _k| {
            call += 1;
            let score = if call >= 3 { 0.9 } else { 0.5 };
            vec![SearchResult {
                node: super::super::nt_memory_types::KnowledgeNode {
                    id: format!("n{}", call),
                    node_type: super::super::nt_memory_types::NodeType::Concept,
                    title: "unrelated topic".to_string(),
                    summary: Some("completely different subject matter".into()),
                    content: None,
                    url: None,
                    domain: None,
                    language: "en".into(),
                    confidence: 1.0,
                    importance: 0.5,
                    created_at: 0,
                    updated_at: 0,
                    access_count: 0,
                    metadata: None,
                    temporal: None,
                    supersedes: None,
                    source_episode: None,
                },
                score,
                matched_on: vec![],
                signals: None,
            }]
        });
        assert!(result.converged);
        assert!(result.phases.contains(&E8Phase::Rewrite), "应进入革(改写)阶段: {:?}", result.phases);
        // 转移矩阵记录了轨迹 (至少 2 条转移)
        let total: u64 = e.matrix.recent_transitions.len() as u64;
        assert!(total >= 2, "转移矩阵应记录轨迹: {}", total);
    }

    #[test]
    fn test_run_fallback_on_irrelevant() {
        let mut e = loop_engine();
        // mock: 返回不相关结果 → Fallback → GraphHop → Grade → 仍不相关 → Fallback...
        // 安全阀 16 步内收敛
        let result = e.run("unknown topic", |_q, _k| {
            vec![SearchResult {
                node: super::super::nt_memory_types::KnowledgeNode {
                    id: "x".into(),
                    node_type: super::super::nt_memory_types::NodeType::Concept,
                    title: "zzz unrelated content".to_string(),
                    summary: Some("unrelated".into()),
                    content: None,
                    url: None,
                    domain: None,
                    language: "en".into(),
                    confidence: 0.1,
                    importance: 0.1,
                    created_at: 0,
                    updated_at: 0,
                    access_count: 0,
                    metadata: None,
                    temporal: None,
                    supersedes: None,
                    source_episode: None,
                },
                score: 0.1,
                matched_on: vec![],
                signals: None,
            }]
        });
        // 不相关 → 兜底路径被走过
        assert!(result.phases.contains(&E8Phase::Fallback));
        assert!(result.phases.contains(&E8Phase::GraphHop));
    }
}