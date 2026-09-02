//! 模板复用标签系统
//!
//! 管理动态漫技能模板的标签、分类、复用关系
//! 支持跨模块一致性检查和模板检索

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// ============================================================================
// 模板标签定义
// ============================================================================

/// 模板标签
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateTag {
    /// 标签ID
    pub id: String,
    /// 标签名称
    pub name: String,
    /// 标签描述
    pub description: String,
    /// 标签分类
    pub category: TagCategory,
    /// 关联的动态等级
    pub dynamic_level: Option<String>,
    /// 关联的分段模式
    pub segment_pattern: Vec<String>,
    /// 关联的情感曲线
    pub emotion_curve: Option<String>,
    /// 支持的平台
    pub supported_platforms: Vec<String>,
    /// 标签权重 (用于优先级排序)
    pub weight: f32,
    /// 标签使用次数
    pub usage_count: u32,
}

/// 标签分类
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TagCategory {
    /// 动态等级
    DynamicLevel,
    /// 节奏模式
    RhythmPattern,
    /// 转场类型
    TransitionType,
    /// 情感类型
    EmotionType,
    /// 场景类型
    SceneType,
    /// 角色类型
    CharacterType,
    /// 内容类型
    ContentType,
}

// ============================================================================
// 模板定义
// ============================================================================

/// 动漫技能模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTemplate {
    /// 模板ID
    pub id: String,
    /// 模板名称
    pub name: String,
    /// 模板描述
    pub description: String,
    /// 模板标签
    pub tags: Vec<String>,
    /// 模板内容 (JSON格式)
    pub content: serde_json::Value,
    /// 模板版本
    pub version: String,
    /// 模板创建时间
    pub created_at: u64,
    /// 模板更新时间
    pub updated_at: u64,
    /// 模板使用次数
    pub usage_count: u32,
    /// 模板评分 (0-5)
    pub rating: f32,
    /// 模板作者
    pub author: String,
    /// 模板来源
    pub source: String,
}

// ============================================================================
// 模板复用标签系统
// ============================================================================

/// 模板复用标签系统
/// 管理模板标签、模板、复用关系
pub struct TemplateTagRegistry {
    /// 所有标签
    tags: HashMap<String, TemplateTag>,
    /// 所有模板
    templates: HashMap<String, SkillTemplate>,
    /// 标签复用关系 (标签ID -> 复用的标签ID列表)
    tag_reuse: HashMap<String, Vec<String>>,
    /// 模板复用关系 (模板ID -> 复用的模板ID列表)
    template_reuse: HashMap<String, Vec<String>>,
}

impl TemplateTagRegistry {
    /// 创建空注册中心
    pub fn new() -> Self {
        Self {
            tags: HashMap::new(),
            templates: HashMap::new(),
            tag_reuse: HashMap::new(),
            template_reuse: HashMap::new(),
        }
    }
    
    /// 添加标签
    pub fn add_tag(&mut self, tag: TemplateTag) {
        self.tags.insert(tag.id.clone(), tag);
    }
    
    /// 添加模板
    pub fn add_template(&mut self, template: SkillTemplate) {
        self.templates.insert(template.id.clone(), template);
    }
    
    /// 添加标签复用关系
    pub fn add_tag_reuse(&mut self, source_tag_id: &str, target_tag_id: &str) {
        if self.tags.contains_key(source_tag_id) && self.tags.contains_key(target_tag_id) {
            self.tag_reuse
                .entry(source_tag_id.to_string())
                .or_insert_with(Vec::new)
                .push(target_tag_id.to_string());
        }
    }
    
    /// 添加模板复用关系
    pub fn add_template_reuse(&mut self, source_template_id: &str, target_template_id: &str) {
        if self.templates.contains_key(source_template_id) && self.templates.contains_key(target_template_id) {
            self.template_reuse
                .entry(source_template_id.to_string())
                .or_insert_with(Vec::new)
                .push(target_template_id.to_string());
        }
    }
    
    /// 根据标签查找模板
    pub fn find_templates_by_tag(&self, tag_id: &str) -> Vec<&SkillTemplate> {
        self.templates.values()
            .filter(|t| t.tags.contains(&tag_id.to_string()))
            .collect()
    }
    
    /// 根据分类查找标签
    pub fn find_tags_by_category(&self, category: TagCategory) -> Vec<&TemplateTag> {
        self.tags.values()
            .filter(|t| t.category == category)
            .collect()
    }
    
    /// 查找最常用的模板
    pub fn find_most_used_templates(&self, limit: usize) -> Vec<&SkillTemplate> {
        let mut templates: Vec<&SkillTemplate> = self.templates.values().collect();
        templates.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));
        templates.into_iter().take(limit).collect()
    }
    
    /// 查找评分最高的模板
    pub fn find_highest_rated_templates(&self, limit: usize) -> Vec<&SkillTemplate> {
        let mut templates: Vec<&SkillTemplate> = self.templates.values().collect();
        templates.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap_or(std::cmp::Ordering::Equal));
        templates.into_iter().take(limit).collect()
    }
    
    /// 获取标签的复用链
    pub fn get_tag_reuse_chain(&self, tag_id: &str, depth: usize) -> Vec<String> {
        let mut chain = Vec::new();
        let mut visited = std::collections::HashSet::new();
        
        self._get_tag_reuse_chain_recursive(tag_id, depth, &mut chain, &mut visited);
        chain
    }
    
    fn _get_tag_reuse_chain_recursive(&self, tag_id: &str, depth: usize, chain: &mut Vec<String>, visited: &mut std::collections::HashSet<String>) {
        if depth == 0 || visited.contains(tag_id) {
            return;
        }
        
        visited.insert(tag_id.to_string());
        
        if let Some(reused_tags) = self.tag_reuse.get(tag_id) {
            for reused_tag_id in reused_tags {
                chain.push(reused_tag_id.clone());
                self._get_tag_reuse_chain_recursive(reused_tag_id, depth - 1, chain, visited);
            }
        }
    }
    
    /// 获取模板的复用链
    pub fn get_template_reuse_chain(&self, template_id: &str, depth: usize) -> Vec<String> {
        let mut chain = Vec::new();
        let mut visited = std::collections::HashSet::new();
        
        self._get_template_reuse_chain_recursive(template_id, depth, &mut chain, &mut visited);
        chain
    }
    
    fn _get_template_reuse_chain_recursive(&self, template_id: &str, depth: usize, chain: &mut Vec<String>, visited: &mut std::collections::HashSet<String>) {
        if depth == 0 || visited.contains(template_id) {
            return;
        }
        
        visited.insert(template_id.to_string());
        
        if let Some(reused_templates) = self.template_reuse.get(template_id) {
            for reused_template_id in reused_templates {
                chain.push(reused_template_id.clone());
                self._get_template_reuse_chain_recursive(reused_template_id, depth - 1, chain, visited);
            }
        }
    }
    
    /// 统计信息
    pub fn statistics(&self) -> RegistryStatistics {
        let mut category_counts: HashMap<String, usize> = HashMap::new();
        for tag in self.tags.values() {
            let category_name = format!("{:?}", tag.category);
            *category_counts.entry(category_name).or_insert(0) += 1;
        }
        
        RegistryStatistics {
            tag_count: self.tags.len(),
            template_count: self.templates.len(),
            tag_reuse_count: self.tag_reuse.len(),
            template_reuse_count: self.template_reuse.len(),
            category_counts,
            most_used_tags: self.find_most_used_tags(5),
        }
    }
    
    fn find_most_used_tags(&self, limit: usize) -> Vec<(String, u32)> {
        let mut tags: Vec<(&TemplateTag)> = self.tags.values().collect();
        tags.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));
        tags.into_iter()
            .take(limit)
            .map(|t| (t.id.clone(), t.usage_count))
            .collect()
    }
}

/// 注册中心统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStatistics {
    /// 标签数量
    pub tag_count: usize,
    /// 模板数量
    pub template_count: usize,
    /// 标签复用关系数量
    pub tag_reuse_count: usize,
    /// 模板复用关系数量
    pub template_reuse_count: usize,
    /// 分类统计
    pub category_counts: HashMap<String, usize>,
    /// 最常用标签
    pub most_used_tags: Vec<(String, u32)>,
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_template_tag_registry() {
        let mut registry = TemplateTagRegistry::new();
        
        // 添加标签
        let tag1 = TemplateTag {
            id: "dynamic_micro".to_string(),
            name: "微动态".to_string(),
            description: "头发轻飘/眼皮颤动/手指微动".to_string(),
            category: TagCategory::DynamicLevel,
            dynamic_level: Some("Micro".to_string()),
            segment_pattern: vec![],
            emotion_curve: None,
            supported_platforms: vec![],
            weight: 1.0,
            usage_count: 10,
        };
        
        let tag2 = TemplateTag {
            id: "rhythm_slow".to_string(),
            name: "慢节奏".to_string(),
            description: "铺垫/抒情".to_string(),
            category: TagCategory::RhythmPattern,
            dynamic_level: None,
            segment_pattern: vec!["Setup".to_string()],
            emotion_curve: None,
            supported_platforms: vec![],
            weight: 1.0,
            usage_count: 5,
        };
        
        registry.add_tag(tag1);
        registry.add_tag(tag2);
        
        // 添加模板
        let template = SkillTemplate {
            id: "template_001".to_string(),
            name: "情感叙事模板".to_string(),
            description: "用于情感类动态漫".to_string(),
            tags: vec!["dynamic_micro".to_string(), "rhythm_slow".to_string()],
            content: serde_json::json!({"type": "emotional"}),
            version: "1.0".to_string(),
            created_at: 0,
            updated_at: 0,
            usage_count: 20,
            rating: 4.5,
            author: "system".to_string(),
            source: "builtin".to_string(),
        };
        
        registry.add_template(template);
        
        // 验证
        let stats = registry.statistics();
        assert_eq!(stats.tag_count, 2);
        assert_eq!(stats.template_count, 1);
    }
    
    #[test]
    fn test_find_templates_by_tag() {
        let mut registry = TemplateTagRegistry::new();
        
        let tag = TemplateTag {
            id: "test_tag".to_string(),
            name: "测试标签".to_string(),
            description: "测试".to_string(),
            category: TagCategory::ContentType,
            dynamic_level: None,
            segment_pattern: vec![],
            emotion_curve: None,
            supported_platforms: vec![],
            weight: 1.0,
            usage_count: 0,
        };
        
        registry.add_tag(tag);
        
        let template = SkillTemplate {
            id: "test_template".to_string(),
            name: "测试模板".to_string(),
            description: "测试".to_string(),
            tags: vec!["test_tag".to_string()],
            content: serde_json::json!({}),
            version: "1.0".to_string(),
            created_at: 0,
            updated_at: 0,
            usage_count: 0,
            rating: 0.0,
            author: "system".to_string(),
            source: "builtin".to_string(),
        };
        
        registry.add_template(template);
        
        let found = registry.find_templates_by_tag("test_tag");
        assert_eq!(found.len(), 1);
    }
}