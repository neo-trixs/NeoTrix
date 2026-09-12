//! 视频规格系统模块
//!
//! 类型化视频规格定义，替代自由文本提示
//! 支持场景列表、时长、资产引用、CTA 结构

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// ============================================================================
// 规格定义
// ============================================================================

/// 内容类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub(crate) enum VideoContentType {
    /// 广告
    Advertisement,
    /// 教育视频
    Educational,
    /// 短视频
    ShortVideo,
    /// 电影
    Film,
    /// MV
    MusicVideo,
    /// 纪录片
    Documentary,
    /// 游戏过场
    GameCutscene,
    /// 漫剧
    ComicDrama,
}

/// 输出格式
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum OutputFormat {
    /// MP4 (H.264)
    MP4H264,
    /// MP4 (H.265)
    MP4H265,
    /// WebM (VP9)
    WebMVP9,
    /// MOV (ProRes)
    MOVProRes,
    /// GIF
    GIF,
}

/// 场景类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SceneType {
    /// 开场
    Opening,
    /// 铺垫
    Setup,
    /// 冲突
    Conflict,
    /// 高潮
    Climax,
    /// 转折
    Transition,
    /// 结局
    Resolution,
    /// CTA
    CallToAction,
}

/// 资产引用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct AssetReference {
    /// 资产ID
    pub asset_id: String,
    /// 资产路径
    pub asset_path: String,
    /// 资产类型
    pub asset_type: String,
    /// 使用方式
    pub usage: String,
    /// 权重
    pub weight: f32,
}

/// 场景规格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SceneSpec {
    /// 场景ID
    pub id: String,
    /// 场景序号
    pub order: u32,
    /// 场景类型
    pub scene_type: SceneType,
    /// 时长 (秒)
    pub duration_secs: f32,
    /// 画面描述
    pub description: String,
    /// 涉及的实体
    pub entities: Vec<String>,
    /// 景别
    pub shot_size: String,
    /// 镜头运动
    pub camera_movement: String,
    /// 对白
    pub dialogue: Option<String>,
    /// 旁白
    pub narration: Option<String>,
    /// 正向提示词
    pub positive_prompt: String,
    /// 负向提示词
    pub negative_prompt: String,
    /// 资产引用
    pub asset_references: Vec<AssetReference>,
    /// 标签
    pub tags: Vec<String>,
}

/// CTA 结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CTASpec {
    /// CTA 文本
    pub text: String,
    /// CTA 类型
    pub cta_type: String,
    /// 目标链接
    pub target_url: Option<String>,
    /// 显示时长 (秒)
    pub display_duration_secs: f32,
    /// 显示位置
    pub position: String,
}

/// 视频规格
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct VideoSpec {
    /// 规格ID
    pub id: String,
    /// 内容类型
    pub content_type: VideoContentType,
    /// 标题
    pub title: String,
    /// 描述
    pub description: Option<String>,
    /// 输出格式
    pub output_format: OutputFormat,
    /// 宽度
    pub width: u32,
    /// 高度
    pub height: u32,
    /// 帧率
    pub fps: u32,
    /// 总时长 (秒)
    pub total_duration_secs: f32,
    /// 场景列表
    pub scenes: Vec<SceneSpec>,
    /// CTA 结构
    pub cta: Option<CTASpec>,
    /// 全局标签
    pub tags: Vec<String>,
    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
    /// 创建时间
    pub created_at: u64,
    /// 更新时间
    pub updated_at: u64,
}

/// 规格验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SpecValidationResult {
    /// 是否有效
    pub valid: bool,
    /// 错误列表
    pub errors: Vec<SpecError>,
    /// 警告列表
    pub warnings: Vec<SpecWarning>,
}

/// 规格错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SpecError {
    /// 错误代码
    pub code: String,
    /// 错误描述
    pub message: String,
    /// 相关字段
    pub field: Option<String>,
}

/// 规格警告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SpecWarning {
    /// 警告代码
    pub code: String,
    /// 警告描述
    pub message: String,
    /// 相关字段
    pub field: Option<String>,
}

/// 规格构建器
pub(crate) struct VideoSpecBuilder {
    spec: VideoSpec,
}

impl VideoSpecBuilder {
    /// 创建构建器
    pub fn new(id: &str, content_type: VideoContentType) -> Self {
        Self {
            spec: VideoSpec {
                id: id.to_string(),
                content_type,
                title: String::new(),
                description: None,
                output_format: OutputFormat::MP4H264,
                width: 1920,
                height: 1080,
                fps: 30,
                total_duration_secs: 0.0,
                scenes: vec![],
                cta: None,
                tags: vec![],
                metadata: HashMap::new(),
                created_at: 0,
                updated_at: 0,
            },
        }
    }
    
    /// 设置标题
    pub fn title(mut self, title: &str) -> Self {
        self.spec.title = title.to_string();
        self
    }
    
    /// 设置描述
    pub fn description(mut self, desc: &str) -> Self {
        self.spec.description = Some(desc.to_string());
        self
    }
    
    /// 设置输出格式
    pub fn output_format(mut self, format: OutputFormat) -> Self {
        self.spec.output_format = format;
        self
    }
    
    /// 设置分辨率
    pub fn resolution(mut self, width: u32, height: u32) -> Self {
        self.spec.width = width;
        self.spec.height = height;
        self
    }
    
    /// 设置帧率
    pub fn fps(mut self, fps: u32) -> Self {
        self.spec.fps = fps;
        self
    }
    
    /// 添加场景
    pub fn add_scene(mut self, scene: SceneSpec) -> Self {
        self.spec.total_duration_secs += scene.duration_secs;
        self.spec.scenes.push(scene);
        self
    }
    
    /// 设置 CTA
    pub fn cta(mut self, cta: CTASpec) -> Self {
        self.spec.cta = Some(cta);
        self
    }
    
    /// 添加标签
    pub fn tag(mut self, tag: &str) -> Self {
        self.spec.tags.push(tag.to_string());
        self
    }
    
    /// 构建规格
    pub fn build(self) -> VideoSpec {
        self.spec
    }
}

// ============================================================================
// 规格验证器
// ============================================================================

/// 规格验证器
pub(crate) struct SpecValidator {
    /// 验证规则
    rules: Vec<Box<dyn Fn(&VideoSpec) -> Vec<SpecError> + Send + Sync>>,
}

impl SpecValidator {
    /// 创建验证器
    pub fn new() -> Self {
        Self {
            rules: vec![
                Box::new(|spec| {
                    let mut errors = vec![];
                    if spec.scenes.is_empty() {
                        errors.push(SpecError {
                            code: "EMPTY_SCENES".to_string(),
                            message: "至少需要一个场景".to_string(),
                            field: Some("scenes".to_string()),
                        });
                    }
                    errors
                }),
                Box::new(|spec| {
                    let mut errors = vec![];
                    if spec.total_duration_secs <= 0.0 {
                        errors.push(SpecError {
                            code: "INVALID_DURATION".to_string(),
                            message: "总时长必须大于0".to_string(),
                            field: Some("total_duration_secs".to_string()),
                        });
                    }
                    errors
                }),
                Box::new(|spec| {
                    let mut errors = vec![];
                    if spec.width == 0 || spec.height == 0 {
                        errors.push(SpecError {
                            code: "INVALID_RESOLUTION".to_string(),
                            message: "分辨率必须大于0".to_string(),
                            field: Some("width/height".to_string()),
                        });
                    }
                    errors
                }),
            ],
        }
    }
    
    /// 验证规格
    pub fn validate(&self, spec: &VideoSpec) -> SpecValidationResult {
        let mut all_errors = vec![];
        let mut warnings = vec![];
        
        for rule in &self.rules {
            all_errors.extend(rule(spec));
        }
        
        // 添加警告
        if spec.total_duration_secs > 60.0 {
            warnings.push(SpecWarning {
                code: "LONG_DURATION".to_string(),
                message: "视频时长超过60秒，可能影响用户参与度".to_string(),
                field: Some("total_duration_secs".to_string()),
            });
        }
        
        let valid = all_errors.is_empty();
        
        SpecValidationResult {
            valid,
            errors: all_errors,
            warnings,
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
    fn test_video_spec_builder() {
        let spec = VideoSpecBuilder::new("spec_001", VideoContentType::Advertisement)
            .title("测试广告")
            .resolution(1920, 1080)
            .fps(30)
            .add_scene(SceneSpec {
                id: "scene_001".to_string(),
                order: 1,
                scene_type: SceneType::Opening,
                duration_secs: 5.0,
                description: "开场".to_string(),
                entities: vec![],
                shot_size: "Wide".to_string(),
                camera_movement: "Pan".to_string(),
                dialogue: None,
                narration: None,
                positive_prompt: String::new(),
                negative_prompt: String::new(),
                asset_references: vec![],
                tags: vec![],
            })
            .tag("广告")
            .build();
        
        assert_eq!(spec.id, "spec_001");
        assert_eq!(spec.scenes.len(), 1);
        assert_eq!(spec.total_duration_secs, 5.0);
    }
    
    #[test]
    fn test_spec_validation() {
        let validator = SpecValidator::new();
        
        let spec = VideoSpecBuilder::new("spec_002", VideoContentType::ShortVideo)
            .title("测试视频")
            .build();
        
        let result = validator.validate(&spec);
        assert!(!result.valid);
        assert!(!result.errors.is_empty());
    }
}