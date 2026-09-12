//! 动态-音效同步模式库
//!
//! 管理动态效果和音效的同步模式
//! 支持动态漫制作中的音画同步

use serde::{Serialize, Deserialize};

// ============================================================================
// 同步模式定义
// ============================================================================

/// 动态-音效同步模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _AudioSyncPattern {
    /// 模式ID
    pub id: String,
    /// 模式名称
    pub name: String,
    /// 模式描述
    pub description: String,
    /// 适用动态等级
    pub dynamic_level: String,
    /// 适用音效类型
    pub audio_type: AudioType,
    /// 同步时机
    pub sync_timing: _SyncTiming,
    /// 技术参数
    pub technical_params: _TechnicalParams,
    /// 使用场景
    pub usage_scenarios: Vec<String>,
}

/// 音效类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AudioType {
    /// 环境音
    Ambient,
    /// 动作音效
    Action,
    /// 配乐
    Music,
    /// 对白
    Dialogue,
    /// 特殊音效
    Special,
}

/// 同步时机
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _SyncTiming {
    /// 开始偏移 (毫秒)
    pub start_offset_ms: i32,
    /// 结束偏移 (毫秒)
    pub end_offset_ms: i32,
    /// 是否需要精确同步
    pub precise_sync: bool,
    /// 同步精度 (毫秒)
    pub precision_ms: u32,
}

/// 技术参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _TechnicalParams {
    /// 淡入时长 (毫秒)
    pub fade_in_ms: u32,
    /// 淡出时长 (毫秒)
    pub fade_out_ms: u32,
    /// 音量 (0.0-1.0)
    pub volume: f32,
    /// 是否循环
    pub loop_playback: bool,
    /// 优先级 (0-10)
    pub priority: u32,
}

// ============================================================================
// 预设同步模式
// ============================================================================

impl _AudioSyncPattern {
    /// 获取所有预设同步模式
    pub fn _preset_patterns() -> Vec<_AudioSyncPattern> {
        vec![
            // 动作音效同步
            _AudioSyncPattern {
                id: "action_hit".to_string(),
                name: "击打同步".to_string(),
                description: "动作击打与音效精确同步".to_string(),
                dynamic_level: "Macro".to_string(),
                audio_type: AudioType::Action,
                sync_timing: _SyncTiming {
                    start_offset_ms: 0,
                    end_offset_ms: 500,
                    precise_sync: true,
                    precision_ms: 16,
                },
                technical_params: _TechnicalParams {
                    fade_in_ms: 0,
                    fade_out_ms: 100,
                    volume: 0.8,
                    loop_playback: false,
                    priority: 8,
                },
                usage_scenarios: vec!["打斗".to_string(), "击打".to_string(), "碰撞".to_string()],
            },
            // 环境音同步
            _AudioSyncPattern {
                id: "ambient_wind".to_string(),
                name: "风声同步".to_string(),
                description: "环境风声与画面元素同步".to_string(),
                dynamic_level: "Micro".to_string(),
                audio_type: AudioType::Ambient,
                sync_timing: _SyncTiming {
                    start_offset_ms: -200,
                    end_offset_ms: 200,
                    precise_sync: false,
                    precision_ms: 100,
                },
                technical_params: _TechnicalParams {
                    fade_in_ms: 500,
                    fade_out_ms: 500,
                    volume: 0.3,
                    loop_playback: true,
                    priority: 2,
                },
                usage_scenarios: vec!["户外".to_string(), "高处".to_string(), "空旷".to_string()],
            },
            // 配乐节奏同步
            _AudioSyncPattern {
                id: "music_beat".to_string(),
                name: "配乐节拍同步".to_string(),
                description: "画面动作与配乐节拍同步".to_string(),
                dynamic_level: "Medium".to_string(),
                audio_type: AudioType::Music,
                sync_timing: _SyncTiming {
                    start_offset_ms: 0,
                    end_offset_ms: 0,
                    precise_sync: true,
                    precision_ms: 32,
                },
                technical_params: _TechnicalParams {
                    fade_in_ms: 200,
                    fade_out_ms: 200,
                    volume: 0.6,
                    loop_playback: false,
                    priority: 5,
                },
                usage_scenarios: vec!["卡点".to_string(), "节奏感".to_string(), "舞蹈".to_string()],
            },
            // 对白口型同步
            _AudioSyncPattern {
                id: "dialogue_lip_sync".to_string(),
                name: "对白口型同步".to_string(),
                description: "角色对白与口型动画同步".to_string(),
                dynamic_level: "Medium".to_string(),
                audio_type: AudioType::Dialogue,
                sync_timing: _SyncTiming {
                    start_offset_ms: 0,
                    end_offset_ms: 0,
                    precise_sync: true,
                    precision_ms: 16,
                },
                technical_params: _TechnicalParams {
                    fade_in_ms: 0,
                    fade_out_ms: 0,
                    volume: 1.0,
                    loop_playback: false,
                    priority: 10,
                },
                usage_scenarios: vec!["对话".to_string(), "独白".to_string(), "旁白".to_string()],
            },
            // 转场音效同步
            _AudioSyncPattern {
                id: "transition_swoosh".to_string(),
                name: "转场音效同步".to_string(),
                description: "转场效果与音效同步".to_string(),
                dynamic_level: "Medium".to_string(),
                audio_type: AudioType::Special,
                sync_timing: _SyncTiming {
                    start_offset_ms: -100,
                    end_offset_ms: 100,
                    precise_sync: true,
                    precision_ms: 32,
                },
                technical_params: _TechnicalParams {
                    fade_in_ms: 50,
                    fade_out_ms: 100,
                    volume: 0.7,
                    loop_playback: false,
                    priority: 6,
                },
                usage_scenarios: vec!["硬切".to_string(), "渐变".to_string(), "闪白".to_string()],
            },
        ]
    }
    
    /// 根据动态等级查找同步模式
    pub fn _find_by_dynamic_level(level: &str) -> Vec<_AudioSyncPattern> {
        Self::_preset_patterns()
            .into_iter()
            .filter(|p| p.dynamic_level == level)
            .collect()
    }
    
    /// 根据音效类型查找同步模式
    pub fn _find_by_audio_type(audio_type: AudioType) -> Vec<_AudioSyncPattern> {
        Self::_preset_patterns()
            .into_iter()
            .filter(|p| p.audio_type == audio_type)
            .collect()
    }
    
    /// 根据使用场景查找同步模式
    pub fn _find_by_scenario(scenario: &str) -> Vec<_AudioSyncPattern> {
        Self::_preset_patterns()
            .into_iter()
            .filter(|p| p.usage_scenarios.iter().any(|s| s.contains(scenario)))
            .collect()
    }
}

// ============================================================================
// 同步检查器
// ============================================================================

/// 同步检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _SyncCheckResult {
    /// 是否同步
    pub is_synced: bool,
    /// 同步误差 (毫秒)
    pub sync_error_ms: f32,
    /// 建议
    pub suggestion: String,
}

/// 同步检查器
pub struct _SyncChecker;

impl _SyncChecker {
    /// 检查动态和音效是否同步
    pub fn _check_sync(
        dynamic_start_ms: u32,
        dynamic_end_ms: u32,
        audio_start_ms: u32,
        audio_end_ms: u32,
        pattern: &_AudioSyncPattern,
    ) -> _SyncCheckResult {
        let start_diff = dynamic_start_ms as i32 - audio_start_ms as i32 + pattern.sync_timing.start_offset_ms;
        let end_diff = dynamic_end_ms as i32 - audio_end_ms as i32 + pattern.sync_timing.end_offset_ms;
        
        let avg_error = ((start_diff.abs() + end_diff.abs()) / 2) as f32;
        
        let is_synced = if pattern.sync_timing.precise_sync {
            avg_error <= pattern.sync_timing.precision_ms as f32
        } else {
            avg_error <= 200.0 // 宽松模式
        };
        
        let suggestion = if is_synced {
            "同步良好".to_string()
        } else if start_diff > 0 {
            format!("音效需要提前 {}ms", start_diff)
        } else {
            format!("音效需要延后 {}ms", start_diff.abs())
        };
        
        _SyncCheckResult {
            is_synced,
            sync_error_ms: avg_error,
            suggestion,
        }
    }
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_preset_patterns() {
        let patterns = _AudioSyncPattern::_preset_patterns();
        assert!(patterns.len() > 0);
        
        let hit_pattern = patterns.iter().find(|p| p.id == "action_hit");
        assert!(hit_pattern.is_some());
    }
    
    #[test]
    fn test_find_by_dynamic_level() {
        let patterns = _AudioSyncPattern::_find_by_dynamic_level("Macro");
        assert!(patterns.iter().all(|p| p.dynamic_level == "Macro"));
    }
    
    #[test]
    fn test_find_by_audio_type() {
        let patterns = _AudioSyncPattern::_find_by_audio_type(AudioType::Ambient);
        assert!(patterns.iter().all(|p| p.audio_type == AudioType::Ambient));
    }
    
    #[test]
    fn test_sync_checker() {
        let pattern = _AudioSyncPattern {
            id: "test".to_string(),
            name: "测试".to_string(),
            description: "测试".to_string(),
            dynamic_level: "Macro".to_string(),
            audio_type: AudioType::Action,
            sync_timing: _SyncTiming {
                start_offset_ms: 0,
                end_offset_ms: 0,
                precise_sync: true,
                precision_ms: 16,
            },
            technical_params: _TechnicalParams {
                fade_in_ms: 0,
                fade_out_ms: 0,
                volume: 1.0,
                loop_playback: false,
                priority: 10,
            },
            usage_scenarios: vec![],
        };
        
        // 同步
        let result = _SyncChecker::_check_sync(100, 200, 100, 200, &pattern);
        assert!(result.is_synced);
        
        // 不同步
        let result = _SyncChecker::_check_sync(100, 200, 150, 250, &pattern);
        assert!(!result.is_synced);
    }
}