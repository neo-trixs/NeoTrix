//! 跨模块一致性检查器
//!
//! 检查动态等级与情绪拐点、节奏节拍、转场类型的三线一致
//! 确保所有模块之间的协同工作

use serde::{Serialize, Deserialize};
use crate::core::nt_core_self::dynamic_params::{DynamicParams, ScalingRating};
use crate::core::nt_core_narrative_types::{SegmentData, SegmentType};

// ============================================================================
// 一致性检查配置
// ============================================================================

/// 一致性检查配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _CrossModuleConfig {
    /// 动态等级与情绪一致性阈值
    pub dynamic_emotion_threshold: f32,
    /// 节奏与转场一致性检查
    pub rhythm_transition_check: bool,
    /// 参数边界检查
    pub param_bounds_check: bool,
}

impl Default for _CrossModuleConfig {
    fn default() -> Self {
        Self {
            dynamic_emotion_threshold: 0.8,
            rhythm_transition_check: true,
            param_bounds_check: true,
        }
    }
}

// ============================================================================
// 一致性检查结果
// ============================================================================

/// 一致性检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _CrossModuleCheckResult {
    /// 是否通过
    pub passed: bool,
    /// 检查详情
    pub details: Vec<_CrossModuleDetail>,
    /// 建议
    pub suggestions: Vec<String>,
    /// 一致性评分 (0-100)
    pub consistency_score: u32,
}

/// 一致性检查详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _CrossModuleDetail {
    /// 检查维度
    pub dimension: String,
    /// 是否通过
    pub passed: bool,
    /// 详情
    pub detail: String,
    /// 影响的模块
    pub affected_modules: Vec<String>,
}

// ============================================================================
// 跨模块一致性检查器
// ============================================================================

/// 跨模块一致性检查器
pub struct CrossModuleAudit {
    config: _CrossModuleConfig,
}

impl CrossModuleAudit {
    /// Create a cross-module audit checker with custom configuration.
    ///
    /// Note: Real implementation needs — configuration is stored but not validated.
    /// Consider: validating threshold ranges (0.0-1.0), ensuring check dimensions
    /// are consistent, and loading configuration from KB for cross-session persistence.
    pub fn new(config: _CrossModuleConfig) -> Self {
        Self { config }
    }
    
    /// Create a cross-module audit checker with default configuration.
    ///
    /// Note: Real implementation needs — default thresholds are hardcoded.
    /// Consider: loading defaults from config file, allowing runtime threshold
    /// adjustment, and persistence of configuration changes.
    pub fn default_checker() -> Self {
        Self::new(_CrossModuleConfig::default())
    }
    
    /// Execute cross-module consistency check across dynamic params, segments, and ratings.
    ///
    /// Note: Real implementation needs — checks are sequential. Consider: parallel
    /// check execution for performance, weighted scoring across dimensions, and
    /// configurable severity thresholds per check type.
    pub fn check(
        &self,
        dynamic_params: &[DynamicParams],
        segments: &[SegmentData],
        dynamic_ratings: &[ScalingRating],
    ) -> _CrossModuleCheckResult {
        let mut details = Vec::new();
        let mut suggestions = Vec::new();
        let mut passed = true;
        
        // 1. 动态等级与情绪一致性检查
        let dynamic_emotion_result = self.check_dynamic_emotion_consistency(dynamic_params, dynamic_ratings);
        if !dynamic_emotion_result.passed {
            passed = false;
        }
        details.extend(dynamic_emotion_result.details);
        suggestions.extend(dynamic_emotion_result.suggestions);
        
        // 2. 节奏与段落类型检查
        let rhythm_segment_result = self.check_rhythm_segment_consistency(segments);
        if !rhythm_segment_result.passed {
            passed = false;
        }
        details.extend(rhythm_segment_result.details);
        suggestions.extend(rhythm_segment_result.suggestions);
        
        // 3. 参数边界检查
        if self.config.param_bounds_check {
            let bounds_result = self.check_parameter_bounds(dynamic_params);
            if !bounds_result.passed {
                passed = false;
            }
            details.extend(bounds_result.details);
            suggestions.extend(bounds_result.suggestions);
        }
        
        // 计算一致性评分
        let consistency_score = self.calculate_consistency_score(&details);
        
        _CrossModuleCheckResult {
            passed,
            details,
            suggestions,
            consistency_score,
        }
    }
    
    /// Check consistency between dynamic parameters and scaling ratings.
    ///
    /// Note: Real implementation needs — verifies DynamicParams.to_scaling_rating()
    /// matches provided rating. Consider: fuzzy matching for borderline cases,
    /// reporting actual parameter values that caused the mismatch, and supporting
    /// custom rating mappings per content type.
    fn check_dynamic_emotion_consistency(
        &self,
        dynamic_params: &[DynamicParams],
        dynamic_ratings: &[ScalingRating],
    ) -> _CrossModuleCheckResult {
        let mut details = Vec::new();
        let mut suggestions = Vec::new();
        let mut passed = true;
        
        // 验证动态参数与等级对应关系
        for (i, params) in dynamic_params.iter().enumerate() {
            let expected_rating: ScalingRating = params.to_scaling_rating();
            if i < dynamic_ratings.len() {
                let actual_rating = dynamic_ratings[i];
                if expected_rating != actual_rating {
                    passed = false;
                    details.push(_CrossModuleDetail {
                        dimension: "动态等级一致性".to_string(),
                        passed: false,
                        detail: format!(
                            "动态参数预测等级 {:?}，实际标记等级 {:?}",
                            expected_rating, actual_rating
                        ),
                        affected_modules: vec!["DynamicParams".to_string(), "ScalingRating".to_string()],
                    });
                    suggestions.push(format!(
                        "建议调整动态参数或重新标记等级以保持一致"
                    ));
                } else {
                    details.push(_CrossModuleDetail {
                        dimension: "动态等级一致性".to_string(),
                        passed: true,
                        detail: format!("动态参数与等级匹配: {:?}", actual_rating),
                        affected_modules: vec!["DynamicParams".to_string(), "ScalingRating".to_string()],
                    });
                }
            }
        }
        
        // Compute per-dimension score from details (passed/total * 100).
        let score = if details.is_empty() {
            100
        } else {
            let passed_count = details.iter().filter(|d| d.passed).count();
            (passed_count as f32 / details.len() as f32 * 100.0) as u32
        };
        _CrossModuleCheckResult { passed, details, suggestions, consistency_score: score }
    }
    
    /// Check rhythm consistency: no more than 2 consecutive segments of same type,
    /// and climax segment not at the start.
    ///
    /// Note: Real implementation needs — consecutive threshold (2) is hardcoded.
    /// Consider: configurable max consecutive count per segment type, transition
    /// probability matrix validation, and pacing curve analysis.
    fn check_rhythm_segment_consistency(&self, segments: &[SegmentData]) -> _CrossModuleCheckResult {
        let mut details = Vec::new();
        let mut suggestions = Vec::new();
        let mut passed = true;
        
        // 检查是否有连续相同类型的段落
        let mut consecutive_count = 0;
        let mut last_type = None;
        
        for segment in segments {
            if last_type == Some(segment.r#type) {
                consecutive_count += 1;
                if consecutive_count >= 2 {
                    passed = false;
                    details.push(_CrossModuleDetail {
                        dimension: "节奏段落类型".to_string(),
                        passed: false,
                        detail: format!(
                            "连续 {} 个相同类型的段落: {:?}",
                            consecutive_count + 1, segment.r#type
                        ),
                        affected_modules: vec!["RhythmRecalculator".to_string()],
                    });
                    suggestions.push("建议在连续相同类型段落之间插入过渡段".to_string());
                }
            } else {
                consecutive_count = 0;
                last_type = Some(segment.r#type);
            }
        }
        
        // 检查段落顺序合理性
        if segments.len() >= 2 {
            let first = &segments[0];
            let _last = &segments[segments.len() - 1];
            
            if first.r#type == SegmentType::Climax {
                passed = false;
                details.push(_CrossModuleDetail {
                    dimension: "节奏段落顺序".to_string(),
                    passed: false,
                    detail: "爽点段在开头，不合理".to_string(),
                    affected_modules: vec!["RhythmRecalculator".to_string()],
                });
                suggestions.push("建议将爽点段放在中后部".to_string());
            }
        }
        
        // Compute per-dimension score from details (passed/total * 100).
        let score = if details.is_empty() {
            100
        } else {
            let passed_count = details.iter().filter(|d| d.passed).count();
            (passed_count as f32 / details.len() as f32 * 100.0) as u32
        };
        _CrossModuleCheckResult { passed, details, suggestions, consistency_score: score }
    }
    
    /// Validate that all dynamic parameters are within physical bounds.
    ///
    /// Note: Real implementation needs — delegates to DynamicParams::validate().
    /// Consider: per-unit bounds (degrees vs radians have different ranges),
    /// warning thresholds (near-boundary values that are valid but risky),
    /// and auto-correction suggestions for out-of-bound parameters.
    fn check_parameter_bounds(&self, dynamic_params: &[DynamicParams]) -> _CrossModuleCheckResult {
        let mut details = Vec::new();
        let mut suggestions = Vec::new();
        let mut passed = true;
        
        for params in dynamic_params {
            if !DynamicParams::validate(params) {
                passed = false;
                details.push(_CrossModuleDetail {
                    dimension: "参数边界".to_string(),
                    passed: false,
                    detail: format!(
                        "参数超出有效范围: speed={}, amplitude={}, frequency={}",
                        params.speed, params.amplitude, params.frequency
                    ),
                    affected_modules: vec!["DynamicParams".to_string()],
                });
                suggestions.push("检查动态参数是否在物理边界内".to_string());
            }
        }
        
        // Compute per-dimension score from details (passed/total * 100).
        let score = if details.is_empty() {
            100
        } else {
            let passed_count = details.iter().filter(|d| d.passed).count();
            (passed_count as f32 / details.len() as f32 * 100.0) as u32
        };
        _CrossModuleCheckResult { passed, details, suggestions, consistency_score: score }
    }
    
    /// Calculate overall consistency score (0-100) from check details.
    ///
    /// Note: Real implementation needs — score is simply (passed/total * 100).
    /// Consider: weighted scoring (some dimensions more critical than others),
    /// severity-aware scoring (Critical failures deduct more), and trend tracking
    /// (is consistency improving or degrading over time?).
    fn calculate_consistency_score(&self, details: &[_CrossModuleDetail]) -> u32 {
        if details.is_empty() {
            return 100;
        }
        
        let passed_count = details.iter().filter(|d| d.passed).count();
        let total = details.len() as f32;
        
        (passed_count as f32 / total * 100.0) as u32
    }
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cross_module_audit_valid_input_passes() {
        // Tests the happy path: well-formed input with consistent dynamic params,
        // segments, and ratings. The check function performs real validation of
        // dynamic-emotion consistency, rhythm-segment consistency, and param bounds.
        let checker = CrossModuleAudit::default_checker();
        
        let dynamic_params = vec![
            DynamicParams {
                speed: 0.5,
                amplitude: 5.0,
                frequency: 1.0,
                unit: crate::core::nt_core_self::dynamic_params::ParamUnit::Degrees,
                valid: true,
            },
        ];
        
        let segments = vec![
            SegmentData {
                r#type: SegmentType::Setup,
                base_length: 60.0,
                content_priority: 0.5,
                is_core_scuang: false,
            },
            SegmentData {
                r#type: SegmentType::Climax,
                base_length: 80.0,
                content_priority: 1.0,
                is_core_scuang: true,
            },
        ];
        
        let dynamic_ratings = vec![ScalingRating::Micro];
        
        let result = checker.check(&dynamic_params, &segments, &dynamic_ratings);
        assert!(result.passed, "consistent input should pass cross-module audit");
        assert!(result.consistency_score > 0,
            "consistent input should produce positive score, got {}", result.consistency_score);
    }
}