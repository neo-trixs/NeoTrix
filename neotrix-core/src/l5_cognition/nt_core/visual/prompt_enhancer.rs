//! 提示词增强器模块
//!
//! 自动优化和丰富用户提示词
//! 支持风格注入、负面提示词生成、质量标签

use serde::{Serialize, Deserialize};


// ============================================================================
// 提示词定义
// ============================================================================

/// 增强策略
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum _EnhancementStrategy {
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
pub struct _StylePreset {
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
pub struct _EnhancerConfig {
    /// 默认策略
    pub default_strategy: _EnhancementStrategy,
    /// 风格预设
    pub style_presets: Vec<_StylePreset>,
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
pub struct _EnhancementResult {
    /// 原始提示词
    pub original: String,
    /// 增强后的提示词
    pub enhanced: String,
    /// 生成的负面提示词
    pub negative: String,
    /// 应用的策略
    pub strategy: _EnhancementStrategy,
    /// 添加的标签
    pub added_tags: Vec<String>,
    /// 增强耗时 (毫秒)
    pub enhancement_time_ms: u64,
}

// ============================================================================
// 提示词增强器
// ============================================================================

/// 提示词增强器
pub struct _PromptEnhancer {
    /// 配置
    config: _EnhancerConfig,
    /// 增强历史
    history: Vec<_EnhancementResult>,
}

impl _PromptEnhancer {
    /// Create prompt enhancer with default config.
    ///
    /// Note: Default config includes "cinematic" and "anime" style presets,
    /// 5 quality tags, 5 negative prompts, and 500 char max length.
    /// Real implementation needs:
    /// - More style presets (realistic, watercolor, pixel art, etc.)
    /// - Platform-specific quality tags (SD WebUI vs ComfyUI)
    /// - Dynamic max_prompt_length based on model tokenizer limits
    pub fn new() -> Self {
        Self {
            config: _EnhancerConfig {
                default_strategy: _EnhancementStrategy::Combined,
                style_presets: vec![
                    _StylePreset {
                        id: "cinematic".to_string(),
                        name: "电影感".to_string(),
                        positive_prefix: "cinematic lighting, film grain, ".to_string(),
                        negative_prefix: "cartoon, anime, ".to_string(),
                        quality_tags: vec!["masterpiece".to_string(), "best quality".to_string(), "highly detailed".to_string()],
                        style_tags: vec!["cinematic".to_string(), "film".to_string()],
                    },
                    _StylePreset {
                        id: "anime".to_string(),
                        name: "动漫风格".to_string(),
                        positive_prefix: "anime style, ".to_string(),
                        negative_prefix: "realistic, photo, ".to_string(),
                        quality_tags: vec!["masterpiece".to_string(), "best quality".to_string()],
                        style_tags: vec!["anime".to_string(), "manga".to_string()],
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
    
    /// Create prompt enhancer with custom configuration.
    ///
    /// Note: Config validation should check:
    /// - max_prompt_length > 0
    /// - quality_tags and negative_prompts are non-empty for Combined strategy
    /// - style_presets have valid IDs (no duplicates)
    pub fn with_config(config: _EnhancerConfig) -> Self {
        Self {
            config,
            history: vec![],
        }
    }
    
    /// Enhance a prompt using the specified strategy.
    ///
    /// Note: Applies quality tags (prepended), style prefix (if style_id provided),
    /// and auto-generates negative prompt. Truncates to max_prompt_length.
    /// Real implementation needs:
    /// - Token-aware truncation (not byte-level)
    /// - Style-specific negative prompt generation
    /// - Deduplication of quality tags already present in prompt
    /// - Platform-specific formatting (ComfyUI node syntax vs SD WebUI syntax)
    pub fn enhance(
        &mut self,
        prompt: &str,
        strategy: _EnhancementStrategy,
        style_id: Option<&str>,
    ) -> _EnhancementResult {
        let start = std::time::Instant::now();
        
        let mut enhanced = prompt.to_string();
        let mut negative = String::new();
        let mut added_tags = vec![];
        
        // 应用质量标签
        if strategy == _EnhancementStrategy::Quality || strategy == _EnhancementStrategy::Combined {
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
        
        let result = _EnhancementResult {
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
    ///
    /// Note: Merges additional negative tags into base, deduplicating by substring match.
    /// Real implementation needs:
    /// - Semantic deduplication (not just substring) to avoid redundant tokens
    /// - Platform-aware negative prompt length limits (SD WebUI ~75 tokens)
    /// - Priority ordering (NSFW > violence > quality) for truncation
    pub(crate) fn _enhance_negative(&self, base_negative: &str, additional: &[String]) -> String {
        let mut negative = base_negative.to_string();
        
        for tag in additional {
            if !negative.contains(tag) {
                negative = format!("{}, {}", negative, tag);
            }
        }
        
        negative
    }
    
    /// Get enhancement statistics.
    ///
    /// Note: avg_added_tags is mean number of tags added per enhancement.
    pub fn statistics(&self) -> _EnhancerStats {
        let total_enhanced = self.history.len();
        let avg_added_tags = if total_enhanced > 0 {
            self.history.iter().map(|r| r.added_tags.len()).sum::<usize>() as f32 / total_enhanced as f32
        } else {
            0.0
        };
        
        _EnhancerStats {
            total_enhanced,
            avg_added_tags,
        }
    }
}

/// 增强统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _EnhancerStats {
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
        let mut enhancer = _PromptEnhancer::new();
        
        let result = enhancer.enhance(
            "a girl in a garden",
            _EnhancementStrategy::Combined,
            Some("cinematic"),
        );
        
        assert!(result.enhanced.contains("masterpiece"));
        assert!(result.enhanced.contains("cinematic"));
        assert!(!result.negative.is_empty());
    }
}