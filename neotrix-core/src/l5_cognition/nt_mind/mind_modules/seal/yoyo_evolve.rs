//! NT-MIND — yoyo-evolve 吸收 (github.com/yologdev/yoyo-evolve, Batch2 #9, C2 集成).
//!
//! yoyo-evolve: 进化机制 `patch.proposed → eval.finished → decision.created → Promoted`,
//! 与 NeoTrix SEAL pipeline 的 `make_stage!` 宏四阶段 **直接同构**。
//!
//! `make_stage!` 宏定义 (neotrix-core/src/lib.rs:60) 生成阶段结构体:
//! ```ignore
//! make_stage!(Explore); make_stage!(Distill);
//! make_stage!(SelfTest); make_stage!(Absorb);
//! ```
//! 每个阶段是一个可 `new()` 的 unit struct。本模块用等价语义的 `Stage` enum +
//! `transition` 状态机表达同一四阶段闭环, 可作为 SEAL pipeline 的阶段路由前端。
//!
//! # GASP 四阶段 → make_stage! 四阶段 直接映射
//! | GASP stage           | enum Stage    | make_stage! 阶段 (SEAL)        | 门禁                    |
//! |----------------------|---------------|--------------------------------|-------------------------|
//! | patch.proposed       | Proposed      | Explore / Distill (提 mutation) | —                       |
//! | eval.finished        | Eval          | SelfTest                       | R-P42 验证门槛 (eval gate) |
//! | decision.created     | Decision      | Absorb (accept/reject gate)    | R-P42/R-P43 吸收协议     |
//! | Promoted             | Promoted      | Promote (wire to prod path)    | Dark Forest 接线门禁     |

use crate::core::nt_core_self_test::SelfTest;

/// 进化阶段机 — 与 `make_stage!` 四阶段一一对应。
///
/// `Proposed → Eval → Decision → Promoted` 构成单一前进方向闭环
/// (Promoted 后回到 Proposed 开启下一代际, 即代际演进)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    /// patch.proposed — 提出修改 (对应 make_stage! Explore/Distill 提 mutation)。
    Proposed,
    /// eval.finished — 评估完成 (对应 make_stage! SelfTest, R-P42 验证门槛)。
    Eval,
    /// decision.created — 决策生成 (对应 make_stage! Absorb accept/reject gate)。
    Decision,
    /// Promoted — 晋升落地 (对应 make_stage! Promote, 接线生产路径)。
    Promoted,
}

impl Stage {
    /// 阶段顺序 (用于验证非空闭环与索引映射)。
    pub fn lifecycle() -> &'static [Stage] {
        &[
            Stage::Proposed,
            Stage::Eval,
            Stage::Decision,
            Stage::Promoted,
        ]
    }

    /// 该阶段映射到的 SEAL `make_stage!` 阶段名 (GASP→SEAL 对照)。
    pub fn _seal_stage(&self) -> &'static str {
        match self {
            Stage::Proposed => "Explore/Distill (propose mutation)",
            Stage::Eval => "SelfTest (R-P42 eval gate)",
            Stage::Decision => "Absorb (accept/reject gate)",
            Stage::Promoted => "Promote (wire to production path)",
        }
    }

    /// 该阶段对应的 GASP 事件名。
    pub(crate) fn _gasp_event(&self) -> &'static str {
        match self {
            Stage::Proposed => "patch.proposed",
            Stage::Eval => "eval.finished",
            Stage::Decision => "decision.created",
            Stage::Promoted => "Promoted",
        }
    }
}

/// 阶段机错误: 非法转移 (后退/越级/未定义)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum _TransitionError {
    IllegalBackward,
    IllegalJump,
    AlreadyTerminal,
}

/// 进化阶段机 — 持有当前阶段, 提供 `transition` 驱动 GASP→SEAL 四阶段推进。
///
/// 状态转移严格单向: Proposed→Eval→Decision→Promoted→(新代际)Proposed。
/// 每步 `transition` 即一次 `make_stage!` 阶段推进。
pub struct _EvolutionMachine {
    stage: Stage,
    generation: u64,
    eval_passed: bool,
}

impl _EvolutionMachine {
    pub fn new() -> Self {
        Self {
            stage: Stage::Proposed,
            generation: 0,
            eval_passed: false,
        }
    }

    pub fn stage(&self) -> Stage {
        self.stage
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// 驱动一次阶段转移, 直接映射 make_stage! 四阶段推进。
    ///
    /// - Proposed → Eval: 进入评估 (尚未过门禁)。
    /// - Eval → Decision: 必须 `eval_passed` (R-P42 验证门槛), 否则 IllegalJump。
    /// - Decision → Promoted: 决策通过, 晋升落地。
    /// - Promoted → Proposed: 开启下一代际 (代际 +1)。
    pub fn transition(&mut self) -> Result<Stage, _TransitionError> {
        match self.stage {
            Stage::Proposed => {
                self.stage = Stage::Eval;
                Ok(self.stage)
            }
            Stage::Eval => {
                if !self.eval_passed {
                    return Err(_TransitionError::IllegalJump);
                }
                self.stage = Stage::Decision;
                Ok(self.stage)
            }
            Stage::Decision => {
                self.stage = Stage::Promoted;
                Ok(self.stage)
            }
            Stage::Promoted => {
                self.generation += 1;
                self.stage = Stage::Proposed;
                self.eval_passed = false;
                Ok(self.stage)
            }
        }
    }

    /// 标记评估通过 (R-P42 eval gate), 允许 Eval→Decision。
    pub(crate) fn _mark_eval_passed(&mut self) {
        self.eval_passed = true;
    }

    /// 当前阶段映射到 SEAL make_stage! 名。
    pub(crate) fn _seal_stage(&self) -> &'static str {
        self.stage._seal_stage()
    }
}

impl Default for _EvolutionMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl SelfTest for _EvolutionMachine {
    fn name(&self) -> &'static str {
        "_EvolutionMachine"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut errs = Vec::new();
        // 阶段必须落在四阶段生命周期内 (非空闭环)。
        if !Stage::lifecycle().contains(&self.stage) {
            errs.push(format!("stage {:?} not in SEAL lifecycle", self.stage));
        }
        // Eval 阶段必须已有 eval 结果 (R-P42 门禁不漏)。
        if self.stage == Stage::Eval && !self.eval_passed {
            errs.push("stuck in Eval without R-P42 eval gate passed".into());
        }
        if errs.is_empty() { Ok(()) } else { Err(errs) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_four_stage_lifecycle_matches_make_stage() {
        assert_eq!(Stage::lifecycle().len(), 4);
        assert_eq!(Stage::Proposed._seal_stage(), "Explore/Distill (propose mutation)");
        assert_eq!(Stage::Promoted._seal_stage(), "Promote (wire to production path)");
    }

    #[test]
    fn test_transition_requires_rp42_eval_gate() {
        let mut m = _EvolutionMachine::new();
        assert_eq!(m.stage(), Stage::Proposed);
        assert_eq!(m.transition().unwrap(), Stage::Eval);
        // Eval 未过 R-P42 门禁 → 拒绝跳转 Decision
        assert_eq!(m.transition(), Err(_TransitionError::IllegalJump));
        m._mark_eval_passed();
        assert_eq!(m.transition().unwrap(), Stage::Decision);
        assert_eq!(m.transition().unwrap(), Stage::Promoted);
        // Promoted → 新代际 Proposed
        assert_eq!(m.transition().unwrap(), Stage::Proposed);
        assert_eq!(m.generation(), 1);
    }

    #[test]
    fn test_selftest_flags_unevaluated_eval_stage() {
        let mut m = _EvolutionMachine::new();
        m.transition().unwrap(); // → Eval, eval_passed=false
        assert_eq!(m.stage(), Stage::Eval);
        assert!(m.self_test().is_err());
        m._mark_eval_passed();
        assert!(m.self_test().is_ok());
    }
}
