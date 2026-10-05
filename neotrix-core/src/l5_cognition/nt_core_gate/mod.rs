//! nt_core_gate — 公正评审门控节点 (Unbiased Judge Panel Gate)
//!
//! 把「门禁读证据、按规则裁决」落地为确定性 + 跨家族法官聚合两层，覆盖 2026 文献
//! 与 reverse-skill / STRATUS / evidence-gate 实证的四条支柱:
//!
//! 1. **多家族公正评委** — `JudgePanel` 聚合不同 `JudgeFamily` 的法官, 计算评分者间
//!    一致率 (inter-rater agreement) 与中位数聚合; 高分歧 → 转人工 (Anthropic pass^k
//!    纪律)。去偏 (DebiasConfig): 家族分离 (评委 ≠ 生成方家族, 消 self-preference,
//!    Wataoka 2024 / Autorubric)、verbosity 惩罚 (Wang et al. 2023 冗长偏差 ~15%)、
//!    小量表 1-4 (FutureAGI 建议)。位置偏差由 `run_ensemble` 的 N 次重复 + pass^k
//!    双评一致吸收。
//! 2. **eval 即护栏** — `GuardrailReport::evaluate`: 低 grounding → Reject;
//!    无证据引用的幻觉声明 → Quarantine (逐句引用 + 集合差检测, agentpatterns.ai
//!    per-line-citation 模式); schema 字段缺失 → Reject (Forbes: JSON 必须可解析)。
//! 3. **轨迹分级 + 忠实度** — `FaithfulnessReport::audit` 对 claim→evidence 做集合差;
//!    `CalibrationSet` 用真实 clean/broken 日志黄金集校准 pass^k (TRACE 2602.21230
//!    高分化错误, Anthropic demystifying-evals Step 6 check transcripts)。
//! 4. **爆炸半径门控** — `ActionTier::classify` 按工具可逆性 (只读/可逆/可补偿/不可逆/
//!    扩权) 分级自治 (DigitalApplied 四层 / TianPan 六类 / STRATUS TNR); 不可逆 → 强制
//!    人工审批, 确定性检查优先于置信度分数。
//!
//! 纪律: R-P6 float 用 `.max(0.0).min(1.0)`; 确定性优先 — LLM 分数只在机械检查通过后
//! 才进入聚合。

pub mod nt_guardrail;
pub mod nt_judge;
pub mod nt_panel_debate;
pub mod nt_provenance;
pub mod nt_tool_registry;
pub mod nt_trajectory;
pub mod nt_types;

pub use nt_guardrail::*;
pub use nt_judge::*;
pub use nt_panel_debate::*;
pub use nt_tool_registry::*;
pub use nt_trajectory::*;
pub use nt_types::*;

#[cfg(test)]
mod tests;
