//! # nt_crystal_dialogue — 对话窗口 + 内需循环
//!
//! 晶体任务闭环开了一个"和人类对话的窗口"，本模块把它做完整，
//! 并接成 NeoTrix 的整套内需循环：
//!
//! ```text
//! NtTaskLoopReport ──▶ NtDialogueWindow::demands_from（内需 → 需求单）
//!                             │
//!                             ▼
//!                      render（窗口文本摆给人看）
//!                             │
//!                             ▼
//!                      NtHumanChannel::prompt（人回话：批准/给答案/沉默）
//!                             │
//!                             ▼
//!                      回灌：人的文字 → 高权重 human 答案参与重熔；
//!                            批准+空文 → 该需求单直接关闭
//!                             │
//!                   ┌─────────┴──────────┐
//!                   ▼                    ▼
//!             NtInnerLoop::drive 多轮收敛：
//!             无内需 → Converged；人不回 → Stalled（挂起不断轮）；超轮 → MaxRounds
//! ```
//!
//! ## 内需来源（全部由上一轮 report 派生，见 `classify_demand` 前缀表）
//! - 人工复核 / 低置信复核 / 少数派核查 / 矛盾裁决 / 失败重试 / 跳过确认
//!
//! ## 需求单 ID 稳定性
//! ID = kind + 文本哈希，多轮间同一内需 ID 不变，人"已阅批准"才能精确关闭它。
//!
//! # Safety
//! - 纯内存 + trait 注入的人类通道，无 IO、无锁、无 unsafe (R-P1)。
//! - 生产代码无 `unwrap/expect/panic`。

use super::nt_crystal_task_fusion::{
    NtCrystalTaskLoop, NtProgressSink, NtScoredAnswer, NtTaskLoopConfig, NtTaskLoopReport,
};
use super::CrystalCore;
use crate::l5_cognition::nt_crystal_core::NtLlmAsk;
use crate::l5_cognition::nt_jev::NoulAnswer;
use std::collections::HashSet;
use std::hash::{DefaultHasher, Hash, Hasher};

// ============================================================================
// 需求单
// ============================================================================

/// 内需种类。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NtDemandKind {
    /// 人工复核融合结论
    ReviewFusion,
    /// 复核低置信结论
    RecheckLowConf,
    /// 核查少数派观点
    RecheckMinority,
    /// 裁决矛盾
    Adjudicate,
    /// 重试失败子任务
    RetryFailed,
    /// 确认被跳过的子任务
    ConfirmSkipped,
    /// 未识别（兜底也上窗口，不丢内需）
    Other,
}

impl NtDemandKind {
    pub fn tag(&self) -> &'static str {
        match self {
            Self::ReviewFusion => "review",
            Self::RecheckLowConf => "recheck",
            Self::RecheckMinority => "minority",
            Self::Adjudicate => "adjudicate",
            Self::RetryFailed => "retry",
            Self::ConfirmSkipped => "confirm",
            Self::Other => "other",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::ReviewFusion => "人工复核",
            Self::RecheckLowConf => "低置信复核",
            Self::RecheckMinority => "少数派核查",
            Self::Adjudicate => "矛盾裁决",
            Self::RetryFailed => "失败重试",
            Self::ConfirmSkipped => "跳过确认",
            Self::Other => "其他内需",
        }
    }
}

/// 前缀 → 内需种类（与 `nt_crystal_task_fusion::follow_ups` 生成前缀严格对齐）。
fn classify_demand(text: &str) -> NtDemandKind {
    if text.starts_with("人工复核融合结论后推进：") {
        NtDemandKind::ReviewFusion
    } else if text.starts_with("复核低置信结论：") {
        NtDemandKind::RecheckLowConf
    } else if text.starts_with("核查少数派观点：") {
        NtDemandKind::RecheckMinority
    } else if text.starts_with("裁决矛盾：") {
        NtDemandKind::Adjudicate
    } else if text.starts_with("重试子任务 ") {
        NtDemandKind::RetryFailed
    } else if text.starts_with("人工确认子任务") {
        NtDemandKind::ConfirmSkipped
    } else {
        NtDemandKind::Other
    }
}

/// 窗口需求单。
#[derive(Debug, Clone)]
pub struct NtDemand {
    pub id: String,
    pub kind: NtDemandKind,
    pub text: String,
}

fn demand_id(kind: NtDemandKind, text: &str) -> String {
    let mut h = DefaultHasher::new();
    text.hash(&mut h);
    format!("{}-{:08x}", kind.tag(), h.finish() & 0xffff_ffff)
}

// ============================================================================
// 人类通道
// ============================================================================

/// 人的一次回复。
#[derive(Debug, Clone)]
pub struct NtHumanReply {
    /// 指向的需求单 ID（None = 对整轮的总体意见）。
    pub demand_id: Option<String>,
    /// 回复正文（空 = 仅表态不给新信息）。
    pub text: String,
    /// 是否批准/认可。
    pub approved: bool,
}

/// 人类通道抽象：同步 trait，UI/CLI/测试各自实现。
/// 返回空 Vec = 人沉默 → 循环挂起（Stalled），不断轮、不瞎猜。
pub trait NtHumanChannel: Send + Sync {
    fn prompt(&self, window: &str, demands: &[NtDemand]) -> Vec<NtHumanReply>;
}

// ============================================================================
// 对话窗口（无状态函数集）
// ============================================================================

/// 对话窗口：report → 需求单 → 窗口文本。
pub struct NtDialogueWindow;

impl NtDialogueWindow {
    /// 从报告派生本轮内需，跳过已关闭的需求单 ID。
    pub fn demands_from(
        report: &NtTaskLoopReport,
        resolved: &HashSet<String>,
    ) -> Vec<NtDemand> {
        let mut out = Vec::new();
        for text in &report.follow_ups {
            let kind = classify_demand(text);
            let id = demand_id(kind, text);
            if resolved.contains(&id) {
                continue;
            }
            out.push(NtDemand {
                id,
                kind,
                text: text.clone(),
            });
        }
        out
    }

    /// 渲染窗口文本：结论 + 校准 + 需求单列表。
    pub fn render(report: &NtTaskLoopReport, demands: &[NtDemand]) -> String {
        let mut w = String::new();
        w.push_str("══ 对话窗口 · 晶体任务闭环 ══\n");
        w.push_str(&format!("目标：{}\n", report.goal));
        if report.fused.text.is_empty() {
            w.push_str("融合结论：（暂无结论）\n");
        } else {
            w.push_str(&format!(
                "融合结论（置信 {:.2} · 校准 {:.2}）：{}\n",
                report.fused.confidence, report.calibration, report.fused.text
            ));
        }
        if demands.is_empty() {
            w.push_str("内需：无，本轮可关闭。\n");
        } else {
            w.push_str(&format!("内需：{} 项\n", demands.len()));
            for d in demands {
                // ⚠️ 这里的 `[{}]` 是**显示层加的装饰**（需求单真实 id 是裸的）。
                //
                // 2026-10-08 实测事故：这个括号曾让 `ntcode --line` **永远收敛不了** ——
                // 用户**照抄屏幕上这串**（`ok [review-fc47da46]`），
                // 而 `l1_action::nt_stdin_human::normalize_demand_id` 当时**不剥**括号
                // ⇒ `demand_id` 存成 `[review-…]`、与真实裸 id 比对不上 ⇒ 批准静默失效
                // ⇒ 同一道复核门在下一轮原样重上（实测 `终态：Stalled`、exit 2）。
                // 现在解析层会剥成对外层括号（`normalize_demand_id`），两侧归一。
                //
                // ⛔ **改这里之前先读那条纪律**：若将来决定不再显示方括号，
                // 解析层的容忍**可以保留**（两种输入都成立）；
                // 但若新增别的装饰（`#` 前缀、`>` 引用号、颜色转义…），
                // **必须同时**告诉解析层，否则同一个 bug 会以另一种样子回来。
                // 判据与全部实测见 `docs/architecture/LESSONS-2026-10-08-cli-plugin-and-shared-index.md` L1。
                w.push_str(&format!("  [{}] {}：{}\n", d.id, d.kind.label(), d.text));
            }
            w.push_str("请回复：批准（可附demand id）/ 给出答案 / 沉默=挂起。\n");
        }
        w
    }
}

// ============================================================================
// 内需循环驱动器
// ============================================================================

/// 循环终态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NtLoopStatus {
    /// 内需清空，收敛。
    Converged,
    /// 人沉默，挂起（不断轮、不瞎猜，等人回来）。
    Stalled,
    /// 达到最大轮次仍有内需。
    MaxRounds,
}

/// 内需循环产出。
#[derive(Debug, Clone)]
pub struct NtInnerLoopOutcome {
    pub status: NtLoopStatus,
    pub report: NtTaskLoopReport,
    /// 人机对话实录（窗口原文 + 人 reply 摘要）。
    pub transcript: Vec<String>,
    pub rounds: usize,
}

/// 内需循环驱动器：多轮 run → 上窗 → 回灌 → 重熔，直到收敛/挂起/超轮。
pub struct NtInnerLoop {
    config: NtTaskLoopConfig,
    max_rounds: usize,
}

impl NtInnerLoop {
    pub fn new(config: NtTaskLoopConfig, max_rounds: usize) -> Self {
        Self {
            config,
            max_rounds: max_rounds.max(1),
        }
    }

    pub fn drive(
        &self,
        goal: &str,
        core: &mut CrystalCore,
        llm: &dyn NtLlmAsk,
        human: &dyn NtHumanChannel,
    ) -> NtInnerLoopOutcome {
        self.drive_with_sink(goal, core, llm, human, None)
    }

    /// 带进度接收器的驱动（TUI 工作相实时渲染 + Esc 取消走这里）。
    pub fn drive_with_sink(
        &self,
        goal: &str,
        core: &mut CrystalCore,
        llm: &dyn NtLlmAsk,
        human: &dyn NtHumanChannel,
        sink: Option<&dyn NtProgressSink>,
    ) -> NtInnerLoopOutcome {
        let engine = NtCrystalTaskLoop::new(self.config);
        let mut extra: Vec<NtScoredAnswer> = Vec::new();
        let mut resolved: HashSet<String> = HashSet::new();
        let mut transcript: Vec<String> = Vec::new();

        for round in 1..=self.max_rounds {
            let mut report = engine.run_with_sink_parallel(goal, core, llm, sink);
            if !extra.is_empty() {
                report.answers.extend(extra.iter().cloned());
                report.fused = engine.fuse(&report.answers);
                report.calibration = engine.calibrate(&report.fused);
                report.follow_ups =
                    engine.follow_ups(goal, &report.subtasks, &report.fused, &report.failed);
                report
                    .decisions
                    .insert("fused".to_string(), crate::l5_cognition::nt_jev::JevDecision::Noul(report.fused.verdict.clone()));
            }

            let demands = NtDialogueWindow::demands_from(&report, &resolved);
            if demands.is_empty() {
                transcript.push(format!("[round{round}-system] 内需清空，收敛。"));
                Self::record_outcome(core, goal, &report, NtLoopStatus::Converged, &transcript);
                return NtInnerLoopOutcome {
                    status: NtLoopStatus::Converged,
                    report,
                    transcript,
                    rounds: round,
                };
            }

            let window = NtDialogueWindow::render(&report, &demands);
            transcript.push(format!(
                "[round{round}-system] 内需{}项已上窗。",
                demands.len()
            ));
            let replies = human.prompt(&window, &demands);
            if replies.is_empty() {
                transcript.push(format!("[round{round}-system] 人沉默，挂起。"));
                Self::record_outcome(core, goal, &report, NtLoopStatus::Stalled, &transcript);
                return NtInnerLoopOutcome {
                    status: NtLoopStatus::Stalled,
                    report,
                    transcript,
                    rounds: round,
                };
            }

            for r in &replies {
                transcript.push(format!(
                    "[round{round}-human] {}：{}",
                    r.demand_id.as_deref().unwrap_or("总体"),
                    if r.text.trim().is_empty() {
                        if r.approved {
                            "批准"
                        } else {
                            "驳回（无文字）"
                        }
                    } else {
                        r.text.trim()
                    }
                ));
                if r.approved {
                    if let Some(id) = &r.demand_id {
                        resolved.insert(id.clone());
                    }
                }
                if !r.text.trim().is_empty() {
                    // 人的文字 = 最高权重选民，直接参与重熔
                    let conf = if r.approved { 0.95 } else { 0.6 };
                    extra.push(NtScoredAnswer {
                        subtask_id: format!(
                            "human-r{round}-{}",
                            r.demand_id.as_deref().unwrap_or("general")
                        ),
                        text: r.text.trim().to_string(),
                        confidence: conf,
                        verdict: NoulAnswer::new(conf),
                        model: "human".to_string(),
                    });
                }
            }
        }

        // 超轮：返回最后一轮现场（内需仍在， transcript 留痕）
        let mut report = engine.run(goal, core, llm);
        if !extra.is_empty() {
            report.answers.extend(extra.iter().cloned());
            report.fused = engine.fuse(&report.answers);
            report.calibration = engine.calibrate(&report.fused);
            report.follow_ups =
                engine.follow_ups(goal, &report.subtasks, &report.fused, &report.failed);
        }
        transcript.push(format!(
            "[system] 达到最大轮次{}，仍有内需未清。",
            self.max_rounds
        ));
        Self::record_outcome(core, goal, &report, NtLoopStatus::MaxRounds, &transcript);
        NtInnerLoopOutcome {
            status: NtLoopStatus::MaxRounds,
            report,
            transcript,
            rounds: self.max_rounds,
        }
    }

    /// 终态写回晶体记忆（只写内存，落盘由调用方 `persist` 显式触发，
    /// 测试不碰磁盘）。
    /// - 任何终态都记 Episode（情境=目标，行动=实录尾，复盘=少数派/矛盾）。
    /// - Converged 且有结论再记一条可复用 Solution（置信≥0.7 才算可复用）。
    fn record_outcome(
        core: &mut CrystalCore,
        goal: &str,
        report: &super::nt_crystal_task_fusion::NtTaskLoopReport,
        status: NtLoopStatus,
        transcript: &[String],
    ) {
        let tail: Vec<&str> = transcript
            .iter()
            .rev()
            .take(3)
            .map(|s| s.as_str())
            .collect();
        let mut tail_rev = tail.clone();
        tail_rev.reverse();
        let mut reflection = report.fused.minority.join("；");
        if !report.fused.contradictions.is_empty() {
            if !reflection.is_empty() {
                reflection.push_str("；");
            }
            reflection.push_str(&report.fused.contradictions.join("；"));
        }
        if reflection.is_empty() {
            reflection = "本轮无内需残留".to_string();
        }
        core.experience.record_episode(
            trunc(goal, 200),
            trunc(&tail_rev.join("\n"), 300),
            format!("{status:?}（融合置信 {:.2}）", report.fused.confidence),
            trunc(&reflection, 300),
            "dialogue",
            report.fused.confidence.clamp(0.0, 1.0),
        );
        if status == NtLoopStatus::Converged && !report.fused.text.is_empty() {
            core.experience.record_success(
                trunc(goal, 200),
                trunc(&report.fused.text, 500),
                format!("calibration {:.2}", report.calibration),
                report.fused.confidence >= 0.7,
                "crystal-loop-fusion",
                "dialogue",
            );
        }
    }

    /// 落盘晶体记忆（bin 显式调用，失败如实返回）。
    ///
    /// 并发安全（尽力而为）：共享 `crystal.json` 会被多会话同时写，
    /// 直接全量覆盖必丢别人的更新。这里加文件锁 + 读-合并-写：
    /// 只把本轮新增的 Episode/Solution/Lesson 按 id 并入，别人先写的都保留。
    /// 注意：锁只能约束同样加锁的写入方；窗口期已压到毫秒级，残余竞态如实接受。
    pub fn persist(core: &CrystalCore) -> Result<(), String> {
        Self::persist_to(core, &super::crystal_root().join("crystal.json"))
    }

    pub(crate) fn persist_to(core: &CrystalCore, path: &std::path::Path) -> Result<(), String> {
        use fs2::FileExt;
        use std::io::{Read, Seek, SeekFrom, Write};
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| format!("mkdir failed: {e}"))?;
        }
        let mut f = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .map_err(|e| format!("open failed: {e}"))?;
        f.lock_exclusive()
            .map_err(|e| format!("lock failed: {e}"))?;
        let mut buf = String::new();
        f.read_to_string(&mut buf)
            .map_err(|e| format!("read failed: {e}"))?;
        let mut disk: CrystalCore = if buf.trim().is_empty() {
            core.clone()
        } else {
            serde_json::from_str(&buf).map_err(|e| format!("parse failed: {e}"))?
        };
        for ep in &core.experience.episodes {
            if !disk.experience.episodes.iter().any(|e| e.id == ep.id) {
                disk.experience.episodes.push(ep.clone());
            }
        }
        for s in &core.experience.successes {
            if !disk.experience.successes.iter().any(|e| e.id == s.id) {
                disk.experience.successes.push(s.clone());
            }
        }
        for l in &core.experience.failures {
            if !disk.experience.failures.iter().any(|e| e.id == l.id) {
                disk.experience.failures.push(l.clone());
            }
        }
        let data =
            serde_json::to_string_pretty(&disk).map_err(|e| format!("serialize failed: {e}"))?;
        f.set_len(0).map_err(|e| format!("truncate failed: {e}"))?;
        f.seek(SeekFrom::Start(0))
            .map_err(|e| format!("seek failed: {e}"))?;
        f.write_all(data.as_bytes())
            .map_err(|e| format!("write failed: {e}"))?;
        f.sync_all().map_err(|e| format!("sync failed: {e}"))?;
        Ok(())
    }
}

/// 截断（字符级，超长加省略号）。
fn trunc(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        return s.to_string();
    }
    let t: String = s.chars().take(n).collect();
    format!("{t}…")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l5_cognition::nt_crystal_core::NtTaskFusionError;
    use std::sync::Mutex;

    struct StubLlm {
        fail_on: Vec<String>,
    }

    impl NtLlmAsk for StubLlm {
        fn ask(&self, prompt: &str) -> Result<super::super::nt_crystal_task_fusion::NtLlmReply, NtTaskFusionError> {
            if self.fail_on.iter().any(|f| prompt.contains(f)) {
                return Err(NtTaskFusionError::Llm("boom".to_string()));
            }
            Ok(super::super::nt_crystal_task_fusion::NtLlmReply {
                text: "stub 完整回答内容足够长可以过门槛".to_string(),
                confidence: 0.6,
                model: "stub".to_string(),
            })
        }
    }

    struct ScriptHuman {
        scripts: Mutex<Vec<Vec<NtHumanReply>>>,
    }

    impl ScriptHuman {
        fn new(scripts: Vec<Vec<NtHumanReply>>) -> Self {
            Self {
                scripts: Mutex::new(scripts),
            }
        }
    }

    impl NtHumanChannel for ScriptHuman {
        fn prompt(&self, _window: &str, _demands: &[NtDemand]) -> Vec<NtHumanReply> {
            self.scripts.lock().unwrap().pop().unwrap_or_default()
        }
    }

    fn config() -> NtTaskLoopConfig {
        NtTaskLoopConfig::default()
    }

    #[test]
    fn test_classify_demand_prefixes() {
        assert_eq!(
            classify_demand("人工复核融合结论后推进：X"),
            NtDemandKind::ReviewFusion
        );
        assert_eq!(
            classify_demand("重试子任务 st-1（上次失败：boom）"),
            NtDemandKind::RetryFailed
        );
        assert_eq!(
            classify_demand("裁决矛盾：A ↔ B"),
            NtDemandKind::Adjudicate
        );
        assert_eq!(
            classify_demand("核查少数派观点：xxx"),
            NtDemandKind::RecheckMinority
        );
        assert_eq!(
            classify_demand("人工确认子任务「标题」是否执行"),
            NtDemandKind::ConfirmSkipped
        );
        assert_eq!(
            classify_demand("复核低置信结论：G"),
            NtDemandKind::RecheckLowConf
        );
        assert_eq!(classify_demand("来历不明的内需"), NtDemandKind::Other);
    }

    #[test]
    fn test_demand_ids_stable_and_resolved_filtered() {
        let core = CrystalCore::new("e");
        let llm = StubLlm { fail_on: vec!["步骤".to_string(), "支付".to_string(), "数据库".to_string(), "接入".to_string()] };
        let engine = NtCrystalTaskLoop::new(config());
        // 全部失败 → 必有 RetryFailed 内需
        let report = engine.run("先设计数据库然后接入支付功能", &core, &llm);
        assert!(!report.failed.is_empty());
        let a = NtDialogueWindow::demands_from(&report, &HashSet::new());
        let b = NtDialogueWindow::demands_from(&report, &HashSet::new());
        assert_eq!(a.len(), b.len());
        assert!(a.iter().zip(b.iter()).all(|(x, y)| x.id == y.id));
        assert!(a.iter().any(|d| d.kind == NtDemandKind::RetryFailed));
        let resolved: HashSet<String> = a.iter().map(|d| d.id.clone()).collect();
        assert!(NtDialogueWindow::demands_from(&report, &resolved).is_empty());
    }

    #[test]
    fn test_drive_converges_when_human_resolves_retry() {
        let mut core = CrystalCore::new("e");
        let llm = StubLlm { fail_on: vec!["炸".to_string()] };
        // 先跑一轮拿到 retry 需求单 id（确定性：同输入同 id）
        let engine = NtCrystalTaskLoop::new(config());
        let pre = engine.run("一定会炸的问题", &core, &llm);
        let ds = NtDialogueWindow::demands_from(&pre, &HashSet::new());
        let retry_id = ds
            .iter()
            .find(|d| d.kind == NtDemandKind::RetryFailed)
            .unwrap()
            .id
            .clone();
        let human = ScriptHuman::new(vec![vec![NtHumanReply {
            demand_id: Some(retry_id),
            text: "已手动执行完毕，结果符合预期要求，可以归档".to_string(),
            approved: true,
        }]]);
        let outcome = NtInnerLoop::new(config(), 3).drive("一定会炸的问题", &mut core, &llm, &human);
        assert_eq!(outcome.status, NtLoopStatus::Converged);
        assert_eq!(outcome.rounds, 2);
        assert!(!outcome.transcript.is_empty());
        // 写回：Converged 有结论 → Episode + Solution 各一条
        assert_eq!(core.experience.episodes.len(), 1);
        assert_eq!(core.experience.successes.len(), 1);
    }

    #[test]
    fn test_drive_stalls_on_silence() {
        let mut core = CrystalCore::new("e");
        let llm = StubLlm { fail_on: vec!["炸".to_string()] };
        let human = ScriptHuman::new(vec![]);
        let outcome = NtInnerLoop::new(config(), 3).drive("一定会炸的问题", &mut core, &llm, &human);
        assert_eq!(outcome.status, NtLoopStatus::Stalled);
        assert_eq!(outcome.rounds, 1);
        // 写回：Stalled 只记 Episode，不记 Solution
        assert_eq!(core.experience.episodes.len(), 1);
        assert!(core.experience.successes.is_empty());
    }

    #[test]
    fn test_drive_max_rounds_when_never_resolved() {
        let mut core = CrystalCore::new("e");
        let llm = StubLlm { fail_on: vec!["炸".to_string()] };
        // 每轮都给新文字但从不带 demand_id → 需求单永不清 → 打满轮次
        let human = ScriptHuman::new(vec![
            vec![NtHumanReply { demand_id: None, text: "第一轮补充说明文字足够长".to_string(), approved: false }],
            vec![NtHumanReply { demand_id: None, text: "第二轮补充说明文字足够长".to_string(), approved: false }],
            vec![NtHumanReply { demand_id: None, text: "第三轮补充说明文字足够长".to_string(), approved: false }],
        ]);
        // 注意 ScriptHuman 用 pop 取脚本：逆序压入
        let outcome = NtInnerLoop::new(config(), 3).drive("一定会炸的问题", &mut core, &llm, &human);
        assert_eq!(outcome.status, NtLoopStatus::MaxRounds);
        assert_eq!(outcome.rounds, 3);
    }

    #[test]
    fn test_render_window_shows_fusion_and_demands() {
        let core = CrystalCore::new("e");
        let llm = StubLlm { fail_on: vec![] };
        let engine = NtCrystalTaskLoop::new(config());
        let report = engine.run("先设计数据库然后接入支付功能", &core, &llm);
        let demands = NtDialogueWindow::demands_from(&report, &HashSet::new());
        let w = NtDialogueWindow::render(&report, &demands);
        assert!(w.contains("对话窗口"));
        assert!(w.contains("目标："));
    }

    #[test]
    fn test_persist_merges_without_clobbering() {
        // 磁盘已有别人的条目 → 我方 persist 只并入新增，谁也不丢谁
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("crystal.json");
        let mut other = CrystalCore::new("other");
        other
            .experience
            .record_episode("ctx", "act", "res", "ref", "d", 0.8);
        NtInnerLoop::persist_to(&other, &path).unwrap();
        let mut mine = CrystalCore::new("mine");
        mine.experience.record_success("prob", "appr", "res", true, "pat", "d");
        NtInnerLoop::persist_to(&mine, &path).unwrap();
        let raw = std::fs::read_to_string(&path).unwrap();
        let back: CrystalCore = serde_json::from_str(&raw).unwrap();
        assert_eq!(back.experience.episodes.len(), 1);
        assert_eq!(back.experience.successes.len(), 1);
        // 幂等：重复 persist 不产生副本
        NtInnerLoop::persist_to(&mine, &path).unwrap();
        let raw = std::fs::read_to_string(&path).unwrap();
        let back: CrystalCore = serde_json::from_str(&raw).unwrap();
        assert_eq!(back.experience.episodes.len(), 1);
        assert_eq!(back.experience.successes.len(), 1);
    }
}
