//! SEALAlgebra - SEAL 循环的代数验证
//! 借鉴 gstack 矩阵分解思想：验证 T_seal 的谱半径和收敛性
//! 
//! 核心思想（来自线性代数）：
//! - 每个 MicroEdit 对应一个小变换矩阵 T_i
//! - 复合变换 T = T_n ◦ ... ◦ T_1 的谱半径 ρ(T) 决定收敛性
//! - 如果 ρ(T) < 1，则迭代收敛；否则可能发散

use super::self_edit::MicroEdit;
// use super::core::CapabilityVector; // 临时禁用：unused

/// SEAL 代数验证器
/// 借鉴 MemOS MemCube 的可演化思想：验证变换序列是否稳定
pub struct SEALAlgebra {
    /// 谱半径阈值（默认 1.0）
    spectral_radius_threshold: f64,
    
    /// 最大允许幅度（防止单个 MicroEdit 变化过大）
    max_adjustment: f64,
}

impl Default for SEALAlgebra {
    fn default() -> Self { Self::new() }
}

impl SEALAlgebra {
    /// 创建新的 SEALAlgebra 验证器
    pub fn new() -> Self {
        Self {
            spectral_radius_threshold: 1.0,
            max_adjustment: 0.5,
        }
    }
    
    /// 计算变换序列的谱半径（简化版本）
    /// 借鉴 gstack 矩阵分解：每个小变换的条件数小，数值稳定
    /// 
    /// 简化假设：每个 AdjustDimension 对应一个缩放矩阵
    /// 谱半径 ≈ max |amount| across all AdjustDimension
    pub fn compute_spectral_radius(&self, edits: &[MicroEdit]) -> f64 {
        let mut max_magnitude = 0.0f64;
        
        for edit in edits {
            if let MicroEdit::AdjustDimension(_, amount) = edit {
                let abs_amount = amount.abs();
                if abs_amount > max_magnitude {
                    max_magnitude = abs_amount;
                }
            }
            // BatchAdjust 取最大幅度
            if let MicroEdit::BatchAdjust(pairs) = edit {
                for (_, amount) in pairs {
                    let abs_amount = amount.abs();
                    if abs_amount > max_magnitude {
                        max_magnitude = abs_amount;
                    }
                }
            }
            // UpdateLearningRate 和 NormalizeVector 不改变谱半径（假设）
            // AddExtension 和 SetProvenance 也不改变（它们是结构变换）
        }
        
        max_magnitude
    }
    
    /// 验证变换序列是否收敛
    /// 借鉴 dbskill 的决策边界思想：如果谱半径超过阈值，拒绝
    pub fn validate_convergence(&self, edits: &[MicroEdit]) -> bool {
        let spectral_radius = self.compute_spectral_radius(edits);
        spectral_radius < self.spectral_radius_threshold
    }
    
    /// 验证单个 MicroEdit 的幅度是否过大
    pub fn validate_micro_edit(&self, edit: &MicroEdit) -> bool {
        if let MicroEdit::AdjustDimension(_, amount) = edit {
            amount.abs() <= self.max_adjustment
        } else if let MicroEdit::BatchAdjust(pairs) = edit {
            pairs.iter().all(|(_, amount)| amount.abs() <= self.max_adjustment)
        } else {
            true  // 其他类型的 MicroEdit 默认通过
        }
    }
    
    /// 验证整个变换序列（包括每个 MicroEdit 和整体谱半径）
    pub fn validate_all(&self, edits: &[MicroEdit]) -> (bool, Vec<String>) {
        let mut errors = Vec::new();
        let mut all_valid = true;
        
        // 1. 验证每个 MicroEdit 的幅度
        for (i, edit) in edits.iter().enumerate() {
            if !self.validate_micro_edit(edit) {
                errors.push(format!(
                    "MicroEdit {}: 幅度过大（超过 {}）",
                    i, self.max_adjustment
                ));
                all_valid = false;
            }
        }
        
        // 2. 验证整体谱半径
        if !self.validate_convergence(edits) {
            errors.push(format!(
                "谱半径 {} 超过阈值 {}，可能导致不收敛",
                self.compute_spectral_radius(edits),
                self.spectral_radius_threshold
            ));
            all_valid = false;
        }
        
        (all_valid, errors)
    }
    
    /// 设置谱半径阈值
    pub fn set_threshold(&mut self, threshold: f64) {
        self.spectral_radius_threshold = threshold.max(0.0);
    }
    
    /// 设置最大调整幅度
    pub fn set_max_adjustment(&mut self, max: f64) {
        self.max_adjustment = max.clamp(0.0, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::self_edit::MicroEdit;
    
    #[test]
    fn test_spectral_radius() {
        let algebra = SEALAlgebra::new();
        
        let edits = vec![
            MicroEdit::AdjustDimension("typography".to_string(), 0.1),
            MicroEdit::AdjustDimension("grid".to_string(), 0.2),
            MicroEdit::NormalizeVector,
        ];
        
        let radius = algebra.compute_spectral_radius(&edits);
        assert_eq!(radius, 0.2, "谱半径应该是最大幅度 0.2");
    }
    
    #[test]
    fn test_validate_convergence() {
        let algebra = SEALAlgebra::new();
        
        // 小幅度，应该收敛
        let valid_edits = vec![
            MicroEdit::AdjustDimension("typography".to_string(), 0.1),
            MicroEdit::AdjustDimension("grid".to_string(), 0.05),
        ];
        assert!(algebra.validate_convergence(&valid_edits), "小幅度应该收敛");
        
        // 大幅度，可能不收敛
        let invalid_edits = vec![
            MicroEdit::AdjustDimension("typography".to_string(), 1.5),
        ];
        assert!(!algebra.validate_convergence(&invalid_edits), "大幅度应该不收敛");
    }
    
    #[test]
    fn test_validate_micro_edit() {
        let algebra = SEALAlgebra::new();
        
        // 幅度正常
        let valid = MicroEdit::AdjustDimension("test".to_string(), 0.3);
        assert!(algebra.validate_micro_edit(&valid), "0.3 应该通过");
        
        // 幅度过大
        let invalid = MicroEdit::AdjustDimension("test".to_string(), 0.8);
        assert!(!algebra.validate_micro_edit(&invalid), "0.8 应该不通过");
    }
}
