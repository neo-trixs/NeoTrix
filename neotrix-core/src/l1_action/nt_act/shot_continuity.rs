//! 镜头衔接模块
//!
//! 实现首尾帧链接、场景转场连续性
//! 确保镜头之间的视觉连贯性

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// ============================================================================
// 镜头衔接定义
// ============================================================================

/// 衔接类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ContinuityType {
    /// 首尾帧链接 (上一镜尾帧 = 下一镜首帧)
    FirstLastFrameLink,
    /// 场景连续性
    SceneContinuity,
    /// 角色连续性
    CharacterContinuity,
    /// 动作连续性
    ActionContinuity,
    /// 情绪连续性
    EmotionContinuity,
}

/// 转场类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TransitionType {
    /// 硬切
    HardCut,
    /// 渐变
    Fade,
    /// 叠化
    Dissolve,
    /// 滑动
    Slide,
    /// 缩放
    Zoom,
    /// 擦除
    Wipe,
    /// 无转场
    None,
}

/// 镜头衔接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinuityConfig {
    /// 是否启用首尾帧链接
    pub enable_first_last_frame_link: bool,
    /// 是否启用场景连续性检查
    pub enable_scene_continuity: bool,
    /// 是否启用角色连续性检查
    pub enable_character_continuity: bool,
    /// 默认转场类型
    pub default_transition: TransitionType,
    /// 转场时长 (秒)
    pub transition_duration: f32,
    /// 连续性阈值 (0.0-1.0)
    pub continuity_threshold: f32,
}

/// 镜头衔接结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinuityResult {
    /// 是否通过
    pub passed: bool,
    /// 衔接类型
    pub continuity_type: ContinuityType,
    /// 连续性分数 (0.0-1.0)
    pub continuity_score: f32,
    /// 问题描述
    pub issues: Vec<String>,
    /// 建议
    pub suggestions: Vec<String>,
}

/// 镜头衔接检查器
pub struct ShotContinuityChecker {
    /// 配置
    config: ContinuityConfig,
    /// 检查历史
    history: Vec<ContinuityResult>,
}

impl ShotContinuityChecker {
    /// 创建检查器
    pub fn new() -> Self {
        Self {
            config: ContinuityConfig {
                enable_first_last_frame_link: true,
                enable_scene_continuity: true,
                enable_character_continuity: true,
                default_transition: TransitionType::HardCut,
                transition_duration: 0.5,
                continuity_threshold: 0.8,
            },
            history: vec![],
        }
    }
    
    /// 使用配置创建
    pub fn with_config(config: ContinuityConfig) -> Self {
        Self {
            config,
            history: vec![],
        }
    }
    
    /// 检查首尾帧链接
    pub fn check_first_last_frame_link(
        &mut self,
        prev_shot_end_frame: &str,
        current_shot_start_frame: &str,
    ) -> ContinuityResult {
        // TODO: 实际调用图像相似度检查
        let score = 0.92; // 模拟检查结果
        
        let result = ContinuityResult {
            passed: score >= self.config.continuity_threshold,
            continuity_type: ContinuityType::FirstLastFrameLink,
            continuity_score: score,
            issues: if score < self.config.continuity_threshold {
                vec!["首尾帧不一致".to_string()]
            } else {
                vec![]
            },
            suggestions: if score < self.config.continuity_threshold {
                vec!["调整尾帧或首帧描述以保持一致".to_string()]
            } else {
                vec![]
            },
        };
        
        self.history.push(result.clone());
        result
    }
    
    /// 检查场景连续性
    pub fn check_scene_continuity(
        &mut self,
        prev_scene: &str,
        current_scene: &str,
        transition_type: TransitionType,
    ) -> ContinuityResult {
        let passed = prev_scene == current_scene || transition_type != TransitionType::HardCut;
        
        let result = ContinuityResult {
            passed,
            continuity_type: ContinuityType::SceneContinuity,
            continuity_score: if passed { 1.0 } else { 0.5 },
            issues: if !passed {
                vec![format!("场景从{}切换到{}，缺少转场", prev_scene, current_scene)]
            } else {
                vec![]
            },
            suggestions: if !passed {
                vec!["添加转场效果以平滑场景切换".to_string()]
            } else {
                vec![]
            },
        };
        
        self.history.push(result.clone());
        result
    }
    
    /// 检查角色连续性
    pub fn check_character_continuity(
        &mut self,
        prev_characters: &[String],
        current_characters: &[String],
    ) -> ContinuityResult {
        let prev_set: std::collections::HashSet<&String> = prev_characters.iter().collect();
        let current_set: std::collections::HashSet<&String> = current_characters.iter().collect();
        
        let common: Vec<&String> = prev_set.intersection(&current_set).copied().collect();
        let score = if prev_set.is_empty() && current_set.is_empty() {
            1.0
        } else if prev_set.is_empty() || current_set.is_empty() {
            0.5
        } else {
            common.len() as f32 / prev_set.len().max(current_set.len()) as f32
        };
        
        let result = ContinuityResult {
            passed: score >= self.config.continuity_threshold,
            continuity_type: ContinuityType::CharacterContinuity,
            continuity_score: score,
            issues: if score < self.config.continuity_threshold {
                vec!["角色变化过大".to_string()]
            } else {
                vec![]
            },
            suggestions: if score < self.config.continuity_threshold {
                vec!["确保主要角色在连续镜头中出现".to_string()]
            } else {
                vec![]
            },
        };
        
        self.history.push(result.clone());
        result
    }
    
    /// 检查动作连续性
    pub fn check_action_continuity(
        &mut self,
        prev_action: Option<&str>,
        current_action: Option<&str>,
    ) -> ContinuityResult {
        let score = match (prev_action, current_action) {
            (Some(_), Some(_)) => 0.85,
            (Some(_), None) => 0.7,
            (None, Some(_)) => 0.7,
            (None, None) => 1.0,
        };
        
        let result = ContinuityResult {
            passed: score >= self.config.continuity_threshold,
            continuity_type: ContinuityType::ActionContinuity,
            continuity_score: score,
            issues: vec![],
            suggestions: vec![],
        };
        
        self.history.push(result.clone());
        result
    }
    
    /// 执行完整检查
    pub fn check_full_continuity(
        &mut self,
        prev_shot: &StoryboardShot,
        current_shot: &StoryboardShot,
    ) -> Vec<ContinuityResult> {
        let mut results = vec![];
        
        if self.config.enable_first_last_frame_link {
            if let (Some(ref prev_end), Some(ref curr_start)) = (&prev_shot.end_frame_desc, &current_shot.start_frame_desc) {
                results.push(self.check_first_last_frame_link(prev_end, curr_start));
            }
        }
        
        if self.config.enable_scene_continuity {
            results.push(self.check_scene_continuity(
                &prev_shot.scene,
                &current_shot.scene,
                self.config.default_transition,
            ));
        }
        
        if self.config.enable_character_continuity {
            results.push(self.check_character_continuity(
                &prev_shot.characters,
                &current_shot.characters,
            ));
        }
        
        results.push(self.check_action_continuity(
            prev_shot.action.as_deref(),
            current_shot.action.as_deref(),
        ));
        
        results
    }
    
    /// 获取检查统计
    pub fn statistics(&self) -> ContinuityStats {
        let total_checks = self.history.len();
        let passed_checks = self.history.iter().filter(|r| r.passed).count();
        let avg_score = if total_checks > 0 {
            self.history.iter().map(|r| r.continuity_score).sum::<f32>() / total_checks as f32
        } else {
            0.0
        };
        
        ContinuityStats {
            total_checks,
            passed_checks,
            failed_checks: total_checks - passed_checks,
            avg_continuity_score: avg_score,
            pass_rate: if total_checks > 0 {
                passed_checks as f32 / total_checks as f32
            } else {
                0.0
            },
        }
    }
}

/// 简化的镜头信息
#[derive(Debug, Clone)]
pub struct StoryboardShot {
    pub id: String,
    pub scene: String,
    pub characters: Vec<String>,
    pub action: Option<String>,
    pub start_frame_desc: Option<String>,
    pub end_frame_desc: Option<String>,
}

/// 衔接统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinuityStats {
    /// 总检查次数
    pub total_checks: usize,
    /// 通过次数
    pub passed_checks: usize,
    /// 失败次数
    pub failed_checks: usize,
    /// 平均连续性分数
    pub avg_continuity_score: f32,
    /// 通过率
    pub pass_rate: f32,
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_shot_continuity_checker() {
        let mut checker = ShotContinuityChecker::new();
        
        // 检查首尾帧链接
        let result = checker.check_first_last_frame_link("尾帧描述", "首帧描述");
        assert!(result.passed);
        
        // 检查场景连续性
        let result = checker.check_scene_continuity("教室", "教室", TransitionType::HardCut);
        assert!(result.passed);
        
        // 检查角色连续性
        let result = checker.check_character_continuity(
            &["角色A".to_string()],
            &["角色A".to_string(), "角色B".to_string()],
        );
        assert!(result.passed);
        
        let stats = checker.statistics();
        assert_eq!(stats.total_checks, 3);
        assert_eq!(stats.passed_checks, 3);
    }
    
    #[test]
    fn test_scene_continuity_failure() {
        let mut checker = ShotContinuityChecker::new();
        
        let result = checker.check_scene_continuity("教室", "户外", TransitionType::HardCut);
        assert!(!result.passed);
        assert!(!result.issues.is_empty());
    }
}