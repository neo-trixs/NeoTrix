//! 动态记忆库模块
//!
//! 实体级一致性管理：角色、道具、场景的跨镜头记忆
//! 支持双查询机制：长上下文（身份）+ 短上下文（生成）

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// ============================================================================
// 实体定义
// ============================================================================

/// 实体类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EntityType {
    /// 角色
    Character,
    /// 道具
    Prop,
    /// 场景
    Background,
}

/// 实体状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityState {
    /// 实体类型
    pub entity_type: EntityType,
    /// 实体名称
    pub name: String,
    /// 描述
    pub description: String,
    /// 视觉特征向量 (DINOv2 等)
    pub visual_features: Vec<f32>,
    /// 文本属性
    pub attributes: HashMap<String, String>,
    /// 参考图片路径
    pub reference_image_path: Option<String>,
    /// 创建时间
    pub created_at: u64,
    /// 最后更新时间
    pub last_updated_at: u64,
    /// 出现的镜头列表
    pub appeared_in_shots: Vec<u32>,
}

/// 记忆条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// 实体ID
    pub entity_id: String,
    /// 实体状态
    pub state: EntityState,
    /// 语义相似度分数 (用于检索)
    pub relevance_score: f32,
    /// 是否为身份关键条目
    pub is_identity_critical: bool,
}

/// 记忆库配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _MemoryBankConfig {
    /// 每个实体类型的最大记忆数
    pub max_entries_per_type: usize,
    /// 身份检索阈值
    pub identity_retrieval_threshold: f32,
    /// 上下文检索阈值
    pub context_retrieval_threshold: f32,
    /// 是否启用自动更新
    pub enable_auto_update: bool,
    /// 记忆衰减因子
    pub decay_factor: f32,
}

/// 检索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalResult {
    /// 检索到的记忆条目
    pub entries: Vec<MemoryEntry>,
    /// 平均相关度
    pub avg_relevance: f32,
    /// 检索耗时 (毫秒)
    pub retrieval_time_ms: u64,
}

/// 双查询配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _DualQueryConfig {
    /// 长上下文查询（身份保留）
    pub story_query: String,
    /// 短上下文查询（生成连续性）
    pub shot_query: String,
    /// 长上下文检索数量
    pub long_context_k: usize,
    /// 短上下文检索数量
    pub short_context_k: usize,
}

// ============================================================================
// 动态记忆库
// ============================================================================

/// 动态记忆库
/// 管理实体级一致性，支持双查询检索
pub struct _DynamicMemoryBank {
    /// 配置
    config: _MemoryBankConfig,
    /// 记忆库：实体类型 -> (实体ID -> 记忆条目)
    banks: HashMap<EntityType, HashMap<String, MemoryEntry>>,
    /// 候选池 (用于学习检索)
    candidate_pool: Vec<(String, String, f32)>, // (query, entity_id, score)
}

impl _DynamicMemoryBank {
    /// 创建记忆库
    pub fn new() -> Self {
        Self {
            config: _MemoryBankConfig {
                max_entries_per_type: 100,
                identity_retrieval_threshold: 0.7,
                context_retrieval_threshold: 0.5,
                enable_auto_update: true,
                decay_factor: 0.95,
            },
            banks: HashMap::new(),
            candidate_pool: vec![],
        }
    }
    
    /// 使用配置创建
    pub fn with_config(config: _MemoryBankConfig) -> Self {
        Self {
            config,
            banks: HashMap::new(),
            candidate_pool: vec![],
        }
    }
    
    /// 存储实体
    pub fn _store_entity(&mut self, entity_id: &str, state: EntityState) {
        let bank = self.banks.entry(state.entity_type).or_insert_with(HashMap::new);
        
        let entry = MemoryEntry {
            entity_id: entity_id.to_string(),
            state,
            relevance_score: 1.0,
            is_identity_critical: true,
        };
        
        bank.insert(entity_id.to_string(), entry);
    }
    
    /// 长上下文检索（身份保留）
    pub fn _retrieve_identity(&self, query: &str, k: usize) -> RetrievalResult {
        let start = std::time::Instant::now();
        let mut results: Vec<MemoryEntry> = vec![];
        
        for bank in self.banks.values() {
            for entry in bank.values() {
                let relevance = self.calculate_semantic_similarity(query, &entry.state.description);
                if relevance >= self.config.identity_retrieval_threshold {
                    let mut entry = entry.clone();
                    entry.relevance_score = relevance;
                    results.push(entry);
                }
            }
        }
        
        results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        results.truncate(k);
        
        let avg_relevance = if results.is_empty() {
            0.0
        } else {
            results.iter().map(|r| r.relevance_score).sum::<f32>() / results.len() as f32
        };
        
        RetrievalResult {
            entries: results,
            avg_relevance,
            retrieval_time_ms: start.elapsed().as_millis() as u64,
        }
    }
    
    /// 短上下文检索（生成连续性）
    pub fn _retrieve_context(&self, query: &str, k: usize) -> RetrievalResult {
        let start = std::time::Instant::now();
        let mut results: Vec<MemoryEntry> = vec![];
        
        for bank in self.banks.values() {
            for entry in bank.values() {
                let relevance = self.calculate_semantic_similarity(query, &entry.state.description);
                if relevance >= self.config.context_retrieval_threshold {
                    let mut entry = entry.clone();
                    entry.relevance_score = relevance;
                    results.push(entry);
                }
            }
        }
        
        results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
        results.truncate(k);
        
        let avg_relevance = if results.is_empty() {
            0.0
        } else {
            results.iter().map(|r| r.relevance_score).sum::<f32>() / results.len() as f32
        };
        
        RetrievalResult {
            entries: results,
            avg_relevance,
            retrieval_time_ms: start.elapsed().as_millis() as u64,
        }
    }
    
    /// 双查询检索
    pub fn _dual_query_retrieve(&self, config: &_DualQueryConfig) -> (RetrievalResult, RetrievalResult) {
        let identity_result = self._retrieve_identity(&config.story_query, config.long_context_k);
        let context_result = self._retrieve_context(&config.shot_query, config.short_context_k);
        
        (identity_result, context_result)
    }
    
    /// 更新实体状态
    pub fn update_entity(&mut self, entity_id: &str, new_state: EntityState) -> Result<(), String> {
        if let Some(bank) = self.banks.get_mut(&new_state.entity_type) {
            if let Some(entry) = bank.get_mut(entity_id) {
                entry.state = new_state;
                entry.state.last_updated_at = current_timestamp();
                Ok(())
            } else {
                Err(format!("实体 {} 不存在", entity_id))
            }
        } else {
            Err(format!("实体类型 {:?} 不存在", new_state.entity_type))
        }
    }
    
    /// 标记实体出现
    pub fn _mark_appearance(&mut self, entity_id: &str, entity_type: EntityType, shot_number: u32) {
        if let Some(bank) = self.banks.get_mut(&entity_type) {
            if let Some(entry) = bank.get_mut(entity_id) {
                entry.state.appeared_in_shots.push(shot_number);
            }
        }
    }
    
    /// 计算语义相似度
    ///
    /// 注意：此为嵌入模型未接入时的降级方案。使用关键词重叠度估算相似度。
    /// 接入 DINOv2/CLIP 后应替换为向量余弦相似度。
    fn calculate_semantic_similarity(&self, query: &str, description: &str) -> f32 {
        let query_words: Vec<&str> = query.split_whitespace().collect();
        let desc_words: Vec<&str> = description.split_whitespace().collect();
        
        // 关键词重叠度 + 长度惩罚
        let mut matches = 0;
        for qw in &query_words {
            if desc_words.iter().any(|dw| dw.contains(qw) || qw.contains(dw)) {
                matches += 1;
            }
        }
        
        let base_score = if query_words.is_empty() {
            0.0
        } else {
            matches as f32 / query_words.len() as f32
        };
        
        // 关键词重叠度上限为 0.6 — 表示这只是近似匹配
        // 真实语义相似度应通过向量嵌入计算
        (base_score * 0.6).min(0.6)
    }
    
    /// 获取记忆库统计
    pub fn statistics(&self) -> _MemoryBankStats {
        let total_entities: usize = self.banks.values().map(|b| b.len()).sum();
        let by_type: HashMap<String, usize> = self.banks.iter()
            .map(|(k, v)| (format!("{:?}", k), v.len()))
            .collect();
        
        let identity_critical: usize = self.banks.values()
            .flat_map(|b| b.values())
            .filter(|e| e.is_identity_critical)
            .count();
        
        _MemoryBankStats {
            total_entities,
            entities_by_type: by_type,
            identity_critical_entities: identity_critical,
            candidate_pool_size: self.candidate_pool.len(),
        }
    }
}

/// 记忆库统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _MemoryBankStats {
    /// 总实体数
    pub total_entities: usize,
    /// 按类型统计
    pub entities_by_type: HashMap<String, usize>,
    /// 身份关键实体数
    pub identity_critical_entities: usize,
    /// 候选池大小
    pub candidate_pool_size: usize,
}

/// 获取当前时间戳
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

// ============================================================================
// 测试模块
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_bank() {
        let mut bank = _DynamicMemoryBank::new();
        
        // 存储角色
        bank._store_entity("char_001", EntityState {
            entity_type: EntityType::Character,
            name: "主角".to_string(),
            description: "年轻男性，棕色头发，蓝色眼睛".to_string(),
            visual_features: vec![],
            attributes: HashMap::new(),
            reference_image_path: None,
            created_at: 0,
            last_updated_at: 0,
            appeared_in_shots: vec![],
        });
        
        // 检索
        let result = bank._retrieve_identity("年轻男性 棕色头发", 10);
        assert!(!result.entries.is_empty());
        
        let stats = bank.statistics();
        assert_eq!(stats.total_entities, 1);
    }
    
    #[test]
    fn test_dual_query() {
        let mut bank = _DynamicMemoryBank::new();
        
        bank._store_entity("char_001", EntityState {
            entity_type: EntityType::Character,
            name: "主角".to_string(),
            description: "年轻男性，棕色头发".to_string(),
            visual_features: vec![],
            attributes: HashMap::new(),
            reference_image_path: None,
            created_at: 0,
            last_updated_at: 0,
            appeared_in_shots: vec![],
        });
        
        let (identity, context) = bank._dual_query_retrieve(&_DualQueryConfig {
            story_query: "主角的故事".to_string(),
            shot_query: "主角在教室".to_string(),
            long_context_k: 5,
            short_context_k: 3,
        });
        
        assert!(!identity.entries.is_empty());
    }
}