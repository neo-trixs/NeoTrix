//! 主体库管理模块
//!
//! 管理角色、场景、特效等标准化资产
//! 支持资产沉淀、复用、版本控制

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
    /// 配音
    Voice,
    /// 音效
    Sound,
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
    /// 资产描述
    pub description: String,
    /// 资产标签
    pub tags: Vec<String>,
    /// 资产版本
    pub version: String,
    /// 创建时间
    pub created_at: u64,
    /// 更新时间
    pub updated_at: u64,
    /// 使用次数
    pub usage_count: u32,
    /// 资产评分 (0-5)
    pub rating: f32,
    /// 资产作者
    pub author: String,
    /// 资产来源
    pub source: String,
}

/// 角色资产
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _CharacterAsset {
    /// 基础元数据
    pub metadata: AssetMetadata,
    /// 视觉描述
    pub visual_description: String,
    /// 性别
    pub gender: Option<String>,
    /// 年龄
    pub age: Option<String>,
    /// 发型
    pub hairstyle: Option<String>,
    /// 发色
    pub hair_color: Option<String>,
    /// 服装
    pub outfit: Option<String>,
    /// 配饰
    pub accessories: Vec<String>,
    /// 三视图路径
    pub three_view_paths: Vec<String>,
    /// LoRA 模型路径
    pub lora_path: Option<String>,
    /// IP-Adapter 特征向量
    pub ip_adapter_features: Option<Vec<f32>>,
    /// 动作库
    pub action_library: Vec<String>,
    /// 情感表达库
    pub emotion_library: Vec<String>,
}

/// 场景资产
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _SceneAsset {
    /// 基础元数据
    pub metadata: AssetMetadata,
    /// 场景描述
    pub scene_description: String,
    /// 场景类型
    pub scene_type: String,
    /// 光照条件
    pub lighting: Option<String>,
    /// 天气条件
    pub weather: Option<String>,
    /// 时间段
    pub time_of_day: Option<String>,
    /// 视角
    pub perspective: Option<String>,
    /// 图层路径
    pub layer_paths: Vec<String>,
    /// 场景模板路径
    pub template_path: Option<String>,
}

/// 特效资产
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _EffectAsset {
    /// 基础元数据
    pub metadata: AssetMetadata,
    /// 特效描述
    pub effect_description: String,
    /// 特效类型
    pub effect_type: String,
    /// 持续时间 (毫秒)
    pub duration_ms: u32,
    /// 触发条件
    pub trigger_condition: Option<String>,
    /// 特效参数
    pub parameters: HashMap<String, serde_json::Value>,
}

/// 配音资产
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _VoiceAsset {
    /// 基础元数据
    pub metadata: AssetMetadata,
    /// 音色描述
    pub voice_description: String,
    /// 音色类型
    pub voice_type: String,
    /// 语言
    pub language: String,
    /// 情感风格
    pub emotion_style: Vec<String>,
    /// TTS 模型路径
    pub tts_model_path: Option<String>,
    /// 音频样本路径
    pub sample_paths: Vec<String>,
}

// ============================================================================
// 主体库
// ============================================================================

/// 主体库
/// 管理所有标准化资产
pub struct AssetRegistry {
    /// 角色资产
    characters: HashMap<String, _CharacterAsset>,
    /// 场景资产
    scenes: HashMap<String, _SceneAsset>,
    /// 特效资产
    effects: HashMap<String, _EffectAsset>,
    /// 配音资产
    voices: HashMap<String, _VoiceAsset>,
    /// 资产关系 (资产ID -> 关联资产ID列表)
    relations: HashMap<String, Vec<String>>,
}

impl AssetRegistry {
    /// 创建空主体库
    pub fn new() -> Self {
        Self {
            characters: HashMap::new(),
            scenes: HashMap::new(),
            effects: HashMap::new(),
            voices: HashMap::new(),
            relations: HashMap::new(),
        }
    }
    
    // ===== 角色管理 =====
    
    /// 添加角色
    pub fn _add_character(&mut self, character: _CharacterAsset) {
        self.characters.insert(character.metadata.id.clone(), character);
    }
    
    /// 获取角色
    pub fn _get_character(&self, id: &str) -> Option<&_CharacterAsset> {
        self.characters.get(id)
    }
    
    /// 获取角色可变引用
    pub fn _get_character_mut(&mut self, id: &str) -> Option<&mut _CharacterAsset> {
        self.characters.get_mut(id)
    }
    
    /// 列出所有角色
    pub fn _list_characters(&self) -> Vec<&_CharacterAsset> {
        self.characters.values().collect()
    }
    
    /// 搜索角色
    pub fn _search_characters(&self, query: &str) -> Vec<&_CharacterAsset> {
        self.characters.values()
            .filter(|c| {
                c.metadata.name.contains(query) ||
                c.metadata.description.contains(query) ||
                c.metadata.tags.iter().any(|t| t.contains(query)) ||
                c.visual_description.contains(query)
            })
            .collect()
    }
    
    // ===== 场景管理 =====
    
    /// 添加场景
    pub fn add_scene(&mut self, scene: _SceneAsset) {
        self.scenes.insert(scene.metadata.id.clone(), scene);
    }
    
    /// 获取场景
    pub fn _get_scene(&self, id: &str) -> Option<&_SceneAsset> {
        self.scenes.get(id)
    }
    
    /// 列出所有场景
    pub fn _list_scenes(&self) -> Vec<&_SceneAsset> {
        self.scenes.values().collect()
    }
    
    /// 搜索场景
    pub fn _search_scenes(&self, query: &str) -> Vec<&_SceneAsset> {
        self.scenes.values()
            .filter(|s| {
                s.metadata.name.contains(query) ||
                s.metadata.description.contains(query) ||
                s.scene_description.contains(query)
            })
            .collect()
    }
    
    // ===== 特效管理 =====
    
    /// 添加特效
    pub fn add_effect(&mut self, effect: _EffectAsset) {
        self.effects.insert(effect.metadata.id.clone(), effect);
    }
    
    /// 获取特效
    pub fn _get_effect(&self, id: &str) -> Option<&_EffectAsset> {
        self.effects.get(id)
    }
    
    /// 列出所有特效
    pub fn _list_effects(&self) -> Vec<&_EffectAsset> {
        self.effects.values().collect()
    }
    
    // ===== 配音管理 =====
    
    /// 添加配音
    pub fn _add_voice(&mut self, voice: _VoiceAsset) {
        self.voices.insert(voice.metadata.id.clone(), voice);
    }
    
    /// 获取配音
    pub fn _get_voice(&self, id: &str) -> Option<&_VoiceAsset> {
        self.voices.get(id)
    }
    
    /// 列出所有配音
    pub fn _list_voices(&self) -> Vec<&_VoiceAsset> {
        self.voices.values().collect()
    }
    
    // ===== 关系管理 =====
    
    /// 添加资产关系
    pub fn add_relation(&mut self, asset_id: &str, related_id: &str) {
        self.relations
            .entry(asset_id.to_string())
            .or_insert_with(Vec::new)
            .push(related_id.to_string());
    }
    
    /// 获取关联资产
    pub fn _get_related_assets(&self, asset_id: &str) -> Vec<&str> {
        self.relations.get(asset_id)
            .map(|v| v.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }
    
    // ===== 资产复用 =====
    
    /// 复用资产
    pub fn _reuse_asset(&self, asset_id: &str) -> Option<AssetMetadata> {
        match self.characters.get(asset_id) {
            Some(c) => {
                let mut meta = c.metadata.clone();
                meta.usage_count += 1;
                Some(meta)
            }
            None => match self.scenes.get(asset_id) {
                Some(s) => {
                    let mut meta = s.metadata.clone();
                    meta.usage_count += 1;
                    Some(meta)
                }
                None => match self.effects.get(asset_id) {
                    Some(e) => {
                        let mut meta = e.metadata.clone();
                        meta.usage_count += 1;
                        Some(meta)
                    }
                    None => self.voices.get(asset_id).map(|v| {
                        let mut meta = v.metadata.clone();
                        meta.usage_count += 1;
                        meta
                    })
                }
            }
        }
    }
    
    // ===== 统计 =====
    
    /// 获取统计信息
    pub fn statistics(&self) -> RegistryStats {
        RegistryStats {
            character_count: self.characters.len(),
            scene_count: self.scenes.len(),
            effect_count: self.effects.len(),
            voice_count: self.voices.len(),
            relation_count: self.relations.len(),
            total_assets: self.characters.len() + self.scenes.len() + self.effects.len() + self.voices.len(),
        }
    }
}

/// 主体库统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    /// 角色数量
    pub character_count: usize,
    /// 场景数量
    pub scene_count: usize,
    /// 特效数量
    pub effect_count: usize,
    /// 配音数量
    pub voice_count: usize,
    /// 关系数量
    pub relation_count: usize,
    /// 总资产数
    pub total_assets: usize,
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_asset_registry() {
        let mut registry = AssetRegistry::new();
        
        // 添加角色
        let character = _CharacterAsset {
            metadata: AssetMetadata {
                id: "char_001".to_string(),
                name: "主角".to_string(),
                asset_type: AssetType::Character,
                description: "男主角".to_string(),
                tags: vec!["主角".to_string()],
                version: "1.0".to_string(),
                created_at: 0,
                updated_at: 0,
                usage_count: 0,
                rating: 4.5,
                author: "system".to_string(),
                source: "builtin".to_string(),
            },
            visual_description: "黑发，墨绿长衫".to_string(),
            gender: Some("男".to_string()),
            age: Some("25".to_string()),
            hairstyle: Some("长发".to_string()),
            hair_color: Some("黑色".to_string()),
            outfit: Some("墨绿长衫".to_string()),
            accessories: vec!["青色玉佩".to_string()],
            three_view_paths: vec![],
            lora_path: None,
            ip_adapter_features: None,
            action_library: vec![],
            emotion_library: vec![],
        };
        
        registry._add_character(character);
        
        // 添加场景
        let scene = _SceneAsset {
            metadata: AssetMetadata {
                id: "scene_001".to_string(),
                name: "演武场".to_string(),
                asset_type: AssetType::Scene,
                description: "比武场景".to_string(),
                tags: vec!["战斗".to_string()],
                version: "1.0".to_string(),
                created_at: 0,
                updated_at: 0,
                usage_count: 0,
                rating: 4.0,
                author: "system".to_string(),
                source: "builtin".to_string(),
            },
            scene_description: "古代演武场".to_string(),
            scene_type: "outdoor".to_string(),
            lighting: Some("日光".to_string()),
            weather: Some("晴朗".to_string()),
            time_of_day: Some("白天".to_string()),
            perspective: Some("全景".to_string()),
            layer_paths: vec![],
            template_path: None,
        };
        
        registry.add_scene(scene);
        
        // 验证
        let stats = registry.statistics();
        assert_eq!(stats.character_count, 1);
        assert_eq!(stats.scene_count, 1);
        assert_eq!(stats.total_assets, 2);
    }
    
    #[test]
    fn test_search_characters() {
        let mut registry = AssetRegistry::new();
        
        let character = _CharacterAsset {
            metadata: AssetMetadata {
                id: "char_001".to_string(),
                name: "林天".to_string(),
                asset_type: AssetType::Character,
                description: "主角".to_string(),
                tags: vec!["主角".to_string()],
                version: "1.0".to_string(),
                created_at: 0,
                updated_at: 0,
                usage_count: 0,
                rating: 0.0,
                author: "system".to_string(),
                source: "builtin".to_string(),
            },
            visual_description: "黑发".to_string(),
            gender: None,
            age: None,
            hairstyle: None,
            hair_color: None,
            outfit: None,
            accessories: vec![],
            three_view_paths: vec![],
            lora_path: None,
            ip_adapter_features: None,
            action_library: vec![],
            emotion_library: vec![],
        };
        
        registry._add_character(character);
        
        let found = registry._search_characters("林天");
        assert_eq!(found.len(), 1);
        
        let found = registry._search_characters("不存在");
        assert_eq!(found.len(), 0);
    }
}