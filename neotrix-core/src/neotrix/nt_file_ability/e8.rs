//! E8 状态转移 (Ext-6) — 每类操作驱动一次 E8 状态转移。

use crate::core::nt_core_hex::ReasoningHexagram;

use super::core::FileAbility;

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
    pub fn target_state(&self) -> ReasoningHexagram {
        match self {
            // 探测: 具体+分析+专注
            Self::Detect => ReasoningHexagram::new(0b001001),
            // 提取: 具体+分析+深度
            Self::Extract => ReasoningHexagram::new(0b001100),
            // 转换: 具体+生成+协作 (format transformation)
            Self::Transform => ReasoningHexagram::new(0b001011),
            // 编辑: 具体+分析+协作
            Self::Edit => ReasoningHexagram::new(0b001110),
            // 嵌入: 抽象+生成+深度 (semantic encoding)
            Self::Embed => ReasoningHexagram::new(0b111100),
            // 审计: 抽象+分析+深度
            Self::Audit => ReasoningHexagram::new(0b101100),
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

impl FileAbility {
    /// 当前 E8 推理状态
    pub fn e8_state(&self) -> ReasoningHexagram {
        self.e8_state
    }

    /// 执行一次状态转移: 将当前状态向目标状态单步推进 (flip 最近的一个差异轴)
    ///
    /// 返回转移后的新状态。若已到达目标, 返回原状态 (路径长度为 0)。
    pub fn transition(&mut self, op: FileOperation) -> ReasoningHexagram {
        let target = op.target_state();
        let current = self.e8_state;
        let mut best = current;
        let mut best_dist = current.hamming_dist(&target);
        // 从 6 个邻居里选最接近目标的单步 (贪心下降)
        for n in current.neighbors() {
            let d = n.hamming_dist(&target);
            if d < best_dist {
                best_dist = d;
                best = n;
            }
        }
        self.e8_state = best;
        best
    }

    /// 到目标状态的完整转移路径 (E8 ReasoningPath)
    pub fn e8_path_to(&self, target: ReasoningHexagram) -> Vec<ReasoningHexagram> {
        crate::core::nt_core_hex::ReasoningPath::shortest(self.e8_state, target).states
    }

    /// E8 状态名称 (人类可读)
    pub fn e8_mode_name(&self) -> &'static str {
        self.e8_state.mode_name()
    }
}