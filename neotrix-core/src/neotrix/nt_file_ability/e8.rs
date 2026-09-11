//! E8 状态转移 (Ext-6) — 每类操作驱动一次 E8 状态转移。
//!
//! 通过 `E8StateTransition` trait 抽象 NT-CORE 的 ReasoningHexagram，
//! 实现 L1 行动层 → L5 认知层的依赖倒置。

use super::core::FileAbility;
use super::types::E8StateTransition;

/// 文件能力操作 — 每类操作驱动一次 E8 状态转移
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileOperation {
    /// 探测/识别 (open)
    Detect,
    /// 纯文本提取
    Extract,
    /// 格式转换 (markdown/html/导出)
    Transform,
    /// 占位符编辑 (replace_placeholder)
    Edit,
    /// 语义嵌入 (VSA)
    Embed,
    /// 健康巡检 (SelfTest/check_health)
    Audit,
}

impl FileOperation {
    /// 该操作的目标 E8 状态 (6-bit hexagram)
    pub fn target_state_bits(&self) -> u8 {
        match self {
            // 探测: 具体+分析+专注
            Self::Detect => 0b001001,
            // 提取: 具体+分析+深度
            Self::Extract => 0b001100,
            // 转换: 具体+生成+协作 (format transformation)
            Self::Transform => 0b001011,
            // 编辑: 具体+分析+协作
            Self::Edit => 0b001110,
            // 嵌入: 抽象+生成+深度 (semantic encoding)
            Self::Embed => 0b111100,
            // 审计: 抽象+分析+深度
            Self::Audit => 0b101100,
        }
    }

    /// 操作名
    pub fn name(&self) -> &'static str {
        match self {
            Self::Detect => "detect",
            Self::Extract => "extract",
            Self::Transform => "transform",
            Self::Edit => "edit",
            Self::Embed => "embed",
            Self::Audit => "audit",
        }
    }
}

impl E8StateTransition for FileAbility {
    fn current_bits(&self) -> u8 {
        self.e8_state.0
    }

    fn transition_to(&mut self, target_bits: u8) {
        use crate::core::nt_core_hex::ReasoningHexagram;
        let target = ReasoningHexagram::new(target_bits);
        let current = self.e8_state;
        let mut best = current;
        let mut best_dist = current.hamming_dist(&target);
        for n in current.neighbors() {
            let d = n.hamming_dist(&target);
            if d < best_dist {
                best_dist = d;
                best = n;
            }
        }
        self.e8_state = best;
    }

    fn path_to(&self, target_bits: u8) -> Vec<u8> {
        use crate::core::nt_core_hex::{ReasoningHexagram, ReasoningPath};
        let target = ReasoningHexagram::new(target_bits);
        ReasoningPath::shortest(self.e8_state, target)
            .states
            .iter()
            .map(|s| s.0)
            .collect()
    }

    fn mode_name(&self) -> &'static str {
        self.e8_state.mode_name()
    }
}

impl FileAbility {
    /// 当前 E8 推理状态的 6-bit 值 (通过 E8StateTransition trait)
    pub fn e8_state_bits(&self) -> u8 {
        self.e8_state.0
    }

    /// 执行一次状态转移: 将当前状态向目标状态单步推进 (flip 最近的一个差异轴)
    ///
    /// 返回转移后的 6-bit 状态值。若已到达目标, 返回原状态 (路径长度为 0)。
    pub fn transition(&mut self, op: FileOperation) -> u8 {
        let target_bits = op.target_state_bits();
        self.transition_to(target_bits);
        self.e8_state.0
    }

    /// 到目标状态的完整转移路径 (返回路径上各状态的 6-bit 值)
    pub fn e8_path_to(&self, target_bits: u8) -> Vec<u8> {
        self.path_to(target_bits)
    }

    /// E8 状态名称 (人类可读)
    pub fn e8_mode_name(&self) -> &'static str {
        <Self as E8StateTransition>::mode_name(self)
    }
}