//! 视觉一致性管理模块 (通用)
//!
//! 管理视觉元素（角色、物体、场景）跨帧/跨镜头的一致性
//! 支持自动修复、分区控制、多模型适配
//! 适用于：漫剧、真人短剧、动画、广告、教育视频等

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// ============================================================================
// 一致性策略定义
// ============================================================================

/// 一致性修复策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsistencyFixStrategy {
    /// ADetailer 自动修复
    ADetailer,
    /// FaceDetailer 自动修复
    FaceDetailer,
    /// IP-Adapter 特征注入
    IPAdapter,
    /// LoRA 微调
    LoRA,
    /// ReActor 换脸
    ReActor,
    /// 手动 Inpainting
    ManualInpaint,
    /// InstantID
    InstantID,
}

/// 视觉元素类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum VisualElementType {
    /// 角色
    Character,
    /// 物体
    Object,
    /// 场景
    Scene,
    /// 道具
    Prop,
    /// 服装
    Costume,
}

/// 一致性修复配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyFixConfig {
    /// 修复策略
    pub strategy: ConsistencyFixStrategy,
    /// 检测阈值 (0.0-1.0)
    pub detection_threshold: f32,
    /// 修复强度 (0.0-1.0)
    pub fix_strength: f32,
    /// 最大重试次数
    pub max_retries: u32,
    /// 是否启用模型增强
    pub use_model_enhancement: bool,
    /// 模型名称
    pub model_name: Option<String>,
    /// 模型权重
    pub model_weight: f32,
}

/// 一致性修复结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyFixResult {
    /// 是否成功
    pub success: bool,
    /// 修复后的图片路径
    pub fixed_image_path: Option<String>,
    /// 检测到的元素数量
    pub detected_elements: u32,
    /// 修复的元素数量
    pub fixed_elements: u32,
    /// 一致性分数 (0.0-1.0)
    pub consistency_score: f32,
    /// 修复耗时 (毫秒)
    pub fix_time_ms: u64,
    /// 错误信息
    pub error: Option<String>,
}

// ============================================================================
// 分区控制定义
// ============================================================================

/// 分区配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionConfig {
    /// 区域ID
    pub id: String,
    /// 区域名称
    pub name: String,
    /// 区域边界 (x, y, width, height) 归一化坐标
    pub bounds: (f32, f32, f32, f32),
    /// 区域内的元素ID
    pub element_id: Option<String>,
    /// 区域提示词
    pub prompt: String,
    /// 区域权重
    pub weight: f32,
}

/// 分区控制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionalControlConfig {
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
// 视觉一致性管理器
// ============================================================================

/// 视觉一致性管理器
/// 管理视觉元素跨帧/跨镜头的一致性
#[allow(dead_code)]
pub struct VisualConsistencyManager {
    /// 一致性修复配置
    fix_config: ConsistencyFixConfig,
    /// 分区控制配置
    regional_config: RegionalControlConfig,
    /// 参考图缓存 (元素ID -> 图片路径)
    reference_cache: HashMap<String, String>,
    /// 修复历史
    fix_history: Vec<ConsistencyFixResult>,
}

impl VisualConsistencyManager {
    /// 创建管理器
    pub fn new() -> Self {
        Self {
            fix_config: ConsistencyFixConfig {
                strategy: ConsistencyFixStrategy::FaceDetailer,
                detection_threshold: 0.5,
                fix_strength: 0.8,
                max_retries: 2,
                use_model_enhancement: true,
                model_name: None,
                model_weight: 0.7,
            },
            regional_config: RegionalControlConfig {
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
    
    /// 使用配置创建
    pub fn with_config(fix_config: ConsistencyFixConfig, regional_config: RegionalControlConfig) -> Self {
        Self {
            fix_config,
            regional_config,
            reference_cache: HashMap::new(),
            fix_history: vec![],
        }
    }
    
    /// 设置参考图
    pub fn set_reference(&mut self, element_id: &str, image_path: &str) {
        self.reference_cache.insert(element_id.to_string(), image_path.to_string());
    }
    
    /// 获取参考图
    pub fn get_reference(&self, element_id: &str) -> Option<&String> {
        self.reference_cache.get(element_id)
    }
    
    /// 执行一致性修复
    pub fn fix_consistency(
        &mut self,
        image_path: &str,
        _element_id: Option<&str>,
        _element_type: VisualElementType,
    ) -> ConsistencyFixResult {
        // TODO: 实际调用修复逻辑
        let result = ConsistencyFixResult {
            success: true,
            fixed_image_path: Some(format!("{}_fixed.png", image_path)),
            detected_elements: 1,
            fixed_elements: 1,
            consistency_score: 0.95,
            fix_time_ms: 2000,
            error: None,
        };
        
        self.fix_history.push(result.clone());
        result
    }
    
    /// 批量修复
    pub fn batch_fix(
        &mut self,
        image_paths: &[String],
        element_id: Option<&str>,
        element_type: VisualElementType,
    ) -> Vec<ConsistencyFixResult> {
        image_paths.iter()
            .map(|path| self.fix_consistency(path, element_id, element_type))
            .collect()
    }
    
    /// 生成分区提示词
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
    
    /// 设置分区配置
    pub fn set_regional_config(&mut self, config: RegionalControlConfig) {
        self.regional_config = config;
    }
    
    /// 获取统计信息
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
// 向后兼容别名
// ============================================================================

/// 角色一致性增强器 (向后兼容别名)
pub type FaceConsistencyManager = VisualConsistencyManager;

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_visual_consistency_manager() {
        let mut manager = VisualConsistencyManager::new();
        
        // 设置参考图
        manager.set_reference("element_001", "/ref/element_001.png");
        assert!(manager.get_reference("element_001").is_some());
        
        // 执行修复
        let result = manager.fix_consistency(
            "/input/test.png",
            Some("element_001"),
            VisualElementType::Character,
        );
        assert!(result.success);
        assert!(result.consistency_score > 0.9);
        
        // 检查统计
        let stats = manager.statistics();
        assert_eq!(stats.total_fixes, 1);
        assert_eq!(stats.successful_fixes, 1);
    }
    
    #[test]
    fn test_regional_control() {
        let mut manager = VisualConsistencyManager::new();
        
        let config = RegionalControlConfig {
            enabled: true,
            mode: RegionalMode::Horizontal,
            regions: vec![
                RegionConfig {
                    id: "left".to_string(),
                    name: "左侧".to_string(),
                    bounds: (0.0, 0.0, 0.5, 1.0),
                    element_id: Some("element_001".to_string()),
                    prompt: "元素A".to_string(),
                    weight: 1.0,
                },
                RegionConfig {
                    id: "right".to_string(),
                    name: "右侧".to_string(),
                    bounds: (0.5, 0.0, 0.5, 1.0),
                    element_id: Some("element_002".to_string()),
                    prompt: "元素B".to_string(),
                    weight: 1.0,
                },
            ],
            global_prompt: String::new(),
            global_negative: String::new(),
        };
        
        manager.set_regional_config(config);
        
        let prompt = manager.generate_regional_prompt("两个元素对话");
        assert!(prompt.contains("元素A"));
        assert!(prompt.contains("元素B"));
    }
}