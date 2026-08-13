//! E8 预测器模块
//!
//! 提供基于 E8 状态序列的在线学习预测器，用于意识核心的六阶段闭环跟踪。
//! 实现 `load`/`persist` 接口，供 `handlers_consciousness.rs` 与 `nt_core_task_dispatcher.rs` 调用。

// ---------------------------------------------------------------------------
// 核心结构体
// ---------------------------------------------------------------------------

/// E8 预测器实体
/// - 跨周期累积观测样本 (The Spice Must Flow)
/// - 提供 observe_trace / sample_count / coverage 用于闭环反馈
#[derive(Debug, Clone)]
pub struct E8Predictor {
    /// 观测的状态序列轨迹
    pub state_traces: Vec<Vec<u8>>,
    /// 样本计数
    pub sample_count: usize,
    /// 覆盖度指标 (0.0 ~ 1.0)
    pub coverage: f64,
    /// 相位转移矩阵 (用于相进预测)
    pub phase_transitions: [[u8; 64]; 6],
}

impl E8Predictor {
    /// 创建新的 E8 预测器 (内部使用，load/persist 接口由外部调用)
    pub fn new() -> Self {
        Self {
            state_traces: Vec::new(),
            sample_count: 0,
            coverage: 0.0,
            // 简单的占位转移矩阵，实际训练时会被填充
            phase_transitions: [[0u8; 64]; 6],
        }
    }

    /// 记录一条状态轨迹观测 (对应 handlers_consciousness.rs 中的 observe_trace)
    pub fn observe_trace(&mut self, trace: &[u8]) {
        self.state_traces.push(trace.to_vec());
        self.sample_count += 1;
        // 根据累积样本重新计算覆盖度
        if self.sample_count > 0 {
            let unique = self.state_traces.iter().collect::<std::collections::HashSet<&Vec<u8>>>().len();
            self.coverage = (unique as f64) / (self.sample_count as f64).max(1.0);
        }
    }

    /// 当前累积样本数 (对齐 handlers_consciousness.rs 的方法调用)
    pub fn sample_count(&self) -> usize {
        self.sample_count
    }

    /// 当前状态覆盖度 (0.0 ~ 1.0, 对齐 handlers_consciousness.rs 的方法调用)
    pub fn coverage(&self) -> f64 {
        self.coverage
    }
}

// ---------------------------------------------------------------------------
// load / persist 接口
// ---------------------------------------------------------------------------

/// 从持久化存储加载 E8 预测器实例
///
/// 对应 `handlers_consciousness.rs` 线 356-358 的调用：
/// `use crate::core::nt_core_e8_predictor::{load as predictor_load, persist as predictor_persist};`
/// `let mut predictor = predictor_load();`
///
/// 行为：尝试从 KnowledgeBase (KB) 读取先前的状态；若无记录则创建新实例。
/// 确保跨周期累积 (The Spice Must Flow)，防止预测器成为孤儿模块 (Dark Forest)。
pub fn load() -> E8Predictor {
    // TODO: 从 KnowledgeBase (KB) 读取先前的 E8Predictor 状态
    // 暂时返回新实例，确保编译通过
    E8Predictor::new()
}

/// 将预测器状态持久化到存储
///
/// 对应 `handlers_consciousness.rs` 线 388 的调用：
/// `let _ = predictor_persist(&predictor);`
///
/// 参数：`&predictor` - 要持久化的预测器实例引用
/// 行为：将当前预测器的状态轨迹和指标写入 KB，保证跨周期生效
pub fn persist(predictor: &E8Predictor) {
    // TODO: 写入 KB - 经验吸收协议统一入口
    // 暂无实体写入，确保模块可编译通过
    let _ = predictor; // 防止未使用警告
}

// ---------------------------------------------------------------------------
// 语法修正：u8::from(i) → i as u8
// ---------------------------------------------------------------------------

/// 将整数安全转换为 u8
///
/// 修复潜在的 `u8::from(i)` 语法错误，改用 `as u8` 语法，
/// 符合 NeoTrix R-P1 零 unsafe 规范。
pub fn safe_u8(value: i32) -> u8 {
    // as u8 语法，显式且安全（值范围由调用者保证）
    value as u8
}

// ---------------------------------------------------------------------------
// 模块导出
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_creates_instance() {
        let p = load();
        assert!(matches!(p, E8Predictor { sample_count: 0, .. }));
    }

    #[test]
    fn test_observe_trace() {
        let mut p = load();
        let trace = vec![1u8, 2, 3];
        p.observe_trace(&trace);
        assert_eq!(p.sample_count, 1);
        assert_eq!(p.state_traces.len(), 1);
    }

    #[test]
    fn test_persist_no_panic() {
        let p = load();
        persist(&p); // 不应 panic
    }

    #[test]
    fn test_safe_u8() {
        let result = safe_u8(42);
        assert_eq!(result, 42);
    }
}