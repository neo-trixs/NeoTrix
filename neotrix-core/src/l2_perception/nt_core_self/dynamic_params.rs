//! 动态参数元数据规范
//! 映射动态漫技能中的动态参数到 NeoTrix NT-感知系统//! 统一速度、幅度、频率三要素的物理单位和约束规范

use serde::{Serialize, Deserialize};
use std::fmt;

// ============================================================================
// 动态参数元数据结构

/// 统一动态参数元数据
/// 对应动态漫专属设定中的核心动作习惯、微动态特征、声音细节
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DynamicParams {
    /// 速度: 动作完成所需时间 (s/动作)
    pub speed: f32,
    
    /// 幅度: 动作的物理幅度
    /// 单位: 度 (°) 或 像素 (px)
    pub amplitude: f32,
    
    /// 频率: 重复动作的频率
    /// 单位: 次/分
    pub frequency: f32,
    
    /// 物理单位枚举
    pub unit: ParamUnit,
    
    /// 是否在物理边界内有效
    pub valid: bool,
}

/// 动态参数物理单位
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParamUnit {
    /// 秒每动作
    SecondsPerAction,
    /// 度
    Degrees,
    /// 像素
    Pixels,
    /// Hertz (每秒次数)
    Hertz,
}

impl fmt::Display for ParamUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParamUnit::SecondsPerAction => write!(f, "s/动作"),
            ParamUnit::Degrees => write!(f, "°"),
            ParamUnit::Pixels => write!(f, "px"),
            ParamUnit::Hertz => write!(f, "Hz"),
        }
    }
}

impl ParamUnit {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "s/动作" | "seconds_per_action" => ParamUnit::SecondsPerAction,
            "°" | "degrees" | "deg" => ParamUnit::Degrees,
            "px" | "pixels" => ParamUnit::Pixels,
            "hz" | "hertz" => ParamUnit::Hertz,
            _ => ParamUnit::SecondsPerAction,
        }
    }
    
    pub fn description(&self) -> &'static str {
        match self {
            ParamUnit::SecondsPerAction => "每个动作耗时",
            ParamUnit::Degrees => "角度幅度",
            ParamUnit::Pixels => "像素位移",
            ParamUnit::Hertz => "每秒重复次数",
        }
    }
}

// ============================================================================
// 动态等级对应情感强度

/// 爽点动态等级对应情感强度
/// 映射到 EmotionLabel 统一枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScalingRating {
    Micro,
    Medium,
    Macro,
}

impl ScalingRating {
    pub fn from_dynamic_level(level: &str) -> Self {
        match level {
            "微" | "基础" | "Micro" => ScalingRating::Micro,
            "中" | "中度" | "Medium" => ScalingRating::Medium,
            "强" | "特效" | "Macro" => ScalingRating::Macro,
            _ => ScalingRating::Micro,
        }
    }
    
    pub fn description(&self) -> &'static str {
        match self {
            ScalingRating::Micro => "头发轻飘/眼皮颤动/手指微动/镜头缓慢推拉",
            ScalingRating::Medium => "肢体位移/物体小幅运动/镜头切换(溶解/切出)",
            ScalingRating::Macro => "打斗动作/爆炸特效/快速转场(闪白/震动)",
        }
    }
    
    pub fn emotion_level(&self) -> &'static str {
        match self {
            ScalingRating::Micro => "微情绪",
            ScalingRating::Medium => "中情绪",
            ScalingRating::Macro => "强情绪",
        }
    }
}

// ============================================================================
// 验证与约束

impl DynamicParams {
    pub fn validate(&self) -> bool {
        self.speed > 0.0 && self.amplitude >= 0.0 && self.frequency >= 0.0
    }
    
    pub fn is_micro(&self) -> bool {
        self.speed <= 1.0 && self.amplitude <= 10.0 && self.frequency <= 2.0
    }
    
    pub fn is_middle(&self) -> bool {
        (1.0 < self.speed && self.speed <= 3.0)
            || (10.0 < self.amplitude && self.amplitude <= 50.0)
            || (2.0 < self.frequency && self.frequency <= 10.0)
    }
    
    pub fn is_strong(&self) -> bool {
        self.speed > 3.0 || self.amplitude > 50.0 || self.frequency > 10.0
    }
    
    pub fn to_scaling_rating(&self) -> ScalingRating {
        if self.is_micro() {
            ScalingRating::Micro
        } else if self.is_middle() {
            ScalingRating::Medium
        } else {
            ScalingRating::Macro
        }
    }
}

// ============================================================================
// 标准预设

pub const MICRO_PRESET: DynamicParams = DynamicParams {
    speed: 0.5,
    amplitude: 5.0,
    frequency: 1.0,
    unit: ParamUnit::Degrees,
    valid: true,
};

pub const MIDDLE_PRESET: DynamicParams = DynamicParams {
    speed: 2.0,
    amplitude: 30.0,
    frequency: 3.0,
    unit: ParamUnit::Degrees,
    valid: true,
};

pub const STRONG_PRESET: DynamicParams = DynamicParams {
    speed: 5.0,
    amplitude: 100.0,
    frequency: 15.0,
    unit: ParamUnit::Degrees,
    valid: true,
};

// ============================================================================
// 转换实用函数

pub fn parse_from_string(description: &str) -> Option<DynamicParams> {
    let mut params = DynamicParams {
        speed: 1.0,
        amplitude: 10.0,
        frequency: 1.0,
        unit: ParamUnit::Degrees,
        valid: false,
    };
    
    for line in description.lines() {
        let line = line.trim();
        if line.starts_with("速度") || line.starts_with("速度：") {
            let val: f32 = line.split_whitespace()
                .nth(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(1.0);
            params.speed = val;
        } else if line.starts_with("幅度") || line.starts_with("幅度：") {
            let val: f32 = line.split_whitespace()
                .nth(1)
                .and_then(|s| s.trim_end_matches('°').parse().ok())
                .unwrap_or(10.0);
            params.amplitude = val;
            if line.contains("°") || line.contains("度") {
                params.unit = ParamUnit::Degrees;
            }
        } else if line.starts_with("频率") || line.starts_with("频率：") {
            let val: f32 = line.split_whitespace()
                .nth(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(1.0);
            params.frequency = val;
        }
    }
    
    if params.validate() {
        Some(params)
    } else {
        None
    }
}

pub fn format_as_description(params: &DynamicParams) -> String {
    format!(
        "速度 {:.1}s/动作, 幅度 {:.1}{} , 频率 {:.1}{}每动作",
        params.speed,
        params.amplitude,
        params.unit,
        params.frequency,
        params.unit
    )
}

// ============================================================================
// 测试模块

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_param_unit_display() {
        assert_eq!(ParamUnit::Degrees.to_string(), "°");
        assert_eq!(ParamUnit::Pixels.to_string(), "px");
        assert_eq!(ParamUnit::Hertz.to_string(), "Hz");
        assert_eq!(ParamUnit::SecondsPerAction.to_string(), "s/动作");
    }
    
    #[test]
    fn test_param_unit_from_str() {
        assert_eq!(ParamUnit::from_str("degrees"), ParamUnit::Degrees);
        assert_eq!(ParamUnit::from_str("°"), ParamUnit::Degrees);
        assert_eq!(ParamUnit::from_str("px"), ParamUnit::Pixels);
        assert_eq!(ParamUnit::from_str("hz"), ParamUnit::Hertz);
        assert_eq!(ParamUnit::from_str("s/动作"), ParamUnit::SecondsPerAction);
    }
    
    #[test]
    fn test_dynamic_params_validation() {
        let valid = DynamicParams {
            speed: 1.0,
            amplitude: 30.0,
            frequency: 2.0,
            unit: ParamUnit::Degrees,
            valid: true,
        };
        assert!(valid.validate());
        
        let invalid = DynamicParams {
            speed: -1.0,
            amplitude: 30.0,
            frequency: 2.0,
            unit: ParamUnit::Degrees,
            valid: false,
        };
        assert!(!invalid.validate());
    }
    
    #[test]
    fn test_scaling_rating() {
        assert_eq!(ScalingRating::from_dynamic_level("微"), ScalingRating::Micro);
        assert_eq!(ScalingRating::from_dynamic_level("中"), ScalingRating::Medium);
        assert_eq!(ScalingRating::from_dynamic_level("强"), ScalingRating::Macro);
        assert_eq!(ScalingRating::from_dynamic_level("未知"), ScalingRating::Micro);
    }
    
    #[test]
    fn test_micro_preset() {
        assert!(MICRO_PRESET.validate());
        assert!(MICRO_PRESET.is_micro());
        assert!(!MICRO_PRESET.is_middle());
        assert!(!MICRO_PRESET.is_strong());
    }
    
    #[test]
    fn test_strong_preset() {
        assert!(STRONG_PRESET.validate());
        assert!(!STRONG_PRESET.is_micro());
        assert!(!STRONG_PRESET.is_middle());
        assert!(STRONG_PRESET.is_strong());
    }
    
    #[test]
    fn test_parse_from_string() {
        let desc = "速度0.5s/动作, 幅度30°, 频率1次/分";
        let params = parse_from_string(desc);
        assert!(params.is_some());
        let p = params.unwrap();
        assert!(p.validate());
        assert_eq!(p.speed, 0.5);
        assert_eq!(p.amplitude, 30.0);
        assert_eq!(p.frequency, 1.0);
        assert_eq!(p.unit, ParamUnit::Degrees);
    }
    
    #[test]
    fn test_format_as_description() {
        let params = DynamicParams {
            speed: 2.0,
            amplitude: 30.0,
            frequency: 3.0,
            unit: ParamUnit::Degrees,
            valid: true,
        };
        let desc = format_as_description(&params);
        assert!(desc.contains("2.0"));
        assert!(desc.contains("30"));
        assert!(desc.contains("°"));
    }
}