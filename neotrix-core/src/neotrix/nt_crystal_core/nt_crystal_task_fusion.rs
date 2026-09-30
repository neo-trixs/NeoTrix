//! # nt_crystal_task_fusion — 晶体任务闭环
//!
//! 之前设计的任务机制落地：晶体内核智能拆解 → 需要推理的小任务分发给
//! LLM 问答 → 答案智能融合 → 汇总成后续任务。
//!
//! ```text
//! goal ──▶ suggest（晶体记忆感知拆解：Solution/Theory/Episode 命中 + 兜底切分）
//!           │
//!           ▼
//!        dispatch（Reasoning 走 NtLlmAsk 问答；Deterministic 走晶体经验直给，零 LLM 开销）
//!           │
//!           ▼
//!        fuse（置信度加权共识聚类 + 少数派/矛盾报告）──▶ 后续任务队列 follow_ups
//!           │                                              （失败重试 / 矛盾裁决 / 低置信复核 / 人工确认）
//!           ▼
//!        report（含 JEV 决策留痕 + 校准分；follow_ups 可直接再跑 run 闭环）
//! ```
//!
//! ## JEV 接线点（R-P79：外部机制同会话接到生产）
//! - **J1 拆解分级 = JEV Choice**：每个候选子任务在
//!   `{reasoning, deterministic, skip}` 上的概率分布 → `ChoiceAnswer::new`
//!   自动给出 confidence + margin；胶着（margin < 0.05）经
//!   `abstain_if_contested` 弃权转人工确认。
//! - **J2 回复可信 = JEV Noul**：每条 LLM 回答经 `NoulAnswer::new`
//!   归一（"该回答可信？"），`< 0.7` 自动挂 `needs_review`。
//! - **J3 融合裁决 = Noul + RiskTier**：获胜簇权重占比 → Noul
//!   （"融合结论为真？"）；`RiskTier::decide` 定 Automate 直接进后续任务
//!   还是 HumanReview 进人工队列。
//! - **J4 校准尺 = eval::brier_score**：报告 `calibration = 1 - brier`
//!   （簇权重作概率、是否共识作正确性代理），融合质量可度量。
//! - **J5 决策留痕 = JevResultSet**：每子任务分级 Choice + 融合 Noul
//!   全量记录，可审计、可复跑。
//!
//! # Safety
//! - 纯内存 + trait 注入的 LLM，无 IO、无锁、无 unsafe (R-P1)。
//! - 生产代码无 `unwrap/expect/panic`；浮点排序用
//!   `partial_cmp().unwrap_or(Equal)`（与 `nt_core_task_dispatcher` 一致）。
//! - 单条 LLM 失败只记 `failed`，不掀翻整轮（弹性）。

use super::CrystalCore;
use crate::l5_cognition::nt_jev::{
    abstain_if_contested, brier_score, is_abstained, ChoiceAnswer, JevDecision, JevResultSet,
    NoulAnswer, RiskDecision, RiskTier,
};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

// ============================================================================
// 文本相似度（模块内自包含：中英混合关键词 Jaccard）
// ============================================================================

/// 2026-09-29：本地副本已删，改用唯一事实源（同 `nt_shared_mind.rs`）。
/// ⛔ 分词用**窄**口径；宽口径会把标点塞进 bigram。
fn is_cjk(c: char) -> bool {
    neotrix_types::core::nt_cjk::is_cjk_han(c)
}

/// 关键词集：空白/标点切词（≥2字保留）+ 中文二元字（解决无空格中文的部分重叠）。
fn keywords(text: &str) -> HashSet<String> {
    let lower = text.to_lowercase();
    let mut set = HashSet::new();
    for tok in lower.split(|c: char| !(c.is_alphanumeric() || is_cjk(c))) {
        if tok.chars().count() >= 2 {
            set.insert(tok.to_string());
        }
    }
    let cjk: Vec<char> = lower.chars().filter(|c| is_cjk(*c)).collect();
    for w in cjk.windows(2) {
        set.insert(w.iter().collect());
    }
    set
}

fn jaccard(a: &HashSet<String>, b: &HashSet<String>) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let inter = a.intersection(b).count() as f64;
    let union = (a.len() + b.len()) as f64 - inter;
    if union <= 0.0 {
        0.0
    } else {
        (inter / union).clamp(0.0, 1.0)
    }
}

fn truncate_chars(s: &str, n: usize) -> String {
    let t: String = s.chars().take(n).collect();
    if s.chars().count() > n {
        format!("{}…", t)
    } else {
        t
    }
}

// ============================================================================
// 类型
// ============================================================================

/// 子任务路由：推理走 LLM，确定性走晶体经验直给，胶着/低价值跳过。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NtSubtaskRoute {
    Reasoning,
    Deterministic,
    Skip,
}

/// 晶体拆解出的子任务（J1：route 由 JEV Choice 产生并留痕）。
#[derive(Debug, Clone)]
pub struct NtCrystalSubtask {
    pub id: String,
    pub title: String,
    pub question: String,
    pub route: NtSubtaskRoute,
    pub route_decision: JevDecision,
    pub confidence: f64,
    /// 来源：`Solution:<id>` / `Theory:<id>` / `Episode:<id>` / `fallback:split`。
    pub provenance: String,
    /// Deterministic 子任务的本地直给答案（晶体经验原文）。
    pub local_answer: Option<String>,
}

/// LLM 单次问答回复。
#[derive(Debug, Clone)]
pub struct NtLlmReply {
    pub text: String,
    pub confidence: f64,
    pub model: String,
}

/// LLM 问答抽象：同步 trait（与 L1 `ReasoningEngineProvider` 同形），
///
/// 异步 Provider 由调用方桥接（block_on 或产出回复后注入），晶体侧保持纯同步可测。
pub trait NtLlmAsk: Send + Sync {
    fn ask(&self, prompt: &str) -> Result<NtLlmReply, NtTaskFusionError>;

    /// 流式问答：逐块回调，返回 false 即取消。
    /// 默认实现 = 非流式 ask + 全文一次回调（行为与 ask 一致）。
    fn ask_stream(
        &self,
        prompt: &str,
        on_chunk: &dyn Fn(&str) -> bool,
    ) -> Result<NtLlmReply, NtTaskFusionError> {
        let reply = self.ask(prompt)?;
        let _ = on_chunk(&reply.text);
        Ok(reply)
    }
}

/// 进度接收器（TUI 工作相实时渲染用；None = 静默）。
/// 全默认空实现，按需覆盖；`cancelled` 被轮询用于 Esc 中断。
pub trait NtProgressSink: Send + Sync {
    fn on_subtask_start(&self, _subtask_id: &str, _title: &str) {}
    fn on_answer_chunk(&self, _subtask_id: &str, _delta: &str) {}
    fn on_subtask_done(&self, _subtask_id: &str, _success: bool) {}
    fn cancelled(&self) -> bool {
        false
    }
}

/// 融合错误。
#[derive(Debug, thiserror::Error)]
pub enum NtTaskFusionError {
    #[error("LLM ask failed: {0}")]
    Llm(String),
}

/// 带评分的单条答案（J2：verdict 为 JEV Noul）。
#[derive(Debug, Clone)]
pub struct NtScoredAnswer {
    pub subtask_id: String,
    pub text: String,
    pub confidence: f64,
    pub verdict: NoulAnswer,
    pub model: String,
}

/// 答案共识簇。
#[derive(Debug, Clone)]
pub struct NtAnswerCluster {
    pub representative: String,
    pub member_ids: Vec<String>,
    pub weight: f64,
    pub avg_confidence: f64,
}

/// 融合结论（J3：verdict Noul + risk RiskTier 裁决）。
#[derive(Debug, Clone)]
pub struct NtFusedAnswer {
    pub text: String,
    pub confidence: f64,
    pub verdict: NoulAnswer,
    pub risk: RiskDecision,
    pub clusters: Vec<NtAnswerCluster>,
    pub minority: Vec<String>,
    pub contradictions: Vec<String>,
    /// 实际熔入结论的子任务 ID。
    pub used_ids: Vec<String>,
}

/// 整轮报告：follow_ups 可直接再跑 `run`，形成后续任务闭环。
#[derive(Debug, Clone)]
pub struct NtTaskLoopReport {
    pub goal: String,
    pub subtasks: Vec<NtCrystalSubtask>,
    pub answers: Vec<NtScoredAnswer>,
    /// (subtask_id, error)：单点失败记录，不阻断整轮。
    pub failed: Vec<(String, String)>,
    pub fused: NtFusedAnswer,
    pub follow_ups: Vec<String>,
    /// J5：子任务分级 Choice + 融合 Noul，全量留痕。
    pub decisions: JevResultSet,
    /// J4：`1 - brier_score`，融合校准度 0.0..=1.0。
    pub calibration: f64,
}

/// 闭环配置。
#[derive(Debug, Clone, Copy)]
pub struct NtTaskLoopConfig {
    pub max_subtasks: usize,
    pub min_answer_chars: usize,
    pub consensus_jaccard: f64,
    pub contra_lo: f64,
    pub contra_hi: f64,
    pub max_follow_ups: usize,
    pub risk_tier: RiskTier,
    /// 并行执行 Reasoning 子任务（默认 true）。
    pub parallel: bool,
    /// 并行子任务最大并发数（0=不限，由 CPU/模型配额决定）。
    pub max_concurrent: usize,
    /// 重叠检测阈值：Jaccard ≥ 此值视为冗余可跳过（0.0=禁用）。
    pub overlap_threshold: f64,
}

impl Default for NtTaskLoopConfig {
    fn default() -> Self {
        Self {
            max_subtasks: 5,
            min_answer_chars: 8,
            consensus_jaccard: 0.5,
            contra_lo: 0.25,
            contra_hi: 0.5,
            max_follow_ups: 6,
            risk_tier: RiskTier::Recoverable,
            parallel: true,
            max_concurrent: 0,
            overlap_threshold: 0.6,
        }
    }
}

// ============================================================================
// 引擎
// ============================================================================

/// 晶体任务闭环引擎。
pub struct NtCrystalTaskLoop {
    config: NtTaskLoopConfig,
}

/// 拆解候选（内部）。
struct Candidate {
    title: String,
    question: String,
    score: f64,
    provenance: String,
    deterministic: bool,
    local_answer: Option<String>,
}

impl NtCrystalTaskLoop {
    pub fn new(config: NtTaskLoopConfig) -> Self {
        Self { config }
    }

    /// 一轮闭环：拆解 → 分发问答 → 融合 → 后续任务汇总。
    pub fn run(
        &self,
        goal: &str,
        core: &CrystalCore,
        llm: &dyn NtLlmAsk,
    ) -> NtTaskLoopReport {
        self.run_with_sink(goal, core, llm, None)
    }

    /// 带进度接收器的闭环（TUI 工作相实时渲染 + Esc 取消走这里）。
    ///
    /// `config.parallel = true` 时自动走并行路径。
    pub fn run_with_sink(
        &self,
        goal: &str,
        core: &CrystalCore,
        llm: &dyn NtLlmAsk,
        sink: Option<&dyn NtProgressSink>,
    ) -> NtTaskLoopReport {
        if self.config.parallel {
            self.run_with_sink_parallel(goal, core, llm, sink)
        } else {
            self.run_with_sink_sequential(goal, core, llm, sink)
        }
    }

    // ── 1. 晶体记忆感知拆解 ──

    fn suggest(&self, goal: &str, core: &CrystalCore) -> Vec<NtCrystalSubtask> {
        let goal_kws = keywords(goal);
        let mut cands: Vec<Candidate> = Vec::new();

        // L3 成功经验：可复用 Solution 命中 → Deterministic 直给候选
        for sol in &core.experience.successes {
            let text = format!("{} {}", sol.problem, sol.approach);
            let j = jaccard(&goal_kws, &keywords(&text));
            if j > 0.0 {
                let score = j * 0.7 + if sol.reusable { 0.3 } else { 0.0 };
                let det = sol.reusable && score >= 0.3;
                cands.push(Candidate {
                    title: truncate_chars(&sol.problem, 24),
                    question: format!("基于晶体经验解决：{}", sol.problem),
                    score,
                    provenance: format!("Solution:{}", sol.id),
                    deterministic: det,
                    local_answer: Some(sol.approach.clone()),
                });
            }
        }

        // L2 理论：core_claim 命中 → Reasoning 候选
        for theory in core.knowledge.theories.values() {
            let text = format!("{} {}", theory.name, theory.core_claim);
            let j = jaccard(&goal_kws, &keywords(&text));
            if j > 0.0 {
                cands.push(Candidate {
                    title: truncate_chars(&theory.name, 24),
                    question: format!("用「{}」分析：{}（{}）", theory.name, goal, theory.core_claim),
                    score: j * theory.confidence.clamp(0.0, 1.0),
                    provenance: format!("Theory:{}", theory.id),
                    deterministic: false,
                    local_answer: None,
                });
            }
        }

        // L3 情境：高质量 Episode 命中 → Reasoning 候选
        for ep in &core.experience.episodes {
            if ep.quality < 0.6 {
                continue;
            }
            let text = format!("{} {}", ep.context, ep.reflection);
            let j = jaccard(&goal_kws, &keywords(&text));
            if j > 0.0 {
                cands.push(Candidate {
                    title: truncate_chars(&ep.context, 24),
                    question: format!("参考历史情境推理：{}（行动：{}）", ep.context, ep.action),
                    score: j * ep.quality.clamp(0.0, 1.0),
                    provenance: format!("Episode:{}", ep.id),
                    deterministic: false,
                    local_answer: None,
                });
            }
        }

        // 兜底：晶体无命中 → 按连词切分（中英），切不出则整题单问
        if cands.is_empty() {
            let parts = split_goal(goal);
            if parts.len() > 1 {
                for (i, p) in parts.iter().enumerate() {
                    cands.push(Candidate {
                        title: truncate_chars(p, 24),
                        question: format!("步骤{}：{}", i + 1, p),
                        score: 0.1,
                        provenance: "fallback:split".to_string(),
                        deterministic: false,
                        local_answer: None,
                    });
                }
            } else {
                cands.push(Candidate {
                    title: truncate_chars(goal, 24),
                    question: goal.to_string(),
                    score: 0.1,
                    provenance: "fallback:single".to_string(),
                    deterministic: false,
                    local_answer: None,
                });
            }
        }

        // 分数降序、provenance 升序（确定性），截断到上限
        cands.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.provenance.cmp(&b.provenance))
        });
        cands.truncate(self.config.max_subtasks.max(1));

        cands
            .into_iter()
            .enumerate()
            .map(|(i, c)| {
                let (route, decision) = route_choice(&c);
                NtCrystalSubtask {
                    id: format!("st-{}", i + 1),
                    title: c.title,
                    question: c.question,
                    route,
                    confidence: decision.confidence(),
                    route_decision: decision,
                    provenance: c.provenance,
                    local_answer: c.local_answer,
                }
            })
            .collect()
    }

    // ── 2. 分发：Reasoning 问 LLM，Deterministic 晶体直给 ──

    fn dispatch(
        &self,
        subtasks: &[NtCrystalSubtask],
        llm: &dyn NtLlmAsk,
        sink: Option<&dyn NtProgressSink>,
    ) -> (Vec<NtScoredAnswer>, Vec<(String, String)>) {
        let mut answers = Vec::new();
        let mut failed = Vec::new();

        for st in subtasks {
            match st.route {
                NtSubtaskRoute::Skip => continue,
                NtSubtaskRoute::Deterministic => match &st.local_answer {
                    Some(text) if !text.trim().is_empty() => {
                        let conf = st.confidence.clamp(0.0, 1.0).max(0.5);
                        if let Some(s) = sink {
                            s.on_subtask_start(&st.id, &st.title);
                        }
                        answers.push(NtScoredAnswer {
                            subtask_id: st.id.clone(),
                            text: text.clone(),
                            confidence: conf,
                            verdict: NoulAnswer::new(conf),
                            model: "crystal-memory".to_string(),
                        });
                        if let Some(s) = sink {
                            s.on_subtask_done(&st.id, true);
                        }
                    }
                    _ => {
                        failed.push((st.id.clone(), "no local answer".to_string()));
                        if let Some(s) = sink {
                            s.on_subtask_done(&st.id, false);
                        }
                    }
                },
                NtSubtaskRoute::Reasoning => {
                    if let Some(s) = sink {
                        s.on_subtask_start(&st.id, &st.title);
                    }
                    let stream_result = llm.ask_stream(&st.question, &|chunk| {
                        if let Some(s) = sink {
                            s.on_answer_chunk(&st.id, chunk);
                            !s.cancelled()
                        } else {
                            true
                        }
                    });
                    match stream_result {
                        Ok(reply) => {
                            let conf = reply.confidence.clamp(0.0, 1.0);
                            answers.push(NtScoredAnswer {
                                subtask_id: st.id.clone(),
                                text: reply.text,
                                confidence: conf,
                                verdict: NoulAnswer::new(conf),
                                model: reply.model,
                            });
                            if let Some(s) = sink {
                                s.on_subtask_done(&st.id, true);
                            }
                        }
                        Err(e) => {
                            failed.push((st.id.clone(), e.to_string()));
                            if let Some(s) = sink {
                                s.on_subtask_done(&st.id, false);
                            }
                        }
                    }
                }
            }
        }
        (answers, failed)
    }

    // ── 2b. 并行分发：多 Reasoning 子任务同时执行，共享 SharedMind ──

    /// 串行闭环（向后兼容，config.parallel = false 时走这里）。
    fn run_with_sink_sequential(
        &self,
        goal: &str,
        core: &CrystalCore,
        llm: &dyn NtLlmAsk,
        sink: Option<&dyn NtProgressSink>,
    ) -> NtTaskLoopReport {
        let subtasks = self.suggest(goal, core);
        let (answers, failed) = self.dispatch(&subtasks, llm, sink);
        let fused = self.fuse(&answers);
        let calibration = self.calibrate(&fused);
        let follow_ups = self.follow_ups(goal, &subtasks, &fused, &failed);

        let mut decisions: JevResultSet = HashMap::new();
        for st in &subtasks {
            decisions.insert(st.id.clone(), st.route_decision.clone());
        }
        decisions.insert(
            "fused".to_string(),
            JevDecision::Noul(fused.verdict.clone()),
        );

        NtTaskLoopReport {
            goal: goal.to_string(),
            subtasks,
            answers,
            failed,
            fused,
            follow_ups,
            decisions,
            calibration,
        }
    }

    /// 带进度接收器的闭环（并行版本）。
    ///
    /// 与 `run_with_sink` 相同语义，但 Reasoning 子任务通过 `thread::scope` 并行执行，
    /// 共享 `SharedMind` 实时交换发现、检测重叠、避免冗余 LLM 调用。
    pub fn run_with_sink_parallel(
        &self,
        goal: &str,
        core: &CrystalCore,
        llm: &dyn NtLlmAsk,
        sink: Option<&dyn NtProgressSink>,
    ) -> NtTaskLoopReport {
        let subtasks = self.suggest(goal, core);
        let shared = super::SharedMind::new();
        let (answers, failed) = self.dispatch_parallel(&subtasks, llm, sink, &shared);
        let fused = self.fuse(&answers);
        let calibration = self.calibrate(&fused);
        let follow_ups = self.follow_ups(goal, &subtasks, &fused, &failed);

        let mut decisions: JevResultSet = HashMap::new();
        for st in &subtasks {
            decisions.insert(st.id.clone(), st.route_decision.clone());
        }
        decisions.insert(
            "fused".to_string(),
            JevDecision::Noul(fused.verdict.clone()),
        );

        NtTaskLoopReport {
            goal: goal.to_string(),
            subtasks,
            answers,
            failed,
            fused,
            follow_ups,
            decisions,
            calibration,
        }
    }

    /// 并行分发：Deterministic 顺序处理，Reasoning 通过 `thread::scope` 并行。
    ///
    /// - 每个 Reasoning 线程先查 `SharedMind` 重叠：高覆盖则跳过（省 LLM 调用）。
    /// - 每个 Reasoning 线程完成后 `post` 发现到 `SharedMind`。
    /// - `config.overlap_threshold`：Jaccard ≥ 此值视为重叠（0.0=禁用, 0.6=默认）。
    /// - `config.max_concurrent`：> 0 时限制并行线程数（超出部分排队）。
    fn dispatch_parallel(
        &self,
        subtasks: &[NtCrystalSubtask],
        llm: &dyn NtLlmAsk,
        sink: Option<&dyn NtProgressSink>,
        shared: &super::SharedMind,
    ) -> (Vec<NtScoredAnswer>, Vec<(String, String)>) {
        use std::sync::mpsc;

        let overlap_threshold = self.config.overlap_threshold;
        let max_concurrent = self.config.max_concurrent;

        // 1) Deterministic 子任务：顺序处理（零 LLM 开销）
        let mut answers = Vec::new();
        let mut failed = Vec::new();
        for st in subtasks {
            if st.route != NtSubtaskRoute::Deterministic {
                continue;
            }
            match &st.local_answer {
                Some(text) if !text.trim().is_empty() => {
                    let conf = st.confidence.clamp(0.0, 1.0).max(0.5);
                    if let Some(s) = sink {
                        s.on_subtask_start(&st.id, &st.title);
                    }
                    answers.push(NtScoredAnswer {
                        subtask_id: st.id.clone(),
                        text: text.clone(),
                        confidence: conf,
                        verdict: NoulAnswer::new(conf),
                        model: "crystal-memory".to_string(),
                    });
                    if let Some(s) = sink {
                        s.on_subtask_done(&st.id, true);
                    }
                    // 确定性答案也写入共享心智（供后续并行子任务参考）
                    shared.post(super::Discovery {
                        text: text.clone(),
                        source_id: st.id.clone(),
                        confidence: conf,
                    });
                }
                _ => {
                    failed.push((st.id.clone(), "no local answer".to_string()));
                    if let Some(s) = sink {
                        s.on_subtask_done(&st.id, false);
                    }
                }
            }
        }

        // 2) Reasoning 子任务：先过滤重叠，再并行
        let reasoning: Vec<&NtCrystalSubtask> = subtasks
            .iter()
            .filter(|st| st.route == NtSubtaskRoute::Reasoning)
            .collect();

        if reasoning.is_empty() {
            return (answers, failed);
        }

        // 2a) 顺序预过滤：重叠检测（廉价关键词匹配）+ 发布确定性发现
        //     双向检测：(1) 问题 vs 已发布发现 (2) 问题 vs 已排队子任务
        //     确保后续并行线程能看到已有发现，避免竞态。
        let mut to_run: Vec<(usize, &NtCrystalSubtask)> = Vec::new();
        let mut queued_kws: Vec<HashSet<String>> = Vec::new(); // 已排队子任务的关键词
        for (i, st) in reasoning.iter().enumerate() {
            let q_kws = super::SharedMind::keywords_of(&st.question);
            // 双向重叠检测
            let overlaps = if overlap_threshold > 0.0 {
                shared.has_overlap(&st.question, overlap_threshold)
                    || queued_kws.iter().any(|kws| {
                        let inter = q_kws.intersection(kws).count() as f64;
                        let union = (q_kws.len() + kws.len()) as f64 - inter;
                        union > 0.0 && (inter / union) >= overlap_threshold
                    })
            } else {
                false
            };
            if overlaps {
                if let Some(s) = sink {
                    s.on_subtask_start(&st.id, &st.title);
                    s.on_subtask_done(&st.id, true);
                }
                continue;
            }
            queued_kws.push(q_kws);
            to_run.push((i, st));
        }

        if to_run.is_empty() {
            return (answers, failed);
        }

        // 2b) 并行执行：线程间共享已过滤的子任务列表
        let (tx, rx) = mpsc::channel::<(
            usize,
            Result<NtScoredAnswer, (String, String)>,
        )>();

        std::thread::scope(|scope| {
            // max_concurrent 限流：AtomicUsize 计数 + yield 自旋等待
            let active = Arc::new(std::sync::atomic::AtomicUsize::new(0));
            for (i, st) in to_run.iter() {
                let i = *i;

                // 限流：等待活跃线程数 < max_concurrent
                if max_concurrent > 0 {
                    while active.load(std::sync::atomic::Ordering::SeqCst) >= max_concurrent {
                        std::thread::yield_now();
                    }
                }

                if let Some(s) = sink {
                    s.on_subtask_start(&st.id, &st.title);
                }

                shared.register_active(&st.id);
                let tx = tx.clone();
                let shared = shared.clone();
                let st = (*st).clone();
                let sink_ref: Option<&(dyn NtProgressSink + '_)> = sink;
                let active = active.clone();

                active.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                scope.spawn(move || {
                    // RAII guard: 线程结束时自动 decrement active count
                    struct ActiveGuard(Arc<std::sync::atomic::AtomicUsize>);
                    impl Drop for ActiveGuard {
                        fn drop(&mut self) {
                            self.0.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                        }
                    }
                    let _guard = ActiveGuard(active);
                    // panic 安全：单线程 panic 不炸整轮
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        // 注入已有发现作为上下文（避免重复回答）
                        let context = shared.relevant_context(&st.question, 3);
                        let enhanced_question = if context.is_empty() {
                            st.question.clone()
                        } else {
                            format!(
                                "{}\n\n[已有发现供参考，避免重复]\n{}",
                                st.question,
                                context.join("\n")
                            )
                        };

                        llm.ask_stream(&enhanced_question, &|chunk| {
                            if let Some(s) = sink_ref {
                                s.on_answer_chunk(&st.id, chunk);
                                !s.cancelled()
                            } else {
                                true
                            }
                        })
                    }));

                    match result {
                        Ok(Ok(reply)) => {
                            let conf = reply.confidence.clamp(0.0, 1.0);
                            let answer = NtScoredAnswer {
                                subtask_id: st.id.clone(),
                                text: reply.text.clone(),
                                confidence: conf,
                                verdict: NoulAnswer::new(conf),
                                model: reply.model,
                            };
                            shared.post(super::Discovery {
                                text: reply.text,
                                source_id: st.id.clone(),
                                confidence: conf,
                            });
                            shared.unregister_active(&st.id);
                            let _ = tx.send((i, Ok(answer)));
                        }
                        Ok(Err(e)) => {
                            shared.unregister_active(&st.id);
                            let _ = tx.send((i, Err((st.id.clone(), e.to_string()))));
                        }
                        Err(panic) => {
                            shared.unregister_active(&st.id);
                            let msg = if let Some(s) = panic.downcast_ref::<&str>() {
                                s.to_string()
                            } else if let Some(s) = panic.downcast_ref::<String>() {
                                s.clone()
                            } else {
                                "子任务线程 panic".to_string()
                            };
                            let _ = tx.send((i, Err((st.id.clone(), msg))));
                        }
                    }
                });
            }

            drop(tx);

            for (i, result) in rx.into_iter() {
                match result {
                    Ok(answer) => {
                        if let Some(s) = sink {
                            s.on_subtask_done(&reasoning[i].id, true);
                        }
                        answers.push(answer);
                    }
                    Err((id, err)) => {
                        if let Some(s) = sink {
                            s.on_subtask_done(&id, false);
                        }
                        failed.push((id, err));
                    }
                }
            }
        });

        (answers, failed)
    }

    // ── 3. 融合：置信度加权共识聚类 ──

    pub(crate) fn fuse(&self, answers: &[NtScoredAnswer]) -> NtFusedAnswer {
        let mut clusters: Vec<NtAnswerCluster> = Vec::new();

        for a in answers {
            if a.text.chars().count() < self.config.min_answer_chars {
                continue;
            }
            let kws = keywords(&a.text);
            let mut best: Option<usize> = None;
            let mut best_j = 0.0;
            for (i, c) in clusters.iter().enumerate() {
                let j = jaccard(&kws, &keywords(&c.representative));
                if j >= self.config.consensus_jaccard && j > best_j {
                    best = Some(i);
                    best_j = j;
                }
            }
            match best {
                Some(i) => {
                    let c = &mut clusters[i];
                    if a.confidence > c.avg_confidence {
                        c.representative = a.text.clone();
                    }
                    c.member_ids.push(a.subtask_id.clone());
                    c.weight += a.confidence;
                    let n = c.member_ids.len() as f64;
                    c.avg_confidence = if n > 0.0 { c.weight / n } else { 0.0 };
                }
                None => clusters.push(NtAnswerCluster {
                    representative: a.text.clone(),
                    member_ids: vec![a.subtask_id.clone()],
                    weight: a.confidence,
                    avg_confidence: a.confidence,
                }),
            }
        }

        clusters.sort_by(|a, b| {
            b.weight
                .partial_cmp(&a.weight)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.representative.cmp(&b.representative))
        });

        let total: f64 = clusters.iter().map(|c| c.weight).sum();
        let share = clusters
            .first()
            .map(|c| {
                if total > 0.0 {
                    (c.weight / total).clamp(0.0, 1.0)
                } else {
                    0.0
                }
            })
            .unwrap_or(0.0);

        let text = clusters
            .first()
            .map(|c| c.representative.clone())
            .unwrap_or_default();
        let used_ids = clusters
            .first()
            .map(|c| c.member_ids.clone())
            .unwrap_or_default();

        // 少数派：未形成共识的孤簇
        let minority: Vec<String> = clusters
            .iter()
            .skip(1)
            .filter(|c| c.member_ids.len() == 1)
            .take(3)
            .map(|c| truncate_chars(&c.representative, 120))
            .collect();

        // 矛盾：部分重叠但未达共识的簇对
        let mut contradictions = Vec::new();
        for (i, a) in clusters.iter().enumerate() {
            for b in clusters.iter().skip(i + 1) {
                let j = jaccard(&keywords(&a.representative), &keywords(&b.representative));
                if j >= self.config.contra_lo && j < self.config.contra_hi {
                    contradictions.push(format!(
                        "{} ↔ {}",
                        truncate_chars(&a.representative, 60),
                        truncate_chars(&b.representative, 60)
                    ));
                    if contradictions.len() >= 3 {
                        break;
                    }
                }
            }
            if contradictions.len() >= 3 {
                break;
            }
        }

        let verdict = NoulAnswer::new(share);
        let risk = self.config.risk_tier.decide(share, verdict.needs_review);

        NtFusedAnswer {
            text,
            confidence: share,
            verdict,
            risk,
            clusters,
            minority,
            contradictions,
            used_ids,
        }
    }

    // ── J4：校准分 ──

    pub(crate) fn calibrate(&self, fused: &NtFusedAnswer) -> f64 {
        let total: f64 = fused.clusters.iter().map(|c| c.weight).sum();
        if total <= 0.0 || fused.clusters.is_empty() {
            return 0.0;
        }
        let items: Vec<(f64, bool)> = fused
            .clusters
            .iter()
            .map(|c| ((c.weight / total).clamp(0.0, 1.0), c.member_ids.len() >= 2))
            .collect();
        (1.0 - brier_score(&items)).clamp(0.0, 1.0)
    }

    // ── 4. 后续任务汇总 ──

    pub(crate) fn follow_ups(
        &self,
        goal: &str,
        subtasks: &[NtCrystalSubtask],
        fused: &NtFusedAnswer,
        failed: &[(String, String)],
    ) -> Vec<String> {
        let mut ups = Vec::new();

        // 人工复核优先（J3 RiskTier 裁决）
        if fused.risk == RiskDecision::HumanReview {
            ups.push(format!("人工复核融合结论后推进：{}", truncate_chars(goal, 60)));
        }
        // 失败重试
        for (id, err) in failed {
            ups.push(format!("重试子任务 {}（上次失败：{}）", id, truncate_chars(err, 80)));
        }
        // 弃权确认（J1 abstain）
        for st in subtasks {
            if st.route == NtSubtaskRoute::Skip && is_abstained(&st.route_decision) {
                ups.push(format!("人工确认子任务「{}」是否执行", st.title));
            }
        }
        // 矛盾裁决
        for c in &fused.contradictions {
            ups.push(format!("裁决矛盾：{}", c));
        }
        // 少数派核查
        for m in fused.minority.iter().take(2) {
            ups.push(format!("核查少数派观点：{}", m));
        }
        // 低置信复核
        if fused.confidence < 0.5 && !fused.text.is_empty() {
            ups.push(format!("复核低置信结论：{}", truncate_chars(goal, 60)));
        }

        ups.truncate(self.config.max_follow_ups);
        ups
    }
}

// ============================================================================
// J1：路由 Choice（模块级函数，单独可测）
// ============================================================================

/// 候选 → JEV Choice 路由：概率由晶体命中强度推导，胶着自动弃权。
fn route_choice(c: &Candidate) -> (NtSubtaskRoute, JevDecision) {
    let (pr, pd, ps) = if c.deterministic {
        (0.25, 0.70, 0.05)
    } else if c.score >= 0.5 {
        (0.70, 0.20, 0.10)
    } else {
        (0.55, 0.15, 0.30)
    };
    let sum = pr + pd + ps;
    let mut probs = HashMap::new();
    probs.insert("reasoning".to_string(), pr / sum);
    probs.insert("deterministic".to_string(), pd / sum);
    probs.insert("skip".to_string(), ps / sum);
    let top = if pr >= pd && pr >= ps {
        "reasoning"
    } else if pd >= ps {
        "deterministic"
    } else {
        "skip"
    };
    let decision = abstain_if_contested(JevDecision::Choice(ChoiceAnswer::new(
        top.to_string(),
        probs,
    )));
    let route = if is_abstained(&decision) {
        NtSubtaskRoute::Skip
    } else {
        match &decision {
            JevDecision::Choice(c) if c.choice == "reasoning" => NtSubtaskRoute::Reasoning,
            JevDecision::Choice(c) if c.choice == "deterministic" => {
                NtSubtaskRoute::Deterministic
            }
            _ => NtSubtaskRoute::Skip,
        }
    };
    (route, decision)
}

/// 兜底切分：中英连词。
fn split_goal(goal: &str) -> Vec<String> {
    let delims = [
        "然后", "接着", "和", "与", "并", "、", "；", ";", " and ", " then ", " & ",
    ];
    let mut parts = vec![goal.to_string()];
    for d in delims {
        let mut next = Vec::new();
        for p in parts {
            for s in p.split(d) {
                let t = s.trim();
                if !t.is_empty() {
                    next.push(t.to_string());
                }
            }
        }
        parts = next;
    }
    parts
        .into_iter()
        .filter(|p| p.chars().count() >= 4)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct StubAsk {
        replies: HashMap<String, NtLlmReply>,
        fail_on: Vec<String>,
        calls: Mutex<Vec<String>>,
    }

    impl StubAsk {
        fn new() -> Self {
            Self {
                replies: HashMap::new(),
                fail_on: Vec::new(),
                calls: Mutex::new(Vec::new()),
            }
        }
    }

    impl NtLlmAsk for StubAsk {
        fn ask(&self, prompt: &str) -> Result<NtLlmReply, NtTaskFusionError> {
            self.calls.lock().unwrap().push(prompt.to_string());
            if self.fail_on.iter().any(|f| prompt.contains(f)) {
                return Err(NtTaskFusionError::Llm("boom".to_string()));
            }
            Ok(self
                .replies
                .get(prompt)
                .cloned()
                .unwrap_or(NtLlmReply {
                    text: "stub 默认回答内容足够长可以过长度门槛".to_string(),
                    confidence: 0.6,
                    model: "stub".to_string(),
                }))
        }
    }

    fn seeded_core() -> CrystalCore {
        let mut core = CrystalCore::new("test");
        core.experience.record_success(
            "支付功能接入",
            "调用支付网关SDK并做幂等下单",
            "接入成功",
            true,
            "网关幂等模式",
            "pay",
        );
        core.experience.record_episode(
            "数据库慢查询",
            "加索引",
            "恢复正常",
            "先看执行计划再加索引",
            "db",
            0.9,
        );
        core
    }

    fn engine() -> NtCrystalTaskLoop {
        NtCrystalTaskLoop::new(NtTaskLoopConfig::default())
    }

    #[test]
    fn test_suggest_hits_reusable_solution_as_deterministic() {
        let core = seeded_core();
        let subs = engine().suggest("如何接入支付功能", &core);
        assert!(!subs.is_empty());
        let hit = subs
            .iter()
            .find(|s| s.provenance.starts_with("Solution:"))
            .unwrap();
        assert_eq!(hit.route, NtSubtaskRoute::Deterministic);
        assert!(hit.local_answer.is_some());
    }

    #[test]
    fn test_suggest_fallback_splits_goal() {
        let core = CrystalCore::new("empty");
        let subs = engine().suggest("先设计数据库然后接入支付功能", &core);
        assert!(subs.len() >= 2);
        assert!(subs.iter().all(|s| s.provenance == "fallback:split"));
    }

    #[test]
    fn test_route_choice_contested_abstains_to_skip() {
        // 0.34/0.33/0.33：margin < 0.05 → 弃权 → Skip
        let mut probs = HashMap::new();
        probs.insert("reasoning".to_string(), 0.34);
        probs.insert("deterministic".to_string(), 0.33);
        probs.insert("skip".to_string(), 0.33);
        let d = abstain_if_contested(JevDecision::Choice(ChoiceAnswer::new(
            "reasoning".to_string(),
            probs,
        )));
        assert!(is_abstained(&d));
        assert!(d.needs_review());
    }

    #[test]
    fn test_dispatch_routes_reasoning_to_llm_and_skips_llm_for_memory() {
        let core = seeded_core();
        let eng = engine();
        let subs = eng.suggest("如何接入支付功能", &core);
        assert!(subs.iter().any(|s| s.route == NtSubtaskRoute::Deterministic));
        let llm = StubAsk::new();
        let (answers, failed) = eng.dispatch(&subs, &llm, None);
        assert!(failed.is_empty());
        // Deterministic 不耗 LLM：calls 只来自 Reasoning 子任务
        let reasoning_count = subs
            .iter()
            .filter(|s| s.route == NtSubtaskRoute::Reasoning)
            .count();
        assert_eq!(llm.calls.lock().unwrap().len(), reasoning_count);
        assert!(answers.iter().any(|a| a.model == "crystal-memory"));
    }

    #[test]
    fn test_dispatch_failure_recorded_not_fatal() {
        let subs = vec![NtCrystalSubtask {
            id: "st-1".to_string(),
            title: "t".to_string(),
            question: "一定会炸的问题".to_string(),
            route: NtSubtaskRoute::Reasoning,
            route_decision: JevDecision::Noul(NoulAnswer::new(0.9)),
            confidence: 0.9,
            provenance: "test".to_string(),
            local_answer: None,
        }];
        let mut llm = StubAsk::new();
        llm.fail_on.push("会炸".to_string());
        let (answers, failed) = engine().dispatch(&subs, &llm, None);
        assert!(answers.is_empty());
        assert_eq!(failed.len(), 1);
        assert!(failed[0].1.contains("boom"));
    }

    #[test]
    fn test_fuse_weights_by_confidence() {
        let eng = engine();
        let mk = |id: &str, text: &str, conf: f64| NtScoredAnswer {
            subtask_id: id.to_string(),
            text: text.to_string(),
            confidence: conf,
            verdict: NoulAnswer::new(conf),
            model: "m".to_string(),
        };
        let answers = vec![
            mk("a", "结论是采用方案甲进行系统重构工作", 0.9),
            mk("b", "结论是采用方案甲进行系统重构任务", 0.8),
            mk("c", "结论是全部推倒重写毫无保留余地", 0.3),
        ];
        let fused = eng.fuse(&answers);
        assert!(fused.text.contains("方案甲"));
        assert!(fused.confidence > 0.5);
        assert!(!fused.verdict.needs_review);
        assert_eq!(fused.risk, RiskDecision::Automate);
    }

    #[test]
    fn test_fuse_low_share_flags_review_and_followup() {
        let eng = engine();
        let mk = |id: &str, text: &str| NtScoredAnswer {
            subtask_id: id.to_string(),
            text: text.to_string(),
            confidence: 0.4,
            verdict: NoulAnswer::new(0.4),
            model: "m".to_string(),
        };
        let answers = vec![
            mk("a", "苹果是水果中维生素含量最高的一种"),
            mk("b", "汽车发动机需要定期更换机油保养"),
            mk("c", "数据库索引可以显著提升查询速度"),
        ];
        let fused = eng.fuse(&answers);
        assert!(fused.verdict.needs_review);
        assert_eq!(fused.risk, RiskDecision::HumanReview);
        // 后续任务汇总直接消费该 fused（而非 run 的单答案闭环）
        let ups = eng.follow_ups("互不相关的三件事", &[], &fused, &[]);
        assert!(ups.iter().any(|u| u.contains("人工复核")));
    }

    #[test]
    fn test_run_end_to_end_with_memory_and_llm() {
        let core = seeded_core();
        let llm = StubAsk::new();
        let report = engine().run("如何接入支付功能", &core, &llm);
        assert!(!report.answers.is_empty());
        assert!(!report.fused.text.is_empty());
        assert!(report.decisions.contains_key("fused"));
        assert!((0.0..=1.0).contains(&report.calibration));
        // 后续任务可闭环：follow_ups 能再跑
        for up in &report.follow_ups {
            assert!(!up.is_empty());
        }
    }

    // ── 并行分发测试 ──

    /// 脚本 LLM：按顺序返回预设答案，带延迟模拟真实并行。
    struct ScriptParallelAsk {
        answers: std::sync::Mutex<Vec<String>>,
    }

    impl ScriptParallelAsk {
        fn new(answers: Vec<String>) -> Self {
            Self {
                answers: std::sync::Mutex::new(answers),
            }
        }
    }

    impl NtLlmAsk for ScriptParallelAsk {
        fn ask(
            &self,
            _prompt: &str,
        ) -> Result<NtLlmReply, NtTaskFusionError> {
            let mut pool = self.answers.lock().unwrap();
            let text = pool.remove(0);
            Ok(NtLlmReply {
                text,
                confidence: 0.8,
                model: "parallel-test".to_string(),
            })
        }
    }

    /// 带并发度追踪的脚本 LLM（max_concurrent 测试用）。
    struct ScriptParallelAskWithConcurrency {
        answers: std::sync::Mutex<Vec<String>>,
        current: Arc<std::sync::atomic::AtomicUsize>,
        peak: Arc<std::sync::atomic::AtomicUsize>,
    }

    impl NtLlmAsk for ScriptParallelAskWithConcurrency {
        fn ask(
            &self,
            _prompt: &str,
        ) -> Result<NtLlmReply, NtTaskFusionError> {
            let prev = self.current.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
            // 更新峰值
            self.peak.fetch_max(prev, std::sync::atomic::Ordering::SeqCst);
            // 模拟工作
            std::thread::sleep(std::time::Duration::from_millis(10));
            let mut pool = self.answers.lock().unwrap();
            let text = if pool.is_empty() {
                "默认答案".to_string()
            } else {
                pool.remove(0)
            };
            self.current.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
            Ok(NtLlmReply {
                text,
                confidence: 0.8,
                model: "concurrency-test".to_string(),
            })
        }
    }

    /// 构造 Reasoning 子任务（测试辅助）。
    fn mk_reasoning(id: &str, question: &str) -> NtCrystalSubtask {
        NtCrystalSubtask {
            id: id.to_string(),
            title: format!("title-{id}"),
            question: question.to_string(),
            route: NtSubtaskRoute::Reasoning,
            route_decision: JevDecision::Noul(NoulAnswer::new(0.8)),
            confidence: 0.8,
            provenance: "test".into(),
            local_answer: None,
        }
    }

    /// 并行分发 + SharedMind 基础功能验证。
    #[test]
    fn test_dispatch_parallel_basic() {
        use crate::neotrix::nt_crystal_core::SharedMind;
        let eng = engine();
        let subtasks = vec![
            NtCrystalSubtask {
                id: "st-1".into(),
                title: "子任务一".into(),
                question: "什么是幂等性".into(),
                route: NtSubtaskRoute::Reasoning,
                route_decision: JevDecision::Noul(NoulAnswer::new(0.8)),
                confidence: 0.8,
                provenance: "test".into(),
                local_answer: None,
            },
            NtCrystalSubtask {
                id: "st-2".into(),
                title: "子任务二".into(),
                question: "分布式系统一致性".into(),
                route: NtSubtaskRoute::Reasoning,
                route_decision: JevDecision::Noul(NoulAnswer::new(0.7)),
                confidence: 0.7,
                provenance: "test".into(),
                local_answer: None,
            },
        ];
        let llm = ScriptParallelAsk::new(vec![
            "幂等性是多次调用结果相同".into(),
            "分布式一致性保证数据同步".into(),
        ]);
        let shared = SharedMind::new();
        let (answers, failed) = eng.dispatch_parallel(&subtasks, &llm, None, &shared);
        assert_eq!(answers.len(), 2);
        assert!(failed.is_empty());
        // SharedMind 应有 2 条发现
        assert_eq!(shared.snapshot().len(), 2);
    }

    /// 跳过与重叠：子任务二的问题被子任务一的答案覆盖 → 跳过。
    #[test]
    fn test_dispatch_parallel_overlap_skips() {
        use crate::neotrix::nt_crystal_core::SharedMind;
        // 阈值 0.05：中文 Jaccard 天然低，0.05 即可检测同领域重叠
        let eng = NtCrystalTaskLoop::new(NtTaskLoopConfig {
            parallel: true,
            overlap_threshold: 0.05,
            ..Default::default()
        });
        let subtasks = vec![
            NtCrystalSubtask {
                id: "st-1".into(),
                title: "幂等性定义".into(),
                question: "什么是幂等性".into(),
                route: NtSubtaskRoute::Reasoning,
                route_decision: JevDecision::Noul(NoulAnswer::new(0.9)),
                confidence: 0.9,
                provenance: "test".into(),
                local_answer: None,
            },
            NtCrystalSubtask {
                id: "st-2".into(),
                title: "幂等性应用".into(),
                question: "幂等性在 HTTP 中的应用".into(),
                route: NtSubtaskRoute::Reasoning,
                route_decision: JevDecision::Noul(NoulAnswer::new(0.8)),
                confidence: 0.8,
                provenance: "test".into(),
                local_answer: None,
            },
        ];
        let llm = ScriptParallelAsk::new(vec![
            // 只需要 st-1 的答案，st-2 应被跳过
            "幂等性是指操作可重复执行".into(),
        ]);
        let shared = SharedMind::new();
        // 阈值 0.05：中文 Jaccard 天然低，0.05 即可检测同领域重叠
        let (answers, failed) = eng.dispatch_parallel(&subtasks, &llm, None, &shared);
        // 只有 st-1 被执行
        assert_eq!(answers.len(), 1);
        assert_eq!(answers[0].subtask_id, "st-1");
        assert!(failed.is_empty());
    }

    /// Deterministic + Reasoning 混合并行。
    #[test]
    fn test_dispatch_parallel_mixed() {
        use crate::neotrix::nt_crystal_core::SharedMind;
        let eng = engine();
        let subtasks = vec![
            NtCrystalSubtask {
                id: "det-1".into(),
                title: "晶体经验".into(),
                question: "基于经验".into(),
                route: NtSubtaskRoute::Deterministic,
                route_decision: JevDecision::Noul(NoulAnswer::new(0.9)),
                confidence: 0.9,
                provenance: "Solution:1".into(),
                local_answer: Some("经验方案：按步骤执行".into()),
            },
            NtCrystalSubtask {
                id: "reason-1".into(),
                title: "推理分析".into(),
                question: "分析最佳方案".into(),
                route: NtSubtaskRoute::Reasoning,
                route_decision: JevDecision::Noul(NoulAnswer::new(0.8)),
                confidence: 0.8,
                provenance: "Theory:1".into(),
                local_answer: None,
            },
        ];
        let llm = ScriptParallelAsk::new(vec!["最佳方案是方案甲".into()]);
        let shared = SharedMind::new();
        let (answers, failed) = eng.dispatch_parallel(&subtasks, &llm, None, &shared);
        assert_eq!(answers.len(), 2);
        assert!(failed.is_empty());
        // 确定性答案应来自 crystal-memory
        let det = answers.iter().find(|a| a.model == "crystal-memory");
        assert!(det.is_some());
        // 推理答案应来自 parallel-test
        let reason = answers.iter().find(|a| a.model == "parallel-test");
        assert!(reason.is_some());
        // SharedMind 应有 2 条发现
        assert_eq!(shared.snapshot().len(), 2);
    }

    /// 并行闭环：run_with_sink_parallel 端到端。
    #[test]
    fn test_run_with_sink_parallel_end_to_end() {
        let core = seeded_core();
        let llm = ScriptParallelAsk::new(vec![
            "接入支付需要 API 网关".into(),
            "支付安全需要加密认证".into(),
        ]);
        let report = engine().run_with_sink_parallel(
            "如何安全接入支付功能",
            &core,
            &llm,
            None,
        );
        assert!(!report.answers.is_empty());
        assert!(!report.fused.text.is_empty());
        assert!(report.decisions.contains_key("fused"));
    }

    /// panic 安全：LLM 线程 panic 不炸整轮，转为 failed 记录。
    #[test]
    fn test_dispatch_parallel_panic_safety() {
        use crate::neotrix::nt_crystal_core::SharedMind;
        struct PanicAsk;
        impl NtLlmAsk for PanicAsk {
            fn ask(&self, _: &str) -> Result<NtLlmReply, NtTaskFusionError> {
                panic!("boom")
            }
        }
        let eng = NtCrystalTaskLoop::new(NtTaskLoopConfig {
            parallel: true,
            overlap_threshold: 0.0, // 禁用重叠检测
            ..Default::default()
        });
        let subtasks = vec![
            NtCrystalSubtask {
                id: "st-1".into(),
                title: "会炸的任务".into(),
                question: "必炸问题".into(),
                route: NtSubtaskRoute::Reasoning,
                route_decision: JevDecision::Noul(NoulAnswer::new(0.8)),
                confidence: 0.8,
                provenance: "test".into(),
                local_answer: None,
            },
            NtCrystalSubtask {
                id: "st-2".into(),
                title: "正常任务".into(),
                question: "正常问题".into(),
                route: NtSubtaskRoute::Reasoning,
                route_decision: JevDecision::Noul(NoulAnswer::new(0.7)),
                confidence: 0.7,
                provenance: "test".into(),
                local_answer: None,
            },
        ];
        let shared = SharedMind::new();
        let (_answers, failed) = eng.dispatch_parallel(&subtasks, &PanicAsk, None, &shared);
        // panic 的子任务进 failed，另一个也被 panic 炸了（scope 传播）
        // 但不会导致进程 abort
        assert!(failed.len() >= 1);
        assert!(failed.iter().any(|(_, e)| e.contains("panic") || e.contains("boom")));
    }

    /// parallel=false 走串行路径。
    #[test]
    fn test_run_sequential_path() {
        let core = seeded_core();
        let llm = StubAsk::new();
        let eng = NtCrystalTaskLoop::new(NtTaskLoopConfig {
            parallel: false,
            ..Default::default()
        });
        let report = eng.run("如何接入支付功能", &core, &llm);
        assert!(!report.answers.is_empty());
        assert!(!report.fused.text.is_empty());
    }

    /// max_concurrent=1 串行执行（并行但实际单线程）。
    #[test]
    fn test_dispatch_parallel_max_concurrent_1() {
        use crate::neotrix::nt_crystal_core::SharedMind;
        use std::sync::atomic::{AtomicUsize, Ordering};
        let eng = NtCrystalTaskLoop::new(NtTaskLoopConfig {
            parallel: true,
            max_concurrent: 1,
            overlap_threshold: 0.0,
            ..Default::default()
        });
        let peak = Arc::new(AtomicUsize::new(0));
        let current = Arc::new(AtomicUsize::new(0));
        let peak_clone = peak.clone();
        let current_clone = current.clone();
        let llm = ScriptParallelAskWithConcurrency {
            answers: std::sync::Mutex::new(vec![
                "答案甲".into(),
                "答案乙".into(),
                "答案丙".into(),
            ]),
            current: current_clone,
            peak: peak_clone,
        };
        let subtasks = vec![
            mk_reasoning("s1", "问题一"),
            mk_reasoning("s2", "问题二"),
            mk_reasoning("s3", "问题三"),
        ];
        let shared = SharedMind::new();
        let (answers, failed) = eng.dispatch_parallel(&subtasks, &llm, None, &shared);
        assert_eq!(answers.len(), 3);
        assert!(failed.is_empty());
        // max_concurrent=1 → 峰值并行度 ≤ 1
        assert!(peak.load(Ordering::SeqCst) <= 1);
    }
}
