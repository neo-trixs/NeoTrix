//! 转场梯度建议器
//!
//! 根据节奏快慢自动推荐转场类型
//! 慢节奏→渐变/虚入，快节奏→硬切/闪白

use serde::{Serialize, Deserialize};

// ============================================================================
// 转场类型
// ============================================================================

/// 转场类型枚举
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TransitionType {
    /// 硬切
    Cut,
    /// 淡入
    FadeIn,
    /// 淡出
    FadeOut,
    /// 虚入
    DissolveIn,
    /// 虚出
    DissolveOut,
    /// 闪白
    FlashWhite,
    /// 光斑转场
    LightLeak,
    /// 匹配剪辑
    MatchCut,
}

/// 转场适用场景
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct _TransitionUseCase {
    /// 转场类型
    pub transition_type: TransitionType,
    /// 适用节奏
    pub rhythm: _RhythmType,
    /// 适用场景
    pub scenarios: Vec<String>,
    /// 效果描述
    pub effect: String,
    /// 技术要求
    pub technical_requirements: Vec<String>,
}

/// 节奏类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub(crate) enum _RhythmType {
    /// 慢节奏 (铺垫/抒情)
    Slow,
    /// 中节奏 (过渡)
    Medium,
    /// 快节奏 (冲突/高潮)
    Fast,
}

// ============================================================================
// 转场梯度建议器
// ============================================================================

/// 转场梯度建议器
pub(crate) struct _TransitionGradientAdvisor;

impl _TransitionGradientAdvisor {
    /// 根据节奏类型推荐转场类型
    pub fn recommend(rhythm: _RhythmType) -> Vec<TransitionType> {
        match rhythm {
            _RhythmType::Slow => vec![
                TransitionType::FadeIn,
                TransitionType::FadeOut,
                TransitionType::DissolveIn,
                TransitionType::DissolveOut,
            ],
            _RhythmType::Medium => vec![
                TransitionType::DissolveIn,
                TransitionType::DissolveOut,
                TransitionType::MatchCut,
                TransitionType::LightLeak,
            ],
            _RhythmType::Fast => vec![
                TransitionType::Cut,
                TransitionType::FlashWhite,
                TransitionType::MatchCut,
            ],
        }
    }
    
    /// 获取转场使用案例
    pub(crate) fn _get_use_cases() -> Vec<_TransitionUseCase> {
        vec![
            _TransitionUseCase {
                transition_type: TransitionType::Cut,
                rhythm: _RhythmType::Fast,
                scenarios: vec!["快速节奏".to_string(), "场景割裂".to_string(), "紧张氛围".to_string()],
                effect: "无过渡直接切换".to_string(),
                technical_requirements: vec!["无需特殊处理".to_string()],
            },
            _TransitionUseCase {
                transition_type: TransitionType::FadeIn,
                rhythm: _RhythmType::Slow,
                scenarios: vec!["章节开始".to_string(), "时间跳转".to_string(), "回忆开始".to_string()],
                effect: "画面明暗过渡".to_string(),
                technical_requirements: vec!["渐变时长控制".to_string()],
            },
            _TransitionUseCase {
                transition_type: TransitionType::FadeOut,
                rhythm: _RhythmType::Slow,
                scenarios: vec!["章节结束".to_string(), "时间跳转".to_string(), "场景结束".to_string()],
                effect: "画面明暗过渡".to_string(),
                technical_requirements: vec!["渐变时长控制".to_string()],
            },
            _TransitionUseCase {
                transition_type: TransitionType::DissolveIn,
                rhythm: _RhythmType::Medium,
                scenarios: vec!["回忆".to_string(), "梦境".to_string(), "意识流".to_string()],
                effect: "画面模糊后清晰".to_string(),
                technical_requirements: vec!["模糊强度控制".to_string()],
            },
            _TransitionUseCase {
                transition_type: TransitionType::DissolveOut,
                rhythm: _RhythmType::Medium,
                scenarios: vec!["回忆结束".to_string(), "梦境结束".to_string()],
                effect: "画面模糊".to_string(),
                technical_requirements: vec!["模糊强度控制".to_string()],
            },
            _TransitionUseCase {
                transition_type: TransitionType::FlashWhite,
                rhythm: _RhythmType::Fast,
                scenarios: vec!["魔法生效".to_string(), "情绪爆发".to_string(), "强烈冲击".to_string()],
                effect: "特殊视觉过渡".to_string(),
                technical_requirements: vec!["闪白强度控制".to_string(), "音效同步".to_string()],
            },
            _TransitionUseCase {
                transition_type: TransitionType::LightLeak,
                rhythm: _RhythmType::Medium,
                scenarios: vec!["梦幻氛围".to_string(), "浪漫场景".to_string(), "时间流逝".to_string()],
                effect: "光斑效果过渡".to_string(),
                technical_requirements: vec!["光斑颜色匹配".to_string()],
            },
            _TransitionUseCase {
                transition_type: TransitionType::MatchCut,
                rhythm: _RhythmType::Fast,
                scenarios: vec!["动作匹配".to_string(), "形状匹配".to_string(), "色彩匹配".to_string()],
                effect: "匹配剪辑".to_string(),
                technical_requirements: vec!["前后画面元素匹配".to_string()],
            },
        ]
    }
    
    /// 根据内容类型推荐转场
    pub(crate) fn _recommend_for_content(content_type: &str) -> Vec<TransitionType> {
        match content_type {
            "回忆" | "梦境" => vec![TransitionType::DissolveIn, TransitionType::DissolveOut],
            "打斗" | "冲突" => vec![TransitionType::Cut, TransitionType::FlashWhite],
            "抒情" | "温馨" => vec![TransitionType::FadeIn, TransitionType::FadeOut],
            "魔法" | "特效" => vec![TransitionType::FlashWhite, TransitionType::LightLeak],
            _ => vec![TransitionType::Cut, TransitionType::DissolveIn],
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
    fn test_recommend_by_rhythm() {
        let slow_recs = _TransitionGradientAdvisor::recommend(_RhythmType::Slow);
        assert!(slow_recs.contains(&TransitionType::FadeIn));
        
        let fast_recs = _TransitionGradientAdvisor::recommend(_RhythmType::Fast);
        assert!(fast_recs.contains(&TransitionType::Cut));
    }
    
    #[test]
    fn test_recommend_for_content() {
        let fight_recs = _TransitionGradientAdvisor::_recommend_for_content("打斗");
        assert!(fight_recs.contains(&TransitionType::Cut));
        
        let dream_recs = _TransitionGradientAdvisor::_recommend_for_content("梦境");
        assert!(dream_recs.contains(&TransitionType::DissolveIn));
    }
    
    #[test]
    fn test_use_cases() {
        let cases = _TransitionGradientAdvisor::_get_use_cases();
        assert!(cases.len() > 0);
    }
}