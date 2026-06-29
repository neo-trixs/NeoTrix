//! ImpactMatrix - 能力维度对任务类型的影响权重矩阵
//! 借鉴 dbskill 的多维度检索思想：不同任务类型对不同能力维度的敏感度不同
//!
//! 核心思想：
//! - 每个能力维度（typography, grid, color, ...）对不同任务类型的影响不同
//! - 例如：Design 任务对 typography 和 color 敏感，但 CodeAnalysis 不敏感
//! - 用于验证 self-edit 是否会对某些任务类型产生负面影响

use super::super::nt_expert_routing::TaskType;
use super::core::CapabilityVector;
use std::collections::HashMap;

/// ImpactMatrix: 能力维度 → 任务类型的影响权重矩阵
/// 借鉴 MemOS MemCube 的可扩展思想：支持动态添加任务类型
#[derive(Default)]
pub struct ImpactMatrix {
    /// 能力维度数量（通常是 NUM_FIELDS = 23）
    capability_dims: usize,

    /// 任务类型列表（用于索引）
    task_types: Vec<TaskType>,

    /// 权重矩阵：weights[i][j] = 第 i 个能力维度对第 j 种任务类型的影响权重
    /// 0.0 = 无影响，1.0 = 强影响
    weights: Vec<Vec<f64>>,

    /// 逆向索引：TaskType → 矩阵列索引
    task_index: HashMap<TaskType, usize>,
}

impl ImpactMatrix {
    /// 创建新的 ImpactMatrix
    /// capability_dims: 能力维度数量（如 23）
    /// task_types: 初始任务类型列表
    pub fn new(capability_dims: usize, task_types: Vec<TaskType>) -> Self {
        let num_tasks = task_types.len();
        let mut task_index = HashMap::new();
        for (i, tt) in task_types.iter().enumerate() {
            task_index.insert(*tt, i);
        }

        // 初始化权重矩阵为 0.5（默认中等影响）
        let weights = vec![vec![0.5; num_tasks]; capability_dims];

        Self {
            capability_dims,
            task_types,
            weights,
            task_index,
        }
    }

    /// 设置某个能力维度对某个任务类型的影响权重
    pub fn set_weight(&mut self, dim_idx: usize, task_type: &TaskType, weight: f64) {
        if let Some(&task_idx) = self.task_index.get(task_type) {
            if dim_idx < self.capability_dims {
                self.weights[dim_idx][task_idx] = weight.clamp(0.0, 1.0);
            }
        }
    }

    /// 添加新任务类型（借鉴 MemOS MemCube 的可扩展思想）
    pub fn add_task_type(&mut self, task_type: TaskType) {
        if !self.task_index.contains_key(&task_type) {
            let task_idx = self.task_types.len();
            self.task_index.insert(task_type, task_idx);
            self.task_types.push(task_type);

            // 为新任务类型添加权重列（所有能力维度默认 0.5）
            for dim_weights in &mut self.weights {
                dim_weights.push(0.5);
            }
        }
    }

    /// 计算某个 MicroEdit 对特定任务类型的整体影响
    /// 借鉴 dbskill 的 kNN 检索思想：相似度加权
    pub fn compute_impact(
        &self,
        micro_edit: &super::self_edit::MicroEdit,
        task_type: &TaskType,
    ) -> f64 {
        let mut total_impact = 0.0;
        let mut count = 0;

        match micro_edit {
            super::self_edit::MicroEdit::AdjustDimension(dim_name, amount) => {
                // 将维度名转换为索引
                if let Some(dim_idx) = CapabilityVector::index_from_name(dim_name) {
                    if let Some(&task_idx) = self.task_index.get(task_type) {
                        let weight = self.weights[dim_idx][task_idx];
                        total_impact += weight * amount.abs();
                        count += 1;
                    }
                }
            }
            super::self_edit::MicroEdit::BatchAdjust(pairs) => {
                for (dim_name, amount) in pairs {
                    if let Some(dim_idx) = CapabilityVector::index_from_name(dim_name) {
                        if let Some(&task_idx) = self.task_index.get(task_type) {
                            let weight = self.weights[dim_idx][task_idx];
                            total_impact += weight * amount.abs();
                            count += 1;
                        }
                    }
                }
            }
            _ => {
                // 其他类型的 MicroEdit 默认影响较小
                total_impact = 0.1;
                count = 1;
            }
        }

        if count > 0 {
            total_impact / count as f64
        } else {
            0.0
        }
    }

    /// 验证 self-edit 序列是否对某个任务类型有负面影响
    /// 借鉴 dbskill 的决策边界思想：如果影响超过阈值，拒绝
    pub fn validate_self_edit(
        &self,
        edits: &[super::self_edit::MicroEdit],
        task_type: &TaskType,
        threshold: f64,
    ) -> bool {
        let mut total_impact = 0.0;

        for edit in edits {
            total_impact += self.compute_impact(edit, task_type);
        }

        // 如果总影响超过阈值（负面影响太大），返回 false
        total_impact < threshold
    }

    /// 生成默认的 ImpactMatrix（基于领域知识）
    /// 借鉴 dbskill 的知识库思想：从专家知识初始化
    pub fn matrix_default() -> Self {
        let task_types = vec![
            TaskType::Design,
            TaskType::UIDesign,
            TaskType::CodeAnalysis,
            TaskType::CodeGeneration,
            TaskType::CodeReview,
            TaskType::Security,
            TaskType::Planning,
            TaskType::General,
        ];

        let mut matrix = Self::new(23, task_types); // NUM_FIELDS = 23

        // 设置基于领域知识的影响权重
        // Design 相关任务对 typography, color, whitespace 敏感
        matrix.set_weight(0, &TaskType::Design, 0.9); // typography
        matrix.set_weight(2, &TaskType::Design, 0.8); // color
        matrix.set_weight(3, &TaskType::Design, 0.7); // whitespace
        matrix.set_weight(13, &TaskType::Design, 0.95); // accessibility
        matrix.set_weight(14, &TaskType::Design, 0.98); // compound_composition

        // UIDesign 类似 Design
        matrix.set_weight(0, &TaskType::UIDesign, 0.85);
        matrix.set_weight(2, &TaskType::UIDesign, 0.9);
        matrix.set_weight(15, &TaskType::UIDesign, 0.95); // tailwind_proficiency
        matrix.set_weight(19, &TaskType::UIDesign, 0.95); // ai_native_states

        // CodeAnalysis 对 analysis, synthesis 敏感
        matrix.set_weight(10, &TaskType::CodeAnalysis, 0.9); // analysis
        matrix.set_weight(11, &TaskType::CodeAnalysis, 0.85); // synthesis
        matrix.set_weight(8, &TaskType::CodeAnalysis, 0.8); // inference_depth

        // CodeGeneration 类似 CodeAnalysis
        matrix.set_weight(10, &TaskType::CodeGeneration, 0.95);
        matrix.set_weight(11, &TaskType::CodeGeneration, 0.9);
        matrix.set_weight(8, &TaskType::CodeGeneration, 0.85);

        // CodeReview 对 analysis, verification 敏感
        matrix.set_weight(10, &TaskType::CodeReview, 0.85);
        matrix.set_weight(22, &TaskType::CodeReview, 0.9); // verification

        // Security 对 analysis, verification 非常敏感
        matrix.set_weight(10, &TaskType::Security, 0.9);
        matrix.set_weight(22, &TaskType::Security, 0.95);
        matrix.set_weight(21, &TaskType::Security, 0.9); // quality_gates

        // Planning 对 inference_depth, synthesis 敏感
        matrix.set_weight(8, &TaskType::Planning, 0.9);
        matrix.set_weight(11, &TaskType::Planning, 0.85);
        matrix.set_weight(10, &TaskType::Planning, 0.8);

        matrix
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::nt_expert_routing::TaskType;

    #[test]
    fn test_new() {
        let matrix = ImpactMatrix::new(23, vec![TaskType::Design, TaskType::CodeAnalysis]);
        assert_eq!(matrix.capability_dims, 23);
        assert_eq!(matrix.task_types.len(), 2);
    }

    #[test]
    fn test_add_task_type() {
        let mut matrix = ImpactMatrix::new(23, vec![TaskType::Design]);
        matrix.add_task_type(TaskType::CodeAnalysis);
        assert_eq!(matrix.task_types.len(), 2);
    }

    #[test]
    fn test_compute_impact() {
        let mut matrix = ImpactMatrix::new(23, vec![TaskType::Design, TaskType::CodeAnalysis]);
        matrix.set_weight(0, &TaskType::Design, 0.8);
        let edit =
            super::super::self_edit::MicroEdit::AdjustDimension("typography".to_string(), 0.1);
        let imp = matrix.compute_impact(&edit, &TaskType::Design);
        assert!(imp > 0.0, "Design task should be sensitive to typography");
    }
}
