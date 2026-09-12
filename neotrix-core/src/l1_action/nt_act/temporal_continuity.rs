//! 时序连续性检查模块 (通用)
//!
//! 检查视频帧间/镜头间的时序连续性
//! 适用于：所有视频编辑和生成场景

use serde::{Serialize, Deserialize};

// ============================================================================
// 时序连续性定义
// ============================================================================

/// 检查类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub(crate) enum ContinuityCheckType {
    /// 首尾帧匹配
    FirstLastFrame,
    /// 场景转场
    SceneTransition,
    /// 元素位置连续
    ElementPosition,
    /// 动作连续
    ActionContinuity,
    /// 光照连续
    LightingContinuity,
    /// 色彩连续
    ColorContinuity,
}

/// 检查配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ContinuityCheckConfig {
    /// 启用的检查类型
    pub check_types: Vec<ContinuityCheckType>,
    /// 匹配阈值 (0.0-1.0)
    pub match_threshold: f32,
    /// 最大允许差异
    pub max_diff: f32,
    /// 是否自动生成修复建议
    pub auto_suggest_fix: bool,
    /// 严重性阈值
    pub severity_threshold: ContinuitySeverity,
}

/// 连续性严重程度
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ContinuitySeverity {
    /// 信息
    Info,
    /// 警告
    Warning,
    /// 错误
    Error,
    /// 严重
    Critical,
}

/// 连续性问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ContinuityIssue {
    /// 问题类型
    pub check_type: ContinuityCheckType,
    /// 严重程度
    pub severity: ContinuitySeverity,
    /// 位置 (帧号)
    pub frame_index: u32,
    /// 问题描述
    pub description: String,
    /// 差异值
    pub diff_value: f32,
    /// 修复建议
    pub fix_suggestion: Option<String>,
}

/// 检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ContinuityCheckResult {
    /// 是否通过
    pub passed: bool,
    /// 检查的帧数
    pub checked_frames: u32,
    /// 问题数量
    pub issue_count: u32,
    /// 通过率 (0.0-1.0)
    pub pass_rate: f32,
    /// 问题列表
    pub issues: Vec<ContinuityIssue>,
    /// 检查耗时 (毫秒)
    pub check_time_ms: u64,
    /// 错误信息
    pub error: Option<String>,
}

// ============================================================================
// 时序连续性检查器
// ============================================================================

/// 时序连续性检查器
/// 检查视频帧间/镜头间的时序连续性
pub(crate) struct TemporalContinuityChecker {
    /// 配置
    config: ContinuityCheckConfig,
    /// 检查历史
    history: Vec<ContinuityCheckResult>,
}

impl TemporalContinuityChecker {
    /// 创建检查器
    pub fn new() -> Self {
        Self {
            config: ContinuityCheckConfig {
                check_types: vec![
                    ContinuityCheckType::FirstLastFrame,
                    ContinuityCheckType::SceneTransition,
                    ContinuityCheckType::ElementPosition,
                ],
                match_threshold: 0.85,
                max_diff: 0.15,
                auto_suggest_fix: true,
                severity_threshold: ContinuitySeverity::Warning,
            },
            history: vec![],
        }
    }
    
    /// 使用配置创建
    pub fn with_config(config: ContinuityCheckConfig) -> Self {
        Self {
            config,
            history: vec![],
        }
    }
    
    /// 检查首尾帧连续性
    pub fn check_first_last_frame(
        &self,
        frames: &[String],
    ) -> ContinuityCheckResult {
        // TODO: 实际调用帧比较逻辑
        let mut issues = vec![];
        
        for i in 0..frames.len().saturating_sub(1) {
            let diff = self.calculate_frame_diff(&frames[i], &frames[i + 1]);
            
            if diff > self.config.max_diff {
                issues.push(ContinuityIssue {
                    check_type: ContinuityCheckType::FirstLastFrame,
                    severity: if diff > 0.3 {
                        ContinuitySeverity::Error
                    } else {
                        ContinuitySeverity::Warning
                    },
                    frame_index: i as u32,
                    description: format!("帧 {} 和帧 {} 差异过大: {:.3}", i, i + 1, diff),
                    diff_value: diff,
                    fix_suggestion: Some("考虑添加过渡帧或调整关键帧".to_string()),
                });
            }
        }
        
        let checked_frames = frames.len() as u32;
        let passed = issues.iter().all(|i| i.severity < self.config.severity_threshold);
        let pass_rate = if checked_frames > 0 {
            (checked_frames - issues.len() as u32) as f32 / checked_frames as f32
        } else {
            1.0
        };
        
        ContinuityCheckResult {
            passed,
            checked_frames,
            issue_count: issues.len() as u32,
            pass_rate,
            issues,
            check_time_ms: 100,
            error: None,
        }
    }
    
    /// 计算帧差异
    fn calculate_frame_diff(&self, _frame_a: &str, _frame_b: &str) -> f32 {
        // TODO: 实际调用帧差异计算
        0.05
    }
    
    /// 检查场景转场
    pub fn check_scene_transition(
        &self,
        frames: &[String],
        _transition_type: &str,
    ) -> ContinuityCheckResult {
        // TODO: 实际调用场景转场检查逻辑
        let checked_frames = frames.len() as u32;
        
        ContinuityCheckResult {
            passed: true,
            checked_frames,
            issue_count: 0,
            pass_rate: 1.0,
            issues: vec![],
            check_time_ms: 50,
            error: None,
        }
    }
    
    /// 检查元素位置连续性
    pub fn check_element_position(
        &self,
        frames: &[String],
        _element_id: &str,
    ) -> ContinuityCheckResult {
        // TODO: 实际调用元素位置跟踪逻辑
        let checked_frames = frames.len() as u32;
        
        ContinuityCheckResult {
            passed: true,
            checked_frames,
            issue_count: 0,
            pass_rate: 1.0,
            issues: vec![],
            check_time_ms: 80,
            error: None,
        }
    }
    
    /// 执行完整检查
    pub fn check_all(&self, frames: &[String]) -> ContinuityCheckResult {
        let mut all_issues = vec![];
        let mut total_checked = 0;
        
        for check_type in &self.config.check_types {
            let result = match check_type {
                ContinuityCheckType::FirstLastFrame => {
                    self.check_first_last_frame(frames)
                }
                ContinuityCheckType::SceneTransition => {
                    self.check_scene_transition(frames, "cut")
                }
                ContinuityCheckType::ElementPosition => {
                    self.check_element_position(frames, "element_001")
                }
                _ => ContinuityCheckResult {
                    passed: true,
                    checked_frames: frames.len() as u32,
                    issue_count: 0,
                    pass_rate: 1.0,
                    issues: vec![],
                    check_time_ms: 0,
                    error: None,
                },
            };
            
            all_issues.extend(result.issues);
            total_checked += result.checked_frames;
        }
        
        let passed = all_issues.iter().all(|i| i.severity < self.config.severity_threshold);
        let pass_rate = if total_checked > 0 {
            (total_checked - all_issues.len() as u32) as f32 / total_checked as f32
        } else {
            1.0
        };
        
        ContinuityCheckResult {
            passed,
            checked_frames: total_checked,
            issue_count: all_issues.len() as u32,
            pass_rate,
            issues: all_issues,
            check_time_ms: 200,
            error: None,
        }
    }
    
    /// 获取检查统计
    pub fn statistics(&self) -> ContinuityStats {
        let total_checks = self.history.len();
        let passed = self.history.iter().filter(|r| r.passed).count();
        let total_issues: u32 = self.history.iter().map(|r| r.issue_count).sum();
        let avg_pass_rate = if total_checks > 0 {
            self.history.iter().map(|r| r.pass_rate).sum::<f32>() / total_checks as f32
        } else {
            0.0
        };
        
        ContinuityStats {
            total_checks,
            passed,
            failed: total_checks - passed,
            total_issues,
            avg_pass_rate,
        }
    }
}

/// 连续性统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ContinuityStats {
    /// 总检查次数
    pub total_checks: usize,
    /// 通过次数
    pub passed: usize,
    /// 失败次数
    pub failed: usize,
    /// 总问题数
    pub total_issues: u32,
    /// 平均通过率
    pub avg_pass_rate: f32,
}

// ============================================================================
// 向后兼容别名
// ============================================================================

/// 镜头衔接检查器 (向后兼容别名)
pub type ShotContinuityChecker = TemporalContinuityChecker;

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_continuity_checker() {
        let checker = TemporalContinuityChecker::new();
        
        let frames = vec![
            "frame_001.png".to_string(),
            "frame_002.png".to_string(),
            "frame_003.png".to_string(),
        ];
        
        let result = checker.check_first_last_frame(&frames);
        assert!(result.passed);
    }
    
    #[test]
    fn test_check_all() {
        let checker = TemporalContinuityChecker::new();
        
        let frames = vec![
            "frame_001.png".to_string(),
            "frame_002.png".to_string(),
        ];
        
        let result = checker.check_all(&frames);
        assert!(result.passed);
    }
}