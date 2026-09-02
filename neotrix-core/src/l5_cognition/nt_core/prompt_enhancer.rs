//! 提示词增强器模块
//!
//! 自动优化和丰富用户提示词
//! 支持风格注入、负面提示词生成、质量标签

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// ============================================================================
// 提示词定义
// ============================================================================

/// 增强策略
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum EnhancementStrategy {
    /// 质量增强
    Quality,
    /// 风格增强
    Style,
    /// 细节增强
    Detail,
    /// 负面提示词
    Negative,
    /// 组合增强
    Combined,
}

/// 风格预设
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StylePreset {
    /// 预设ID
    pub id: String,
    /// 预设名称
    pub name: String,
    /// 正向提示词前缀
    pub positive_prefix: String,
    /// 负向提示词前缀
    pub negative_prefix: String,
    /// 质量标签
    pub quality_tags: Vec<String>,
    /// 风格标签
    pub style_tags: Vec<String>,
}

/// 增强配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancerConfig {
    /// 默认策略
    pub default_strategy: EnhancementStrategy,
    /// 风格预设
    pub style_presets: Vec<StylePreset>,
    /// 质量标签库
    pub quality_tags: Vec<String>,
    /// 负面提示词库
    pub negative_prompts: Vec<String>,
    /// 最大提示词长度
    pub max_prompt_length: usize,
    /// 是否启用自动负面提示词
    pub auto_negative: bool,
}

/// 增强结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancementResult {
    /// 原始提示词
    pub original: String,
    /// 增强后的提示词
    pub enhanced: String,
    /// 生成的负面提示词
    pub negative: String,
    /// 应用的策略
    pub strategy: EnhancementStrategy,
    /// 添加的标签
    pub added_tags: Vec<String>,
    /// 增强耗时 (毫秒)
    pub enhancement_time_ms: u64,
}

// ============================================================================
// 提示词增强器
// ============================================================================

/// 提示词增强器
pub struct PromptEnhancer {
    /// 配置
    config: EnhancerConfig,
    /// 增强历史
    history: Vec<EnhancementResult>,
}

impl PromptEnhancer {
    /// 创建增强器
    pub fn new() -> Self {
        Self {
            config: EnhancerConfig {
                default_strategy: EnhancementStrategy::Combined,
                style_presets: vec![
                    StylePreset {
                        id: "cinematic".to_string(),
                        name: "电影感".to_string(),
                        positive_prefix: "cinematic lighting, film grain, ".to_string(),
                        negative_prefix: "cartoon, anime, ".to_string(),
                        quality_tags: vec!["masterpiece", "best quality", "highly detailed".to_string()],
                        style_tags: vec!["cinematic", "film".to_string()],
                    },
                    StylePreset {
                        id: "anime".to_string(),
                        name: "动漫风格".to_string(),
                        positive_prefix: "anime style, ".to_string(),
                        negative_prefix: "realistic, photo, ".to_string(),
                        quality_tags: vec!["masterpiece", "best quality".to_string()],
                        style_tags: vec!["anime", "manga".to_string()],
                    },
                ],
                quality_tags: vec![
                    "masterpiece".to_string(),
                    "best quality".to_string(),
                    "highly detailed".to_string(),
                    "8k".to_string(),
                    "ultra detailed".to_string(),
                ],
                negative_prompts: vec![
                    "low quality".to_string(),
                    "worst quality".to_string(),
                    "blurry".to_string(),
                    "deformed".to_string(),
                    "ugly".to_string(),
                ],
                max_prompt_length: 500,
                auto_negative: true,
            },
            history: vec![],
        }
    }
    
    /// 使用配置创建
    pub fn with_config(config: EnhancerConfig) -> Self {
        Self {
            config,
            history: vec![],
        }
    }
    
    /// 增强提示词
    pub fn enhance(
        &mut self,
        prompt: &str,
        strategy: EnhancementStrategy,
        style_id: Option<&str>,
    ) -> EnhancementResult {
        let start = std::time::Instant::now();
        
        let mut enhanced = prompt.to_string();
        let mut negative = String::new();
        let mut added_tags = vec![];
        
        // 应用质量标签
        if strategy == EnhancementStrategy::Quality || strategy == EnhancementStrategy::Combined {
            for tag in &self.config.quality_tags {
                if !enhanced.contains(tag) {
                    enhanced = format!("{}, {}", tag, enhanced);
                    added_tags.push(tag.clone());
                }
            }
        }
        
        // 应用风格预设
        if let Some(style_id) = style_id {
            if let Some(preset) = self.config.style_presets.iter().find(|p| p.id == style_id) {
                enhanced = format!("{}{}", preset.positive_prefix, enhanced);
                negative = format!("{}{}", preset.negative_prefix, negative);
                added_tags.extend(preset.style_tags.clone());
            }
        }
        
        // 生成负面提示词
        if self.config.auto_negative && negative.is_empty() {
            negative = self.config.negative_prompts.join(", ");
        }
        
        // 截断
        if enhanced.len() > self.config.max_prompt_length {
            enhanced.truncate(self.config.max_prompt_length);
        }
        
        let result = EnhancementResult {
            original: prompt.to_string(),
            enhanced,
            negative,
            strategy,
            added_tags,
            enhancement_time_ms: start.elapsed().as_millis() as u64,
        };
        
        self.history.push(result.clone());
        result
    }
    
    /// 增强负面提示词
    pub fn enhance_negative(&self, base_negative: &str, additional: &[String]) -> String {
        let mut negative = base_negative.to_string();
        
        for tag in additional {
            if !negative.contains(tag) {
                negative = format!("{}, {}", negative, tag);
            }
        }
        
        negative
    }
    
    /// 获取统计信息
    pub fn statistics(&self) -> EnhancerStats {
        let total_enhanced = self.history.len();
        let avg_added_tags = if total_enhanced > 0 {
            self.history.iter().map(|r| r.added_tags.len()).sum::<usize>() as f32 / total_enhanced as f32
        } else {
            0.0
        };
        
        EnhancerStats {
            total_enhanced,
            avg_added_tags,
        }
    }
}

/// 增强统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancerStats {
    /// 总增强次数
    pub total_enhanced: usize,
    /// 平均添加标签数
    pub avg_added_tags: f32,
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_prompt_enhancer() {
        let mut enhancer = PromptEnhancer::new();
        
        let result = enhancer.enhance(
            "a girl in a garden",
            EnhancementStrategy::Combined,
            Some("cinematic"),
        );
        
        assert!(result.enhanced.contains("masterpiece"));
        assert!(result.enhanced.contains("cinematic"));
        assert!(!result.negative.is_empty());
    }
}