//! 分镜智能拆解模块
//!
//! 实现 LLM 剧本→分镜自动拆解、镜头运动规划
//! 将剧本转换为结构化分镜脚本

use serde::{Serialize, Deserialize};


// ============================================================================
// 分镜定义
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
}

/// 镜头时长模式
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DurationMode {
    /// 基于对白时长
    DialogueBased,
    /// 基于动作时长
    ActionBased,
    /// 固定时长
    Fixed,
    /// 动态计算
    Dynamic,
}

/// 分镜脚本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Storyboard {
    /// 镜头ID
    pub id: String,
    /// 镜头序号
    pub shot_number: u32,
    /// 画面描述
    pub description: String,
    /// 角色
    pub characters: Vec<String>,
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
    /// 关联角色
    pub linked_characters: Vec<String>,
    /// 标签
    pub tags: Vec<String>,
}

/// 分镜表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryboardScript {
    /// 剧本ID
    pub episode_id: String,
    /// 集数
    pub episode_number: u32,
    /// 集标题
    pub episode_title: String,
    /// 所有镜头
    pub shots: Vec<Storyboard>,
    /// 总时长 (秒)
    pub total_duration_secs: f32,
    /// 角色列表
    pub characters: Vec<String>,
    /// 场景列表
    pub scenes: Vec<String>,
}

/// 分镜拆解配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct _StoryboardConfig {
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
    /// 画风
    pub art_style: String,
    /// 视频比例
    pub aspect_ratio: String,
}

// ============================================================================
// 分镜智能拆解器
// ============================================================================

/// 分镜智能拆解器
pub struct StoryboardExtractor {
    /// 配置
    config: _StoryboardConfig,
    /// 拆解历史
    history: Vec<StoryboardScript>,
}

impl StoryboardExtractor {
    /// 创建拆解器
    pub fn new() -> Self {
        Self {
            config: _StoryboardConfig {
                duration_mode: DurationMode::Dynamic,
                default_shot_duration: 3.0,
                min_shot_duration: 1.5,
                max_shot_duration: 6.0,
                enable_shot_linking: true,
                auto_generate_prompts: true,
                art_style: "anime".to_string(),
                aspect_ratio: "9:16".to_string(),
            },
            history: vec![],
        }
    }
    
    /// 使用配置创建
    pub fn with_config(config: _StoryboardConfig) -> Self {
        Self {
            config,
            history: vec![],
        }
    }
    
    /// 从剧本文本拆解分镜
    ///
    /// **Feature not wired**: This currently uses naive paragraph splitting.
    /// Real implementation needs an LLM call (via `nt_io::LlmProvider`) to parse
    /// the script into structured shots with proper shot sizes, camera movements,
    /// dialogue extraction, and emotion tagging. The naive split produces empty
    /// characters/scenes and uniform shot sizes — unsuitable for production use.
    pub(crate) fn _extract_from_script(
        &mut self,
        episode_id: &str,
        episode_number: u32,
        script_text: &str,
    ) -> StoryboardScript {
        // Feature not wired: LLM-based storyboard extraction is not implemented.
        // Returns naive paragraph-split results. For production, wire to LLM provider
        // with a structured prompt that extracts shots, dialogue, camera, and emotion.
        let shots = self.parse_script_to_shots(script_text);

        let total_duration = shots.iter().map(|s| s.duration_secs).sum();
        let characters = self.extract_characters(&shots);
        let scenes = self.extract_scenes(&shots);

        let script = StoryboardScript {
            episode_id: episode_id.to_string(),
            episode_number,
            episode_title: format!("第{}集", episode_number),
            shots,
            total_duration_secs: total_duration,
            characters,
            scenes,
        };

        self.history.push(script.clone());
        script
    }

    /// 解析剧本为镜头（朴素段落拆分 — 非 LLM 驱动）
    ///
    /// **Feature not wired**: Real implementation needs an LLM call to:
    /// 1. Identify scene boundaries, dialogue, and action blocks
    /// 2. Assign appropriate shot sizes and camera movements per narrative context
    /// 3. Extract character names and emotions from dialogue/narration
    /// 4. Generate positive/negative prompts for video generation
    fn parse_script_to_shots(&self, script_text: &str) -> Vec<Storyboard> {
        // Feature not wired: LLM-based parsing not implemented.
        // Falls back to naive paragraph splitting — each paragraph becomes a shot
        // with default values for all fields except description.
        let paragraphs: Vec<&str> = script_text.lines().filter(|l| !l.trim().is_empty()).collect();

        paragraphs.iter().enumerate().map(|(i, p)| {
            Storyboard {
                id: format!("shot_{}", i + 1),
                shot_number: (i + 1) as u32,
                description: p.to_string(),
                characters: vec![],
                scene: "默认场景".to_string(),
                shot_size: ShotSize::Medium,
                camera_movement: CameraMovement::Static,
                duration_secs: self.config.default_shot_duration,
                dialogue: None,
                narration: None,
                action: None,
                emotion: None,
                positive_prompt: String::new(),
                negative_prompt: String::new(),
                start_frame_desc: None,
                end_frame_desc: None,
                motion_script: None,
                linked_characters: vec![],
                tags: vec![],
            }
        }).collect()
    }
    
    /// 提取角色列表
    fn extract_characters(&self, shots: &[Storyboard]) -> Vec<String> {
        let mut characters = std::collections::HashSet::new();
        for shot in shots {
            for char in &shot.characters {
                characters.insert(char.clone());
            }
        }
        characters.into_iter().collect()
    }
    
    /// 提取场景列表
    fn extract_scenes(&self, shots: &[Storyboard]) -> Vec<String> {
        let mut scenes = std::collections::HashSet::new();
        for shot in shots {
            scenes.insert(shot.scene.clone());
        }
        scenes.into_iter().collect()
    }
    
    /// 优化镜头时长
    pub fn optimize_durations(&self, script: &mut StoryboardScript) {
        for shot in &mut script.shots {
            // 根据对白长度调整时长
            if let Some(ref dialogue) = shot.dialogue {
                let dialogue_duration = dialogue.len() as f32 * 0.15; // 假设每字符0.15秒
                shot.duration_secs = dialogue_duration.max(self.config.min_shot_duration)
                    .min(self.config.max_shot_duration);
            }
            
            // 根据动作复杂度调整时长
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
    pub fn plan_camera_movements(&self, script: &mut StoryboardScript) {
        for (_i, shot) in script.shots.iter_mut().enumerate() {
            // 根据景别和情绪自动规划镜头运动
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
    
    /// 获取拆解统计
    pub fn statistics(&self) -> ExtractorStats {
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
        
        ExtractorStats {
            total_scripts,
            total_shots,
            avg_shots_per_script,
            avg_shot_duration: avg_duration,
        }
    }
}

/// 拆解统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractorStats {
    /// 总剧本数
    pub total_scripts: usize,
    /// 总镜头数
    pub total_shots: usize,
    /// 平均每剧本镜头数
    pub avg_shots_per_script: f32,
    /// 平均镜头时长
    pub avg_shot_duration: f32,
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_storyboard_extractor() {
        let mut extractor = StoryboardExtractor::new();
        
        let script_text = r#"
        第一幕：主角登场
        主角走进教室，环顾四周
        发现没有人，感到困惑
        "#;
        
        let script = extractor._extract_from_script("ep001", 1, script_text);
        assert_eq!(script.episode_number, 1);
        assert!(!script.shots.is_empty());
        
        let stats = extractor.statistics();
        assert_eq!(stats.total_scripts, 1);
    }
    
    #[test]
    fn test_optimize_durations() {
        let mut extractor = StoryboardExtractor::new();
        
        let mut script = StoryboardScript {
            episode_id: "ep001".to_string(),
            episode_number: 1,
            episode_title: "测试".to_string(),
            shots: vec![
                Storyboard {
                    id: "shot_1".to_string(),
                    shot_number: 1,
                    description: "测试".to_string(),
                    characters: vec![],
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
                    linked_characters: vec![],
                    tags: vec![],
                },
            ],
            total_duration_secs: 1.0,
            characters: vec![],
            scenes: vec![],
        };
        
        extractor.optimize_durations(&mut script);
        assert!(script.shots[0].duration_secs > 1.0);
    }
}