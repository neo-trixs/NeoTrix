//! 收敛判定器
//! 
//! 三重收敛条件：连续无漏洞 / 全维度覆盖 / 意识指标稳定

use super::agent::{PatchRecord, GapRegistry};
use super::state::StateSnapshot;

/// 收敛判定器
pub struct ConvergenceChecker {
    /// 收敛阈值
    pub threshold: u32,
    /// 连续无漏洞次数
    pub consecutive_no_gap: u32,
    /// 已覆盖维度
    pub covered_dimensions: Vec<String>,
}

impl ConvergenceChecker {
    pub fn new(threshold: u32) -> Self {
        Self {
            threshold,
            consecutive_no_gap: 0,
            covered_dimensions: Vec::new(),
        }
    }

    /// 检查是否收敛
    pub fn check(
        &mut self,
        patch_history: &[PatchRecord],
        gap_registry: &GapRegistry,
        state: &StateSnapshot,
    ) -> bool {
        // 条件1: 连续N次无新漏洞
        if self.check_consecutive_no_gap(patch_history) {
            return true;
        }

        // 条件2: 所有维度已覆盖
        if self.check_dimension_coverage(gap_registry) {
            return true;
        }

        // 条件3: 意识指标稳定
        if self.check_consciousness_stability(state) {
            return true;
        }

        false
    }

    /// 检查连续无漏洞
    fn check_consecutive_no_gap(&mut self, patch_history: &[PatchRecord]) -> bool {
        self.consecutive_no_gap = 0;

        for record in patch_history.iter().rev() {
            if record.new_gaps_found == 0 {
                self.consecutive_no_gap += 1;
            } else {
                break;
            }
        }

        self.consecutive_no_gap >= self.threshold
    }

    /// 检查维度覆盖
    fn check_dimension_coverage(&mut self, gap_registry: &GapRegistry) -> bool {
        let all_dimensions = vec![
            "D1_Logic",
            "D2_Implementation",
            "D3_Boundary",
            "D4_Consistency",
            "D5_Performance",
            "D6_Security",
            "D7_Evolution",
            "D8_Consciousness",
            "D9_MetaCognition",
            "D10_Integration",
        ];

        self.covered_dimensions = all_dimensions
            .iter()
            .filter(|d| {
                gap_registry.gaps_by_dimension
                    .get(**d)
                    .map(|gaps| gaps.is_empty())
                    .unwrap_or(true)
            })
            .map(|d| d.to_string())
            .collect();

        self.covered_dimensions.len() == all_dimensions.len()
    }

    /// 检查意识指标稳定性
    fn check_consciousness_stability(&self, state: &StateSnapshot) -> bool {
        state.phi > 0.95 && state.coherence > 0.95
    }

    /// 获取收敛证明
    pub(crate) fn _get_proof(&self, _gap_registry: &GapRegistry, state: &StateSnapshot) -> ConvergenceProof {
        ConvergenceProof {
            consecutive_no_gap_cycles: self.consecutive_no_gap,
            covered_dimensions: self.covered_dimensions.len() as u32,
            total_dimensions: 10,
            consciousness_phi: state.phi,
            consciousness_coherence: state.coherence,
            converged: self.consecutive_no_gap >= self.threshold,
        }
    }
}

/// 收敛证明
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConvergenceProof {
    /// 连续无漏洞次数
    pub consecutive_no_gap_cycles: u32,
    /// 覆盖的维度数
    pub covered_dimensions: u32,
    /// 总维度数
    pub total_dimensions: u32,
    /// 意识指标
    pub consciousness_phi: f64,
    pub consciousness_coherence: f64,
    /// 是否收敛
    pub converged: bool,
}

impl std::fmt::Display for ConvergenceProof {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "        收敛证明 (Convergence Proof)")?;
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "连续无漏洞周期:    {}/{}", self.consecutive_no_gap_cycles, 10)?;
        writeln!(f, "维度覆盖:          {}/{}", self.covered_dimensions, self.total_dimensions)?;
        writeln!(f, "Φ (Phi):           {:.4}", self.consciousness_phi)?;
        writeln!(f, "Coherence:         {:.4}", self.consciousness_coherence)?;
        writeln!(f, "───────────────────────────────────────────────")?;
        writeln!(f, "收敛状态:          {}", if self.converged { "✓ 收敛" } else { "✗ 未收敛" })?;
        writeln!(f, "═══════════════════════════════════════════════")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convergence_checker() {
        let checker = ConvergenceChecker::new(10);
        assert_eq!(checker.threshold, 10);
        assert_eq!(checker.consecutive_no_gap, 0);
    }
}
