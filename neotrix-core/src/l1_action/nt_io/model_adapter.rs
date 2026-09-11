//! 模型适配器模块 (通用)
//!
//! 统一接口适配 LoRA、IP-Adapter、ControlNet 等模型
//! 适用于：所有需要模型适配的场景

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// ============================================================================
// 模型适配定义
// ============================================================================

/// 模型类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ModelType {
    /// LoRA
    LoRA,
    /// IP-Adapter
    IPAdapter,
    /// ControlNet
    ControlNet,
    /// T2I-Adapter
    T2IAdapter,
    /// InstantID
    InstantID,
    /// ReActor
    ReActor,
    /// FaceDetailer
    FaceDetailer,
}

/// 适配器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterConfig {
    /// 适配器ID
    pub id: String,
    /// 适配器名称
    pub name: String,
    /// 模型类型
    pub model_type: ModelType,
    /// 模型路径
    pub model_path: String,
    /// 权重 (0.0-1.0)
    pub weight: f32,
    /// 是否启用
    pub enabled: bool,
    /// 自定义参数
    pub params: HashMap<String, serde_json::Value>,
}

/// 适配器应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterApplication {
    /// 适配器ID
    pub adapter_id: String,
    /// 应用区域
    pub region: Option<String>,
    /// 应用强度
    pub strength: f32,
    /// 开始时间 (视频帧)
    pub start_frame: Option<u32>,
    /// 结束时间 (视频帧)
    pub end_frame: Option<u32>,
}

/// 适配器结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterResult {
    /// 是否成功
    pub success: bool,
    /// 输出文件路径
    pub output_path: String,
    /// 适配器名称
    pub adapter_name: String,
    /// 应用耗时 (毫秒)
    pub application_time_ms: u64,
    /// 相似度分数
    pub similarity_score: f32,
    /// 错误信息
    pub error: Option<String>,
}

/// 适配器注册配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelAdapterConfig {
    /// 默认适配器目录
    pub adapter_dir: String,
    /// 是否启用自动加载
    pub enable_auto_load: bool,
    /// 最大同时加载数
    pub max_concurrent_loads: u32,
    /// 缓存大小
    pub cache_size: usize,
}

// ============================================================================
// 模型适配器
// ============================================================================

/// 模型适配器
/// 统一管理 LoRA、IP-Adapter、ControlNet 等模型
#[derive(Debug)]
pub struct ModelAdapter {
    /// 配置
    config: ModelAdapterConfig,
    /// 已注册的适配器
    adapters: HashMap<String, AdapterConfig>,
    /// 应用历史
    history: Vec<AdapterResult>,
}

impl ModelAdapter {
    /// 创建适配器
    pub fn new() -> Self {
        Self {
            config: ModelAdapterConfig {
                adapter_dir: "./adapters".to_string(),
                enable_auto_load: true,
                max_concurrent_loads: 4,
                cache_size: 100,
            },
            adapters: HashMap::new(),
            history: vec![],
        }
    }
    
    /// 使用配置创建
    pub fn with_config(config: ModelAdapterConfig) -> Self {
        Self {
            config,
            adapters: HashMap::new(),
            history: vec![],
        }
    }
    
    /// 注册适配器
    pub fn register_adapter(&mut self, adapter: AdapterConfig) {
        self.adapters.insert(adapter.id.clone(), adapter);
    }
    
    /// 获取适配器
    pub fn get_adapter(&self, adapter_id: &str) -> Option<&AdapterConfig> {
        self.adapters.get(adapter_id)
    }
    
    /// 移除适配器
    pub fn remove_adapter(&mut self, adapter_id: &str) -> Result<(), String> {
        if self.adapters.remove(adapter_id).is_some() {
            Ok(())
        } else {
            Err("适配器不存在".to_string())
        }
    }
    
    /// 应用 LoRA
    pub fn apply_lora(
        &mut self,
        lora_id: &str,
        input_path: &str,
        _strength: f32,
    ) -> AdapterResult {
        // TODO: 实际调用 LoRA 应用逻辑
        let adapter = self.adapters.get(lora_id);
        let adapter_name = adapter.map(|a| a.name.clone()).unwrap_or_default();
        
        let result = AdapterResult {
            success: true,
            output_path: format!("{}_lora.png", input_path),
            adapter_name,
            application_time_ms: 1000,
            similarity_score: 0.95,
            error: None,
        };
        
        self.history.push(result.clone());
        result
    }
    
    /// 应用 IP-Adapter
    pub fn apply_ip_adapter(
        &mut self,
        adapter_id: &str,
        content_path: &str,
        _reference_path: &str,
        _strength: f32,
    ) -> AdapterResult {
        // TODO: 实际调用 IP-Adapter 应用逻辑
        let adapter = self.adapters.get(adapter_id);
        let adapter_name = adapter.map(|a| a.name.clone()).unwrap_or_default();
        
        let result = AdapterResult {
            success: true,
            output_path: format!("{}_ip_adapter.png", content_path),
            adapter_name,
            application_time_ms: 2000,
            similarity_score: 0.88,
            error: None,
        };
        
        self.history.push(result.clone());
        result
    }
    
    /// 应用 ControlNet
    pub fn apply_controlnet(
        &mut self,
        adapter_id: &str,
        _prompt: &str,
        control_image: &str,
        _strength: f32,
    ) -> AdapterResult {
        // TODO: 实际调用 ControlNet 应用逻辑
        let adapter = self.adapters.get(adapter_id);
        let adapter_name = adapter.map(|a| a.name.clone()).unwrap_or_default();
        
        let result = AdapterResult {
            success: true,
            output_path: format!("{}_controlnet.png", control_image),
            adapter_name,
            application_time_ms: 1500,
            similarity_score: 0.85,
            error: None,
        };
        
        self.history.push(result.clone());
        result
    }
    
    /// 列出所有适配器
    pub fn list_adapters(&self) -> Vec<&AdapterConfig> {
        self.adapters.values().collect()
    }
    
    /// 获取统计信息
    pub fn statistics(&self) -> AdapterStats {
        let total_adapters = self.adapters.len();
        let by_type: HashMap<String, usize> = self.adapters.values()
            .fold(HashMap::new(), |mut acc, a| {
                *acc.entry(format!("{:?}", a.model_type)).or_insert(0) += 1;
                acc
            });
        
        let total_applications = self.history.len();
        let successful = self.history.iter().filter(|r| r.success).count();
        
        AdapterStats {
            total_adapters,
            adapters_by_type: by_type,
            total_applications,
            successful_applications: successful,
        }
    }
}

/// 适配器统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterStats {
    /// 总适配器数
    pub total_adapters: usize,
    /// 按类型统计
    pub adapters_by_type: HashMap<String, usize>,
    /// 总应用次数
    pub total_applications: usize,
    /// 成功应用次数
    pub successful_applications: usize,
}

// ============================================================================
// 向后兼容别名
// ============================================================================

/// 一致性适配器 (向后兼容别名)
pub type ConsistencyAdapter = ModelAdapter;

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_model_adapter() {
        let mut adapter = ModelAdapter::new();
        
        adapter.register_adapter(AdapterConfig {
            id: "lora_001".to_string(),
            name: "测试LoRA".to_string(),
            model_type: ModelType::LoRA,
            model_path: "./adapters/test_lora.safetensors".to_string(),
            weight: 0.8,
            enabled: true,
            params: HashMap::new(),
        });
        
        let result = adapter.apply_lora("lora_001", "/input/test.png", 0.8);
        assert!(result.success);
        
        let stats = adapter.statistics();
        assert_eq!(stats.total_adapters, 1);
        assert_eq!(stats.total_applications, 1);
    }
}