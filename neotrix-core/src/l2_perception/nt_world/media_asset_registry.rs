//! 媒体资产库模块 (通用)
//!
//! 管理视觉资产（角色、场景、道具、模板）
//! 适用于：所有视觉内容创作场景

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// ============================================================================
// 资产定义
// ============================================================================

/// 资产类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AssetType {
    /// 角色
    Character,
    /// 场景
    Scene,
    /// 道具
    Prop,
    /// 特效
    Effect,
    /// 音频
    Audio,
    /// 模板
    Template,
    /// 参考图
    Reference,
}

/// 资产状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AssetStatus {
    /// 草稿
    Draft,
    /// 审核中
    Reviewing,
    /// 已发布
    Published,
    /// 已归档
    Archived,
}

/// 资产元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetadata {
    /// 资产ID
    pub id: String,
    /// 资产名称
    pub name: String,
    /// 资产类型
    pub asset_type: AssetType,
    /// 状态
    pub status: AssetStatus,
    /// 文件路径
    pub file_path: String,
    /// 缩略图路径
    pub thumbnail_path: Option<String>,
    /// 标签
    pub tags: Vec<String>,
    /// 描述
    pub description: Option<String>,
    /// 创建时间
    pub created_at: u64,
    /// 更新时间
    pub updated_at: u64,
    /// 版本
    pub version: u32,
    /// 使用次数
    pub usage_count: u32,
    /// 自定义属性
    pub custom_attributes: HashMap<String, serde_json::Value>,
}

/// 资产库配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetRegistryConfig {
    /// 存储路径
    pub storage_path: String,
    /// 最大资产数
    pub max_assets: usize,
    /// 是否启用版本控制
    pub enable_versioning: bool,
    /// 是否启用自动缩略图
    pub enable_auto_thumbnail: bool,
    /// 缩略图尺寸
    pub thumbnail_size: (u32, u32),
    /// 是否启用搜索索引
    pub enable_search_index: bool,
}

/// 搜索查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetSearchQuery {
    /// 关键词
    pub keyword: Option<String>,
    /// 资产类型过滤
    pub asset_type: Option<AssetType>,
    /// 状态过滤
    pub status: Option<AssetStatus>,
    /// 标签过滤
    pub tags: Option<Vec<String>>,
    /// 排序字段
    pub sort_by: Option<String>,
    /// 是否降序
    pub descending: bool,
    /// 分页偏移
    pub offset: usize,
    /// 分页大小
    pub limit: usize,
}

/// 搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetSearchResult {
    /// 总数
    pub total: usize,
    /// 资产列表
    pub assets: Vec<AssetMetadata>,
    /// 查询耗时 (毫秒)
    pub query_time_ms: u64,
}

// ============================================================================
// 媒体资产库
// ============================================================================

/// 媒体资产库
/// 管理视觉资产的创建、存储、检索
pub struct MediaAssetRegistry {
    /// 配置
    config: AssetRegistryConfig,
    /// 资产存储
    assets: HashMap<String, AssetMetadata>,
    /// 标签索引
    tag_index: HashMap<String, Vec<String>>,
    /// 类型索引
    type_index: HashMap<AssetType, Vec<String>>,
}

impl MediaAssetRegistry {
    /// 创建资产库
    pub fn new(storage_path: &str) -> Self {
        Self {
            config: AssetRegistryConfig {
                storage_path: storage_path.to_string(),
                max_assets: 10000,
                enable_versioning: true,
                enable_auto_thumbnail: true,
                thumbnail_size: (256, 256),
                enable_search_index: true,
            },
            assets: HashMap::new(),
            tag_index: HashMap::new(),
            type_index: HashMap::new(),
        }
    }
    
    /// 使用配置创建
    pub fn with_config(config: AssetRegistryConfig) -> Self {
        Self {
            config,
            assets: HashMap::new(),
            tag_index: HashMap::new(),
            type_index: HashMap::new(),
        }
    }
    
    /// 添加资产
    pub fn add_asset(&mut self, metadata: AssetMetadata) -> Result<String, String> {
        if self.assets.len() >= self.config.max_assets {
            return Err("资产库已满".to_string());
        }
        
        let id = metadata.id.clone();
        
        // 更新标签索引
        for tag in &metadata.tags {
            self.tag_index.entry(tag.clone())
                .or_insert_with(Vec::new)
                .push(id.clone());
        }
        
        // 更新类型索引
        self.type_index.entry(metadata.asset_type)
            .or_insert_with(Vec::new)
            .push(id.clone());
        
        self.assets.insert(id.clone(), metadata);
        Ok(id)
    }
    
    /// 获取资产
    pub fn get_asset(&self, asset_id: &str) -> Option<&AssetMetadata> {
        self.assets.get(asset_id)
    }
    
    /// 更新资产
    pub fn update_asset(&mut self, asset_id: &str, metadata: AssetMetadata) -> Result<(), String> {
        if !self.assets.contains_key(asset_id) {
            return Err("资产不存在".to_string());
        }
        
        self.assets.insert(asset_id.to_string(), metadata);
        Ok(())
    }
    
    /// 删除资产
    pub fn delete_asset(&mut self, asset_id: &str) -> Result<(), String> {
        if let Some(metadata) = self.assets.remove(asset_id) {
            // 清理标签索引
            for tag in &metadata.tags {
                if let Some(ids) = self.tag_index.get_mut(tag) {
                    ids.retain(|id| id != asset_id);
                }
            }
            
            // 清理类型索引
            if let Some(ids) = self.type_index.get_mut(&metadata.asset_type) {
                ids.retain(|id| id != asset_id);
            }
            
            Ok(())
        } else {
            Err("资产不存在".to_string())
        }
    }
    
    /// 搜索资产
    pub fn search(&self, query: &AssetSearchQuery) -> AssetSearchResult {
        let start = std::time::Instant::now();
        
        let mut candidates: Vec<&AssetMetadata> = self.assets.values().collect();
        
        // 关键词过滤
        if let Some(ref keyword) = query.keyword {
            candidates.retain(|a| {
                a.name.contains(keyword)
                    || a.description.as_ref().map_or(false, |d| d.contains(keyword))
                    || a.tags.iter().any(|t| t.contains(keyword))
            });
        }
        
        // 类型过滤
        if let Some(asset_type) = query.asset_type {
            candidates.retain(|a| a.asset_type == asset_type);
        }
        
        // 状态过滤
        if let Some(status) = query.status {
            candidates.retain(|a| a.status == status);
        }
        
        // 标签过滤
        if let Some(ref tags) = query.tags {
            candidates.retain(|a| tags.iter().all(|t| a.tags.contains(t)));
        }
        
        let total = candidates.len();
        
        // 排序
        if let Some(ref sort_by) = query.sort_by {
            candidates.sort_by(|a, b| {
                let cmp = match sort_by.as_str() {
                    "name" => a.name.cmp(&b.name),
                    "created_at" => a.created_at.cmp(&b.created_at),
                    "usage_count" => a.usage_count.cmp(&b.usage_count),
                    _ => a.updated_at.cmp(&b.updated_at),
                };
                if query.descending { cmp.reverse() } else { cmp }
            });
        }
        
        // 分页
        let assets: Vec<AssetMetadata> = candidates.into_iter()
            .skip(query.offset)
            .take(query.limit)
            .cloned()
            .collect();
        
        let query_time_ms = start.elapsed().as_millis() as u64;
        
        AssetSearchResult {
            total,
            assets,
            query_time_ms,
        }
    }
    
    /// 获取统计信息
    pub fn statistics(&self) -> AssetStats {
        let total_assets = self.assets.len();
        let by_type: HashMap<String, usize> = self.type_index.iter()
            .map(|(k, v)| (format!("{:?}", k), v.len()))
            .collect();
        
        AssetStats {
            total_assets,
            assets_by_type: by_type,
        }
    }
}

/// 资产统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetStats {
    /// 总资产数
    pub total_assets: usize,
    /// 按类型统计
    pub assets_by_type: HashMap<String, usize>,
}

// ============================================================================
// 向后兼容别名
// ============================================================================

/// 资产注册表 (向后兼容别名)
pub type AssetRegistry = MediaAssetRegistry;

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_asset_registry() {
        let mut registry = MediaAssetRegistry::new("/assets");
        
        let metadata = AssetMetadata {
            id: "asset_001".to_string(),
            name: "测试角色".to_string(),
            asset_type: AssetType::Character,
            status: AssetStatus::Published,
            file_path: "/assets/character_001.png".to_string(),
            thumbnail_path: None,
            tags: vec!["主角".to_string(), "男性".to_string()],
            description: Some("测试角色描述".to_string()),
            created_at: 0,
            updated_at: 0,
            version: 1,
            usage_count: 0,
            custom_attributes: HashMap::new(),
        };
        
        let id = registry.add_asset(metadata).unwrap();
        assert_eq!(id, "asset_001");
        
        let asset = registry.get_asset("asset_001").unwrap();
        assert_eq!(asset.name, "测试角色");
    }
    
    #[test]
    fn test_search() {
        let mut registry = MediaAssetRegistry::new("/assets");
        
        // 添加测试资产
        for i in 0..5 {
            let metadata = AssetMetadata {
                id: format!("asset_{}", i),
                name: format!("角色{}", i),
                asset_type: AssetType::Character,
                status: AssetStatus::Published,
                file_path: format!("/assets/character_{}.png", i),
                thumbnail_path: None,
                tags: vec!["测试".to_string()],
                description: None,
                created_at: i as u64,
                updated_at: i as u64,
                version: 1,
                usage_count: 0,
                custom_attributes: HashMap::new(),
            };
            registry.add_asset(metadata).unwrap();
        }
        
        let result = registry.search(&AssetSearchQuery {
            keyword: Some("角色".to_string()),
            asset_type: None,
            status: None,
            tags: None,
            sort_by: Some("name".to_string()),
            descending: false,
            offset: 0,
            limit: 10,
        });
        
        assert_eq!(result.total, 5);
    }
}