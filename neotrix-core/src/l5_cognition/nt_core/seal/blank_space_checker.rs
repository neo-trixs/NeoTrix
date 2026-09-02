//! 留白量化检查器
//!
//! 检查爽点后的留白是否符合节奏呼吸感要求
//! 确保强冲突/爽点后有 3-5 秒无台词+微动态的缓冲

use serde::{Serialize, Deserialize};
use crate::l5_cognition::nt_core::seal::rhythm_recalculator::{SegmentData, SegmentType};

// ============================================================================
// 留白检查配置
// ============================================================================

/// 留白检查配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlankSpaceConfig {
    /// 爽点后最小留白时长 (秒)
    pub min_blank_after_climax: f32,
    /// 爽点后最大留白时长 (秒)
    pub max_blank_after_climax: f32,
    /// 无爽点最大平淡期 (秒)
    pub max_blank_period: f32,
    /// 每30秒情绪波动要求
    pub emotion_beat_interval: f32,
}

impl Default for BlankSpaceConfig {
    fn default() -> Self {
        Self {
            min_blank_after_climax: 3.0,
            max_blank_after_climax: 5.0,
            max_blank_period: 60.0,
            emotion_beat_interval: 30.0,
        }
    }
}

// ============================================================================
// 留白检查结果
// ============================================================================

/// 留白检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlankSpaceCheckResult {
    /// 是否通过检查
    pub passed: bool,
    /// 检查详情
    pub details: Vec<BlankSpaceDetail>,
    /// 建议
    pub suggestions: Vec<String>,
    /// 评分 (0-100)
    pub score: u32,
}

/// 留白检查详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlankSpaceDetail {
    /// 检查项
    pub check_item: String,
    /// 是否通过
    pub passed: bool,
    /// 详情
    pub detail: String,
    /// 严重程度
    pub severity: Severity,
}

/// 严重程度
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    /// 通过
    Pass,
    /// 警告
    Warning,
    /// 错误
    Error,
}

// ============================================================================
// 留白量化检查器
// ============================================================================

/// 留白量化检查器
pub struct BlankSpaceChecker {
    config: BlankSpaceConfig,
}

impl BlankSpaceChecker {
    /// 创建检查器
    pub fn new(config: BlankSpaceConfig) -> Self {
        Self { config }
    }
    
    /// 使用默认配置创建检查器
    pub fn default_checker() -> Self {
        Self::new(BlankSpaceConfig::default())
    }
    
    /// 检查留白
    pub fn check(&self, segments: &[SegmentData], total_duration: f32) -> BlankSpaceCheckResult {
        let mut details = Vec::new();
        let mut suggestions = Vec::new();
        let mut passed = true;
        
        // 1. 检查爽点后留白
        let climax_result = self.check_blank_after_climax(segments);
        if !climax_result.passed {
            passed = false;
        }
        details.extend(climax_result.details);
        suggestions.extend(climax_result.suggestions);
        
        // 2. 检查无爽点最大平淡期
        let blank_period_result = self.check_max_blank_period(segments, total_duration);
        if !blank_period_result.passed {
            passed = false;
        }
        details.extend(blank_period_result.details);
        suggestions.extend(blank_period_result.suggestions);
        
        // 3. 检查情绪波动频率
        let emotion_beat_result = self.check_emotion_beat_frequency(segments, total_duration);
        if !emotion_beat_result.passed {
            passed = false;
        }
        details.extend(emotion_beat_result.details);
        suggestions.extend(emotion_beat_result.suggestions);
        
        // 计算评分
        let score = self.calculate_score(&details);
        
        BlankSpaceCheckResult {
            passed,
            details,
            suggestions,
            score,
        }
    }
    
    /// 检查爽点后留白
    fn check_blank_after_climax(&self, segments: &[SegmentData]) -> BlankSpaceCheckResult {
        let mut details = Vec::new();
        let mut suggestions = Vec::new();
        let mut passed = true;
        
        // 查找爽点段
        let climax_index = segments.iter().position(|s| s.r#type == SegmentType::Climax);
        
        if let Some(idx) = climax_index {
            // 检查爽点段后面是否有足够的留白
            if idx + 1 < segments.len() {
                let next_segment = &segments[idx + 1];
                let blank_duration = next_segment.base_length;
                
                if blank_duration < self.config.min_blank_after_climax {
                    passed = false;
                    details.push(BlankSpaceDetail {
                        check_item: "爽点后留白".to_string(),
                        passed: false,
                        detail: format!(
                            "爽点后留白 {:.1}秒，不足 {:.1}秒",
                            blank_duration, self.config.min_blank_after_climax
                        ),
                        severity: Severity::Error,
                    });
                    suggestions.push(format!(
                        "建议在爽点后增加 {:.1}-{:.1}秒的留白缓冲",
                        self.config.min_blank_after_climax,
                        self.config.max_blank_after_climax
                    ));
                } else if blank_duration > self.config.max_blank_after_climax {
                    details.push(BlankSpaceDetail {
                        check_item: "爽点后留白".to_string(),
                        passed: true,
                        detail: format!(
                            "爽点后留白 {:.1}秒，略长但可接受",
                            blank_duration
                        ),
                        severity: Severity::Warning,
                    });
                    suggestions.push("爽点后留白略长，考虑缩短以保持节奏".to_string());
                } else {
                    details.push(BlankSpaceDetail {
                        check_item: "爽点后留白".to_string(),
                        passed: true,
                        detail: format!("爽点后留白 {:.1}秒，符合要求", blank_duration),
                        severity: Severity::Pass,
                    });
                }
            } else {
                // 爽点是最后一段
                details.push(BlankSpaceDetail {
                    check_item: "爽点后留白".to_string(),
                    passed: true,
                    detail: "爽点在最后，无需留白".to_string(),
                    severity: Severity::Pass,
                });
            }
        }
        
        BlankSpaceCheckResult { passed, details, suggestions, score: if passed { 100 } else { 0 } }
    }
    
    /// 检查无爽点最大平淡期
    fn check_max_blank_period(&self, segments: &[SegmentData], total_duration: f32) -> BlankSpaceCheckResult {
        let mut details = Vec::new();
        let mut suggestions = Vec::new();
        let mut passed = true;
        
        // 计算连续无爽点段的最大时长
        let mut max_blank = 0.0;
        let mut current_blank = 0.0;
        
        for segment in segments {
            if !segment.is_core_scuang {
                current_blank += segment.base_length;
                max_blank = max_blank.max(current_blank);
            } else {
                current_blank = 0.0;
            }
        }
        
        if max_blank > self.config.max_blank_period {
            passed = false;
            details.push(BlankSpaceDetail {
                check_item: "无爽点最大平淡期".to_string(),
                passed: false,
                detail: format!(
                    "连续无爽点段 {:.1}秒，超过 {:.1}秒限制",
                    max_blank, self.config.max_blank_period
                ),
                severity: Severity::Error,
            });
            suggestions.push(format!(
                "建议在连续无爽点段中增加小爽点或调整节奏，间隔不超过 {:.1}秒",
                self.config.max_blank_period
            ));
        } else {
            details.push(BlankSpaceDetail {
                check_item: "无爽点最大平淡期".to_string(),
                passed: true,
                detail: format!("连续无爽点段 {:.1}秒，在允许范围内", max_blank),
                severity: Severity::Pass,
            });
        }
        
        BlankSpaceCheckResult { passed, details, suggestions, score: if passed { 100 } else { 0 } }
    }
    
    /// 检查情绪波动频率
    fn check_emotion_beat_frequency(&self, segments: &[SegmentData], total_duration: f32) -> BlankSpaceCheckResult {
        let mut details = Vec::new();
        let mut suggestions = Vec::new();
        let mut passed = true;
        
        // 计算爽点数量
        let climax_count = segments.iter().filter(|s| s.is_core_scuang).count();
        
        // 计算应该有多少个情绪节拍 (每30秒至少1个)
        let expected_beats = (total_duration / self.config.emotion_beat_interval).ceil() as usize;
        
        // 实际情绪节拍包括爽点和重要内容段
        let actual_beats = segments.iter()
            .filter(|s| s.is_core_scuang || s.content_priority >= 0.7)
            .count();
        
        if actual_beats < expected_beats {
            passed = false;
            details.push(BlankSpaceDetail {
                check_item: "情绪波动频率".to_string(),
                passed: false,
                detail: format!(
                    "情绪节拍 {} 个，期望至少 {} 个 (每 {:.0}秒 一个)",
                    actual_beats, expected_beats, self.config.emotion_beat_interval
                ),
                severity: Severity::Error,
            });
            suggestions.push(format!(
                "建议每 {:.0}秒 增加一个情绪拐点或动作亮点",
                self.config.emotion_beat_interval
            ));
        } else {
            details.push(BlankSpaceDetail {
                check_item: "情绪波动频率".to_string(),
                passed: true,
                detail: format!("情绪节拍 {} 个，满足要求", actual_beats),
                severity: Severity::Pass,
            });
        }
        
        BlankSpaceCheckResult { passed, details, suggestions, score: if passed { 100 } else { 0 } }
    }
    
    /// 计算评分
    fn calculate_score(&self, details: &[BlankSpaceDetail]) -> u32 {
        if details.is_empty() {
            return 100;
        }
        
        let passed_count = details.iter().filter(|d| d.severity == Severity::Pass).count();
        let warning_count = details.iter().filter(|d| d.severity == Severity::Warning).count();
        let error_count = details.iter().filter(|d| d.severity == Severity::Error).count();
        
        let total = details.len() as f32;
        let score = (passed_count as f32 / total * 80.0)
            + (warning_count as f32 / total * 15.0)
            + (error_count as f32 / total * 5.0);
        
        (100.0 - score) as u32
    }
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_blank_space_checker() {
        let checker = BlankSpaceChecker::default_checker();
        
        let segments = vec![
            SegmentData {
                r#type: SegmentType::Setup,
                base_length: 60.0,
                content_priority: 0.5,
                is_core_scuang: false,
            },
            SegmentData {
                r#type: SegmentType::Conflict,
                base_length: 80.0,
                content_priority: 0.8,
                is_core_scuang: false,
            },
            SegmentData {
                r#type: SegmentType::Climax,
                base_length: 60.0,
                content_priority: 1.0,
                is_core_scuang: true,
            },
            SegmentData {
                r#type: SegmentType::Transition,
                base_length: 40.0,
                content_priority: 0.6,
                is_core_scuang: false,
            },
        ];
        
        let result = checker.check(&segments, 240.0);
        assert!(result.passed);
        assert!(result.score >= 80);
    }
}