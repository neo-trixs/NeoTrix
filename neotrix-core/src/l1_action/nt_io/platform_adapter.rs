//! 平台适配器统一接口
//!
//! 只定义 trait 和注册中心，不进行第三方平台适配
//! 具体适配由外部调用者实现，NeoTrix 只提供统一接口管理

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// ============================================================================
// 平台适配器 trait (统一接口)
// ============================================================================

/// 平台适配器 trait
/// 外部调用者实现此 trait 以对接 NeoTrix
/// NeoTrix 内部不依赖具体平台实现
pub trait PlatformAdapter: Send + Sync {
    /// 将内部表示转换为平台特定格式
    fn convert_to_platform(&self, internal: &InternalPrompt) -> PlatformPrompt;
    
    /// 将平台返回转换为内部表示
    fn convert_from_platform(&self, platform: &PlatformPrompt) -> InternalPrompt;
    
    /// 平台能力查询
    fn capabilities(&self) -> PlatformCapabilities;
    
    /// 平台名称
    fn name(&self) -> &str;
}

// ============================================================================
// 内部表示 (NeoTrix 统一数据结构)
// ============================================================================

/// 内部统一提示词表示
/// 所有平台适配都基于此结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalPrompt {
    /// 风格描述
    pub style: String,
    /// 时长 (秒)
    pub duration: f32,
    /// 画幅比例
    pub aspect_ratio: AspectRatio,
    /// 氛围
    pub atmosphere: String,
    /// 时间轴分段
    pub segments: Vec<TimeSegment>,
    /// 声音设计
    pub sound: SoundDesign,
    /// 参考素材
    pub references: Vec<ReferenceMaterial>,
}

/// 画幅比例
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AspectRatio {
    /// 16:9 横屏
    Landscape,
    /// 9:16 竖屏
    Portrait,
    /// 2.35:1 电影宽屏
    Cinematic,
    /// 1:1 正方形
    Square,
}

/// 时间轴分段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSegment {
    /// 开始时间 (秒)
    pub start: f32,
    /// 结束时间 (秒)
    pub end: f32,
    /// 镜头景别
    pub shot_type: ShotType,
    /// 画面描述
    pub visual: String,
    /// 动作描述
    pub action: String,
    /// 特效描述
    pub effects: String,
}

/// 镜头景别
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ShotType {
    /// 远景
    Wide,
    /// 全景
    Full,
    /// 中景
    Medium,
    /// 近景
    CloseUp,
    /// 特写
    ExtremeCloseUp,
}

/// 声音设计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundDesign {
    /// 配乐风格
    pub music_style: String,
    /// 音效列表
    pub effects: Vec<String>,
    /// 对白/旁白
    pub dialogue: String,
}

/// 参考素材
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceMaterial {
    /// 素材类型
    pub material_type: MaterialType,
    /// 素材描述
    pub description: String,
    /// 用途说明
    pub purpose: String,
}

/// 素材类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum MaterialType {
    /// 图片
    Image,
    /// 视频
    Video,
    /// 音频
    Audio,
}

// ============================================================================
// 平台输出表示
// ============================================================================

/// 平台特定格式提示词
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformPrompt {
    /// 平台名称
    pub platform: String,
    /// 格式化后的提示词
    pub formatted: String,
    /// 平台特定元数据
    pub metadata: HashMap<String, String>,
}

/// 平台能力描述
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformCapabilities {
    /// 最大文件数
    pub max_files: usize,
    /// 最大视频时长 (秒)
    pub max_video_duration: f32,
    /// 最大音频时长 (秒)
    pub max_audio_duration: f32,
    /// 支持的素材格式
    pub supported_formats: Vec<String>,
    /// 支持的画幅比例
    pub supported_aspect_ratios: Vec<AspectRatio>,
}

// ============================================================================
// 适配器注册中心
// ============================================================================

/// 适配器注册中心
/// 管理所有已注册的平台适配器
pub struct PlatformRegistry {
    adapters: HashMap<String, Box<dyn PlatformAdapter>>,
}

impl PlatformRegistry {
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
        }
    }
    
    /// 注册适配器
    pub fn register(&mut self, adapter: Box<dyn PlatformAdapter>) {
        let name = adapter.name().to_string();
        self.adapters.insert(name, adapter);
    }
    
    /// 获取适配器
    pub fn get(&self, platform_name: &str) -> Option<&dyn PlatformAdapter> {
        self.adapters.get(platform_name).map(|a| a.as_ref())
    }
    
    /// 列出所有已注册平台
    pub fn list_platforms(&self) -> Vec<&str> {
        self.adapters.keys().map(|s| s.as_str()).collect()
    }
    
    /// 转换为平台格式
    pub fn convert_to(&self, platform: &str, prompt: &InternalPrompt) -> Option<PlatformPrompt> {
        self.get(platform).map(|adapter| adapter.convert_to_platform(prompt))
    }
    
    /// 从平台格式转换
    pub fn convert_from(&self, platform: &str, platform_prompt: &PlatformPrompt) -> Option<InternalPrompt> {
        self.get(platform).map(|adapter| adapter.convert_from_platform(platform_prompt))
    }
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试用适配器实现
    struct TestAdapter;
    
    impl PlatformAdapter for TestAdapter {
        fn convert_to_platform(&self, internal: &InternalPrompt) -> PlatformPrompt {
            PlatformPrompt {
                platform: "test".to_string(),
                formatted: format!("TEST:{}", internal.style),
                metadata: HashMap::new(),
            }
        }
        
        fn convert_from_platform(&self, platform: &PlatformPrompt) -> InternalPrompt {
            InternalPrompt {
                style: platform.formatted.clone(),
                duration: 15.0,
                aspect_ratio: AspectRatio::Landscape,
                atmosphere: "default".to_string(),
                segments: vec![],
                sound: SoundDesign {
                    music_style: "default".to_string(),
                    effects: vec![],
                    dialogue: String::new(),
                },
                references: vec![],
            }
        }
        
        fn capabilities(&self) -> PlatformCapabilities {
            PlatformCapabilities {
                max_files: 12,
                max_video_duration: 15.0,
                max_audio_duration: 15.0,
                supported_formats: vec!["png".to_string(), "jpg".to_string()],
                supported_aspect_ratios: vec![AspectRatio::Landscape, AspectRatio::Portrait],
            }
        }
        
        fn name(&self) -> &str {
            "TestPlatform"
        }
    }
    
    #[test]
    fn test_registry() {
        let mut registry = PlatformRegistry::new();
        registry.register(Box::new(TestAdapter));
        
        assert!(registry.get("TestPlatform").is_some());
        assert!(registry.list_platforms().contains(&"TestPlatform"));
    }
    
    #[test]
    fn test_conversion() {
        let mut registry = PlatformRegistry::new();
        registry.register(Box::new(TestAdapter));
        
        let internal = InternalPrompt {
            style: "电影级写实".to_string(),
            duration: 15.0,
            aspect_ratio: AspectRatio::Landscape,
            atmosphere: "温馨".to_string(),
            segments: vec![],
            sound: SoundDesign {
                music_style: "钢琴".to_string(),
                effects: vec![],
                dialogue: String::new(),
            },
            references: vec![],
        };
        
        let platform_prompt = registry.convert_to("TestPlatform", &internal).unwrap();
        assert!(platform_prompt.formatted.contains("电影级写实"));
    }
}