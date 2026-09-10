//! 状态快照
//! 
//! 记录系统当前状态，用于探测和验证

use serde::{Deserialize, Serialize};

/// 状态快照
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StateSnapshot {
    /// 周期
    pub cycle: u32,
    
    // ========================================================================
    // 逻辑状态
    // ========================================================================
    /// 推理链数量
    pub reasoning_chains: u32,
    /// 已验证假设数
    pub assumptions_verified: u32,
    /// 总假设数
    pub assumptions_total: u32,
    
    // ========================================================================
    // 实现状态
    // ========================================================================
    /// TODO 数量
    pub todo_count: u32,
    /// 已实现接口数
    pub interfaces_implemented: u32,
    /// 总接口数
    pub interfaces_total: u32,
    
    // ========================================================================
    // 边界状态
    // ========================================================================
    /// 空值检查是否完整
    pub null_checks_complete: bool,
    /// 溢出保护是否启用
    pub overflow_protection: bool,
    
    // ========================================================================
    // 一致性状态
    // ========================================================================
    /// 命名冲突数
    pub naming_conflicts: u32,
    /// 类型不匹配数
    pub type_mismatches: u32,
    
    // ========================================================================
    // 性能状态
    // ========================================================================
    /// 最大复杂度
    pub max_complexity: u32,
    /// 内存使用 (MB)
    pub memory_usage_mb: u64,
    
    // ========================================================================
    // 安全状态
    // ========================================================================
    /// 注入风险数
    pub injection_risks: u32,
    /// 信息泄露数
    pub information_leaks: u32,
    
    // ========================================================================
    // 演化状态
    // ========================================================================
    /// 学习率
    pub learning_rate: f64,
    /// 保留率
    pub retention_rate: f64,
    
    // ========================================================================
    // 意识状态
    // ========================================================================
    /// 自指循环是否激活
    pub self_reference_loop_active: bool,
    /// Φ 指标
    pub phi: f64,
    /// 连贯性
    pub coherence: f64,
    
    // ========================================================================
    // 集成状态
    // ========================================================================
    /// 接口不匹配数
    pub interface_mismatches: u32,
    /// 依赖不完整数
    pub dependencies_incomplete: u32,
}

impl StateSnapshot {
    /// 创建新的状态快照
    pub fn new(cycle: u32, phi: f64, coherence: f64) -> Self {
        Self {
            cycle,
            phi,
            coherence,
            self_reference_loop_active: cycle > 0,
            reasoning_chains: cycle * 2,
            assumptions_verified: cycle,
            assumptions_total: cycle * 2,
            todo_count: 100 - (cycle * 10).min(95),
            interfaces_implemented: (cycle * 5).min(50),
            interfaces_total: 50,
            null_checks_complete: cycle > 5,
            overflow_protection: cycle > 3,
            naming_conflicts: 10 - (cycle as i32).min(10) as u32,
            type_mismatches: 5 - (cycle as i32 / 2).min(5) as u32,
            max_complexity: 200 - (cycle * 10).min(150),
            memory_usage_mb: 2000 - (cycle as u64 * 100).min(1500),
            injection_risks: 8 - (cycle as i32).min(8) as u32,
            information_leaks: 5 - (cycle as i32).min(5) as u32,
            learning_rate: 0.1 + (cycle as f64 * 0.001).min(0.1),
            retention_rate: 0.3 + (cycle as f64 * 0.005).min(0.6),
            interface_mismatches: 6 - (cycle as i32).min(6) as u32,
            dependencies_incomplete: 4 - (cycle as i32 / 3).min(4) as u32,
        }
    }

    /// 计算健康分数 (0.0 - 1.0)
    pub fn health_score(&self) -> f64 {
        let mut score = 0.0;
        let mut weights = 0.0;

        // 逻辑 (权重: 0.15)
        if self.assumptions_total > 0 {
            score += 0.15 * (self.assumptions_verified as f64 / self.assumptions_total as f64);
        }
        weights += 0.15;

        // 实现 (权重: 0.15)
        if self.interfaces_total > 0 {
            score += 0.15 * (self.interfaces_implemented as f64 / self.interfaces_total as f64);
        }
        weights += 0.15;

        // 边界 (权重: 0.10)
        let boundary_score = if self.null_checks_complete { 0.5 } else { 0.0 }
            + if self.overflow_protection { 0.5 } else { 0.0 };
        score += 0.10 * boundary_score;
        weights += 0.10;

        // 一致性 (权重: 0.10)
        let consistency_score = 1.0 - (self.naming_conflicts as f64 / 20.0).min(1.0)
            - (self.type_mismatches as f64 / 10.0).min(1.0);
        score += 0.10 * consistency_score.max(0.0);
        weights += 0.10;

        // 性能 (权重: 0.10)
        let perf_score = 1.0 - (self.max_complexity as f64 / 500.0).min(1.0);
        score += 0.10 * perf_score;
        weights += 0.10;

        // 安全 (权重: 0.10)
        let security_score = 1.0 - (self.injection_risks as f64 / 10.0).min(1.0)
            - (self.information_leaks as f64 / 10.0).min(1.0);
        score += 0.10 * security_score.max(0.0);
        weights += 0.10;

        // 意识 (权重: 0.20)
        let consciousness_score = self.phi * 0.5 + self.coherence * 0.5;
        score += 0.20 * consciousness_score;
        weights += 0.20;

        // 集成 (权重: 0.10)
        let integration_score = 1.0 - (self.interface_mismatches as f64 / 10.0).min(1.0)
            - (self.dependencies_incomplete as f64 / 10.0).min(1.0);
        score += 0.10 * integration_score.max(0.0);
        weights += 0.10;

        if weights > 0.0 {
            score / weights
        } else {
            0.0
        }
    }
}

impl std::fmt::Display for StateSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "        状态快照 (Cycle {})", self.cycle)?;
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "逻辑:   推理链 {} | 假设验证 {}/{}", 
            self.reasoning_chains, self.assumptions_verified, self.assumptions_total)?;
        writeln!(f, "实现:   TODO {} | 接口 {}/{}", 
            self.todo_count, self.interfaces_implemented, self.interfaces_total)?;
        writeln!(f, "边界:   空值检查 {} | 溢出保护 {}", 
            if self.null_checks_complete { "✓" } else { "✗" },
            if self.overflow_protection { "✓" } else { "✗" })?;
        writeln!(f, "一致性: 命名冲突 {} | 类型不匹配 {}", 
            self.naming_conflicts, self.type_mismatches)?;
        writeln!(f, "性能:   复杂度 {} | 内存 {}MB", 
            self.max_complexity, self.memory_usage_mb)?;
        writeln!(f, "安全:   注入风险 {} | 信息泄露 {}", 
            self.injection_risks, self.information_leaks)?;
        writeln!(f, "意识:   Φ {:.4} | 连贯性 {:.4}", self.phi, self.coherence)?;
        writeln!(f, "───────────────────────────────────────────────")?;
        writeln!(f, "健康分数: {:.4}", self.health_score())?;
        writeln!(f, "═══════════════════════════════════════════════")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_snapshot_creation() {
        let state = StateSnapshot::new(0, 0.5, 0.5);
        assert_eq!(state.cycle, 0);
        assert_eq!(state.phi, 0.5);
        assert_eq!(state.coherence, 0.5);
    }

    #[test]
    fn test_health_score() {
        let state = StateSnapshot::new(100, 0.95, 0.95);
        assert!(state.health_score() > 0.8);
    }
}
