//! 验证器引导模块
//!
//! VLM 验证 + 自动重生成循环
//! 支持多维度验证：实体一致性、环境一致性、叙事进展

use serde::{Serialize, Deserialize};


// ============================================================================
// 验证定义
// ============================================================================

/// 验证维度
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum VerificationDimension {
    /// 实体一致性
    EntityConsistency,
    /// 环境一致性
    EnvironmentConsistency,
    /// 叙事进展
    NarrativeProgression,
    /// 空间逻辑
    SpatialLogicalness,
    /// 实体状态
    EntityState,
    /// 环境状态
    EnvironmentState,
    /// 指令遵循
    InstructionFollowing,
    /// 物理合理性
    PhysicalPlausibility,
    /// 运动一致性
    MotionConsistency,
    /// 镜头一致性
    CameraConsistency,
}

/// 验证维度组
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionGroup {
    /// 组名
    pub name: String,
    /// 包含的维度
    pub dimensions: Vec<VerificationDimension>,
    /// 权重
    pub weight: f32,
}

/// 验证评分
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationScore {
    /// 维度
    pub dimension: VerificationDimension,
    /// 分数 (1-10)
    pub score: u8,
    /// 说明
    pub explanation: Option<String>,
}

/// 验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// 是否通过
    pub passed: bool,
    /// 总分 (0.0-1.0)
    pub total_score: f32,
    /// 各维度分数
    pub scores: Vec<VerificationScore>,
    /// 错误类型
    pub error_types: Vec<String>,
    /// 建议修正
    pub suggested_corrections: Vec<String>,
    /// 是否需要重生成
    pub needs_regeneration: bool,
    /// 验证耗时 (毫秒)
    pub verification_time_ms: u64,
}

/// 验证配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifierConfig {
    /// 维度组
    pub dimension_groups: Vec<DimensionGroup>,
    /// 通过阈值 (0.0-1.0)
    pub pass_threshold: f32,
    /// 最大重生成次数
    pub max_regeneration_attempts: u32,
    /// 是否启用自动修正
    pub enable_auto_correction: bool,
    /// 修正提示词模板
    pub correction_prompt_template: String,
}

/// 重生成请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegenerationRequest {
    /// 原始提示词
    pub original_prompt: String,
    /// 验证结果
    pub verification_result: VerificationResult,
    /// 修正后的提示词
    pub corrected_prompt: String,
    /// 重生成模式
    pub regeneration_mode: RegenerationMode,
}

/// 重生成模式
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum RegenerationMode {
    /// 重新生成（新种子）
    Regenerate,
    /// 编辑（保持主体）
    Edit,
    /// 替换（换模型）
    ReplaceModel,
}

// ============================================================================
// 验证器引导器
// ============================================================================

/// 验证器引导器
/// 实现 VLM 验证 + 自动重生成循环
pub struct VerifierAgent {
    /// 配置
    config: VerifierConfig,
    /// 验证历史
    history: Vec<VerificationResult>,
    /// 重生成历史
    regeneration_history: Vec<RegenerationRequest>,
}

impl VerifierAgent {
    /// 创建验证器
    pub fn new() -> Self {
        Self {
            config: VerifierConfig {
                dimension_groups: vec![
                    DimensionGroup {
                        name: "一致性检查".to_string(),
                        dimensions: vec![
                            VerificationDimension::EntityConsistency,
                            VerificationDimension::EnvironmentConsistency,
                            VerificationDimension::CameraConsistency,
                        ],
                        weight: 0.4,
                    },
                    DimensionGroup {
                        name: "叙事检查".to_string(),
                        dimensions: vec![
                            VerificationDimension::NarrativeProgression,
                            VerificationDimension::SpatialLogicalness,
                        ],
                        weight: 0.3,
                    },
                    DimensionGroup {
                        name: "质量检查".to_string(),
                        dimensions: vec![
                            VerificationDimension::InstructionFollowing,
                            VerificationDimension::PhysicalPlausibility,
                            VerificationDimension::MotionConsistency,
                        ],
                        weight: 0.3,
                    },
                ],
                pass_threshold: 0.7,
                max_regeneration_attempts: 3,
                enable_auto_correction: true,
                correction_prompt_template: "修正以下问题：{errors}".to_string(),
            },
            history: vec![],
            regeneration_history: vec![],
        }
    }
    
    /// 使用配置创建
    pub fn with_config(config: VerifierConfig) -> Self {
        Self {
            config,
            history: vec![],
            regeneration_history: vec![],
        }
    }
    
    /// 验证视频片段
    pub fn verify_shot(
        &mut self,
        _shot_id: &str,
        _video_path: &str,
        spec_description: &str,
        memory_context: Option<&str>,
    ) -> VerificationResult {
        let start = std::time::Instant::now();
        
        // TODO: 实际调用 VLM 进行验证
        let scores = self.simulate_verification(spec_description, memory_context);
        
        // 计算总分
        let total_score = self.calculate_total_score(&scores);
        let passed = total_score >= self.config.pass_threshold;
        
        // 检查是否需要重生成
        let needs_regeneration = !passed && self.history.len() < self.config.max_regeneration_attempts as usize;
        
        let result = VerificationResult {
            passed,
            total_score,
            scores,
            error_types: vec![],
            suggested_corrections: vec![],
            needs_regeneration,
            verification_time_ms: start.elapsed().as_millis() as u64,
        };
        
        self.history.push(result.clone());
        result
    }
    
    /// 模拟验证
    fn simulate_verification(&self, _description: &str, _context: Option<&str>) -> Vec<VerificationScore> {
        vec![
            VerificationScore {
                dimension: VerificationDimension::EntityConsistency,
                score: 8,
                explanation: Some("实体外观保持一致".to_string()),
            },
            VerificationScore {
                dimension: VerificationDimension::EnvironmentConsistency,
                score: 7,
                explanation: Some("环境光照略有变化".to_string()),
            },
            VerificationScore {
                dimension: VerificationDimension::NarrativeProgression,
                score: 8,
                explanation: Some("叙事进展自然".to_string()),
            },
            VerificationScore {
                dimension: VerificationDimension::InstructionFollowing,
                score: 9,
                explanation: Some("遵循提示词指令".to_string()),
            },
        ]
    }
    
    /// 计算总分
    fn calculate_total_score(&self, scores: &[VerificationScore]) -> f32 {
        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;
        
        for group in &self.config.dimension_groups {
            let group_scores: Vec<f32> = scores.iter()
                .filter(|s| group.dimensions.contains(&s.dimension))
                .map(|s| s.score as f32 / 10.0)
                .collect();
            
            if !group_scores.is_empty() {
                let group_avg = group_scores.iter().sum::<f32>() / group_scores.len() as f32;
                weighted_sum += group_avg * group.weight;
                total_weight += group.weight;
            }
        }
        
        if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            0.0
        }
    }
    
    /// 生成重生成请求
    pub fn generate_regeneration_request(
        &self,
        original_prompt: &str,
        verification_result: &VerificationResult,
    ) -> RegenerationRequest {
        let corrected_prompt = if self.config.enable_auto_correction {
            self.auto_correct_prompt(original_prompt, verification_result)
        } else {
            original_prompt.to_string()
        };
        
        let regeneration_mode = if verification_result.total_score < 0.5 {
            RegenerationMode::Regenerate
        } else {
            RegenerationMode::Edit
        };
        
        RegenerationRequest {
            original_prompt: original_prompt.to_string(),
            verification_result: verification_result.clone(),
            corrected_prompt,
            regeneration_mode,
        }
    }
    
    /// 自动修正提示词
    fn auto_correct_prompt(&self, prompt: &str, result: &VerificationResult) -> String {
        // TODO: 实际调用 LLM 修正提示词
        let mut corrected = prompt.to_string();
        
        for correction in &result.suggested_corrections {
            corrected.push_str(&format!(" {}", correction));
        }
        
        corrected
    }
    
    /// 获取验证统计
    pub fn statistics(&self) -> VerifierStats {
        let total_verifications = self.history.len();
        let passed = self.history.iter().filter(|r| r.passed).count();
        let avg_score = if total_verifications > 0 {
            self.history.iter().map(|r| r.total_score).sum::<f32>() / total_verifications as f32
        } else {
            0.0
        };
        let total_regenerations = self.regeneration_history.len();
        
        VerifierStats {
            total_verifications,
            passed,
            failed: total_verifications - passed,
            avg_score,
            total_regenerations,
        }
    }
}

/// 验证统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifierStats {
    /// 总验证次数
    pub total_verifications: usize,
    /// 通过次数
    pub passed: usize,
    /// 失败次数
    pub failed: usize,
    /// 平均分数
    pub avg_score: f32,
    /// 总重生成次数
    pub total_regenerations: usize,
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_verifier_agent() {
        let mut verifier = VerifierAgent::new();
        
        let result = verifier.verify_shot(
            "shot_001",
            "/output/shot_001.mp4",
            "主角在教室学习",
            None,
        );
        
        assert!(result.passed);
        assert!(result.total_score > 0.7);
        
        let stats = verifier.statistics();
        assert_eq!(stats.total_verifications, 1);
    }
    
    #[test]
    fn test_regeneration_request() {
        let verifier = VerifierAgent::new();
        
        let result = VerificationResult {
            passed: false,
            total_score: 0.5,
            scores: vec![],
            error_types: vec!["实体漂移".to_string()],
            suggested_corrections: vec!["保持角色外观一致".to_string()],
            needs_regeneration: true,
            verification_time_ms: 100,
        };
        
        let request = verifier.generate_regeneration_request(
            "主角在教室",
            &result,
        );
        
        assert_eq!(request.regeneration_mode, RegenerationMode::Edit);
    }
}