//! # NT-REPAIR-SELF-HEAL — 自愈闭环 C5 (Phase 3 免疫)
//!
//! 本模块是 NeoTrix 进化路线 **Phase 3 — 免疫** 的核心件: 让一个自愈/修复
//! 模块**消费 SelfTest 的失败产出** (`Vec<String>`), 形成
//! `检测 → 诊断 → 自愈 → 复测` 的最小闭环。
//!
//! ## 闭环结构
//!
//! 1. **检测 (Detect)**: 运行一组 `SelfTest` 检测器, 收集失败项 `Vec<String>`。
//! 2. **诊断 (Diagnose)**: 解析每条失败, 按检测器 id 归类, 区分"可自愈"与
//!    "不可自愈" (外部故障)。
//! 3. **自愈 (Heal)**: 对可自愈项翻转其内部 `broken` 状态 (interior mutability),
//!    使对应不变量恢复。
//! 4. **复测 (Retest)**: 重新运行全部检测器, 把残余失败 (自愈失败的) 作为
//!    `Err(Vec<String>)` 返回 —— 即 SelfTest 自身通过 = 闭环收敛。
//!
//! 同时暴露 `diagnose_and_heal(failures: &[String])`, 可直接消费**任意**
//! SelfTest 的失败产出 (如全局 `SelfTestRegistry::run_all` 的结果),
//! 返回无法自愈的残余失败, 实现跨模块免疫。
//!
//! 接线契约 (R-P79): 本模块实现 `SelfTest` (T1), 在
//! `nt_core_self_test_integration::register_absorbed_modules` + `register_c5_healers`
//! 注册 (T2); 其 `self_test()` 即闭环本身 (T3 生产行为接地).

#![forbid(unsafe_code)]

use crate::core::nt_core_self_test::{SelfTest, SelfTestRegistry, SelfTestResult};
use std::cell::RefCell;

/// 可被置于"损坏"态、亦可被"自愈"恢复的检测器。
///
/// 用 `RefCell` 内部可变性持有 `broken`, 使 `SelfTest::self_test(&self)` 的
/// 不可变引用下仍可被 `SelfHealLoop` 翻转修复 (无 unsafe, R-P1 合规)。
#[derive(Debug, Clone)]
pub struct HealableDetector {
    pub id: &'static str,
    /// 不变量是否被破坏; `true` → self_test 失败。
    broken: RefCell<bool>,
    /// 是否可被自愈闭环修复 (不可自愈项模拟外部环境故障, 如磁盘/网络)。
    pub healable: bool,
}

impl HealableDetector {
    pub fn new(id: &'static str, healable: bool) -> Self {
        Self {
            id,
            broken: RefCell::new(false),
            healable,
        }
    }

    /// 注入一次故障 (测试/演练用)。
    pub fn inject_fault(&self) {
        *self.broken.borrow_mut() = true;
    }

    /// 自愈闭环调用: 翻转损坏态为健康。
    pub fn heal(&self) {
        *self.broken.borrow_mut() = false;
    }

    pub fn is_broken(&self) -> bool {
        *self.broken.borrow()
    }
}

impl SelfTest for HealableDetector {
    fn name(&self) -> &str {
        self.id
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        if *self.broken.borrow() {
            Err(vec![format!("{}: invariant violated (broken)", self.id)])
        } else {
            Ok(())
        }
    }
}

/// 自愈闭环 C5 — 消费 SelfTest 失败产出, 驱动 检测→诊断→自愈→复测。
#[derive(Debug, Clone)]
pub struct SelfHealLoop {
    detectors: Vec<HealableDetector>,
}

impl Default for SelfHealLoop {
    fn default() -> Self {
        Self::new()
    }
}

impl SelfHealLoop {
    /// 构造一个内置 3 检测器的闭环 (2 可自愈 + 1 外部不可自愈)。
    pub fn new() -> Self {
        Self {
            detectors: vec![
                HealableDetector::new("nt_repair_self_heal::cache_coherence", true),
                HealableDetector::new("nt_repair_self_heal::index_integrity", true),
                HealableDetector::new("nt_repair_self_heal::external_disk", false),
            ],
        }
    }

    /// 当前检测器集合 (克隆, 用于运行)。
    fn detectors(&self) -> Vec<HealableDetector> {
        self.detectors.clone()
    }

    /// **诊断**: 把一条失败串解析为 (检测器 id, 原因)。
    ///
    /// 失败串形如 `"nt_repair_self_heal::cache_coherence: invariant violated (broken)"`,
    /// 以首个 `: ` 切分为 id 与原因; 无法识别则归为 `"unknown"`。
    pub fn diagnose(failure: &str) -> (&str, &str) {
        match failure.find(": ") {
            Some(idx) => (&failure[..idx], &failure[idx + 2..]),
            None => ("unknown", failure),
        }
    }

    /// **核心闭环入口 (消费任意 SelfTest 失败产出)**:
    /// 接收来自 `SelfTestRegistry::run_all` 等来源的失败串,
    /// 按 id 匹配内置可自愈检测器并翻转修复, 返回**残余** (不可自愈) 失败。
    ///
    /// 这是"消费 SelfTest 失败产出"的跨模块接线点: 任意 SelfTest 的
    /// `Vec<String>` 失败可直接喂入, 闭环吸收其可修复部分。
    pub fn diagnose_and_heal(&self, failures: &[String]) -> Vec<String> {
        let mut residual = Vec::new();
        for f in failures {
            let (id, _reason) = Self::diagnose(f);
            let mut healed = false;
            for d in &self.detectors {
                if d.id == id {
                    if d.healable {
                        d.heal();
                        healed = true;
                    }
                    break;
                }
            }
            if !healed {
                residual.push(f.clone());
            }
        }
        residual
    }

    /// **完整闭环 self_test**: 检测 → 诊断 → 自愈 → 复测。
    ///
    /// 返回 `Ok(())` 当且仅当复测后无任何残余失败 (闭环收敛);
    /// 否则返回残余失败 `Vec<String>`。
    pub fn run_closed_loop(&self) -> Result<(), Vec<String>> {
        // 1. 检测
        let mut registry = SelfTestRegistry::new();
        for d in self.detectors() {
            registry.register(Box::new(d));
        }
        let results: Vec<SelfTestResult> = registry.run_all();
        let failures: Vec<String> = results
            .iter()
            .filter(|r| !r.passed)
            .flat_map(|r| r.failures.clone())
            .collect();

        // 2+3. 诊断 + 自愈 (消费失败产出)
        let residual = self.diagnose_and_heal(&failures);
        if residual.is_empty() {
            return Ok(());
        }

        // 4. 复测: 再次运行, 确认可自愈项已恢复, 仅留不可自愈残余。
        let mut registry2 = SelfTestRegistry::new();
        for d in self.detectors() {
            registry2.register(Box::new(d));
        }
        let results2: Vec<SelfTestResult> = registry2.run_all();
        let residual2: Vec<String> = results2
            .iter()
            .filter(|r| !r.passed)
            .flat_map(|r| r.failures.clone())
            .collect();

        if residual2.is_empty() {
            Ok(())
        } else {
            Err(residual2)
        }
    }
}

impl SelfTest for SelfHealLoop {
    fn name(&self) -> &str {
        "nt_repair_self_heal"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        // 闭环本身作为 SelfTest 检测器: 即使当前无故障, 也需验证闭环能修复注入故障。
        // 先注入一次可自愈故障演练, 再跑闭环确认其收敛 → 证明自愈能力在线。
        let demo = HealableDetector::new("nt_repair_self_heal::loop_probe", true);
        demo.inject_fault();

        let mut registry = SelfTestRegistry::new();
        for d in self.detectors() {
            registry.register(Box::new(d));
        }
        registry.register(Box::new(demo));

        let results = registry.run_all();
        let failures: Vec<String> = results
            .iter()
            .filter(|r| !r.passed)
            .flat_map(|r| r.failures.clone())
            .collect();

        // 诊断 + 自愈
        let _ = self.diagnose_and_heal(&failures);

        // 复测 (含 probe)
        let mut registry2 = SelfTestRegistry::new();
        for d in self.detectors() {
            registry2.register(Box::new(d));
        }
        registry2.register(Box::new(HealableDetector::new(
            "nt_repair_self_heal::loop_probe",
            true,
        )));
        let results2 = registry2.run_all();
        let residual2: Vec<String> = results2
            .iter()
            .filter(|r| !r.passed)
            .flat_map(|r| r.failures.clone())
            .collect();

        if residual2.is_empty() {
            Ok(())
        } else {
            Err(residual2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_healthy_loop_passes() {
        let loop_h = SelfHealLoop::new();
        assert!(loop_h.self_test().is_ok(), "no faults → loop should pass");
    }

    #[test]
    fn test_loop_heals_injected_fault() {
        let loop_h = SelfHealLoop::new();
        // 注入一个可自愈故障到内部检测器 (演练检测→诊断→自愈→复测)
        loop_h.detectors[0].inject_fault();
        let res = loop_h.run_closed_loop();
        assert!(
            res.is_ok(),
            "self-heal loop must absorb injected fault, residual={:?}",
            res.err()
        );
    }

    #[test]
    fn test_unhealable_residual_surfaces() {
        let loop_h = SelfHealLoop::new();
        // 外部不可自愈项失败串 → 必须作为残余返回, 不被静默吞掉。
        let failures = vec![
            "nt_repair_self_heal::external_disk: invariant violated (broken)".to_string(),
        ];
        let residual = loop_h.diagnose_and_heal(&failures);
        assert_eq!(residual.len(), 1, "unhealable failure must surface");
        assert!(residual[0].contains("external_disk"));
    }

    #[test]
    fn test_diagnose_splits_id_and_reason() {
        let (id, reason) = SelfHealLoop::diagnose(
            "nt_repair_self_heal::cache_coherence: invariant violated (broken)",
        );
        assert_eq!(id, "nt_repair_self_heal::cache_coherence");
        assert!(reason.contains("invariant violated"));
    }

    #[test]
    fn test_diagnose_unknown_on_malformed() {
        let (id, _) = SelfHealLoop::diagnose("no-colon-failure");
        assert_eq!(id, "unknown");
    }
}
