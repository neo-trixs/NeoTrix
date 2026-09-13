//! 叙事结构化模块 (通用)
//!
//! 实现文本→结构化叙事、镜头运动规划
//! 适用于：电影、广告、教育视频、漫剧等所有视频内容

use serde::{Serialize, Deserialize};


// ============================================================================
// 叙事定义
// ============================================================================

/// 景别类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ShotSize {
    /// 远景
    ExtremeWide,
    /// 全景
    Wide,
    /// 中景
    Medium,
    /// 中近景
    MediumClose,
    /// 近景
    Close,
    /// 特写
    ExtremeClose,
    /// 大特写
    Macro,
}

/// 镜头运动类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CameraMovement {
    /// 固定镜头
    Static,
    /// 推镜头
    PushIn,
    /// 拉镜头
    PullOut,
    /// 摇镜头
    Pan,
    /// 移镜头
    Dolly,
    /// 跟镜头
    Follow,
    /// 升降镜头
    Crane,
    /// 手持镜头
    Handheld,
    /// 旋转镜头
    Rotate,
    /// 航拍
    Aerial,
}

/// 时长模式
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DurationMode {
    /// 基于对白时长
    DialogueBased,
    /// 基于动作时长
    ActionBased,
    /// 基于音乐节拍
    MusicBased,
    /// 固定时长
    Fixed,
    /// 动态计算
    Dynamic,
}

/// 镜头单元
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct _ShotUnit {
    /// 镜头ID
    pub id: String,
    /// 镜头序号
    pub shot_number: u32,
    /// 画面描述
    pub description: String,
    /// 涉及的元素
    pub elements: Vec<String>,
    /// 场景
    pub scene: String,
    /// 景别
    pub shot_size: ShotSize,
    /// 镜头运动
    pub camera_movement: CameraMovement,
    /// 时长 (秒)
    pub duration_secs: f32,
    /// 对白
    pub dialogue: Option<String>,
    /// 旁白
    pub narration: Option<String>,
    /// 动作描述
    pub action: Option<String>,
    /// 情绪
    pub emotion: Option<String>,
    /// 正向提示词
    pub positive_prompt: String,
    /// 负向提示词
    pub negative_prompt: String,
    /// 首帧描述
    pub start_frame_desc: Option<String>,
    /// 尾帧描述
    pub end_frame_desc: Option<String>,
    /// 动作脚本
    pub motion_script: Option<String>,
    /// 关联元素
    pub linked_elements: Vec<String>,
    /// 标签
    pub tags: Vec<String>,
}

/// 叙事脚本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct _NarrativeScript {
    /// 内容ID
    pub content_id: String,
    /// 内容标题
    pub content_title: String,
    /// 所有镜头
    pub shots: Vec<_ShotUnit>,
    /// 总时长 (秒)
    pub total_duration_secs: f32,
    /// 元素列表
    pub elements: Vec<String>,
    /// 场景列表
    pub scenes: Vec<String>,
}

/// 叙事结构化配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct _NarrativeConfig {
    /// 目标时长模式
    pub duration_mode: DurationMode,
    /// 默认镜头时长 (秒)
    pub default_shot_duration: f32,
    /// 最小镜头时长 (秒)
    pub min_shot_duration: f32,
    /// 最大镜头时长 (秒)
    pub max_shot_duration: f32,
    /// 是否启用镜头衔接
    pub enable_shot_linking: bool,
    /// 是否自动生成提示词
    pub auto_generate_prompts: bool,
    /// 内容风格
    pub content_style: String,
    /// 输出比例
    pub aspect_ratio: String,
    /// 内容类型
    pub content_type: ContentType,
}

/// 内容类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ContentType {
    /// 电影
    Film,
    /// 广告
    Advertisement,
    /// 教育视频
    Educational,
    /// 漫剧
    ComicDrama,
    /// 短视频
    ShortVideo,
    /// MV
    MusicVideo,
    /// 纪录片
    Documentary,
    /// 游戏过场
    GameCutscene,
}

// ============================================================================
// 叙事结构化器
// ============================================================================

/// 叙事结构化器
/// 将文本转换为结构化叙事脚本
pub(crate) struct _NarrativeStructuring {
    /// 配置
    config: _NarrativeConfig,
    /// 结构化历史
    history: Vec<_NarrativeScript>,
}

impl _NarrativeStructuring {
    /// 创建结构化器
    pub fn new() -> Self {
        Self {
            config: _NarrativeConfig {
                duration_mode: DurationMode::Dynamic,
                default_shot_duration: 3.0,
                min_shot_duration: 1.5,
                max_shot_duration: 6.0,
                enable_shot_linking: true,
                auto_generate_prompts: true,
                content_style: "default".to_string(),
                aspect_ratio: "16:9".to_string(),
                content_type: ContentType::ShortVideo,
            },
            history: vec![],
        }
    }
    
    /// 使用配置创建
    pub fn with_config(config: _NarrativeConfig) -> Self {
        Self {
            config,
            history: vec![],
        }
    }
    
    /// 从文本结构化叙事
    pub(crate) fn _structure_from_text(
        &mut self,
        content_id: &str,
        text: &str,
    ) -> _NarrativeScript {
        // TODO: 实际调用 LLM 进行叙事结构化
        let shots = self.parse_text_to_shots(text);
        
        let total_duration = shots.iter().map(|s| s.duration_secs).sum();
        let elements = self.extract_elements(&shots);
        let scenes = self.extract_scenes(&shots);
        
        let script = _NarrativeScript {
            content_id: content_id.to_string(),
            content_title: format!("内容_{}", content_id),
            shots,
            total_duration_secs: total_duration,
            elements,
            scenes,
        };
        
        self.history.push(script.clone());
        script
    }
    
    /// 解析文本为镜头
    ///
    /// 注意：此为 LLM 未接入时的降级方案。按段落分割并启发式分配镜头类型。
    fn parse_text_to_shots(&self, text: &str) -> Vec<_ShotUnit> {
        let paragraphs: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
        
        paragraphs.iter().enumerate().map(|(i, p)| {
            let p_lower = p.to_lowercase();
            
            // 启发式：根据内容判断镜头类型
            let (shot_size, duration_multiplier) = if p_lower.contains("特写") || p_lower.contains("close") || p_lower.contains("脸") {
                (ShotSize::ExtremeClose, 0.7)
            } else if p_lower.contains("全景") || p_lower.contains("wide") || p_lower.contains("场景") {
                (ShotSize::Wide, 1.3)
            } else if p_lower.contains("中景") || p_lower.contains("medium") {
                (ShotSize::Medium, 1.0)
            } else if p_lower.contains("远景") || p_lower.contains("extreme") {
                (ShotSize::ExtremeWide, 1.5)
            } else {
                (ShotSize::Medium, 1.0)
            };
            
            let camera_movement = if p_lower.contains("跟拍") || p_lower.contains("follow") || p_lower.contains("追踪") {
                CameraMovement::Follow
            } else if p_lower.contains("推") || p_lower.contains("zoom") || p_lower.contains("拉近") {
                CameraMovement::PushIn
            } else if p_lower.contains("摇") || p_lower.contains("pan") || p_lower.contains("扫") {
                CameraMovement::Pan
            } else if p_lower.contains("旋转") || p_lower.contains("rotate") {
                CameraMovement::Rotate
            } else if p_lower.contains("手持") || p_lower.contains("handheld") {
                CameraMovement::Handheld
            } else if p_lower.contains("航拍") || p_lower.contains("aerial") {
                CameraMovement::Aerial
            } else {
                CameraMovement::Static
            };
            
            let duration = (self.config.default_shot_duration as f64 * duration_multiplier) as f64;
            
            _ShotUnit {
                id: format!("shot_{}", i + 1),
                shot_number: (i + 1) as u32,
                description: p.to_string(),
                elements: vec![],
                scene: format!("场景_{}", (i / 3) + 1),
                shot_size,
                camera_movement,
                duration_secs: duration,
                dialogue: None,
                narration: None,
                action: None,
                emotion: None,
                positive_prompt: String::new(),
                negative_prompt: String::new(),
                start_frame_desc: None,
                end_frame_desc: None,
                motion_script: None,
                linked_elements: vec![],
                tags: vec![],
            }
        }).collect()
    }
    
    /// 提取元素列表
    fn extract_elements(&self, shots: &[_ShotUnit]) -> Vec<String> {
        let mut elements = std::collections::HashSet::new();
        for shot in shots {
            for elem in &shot.elements {
                elements.insert(elem.clone());
            }
        }
        elements.into_iter().collect()
    }
    
    /// 提取场景列表
    fn extract_scenes(&self, shots: &[_ShotUnit]) -> Vec<String> {
        let mut scenes = std::collections::HashSet::new();
        for shot in shots {
            scenes.insert(shot.scene.clone());
        }
        scenes.into_iter().collect()
    }
    
    /// 优化镜头时长
    pub fn optimize_durations(&self, script: &mut _NarrativeScript) {
        for shot in &mut script.shots {
            if let Some(ref dialogue) = shot.dialogue {
                let dialogue_duration = dialogue.len() as f32 * 0.15;
                shot.duration_secs = dialogue_duration.max(self.config.min_shot_duration)
                    .min(self.config.max_shot_duration);
            }
            
            if let Some(ref action) = shot.action {
                let action_words = action.split_whitespace().count() as f32;
                let action_duration = action_words * 0.3;
                shot.duration_secs = shot.duration_secs.max(action_duration)
                    .min(self.config.max_shot_duration);
            }
        }
        
        script.total_duration_secs = script.shots.iter().map(|s| s.duration_secs).sum();
    }
    
    /// 生成镜头运动规划
    pub fn plan_camera_movements(&self, script: &mut _NarrativeScript) {
        for shot in &mut script.shots {
            if shot.camera_movement == CameraMovement::Static {
                shot.camera_movement = match shot.shot_size {
                    ShotSize::ExtremeWide => CameraMovement::Pan,
                    ShotSize::Wide => CameraMovement::Dolly,
                    ShotSize::Medium => CameraMovement::Static,
                    ShotSize::Close => CameraMovement::PushIn,
                    ShotSize::ExtremeClose => CameraMovement::Static,
                    _ => CameraMovement::Static,
                };
            }
        }
    }
    
    /// 获取结构化统计
    pub fn statistics(&self) -> _NarrativeStats {
        let total_scripts = self.history.len();
        let total_shots: usize = self.history.iter().map(|s| s.shots.len()).sum();
        let avg_shots_per_script = if total_scripts > 0 {
            total_shots as f32 / total_scripts as f32
        } else {
            0.0
        };
        let avg_duration = if total_shots > 0 {
            self.history.iter()
                .flat_map(|s| &s.shots)
                .map(|s| s.duration_secs)
                .sum::<f32>() / total_shots as f32
        } else {
            0.0
        };
        
        _NarrativeStats {
            total_scripts,
            total_shots,
            avg_shots_per_script,
            avg_shot_duration: avg_duration,
        }
    }
}

/// 结构化统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct _NarrativeStats {
    /// 总脚本数
    pub total_scripts: usize,
    /// 总镜头数
    pub total_shots: usize,
    /// 平均每脚本镜头数
    pub avg_shots_per_script: f32,
    /// 平均镜头时长
    pub avg_shot_duration: f32,
}

// ============================================================================
// 向后兼容别名
// ============================================================================

/// 分镜智能拆解器 (向后兼容别名)
pub type StoryboardExtractor = _NarrativeStructuring;

/// 分镜脚本 (向后兼容别名)
pub type StoryboardScript = _NarrativeScript;

/// 分镜 (向后兼容别名)
pub type Storyboard = _ShotUnit;

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_narrative_structuring() {
        let mut structuring = _NarrativeStructuring::new();
        
        let text = r#"
        第一幕：主角登场
        主角走进教室，环顾四周
        发现没有人，感到困惑
        "#;
        
        let script = structuring._structure_from_text("content_001", text);
        assert!(!script.shots.is_empty());
        
        let stats = structuring.statistics();
        assert_eq!(stats.total_scripts, 1);
    }
    
    #[test]
    fn test_optimize_durations() {
        let mut structuring = _NarrativeStructuring::new();
        
        let mut script = _NarrativeScript {
            content_id: "content_001".to_string(),
            content_title: "测试".to_string(),
            shots: vec![
                _ShotUnit {
                    id: "shot_1".to_string(),
                    shot_number: 1,
                    description: "测试".to_string(),
                    elements: vec![],
                    scene: "场景1".to_string(),
                    shot_size: ShotSize::Medium,
                    camera_movement: CameraMovement::Static,
                    duration_secs: 1.0,
                    dialogue: Some("这是一段很长的对白，用来测试时长调整".to_string()),
                    narration: None,
                    action: None,
                    emotion: None,
                    positive_prompt: String::new(),
                    negative_prompt: String::new(),
                    start_frame_desc: None,
                    end_frame_desc: None,
                    motion_script: None,
                    linked_elements: vec![],
                    tags: vec![],
                },
            ],
            total_duration_secs: 1.0,
            elements: vec![],
            scenes: vec![],
        };
        
        structuring.optimize_durations(&mut script);
        assert!(script.shots[0].duration_secs > 1.0);
    }
}