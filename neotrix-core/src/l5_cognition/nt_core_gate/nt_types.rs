//! nt_types — 门控基础类型: 裁决/门级/动作分级/工具规约/去偏配置/组合裁决.
//! 从 `nt_core_gate/mod.rs` 纯搬移, 行为零变更.

use serde::{Deserialize, Serialize};

use super::nt_guardrail::GuardrailReport;
use super::nt_judge::JudgeInput;
use super::nt_panel_debate::JudgePanel;
use super::nt_tool_registry::ToolRegistry;

/// 门控强度 — 由爆炸半径分级驱动 (cost of the gate ≪ cost of the bad merge)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GateLevel {
    /// 轻门: 确定性检查 + 单判, 不阻塞 (docs/draft/低风险)
    Light,
    /// 评审组: 多家族法官 + 一致率 + 黄金集校准 (agent 改动 → trunk / 安全路径)
    Panel,
    /// 人工: 强制人类审批 (不可逆/扩权动作, 确定性检查不能越过)
    Human,
}

/// 门控裁决。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Verdict {
    Pass,
    /// 高分歧或证据不足 — 转人工路由, 不自动放行
    Review,
    /// 机械检查或低分 — 阻断合并
    Block,
}

/// 护栏动作 — eval 结果对运行轨迹的引导。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GuardAction {
    Allow,
    /// 拒绝低 grounding / schema 失败 — 硬阻断
    Reject,
    /// 隔离幻觉声明 — 扣留待人工复核, 不进生产
    Quarantine,
    /// 升级 — 需要更高权限/人工
    Escalate,
}

/// 工具可逆性 — 爆炸半径的原子度量 (STRATUS TNR: 每动作配 undo 算子)。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolReversibility {
    /// 无副作用 (SELECT / GET / 文件读)
    ReadOnly,
    /// 可逆写 (有 undo 算子 + 前置快照)
    Reversible,
    /// 可补偿 (无完美 undo, 但有补偿动作; saga)
    Compensable,
    /// 不可逆 (发送邮件 / 删除生产数据 / 发布) — 强制人工
    Irreversible,
}

/// 工具规约 — 注册表条目, 构建期事实而非运行期猜测 (TianPan: risk class as
/// versioned tool attribute; 同一注册表同时发 tool spec 与 gate config)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub reversibility: ToolReversibility,
    /// 可选 undo 算子名 (Reversible 必须注册)。
    pub undo: Option<String>,
    /// 扩权型 (授权/改权限/换凭据) — 未来的动作空间由它决定, 必须人工。
    pub authority_modifying: bool,
}

impl ToolSpec {
    pub fn read_only(name: &str) -> Self {
        Self {
            name: name.to_string(),
            reversibility: ToolReversibility::ReadOnly,
            undo: None,
            authority_modifying: false,
        }
    }

    pub fn reversible(name: &str, undo: &str) -> Self {
        Self {
            name: name.to_string(),
            reversibility: ToolReversibility::Reversible,
            undo: Some(undo.to_string()),
            authority_modifying: false,
        }
    }

    pub fn irreversible(name: &str) -> Self {
        Self {
            name: name.to_string(),
            reversibility: ToolReversibility::Irreversible,
            undo: None,
            authority_modifying: false,
        }
    }
}

/// 动作分级 — 计划的风险是**路径的函数**, 不是工具最大值; 用路径上最严重者兜底。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionTier {
    /// 只读 — 全自动, 不打断 (过度打断 → confirmation fatigue 反变安全风险)
    Tier1Autonomous,
    /// 可逆写 — 自动 + 全量日志
    Tier2Logged,
    /// 可补偿/外部 — 评审组或 staging 队列
    Tier3Review,
    /// 不可逆/扩权 — 强制人工审批, 无置信度豁免
    Tier4Human,
}

impl ActionTier {
    /// 沿路径分类: 最严重者兜底 (Path-Based Authorization 的保守 floor)。
    pub fn classify(tools: &[ToolSpec]) -> Self {
        let mut tier = ActionTier::Tier1Autonomous;
        for t in tools {
            let this =
                if t.authority_modifying || t.reversibility == ToolReversibility::Irreversible {
                    ActionTier::Tier4Human
                } else if t.reversibility == ToolReversibility::Compensable {
                    ActionTier::Tier3Review
                } else if t.reversibility == ToolReversibility::Reversible {
                    ActionTier::Tier2Logged
                } else {
                    ActionTier::Tier1Autonomous
                };
            if tier_rank(this) > tier_rank(tier) {
                tier = this;
            }
        }
        tier
    }

    pub fn required_gate(self) -> GateLevel {
        match self {
            ActionTier::Tier1Autonomous | ActionTier::Tier2Logged => GateLevel::Light,
            ActionTier::Tier3Review => GateLevel::Panel,
            ActionTier::Tier4Human => GateLevel::Human,
        }
    }
}

fn tier_rank(t: ActionTier) -> u8 {
    match t {
        ActionTier::Tier1Autonomous => 1,
        ActionTier::Tier2Logged => 2,
        ActionTier::Tier3Review => 3,
        ActionTier::Tier4Human => 4,
    }
}

/// 去偏 + 门限配置 — R-P11 Config struct 模式。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebiasConfig {
    /// 冗长惩罚系数: 输出超过 norm 时按比例压分 (verbosity bias ~15%)
    pub verbosity_penalty: f64,
    /// 参考文本长度 (字符), 超过即触发冗长惩罚
    pub verbosity_norm_len: usize,
    /// 冗长惩罚上限 (防过度压制)
    pub verbosity_penalty_cap: f64,
    /// 强制家族分离: 生成方家族已知时, 同族法官的评分被排除
    pub require_family_separation: bool,
    /// self-preference 降权 (同族法官未被排除时的残余惩罚)
    pub self_preference_penalty: f64,
    /// 评分者间一致率下限, 低于此 → Review (转人工)
    pub agreement_review_threshold: f64,
    /// 中位数通过阈值
    pub pass_threshold: f64,
    /// grounding 最低比率 (低于 → Reject)
    pub grounding_min_ratio: f64,
    /// schema 严格模式 (缺字段即 Reject)
    pub schema_strict: bool,
}

impl Default for DebiasConfig {
    fn default() -> Self {
        Self {
            verbosity_penalty: 0.10,
            verbosity_norm_len: 600,
            verbosity_penalty_cap: 0.30,
            require_family_separation: true,
            self_preference_penalty: 0.05,
            agreement_review_threshold: 0.60,
            pass_threshold: 0.60,
            grounding_min_ratio: 0.60,
            schema_strict: true,
        }
    }
}

impl DebiasConfig {
    /// 冗长惩罚: 超过 norm 的部分按比例折算, 封顶。
    pub(crate) fn verbosity_penalty_for(&self, text: &str) -> f64 {
        let len = text.chars().count();
        if len <= self.verbosity_norm_len {
            return 0.0;
        }
        let over = (len - self.verbosity_norm_len) as f64 / self.verbosity_norm_len as f64;
        (over * self.verbosity_penalty)
            .max(0.0)
            .min(self.verbosity_penalty_cap)
    }
}

/// 组合裁决 — 爆炸半径分级 × 护栏 × 评审组, 输出可执行门控动作。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateDecision {
    pub level: GateLevel,
    pub tier: ActionTier,
    pub action: GuardAction,
    pub verdict: Verdict,
    pub reason: String,
}

impl GateDecision {
    /// 沿动作路径决定门控: 确定性检查 (护栏) 优先, 爆炸半径决定强度, LLM 分数殿后。
    pub fn decide(tools: &[ToolSpec], input: &JudgeInput, panel: &JudgePanel) -> Self {
        let tier = ActionTier::classify(tools);
        let level = tier.required_gate();
        let guardrail = GuardrailReport::evaluate(input, &panel.debias);
        let verdict = panel.run(input).verdict;

        // 机械失败 — 确定性检查压过 LLM 分数与自治等级
        if guardrail.action == GuardAction::Reject {
            return Self {
                level: GateLevel::Human,
                tier,
                action: GuardAction::Reject,
                verdict: Verdict::Block,
                reason: format!("机械检查拒绝: {}", guardrail.reason),
            };
        }
        // 幻觉隔离 — 升级人工复核
        if guardrail.action == GuardAction::Quarantine {
            return Self {
                level: GateLevel::Human,
                tier,
                action: GuardAction::Quarantine,
                verdict: Verdict::Review,
                reason: format!("幻觉隔离升级人工: {}", guardrail.reason),
            };
        }
        // 爆炸半径 — 不可逆/扩权动作强制人工, 无置信度豁免
        if level == GateLevel::Human {
            return Self {
                level,
                tier,
                action: GuardAction::Escalate,
                verdict,
                reason: "路径含不可逆/扩权动作, 强制人工审批 (TNR 无豁免)".to_string(),
            };
        }
        // 高分歧 — 转评审组/人工, 不自动放行
        if verdict == Verdict::Review {
            return Self {
                level: GateLevel::Panel,
                tier,
                action: GuardAction::Escalate,
                verdict,
                reason: format!("评审组高分歧 (agreement < 阈值): {:?}", verdict),
            };
        }
        Self {
            level,
            tier,
            action: GuardAction::Allow,
            verdict,
            reason: format!(
                "门控通过: tier={:?}, verdict={:?}, action={:?}",
                tier, verdict, guardrail.action
            ),
        }
    }

    /// 是否允许自治执行 — 唯一放行条件。
    pub fn allows_autonomous(&self) -> bool {
        self.level == GateLevel::Light
            && self.action == GuardAction::Allow
            && self.verdict == Verdict::Pass
    }

    /// 工具级前置检查 — 给定工具名, 返回 (允许, 原因)。
    /// 用法: 在任何工具执行前调用 `GateDecision::check_tool_call("send_email", &registry, &input, &panel)`。
    pub fn check_tool_call(
        tool_name: &str,
        registry: &ToolRegistry,
        _input: &JudgeInput,
        _panel: &JudgePanel,
    ) -> (bool, String) {
        let Some(spec) = registry.get(tool_name) else {
            return (
                false,
                format!("工具 '{}' 未在注册表中, 默认拒绝", tool_name),
            );
        };
        // 单工具快速判定: 只读/可逆 → 允许 (后续完整路径再查); 不可逆/扩权 → 拒绝需人工
        match spec.reversibility {
            ToolReversibility::ReadOnly => (true, "只读工具, 自治放行".to_string()),
            ToolReversibility::Reversible => (true, "可逆工具, 自治放行 (已登记 undo)".to_string()),
            ToolReversibility::Compensable => (false, "可补偿工具, 需评审组审批".to_string()),
            ToolReversibility::Irreversible => (false, "不可逆工具, 强制人工审批".to_string()),
        }
    }

    /// 完整路径检查 — 组合 GateDecision::decide 结果。
    pub fn check_path(tools: &[ToolSpec], input: &JudgeInput, panel: &JudgePanel) -> Self {
        Self::decide(tools, input, panel)
    }
}
