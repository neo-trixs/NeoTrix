//! 角色一致性增强模块
//!
//! 实现 ADetailer/FaceDetailer 自动补脸、Regional Prompting 多角色分区
//! 增强角色跨镜头一致性

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// ============================================================================
// 面部修复定义
// ============================================================================

/// 面部修复策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum _FaceFixStrategy {
    /// ADetailer 自动补脸 (SD WebUI)
    ADetailer,
    /// FaceDetailer 自动补脸 (ComfyUI Impact Pack)
    FaceDetailer,
    /// ReActor 换脸插件
    ReActor,
    /// 手动 Inpainting
    ManualInpaint,
}

/// 面部修复配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _FaceFixConfig {
    /// 修复策略
    pub strategy: _FaceFixStrategy,
    /// 检测阈值 (0.0-1.0)
    pub detection_threshold: f32,
    /// 修复强度 (0.0-1.0)
    pub fix_strength: f32,
    /// 最大重试次数
    pub max_retries: u32,
    /// 是否启用 LoRA
    pub use_lora: bool,
    /// LoRA 名称
    pub lora_name: Option<String>,
    /// LoRA 权重
    pub lora_weight: f32,
}

/// 面部修复结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _FaceFixResult {
    /// 是否成功
    pub success: bool,
    /// 修复后的图片路径
    pub fixed_image_path: Option<String>,
    /// 检测到的面部数量
    pub detected_faces: u32,
    /// 修复的面部数量
    pub fixed_faces: u32,
    /// 一致性分数 (0.0-1.0)
    pub consistency_score: f32,
    /// 修复耗时 (毫秒)
    pub fix_time_ms: u64,
    /// 错误信息
    pub error: Option<String>,
}

// ============================================================================
// 区域提示词定义
// ============================================================================

/// 区域配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionConfig {
    /// 区域ID
    pub id: String,
    /// 区域名称
    pub name: String,
    /// 区域边界 (x, y, width, height) 归一化坐标
    pub bounds: (f32, f32, f32, f32),
    /// 区域内的角色ID
    pub character_id: Option<String>,
    /// 区域提示词
    pub prompt: String,
    /// 区域权重
    pub weight: f32,
}

/// 多角色分区配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _RegionalPromptingConfig {
    /// 是否启用
    pub enabled: bool,
    /// 分区模式
    pub mode: RegionalMode,
    /// 区域列表
    pub regions: Vec<RegionConfig>,
    /// 全局提示词
    pub global_prompt: String,
    /// 全局负面提示词
    pub global_negative: String,
}

/// 分区模式
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum RegionalMode {
    /// 横向分区
    Horizontal,
    /// 纵向分区
    Vertical,
    /// 自定义分区
    Custom,
    /// Attention Couple
    AttentionCouple,
    /// Latent Couple
    LatentCouple,
}

// ============================================================================
// 角色一致性增强器
// ============================================================================

/// 角色一致性增强器
pub struct FaceConsistencyManager {
    /// 面部修复配置
    #[allow(dead_code)]
    face_fix_config: _FaceFixConfig,
    /// 区域提示词配置
    regional_config: _RegionalPromptingConfig,
    /// 角色参考图缓存
    reference_cache: HashMap<String, String>,
    /// 修复历史
    fix_history: Vec<_FaceFixResult>,
}

impl FaceConsistencyManager {
    /// 创建增强器
    ///
    /// STUB: Initializes with default FaceDetailer strategy and 0.5 detection threshold.
    /// Real implementation needs configurable strategy per platform (SD WebUI vs ComfyUI).
    pub fn new() -> Self {
        Self {
            face_fix_config: _FaceFixConfig {
                strategy: _FaceFixStrategy::FaceDetailer,
                detection_threshold: 0.5,
                fix_strength: 0.8,
                max_retries: 2,
                use_lora: true,
                lora_name: None,
                lora_weight: 0.7,
            },
            regional_config: _RegionalPromptingConfig {
                enabled: false,
                mode: RegionalMode::Horizontal,
                regions: vec![],
                global_prompt: String::new(),
                global_negative: String::new(),
            },
            reference_cache: HashMap::new(),
            fix_history: vec![],
        }
    }
    
    /// Set character reference image.
    ///
    /// Note: Stores image path in memory cache keyed by character_id.
    /// Real implementation needs:
    /// - Reference image validation (exists, correct format, reasonable size)
    /// - Reference embedding extraction (ArcFace/CosFace) for similarity comparison
    /// - Persistent cache (not just in-memory) for cross-session consistency
    pub fn set_reference(&mut self, character_id: &str, image_path: &str) {
        self.reference_cache.insert(character_id.to_string(), image_path.to_string());
    }
    
    /// Get character reference image path.
    ///
    /// Note: Returns cached image path if exists. No validation performed.
    /// Real implementation needs:
    /// - TTL-based cache expiration (references may become stale)
    /// - Fallback to KB-stored references if not in memory
    pub fn get_reference(&self, character_id: &str) -> Option<&String> {
        self.reference_cache.get(character_id)
    }
    
    /// 执行面部修复
    ///
    /// Real implementation needs:
    /// - Face detection (InsightFace/RetinaFace/MTCNN)
    /// - Face alignment and preprocessing
    /// - ADetailer/FaceDetailer inpainting pipeline (SD WebUI or ComfyUI)
    /// - Optionally apply LoRA weights for character consistency
    /// - Compute actual consistency score via face embedding similarity (ArcFace/CosFace)
    pub(crate) fn _fix_faces(
        &mut self,
        image_path: &str,
        _character_id: Option<&str>,
    ) -> _FaceFixResult {
        let result = _FaceFixResult {
            success: false,
            fixed_image_path: None,
            detected_faces: 0,
            fixed_faces: 0,
            consistency_score: 0.0,
            fix_time_ms: 0,
            error: Some(format!(
                "Face fix not yet wired. Requires face detector (InsightFace) + \
                 inpainting pipeline (ADetailer/FaceDetailer). Input: {}",
                image_path
            )),
        };

        self.fix_history.push(result.clone());
        result
    }
    
    /// 批量修复面部
    ///
    /// STUB: Delegates each image to `_fix_faces` sequentially.
    /// Real implementation needs:
    /// - Parallel processing with configurable concurrency
    /// - Batch face detection to amortize model load cost
    /// - Cross-frame consistency enforcement (reference embedding matching)
    pub(crate) fn _batch_fix_faces(
        &mut self,
        image_paths: &[String],
        character_id: Option<&str>,
    ) -> Vec<_FaceFixResult> {
        image_paths.iter()
            .map(|path| self._fix_faces(path, character_id))
            .collect()
    }
    
    /// 生成区域提示词
    ///
    /// Note: Concatenates region prompts in `[name: prompt (weight: w)]` syntax.
    /// Real implementation needs platform-specific syntax adaptation:
    /// - ComfyUI: RegionalPrompting node format
    /// - SD WebUI: ADetailer regional prompt syntax
    /// - Attention Couple: attention mask-based regions
    pub fn generate_regional_prompt(&self, base_prompt: &str) -> String {
        if !self.regional_config.enabled {
            return base_prompt.to_string();
        }
        
        let mut prompt = base_prompt.to_string();
        
        for region in &self.regional_config.regions {
            prompt.push_str(&format!(
                " [{}: {} (weight: {})]",
                region.name, region.prompt, region.weight
            ));
        }
        
        prompt
    }
    
    /// Set regional prompting configuration.
    ///
    /// Note: Replaces entire regional config. No validation of region bounds
    /// (should be normalized 0.0-1.0). Real implementation needs:
    /// - Bounds validation (x+w <= 1.0, y+h <= 1.0)
    /// - Overlap detection between regions
    /// - Platform-specific syntax validation (ComfyUI vs SD WebUI)
    pub fn set_regional_config(&mut self, config: _RegionalPromptingConfig) {
        self.regional_config = config;
    }
    
    /// Get repair statistics.
    ///
    /// Note: Computes aggregates from fix_history. avg_consistency_score is
    /// mean of all consistency_score values. reference_count is number of
    /// cached reference images (not total characters).
    pub fn statistics(&self) -> ConsistencyStats {
        let total_fixes = self.fix_history.len();
        let successful_fixes = self.fix_history.iter().filter(|r| r.success).count();
        let avg_consistency = if total_fixes > 0 {
            self.fix_history.iter().map(|r| r.consistency_score).sum::<f32>() / total_fixes as f32
        } else {
            0.0
        };
        
        ConsistencyStats {
            total_fixes,
            successful_fixes,
            failed_fixes: total_fixes - successful_fixes,
            avg_consistency_score: avg_consistency,
            reference_count: self.reference_cache.len(),
        }
    }
}

/// 一致性统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyStats {
    /// 总修复次数
    pub total_fixes: usize,
    /// 成功修复次数
    pub successful_fixes: usize,
    /// 失败修复次数
    pub failed_fixes: usize,
    /// 平均一致性分数
    pub avg_consistency_score: f32,
    /// 参考图数量
    pub reference_count: usize,
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_face_consistency_manager() {
        let mut manager = FaceConsistencyManager::new();
        
        // 设置参考图
        manager.set_reference("char_001", "/ref/char_001.png");
        assert!(manager.get_reference("char_001").is_some());
        
        // Face fix is not wired — expect explicit failure
        let result = manager._fix_faces("/input/test.png", Some("char_001"));
        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.consistency_score == 0.0);
        
        // 检查统计
        let stats = manager.statistics();
        assert_eq!(stats.total_fixes, 1);
        assert_eq!(stats.successful_fixes, 0);
    }
    
    #[test]
    fn test_regional_prompting() {
        let mut manager = FaceConsistencyManager::new();
        
        let config = _RegionalPromptingConfig {
            enabled: true,
            mode: RegionalMode::Horizontal,
            regions: vec![
                RegionConfig {
                    id: "left".to_string(),
                    name: "左侧".to_string(),
                    bounds: (0.0, 0.0, 0.5, 1.0),
                    character_id: Some("char_001".to_string()),
                    prompt: "角色A".to_string(),
                    weight: 1.0,
                },
                RegionConfig {
                    id: "right".to_string(),
                    name: "右侧".to_string(),
                    bounds: (0.5, 0.0, 0.5, 1.0),
                    character_id: Some("char_002".to_string()),
                    prompt: "角色B".to_string(),
                    weight: 1.0,
                },
            ],
            global_prompt: String::new(),
            global_negative: String::new(),
        };
        
        manager.set_regional_config(config);
        
        let prompt = manager.generate_regional_prompt("两个角色对话");
        assert!(prompt.contains("角色A"));
        assert!(prompt.contains("角色B"));
    }
}