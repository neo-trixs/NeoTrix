//! 角色互动谱系图
//!
//! 管理角色之间的互动模式、关系强度、成长弧线
//! 为动态漫人物小传提供结构化的互动网络

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// ============================================================================
// 角色定义
// ============================================================================

/// 角色基础信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    /// 角色ID
    pub id: String,
    /// 姓名
    pub name: String,
    /// 核心标签 (3个关键词)
    pub tags: Vec<String>,
    /// 团队定位
    pub role: TeamRole,
    /// 动态参数
    pub dynamic_params: CharacterDynamicParams,
}

/// 团队定位
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TeamRole {
    /// 主角
    Protagonist,
    /// 配角
    Supporting,
    /// 反派
    Antagonist,
    /// 导师
    Mentor,
    /// 氛围调节者
    AtmosphereRegulator,
}

/// 角色动态参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterDynamicParams {
    /// 核心动作习惯
    pub core_actions: Vec<String>,
    /// 微动态特征
    pub micro_dynamics: Vec<String>,
    /// 声音细节
    pub voice_details: VoiceDetails,
}

/// 声音细节
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceDetails {
    /// 音色
    pub tone: String,
    /// 语速
    pub speed: String,
    /// 口头禅
    pub catchphrases: Vec<String>,
}

// ============================================================================
// 互动关系
// ============================================================================

/// 互动关系定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    /// 角色A ID
    pub character_a: String,
    /// 角色B ID
    pub character_b: String,
    /// 互动类型
    pub interaction_type: InteractionType,
    /// 关系强度 (0.0-1.0)
    pub strength: f32,
    /// 互动模式描述
    pub pattern: String,
    /// 触发场景
    pub trigger_scenes: Vec<String>,
    /// 动态表现
    pub dynamic_manifestation: String,
}

/// 互动类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum InteractionType {
    /// 合作
    Cooperation,
    /// 对抗
    Conflict,
    /// 依赖
    Dependency,
    /// 保护
    Protection,
    /// 误解
    Misunderstanding,
    /// 成长
    Growth,
}

// ============================================================================
// 成长弧线
// ============================================================================

/// 角色成长弧线
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthArc {
    /// 角色ID
    pub character_id: String,
    /// 初始状态
    pub initial_state: String,
    /// 最终状态
    pub final_state: String,
    /// 关键转变点
    pub key_transitions: Vec<Transition>,
    /// 单集变化
    pub episode_changes: Vec<EpisodeChange>,
}

/// 转变点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    /// 触发事件
    pub trigger: String,
    /// 转变前状态
    pub before: String,
    /// 转变后状态
    pub after: String,
    /// 动态变化描述
    pub dynamic_change: String,
}

/// 单集变化
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeChange {
    /// 集数
    pub episode: u32,
    /// 状态变化
    pub state_change: String,
    /// 动态变化
    pub dynamic_change: String,
}

// ============================================================================
// 角色互动谱系图
// ============================================================================

/// 角色互动谱系图
/// 管理角色、互动、成长弧线的完整网络
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterInteractionGraph {
    /// 所有角色
    characters: HashMap<String, Character>,
    /// 所有互动关系
    interactions: Vec<Interaction>,
    /// 所有成长弧线
    growth_arcs: HashMap<String, GrowthArc>,
}

impl CharacterInteractionGraph {
    /// 创建空谱系图
    pub fn new() -> Self {
        Self {
            characters: HashMap::new(),
            interactions: Vec::new(),
            growth_arcs: HashMap::new(),
        }
    }
    
    /// 添加角色
    pub fn add_character(&mut self, character: Character) {
        self.characters.insert(character.id.clone(), character);
    }
    
    /// 添加互动关系
    pub fn add_interaction(&mut self, interaction: Interaction) {
        // 验证角色存在
        if self.characters.contains_key(&interaction.character_a)
            && self.characters.contains_key(&interaction.character_b) {
            self.interactions.push(interaction);
        }
    }
    
    /// 添加成长弧线
    pub fn add_growth_arc(&mut self, arc: GrowthArc) {
        if self.characters.contains_key(&arc.character_id) {
            self.growth_arcs.insert(arc.character_id.clone(), arc);
        }
    }
    
    /// 获取角色的所有互动
    pub fn get_character_interactions(&self, character_id: &str) -> Vec<&Interaction> {
        self.interactions.iter()
            .filter(|i| i.character_a == character_id || i.character_b == character_id)
            .collect()
    }
    
    /// 获取角色的成长弧线
    pub fn get_growth_arc(&self, character_id: &str) -> Option<&GrowthArc> {
        self.growth_arcs.get(character_id)
    }
    
    /// 检查互动一致性
    pub fn check_consistency(&self) -> Vec<ConsistencyIssue> {
        let mut issues = Vec::new();
        
        // 检查孤立角色
        for character in self.characters.values() {
            let interactions = self.get_character_interactions(&character.id);
            if interactions.is_empty() {
                issues.push(ConsistencyIssue {
                    issue_type: IssueType::IsolatedCharacter,
                    character_id: Some(character.id.clone()),
                    description: format!("角色 {} 没有任何互动关系", character.name),
                    severity: Severity::Warning,
                });
            }
        }
        
        // 检查互动强度平衡
        for interaction in &self.interactions {
            if interaction.strength < 0.0 || interaction.strength > 1.0 {
                issues.push(ConsistencyIssue {
                    issue_type: IssueType::InvalidStrength,
                    character_id: None,
                    description: format!("互动强度超出范围: {}", interaction.strength),
                    severity: Severity::Error,
                });
            }
        }
        
        issues
    }
    
    /// 获取统计信息
    pub fn statistics(&self) -> GraphStatistics {
        let mut interaction_counts: HashMap<String, usize> = HashMap::new();
        
        for interaction in &self.interactions {
            *interaction_counts.entry(interaction.character_a.clone()).or_insert(0) += 1;
            *interaction_counts.entry(interaction.character_b.clone()).or_insert(0) += 1;
        }
        
        let most_connected = interaction_counts.iter()
            .max_by_key(|(_, &count)| count)
            .map(|(id, _)| id.clone());
        
        GraphStatistics {
            character_count: self.characters.len(),
            interaction_count: self.interactions.len(),
            growth_arc_count: self.growth_arcs.len(),
            most_connected_character: most_connected,
            average_interaction_strength: self.calculate_average_strength(),
        }
    }
    
    fn calculate_average_strength(&self) -> f32 {
        if self.interactions.is_empty() {
            return 0.0;
        }
        let total: f32 = self.interactions.iter().map(|i| i.strength).sum();
        total / self.interactions.len() as f32
    }
}

// ============================================================================
// 一致性检查
// ============================================================================

/// 一致性问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyIssue {
    /// 问题类型
    pub issue_type: IssueType,
    /// 相关角色ID
    pub character_id: Option<String>,
    /// 问题描述
    pub description: String,
    /// 严重程度
    pub severity: Severity,
}

/// 问题类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum IssueType {
    /// 孤立角色
    IsolatedCharacter,
    /// 互动强度无效
    InvalidStrength,
    /// 循环依赖
    CircularDependency,
    /// 成长弧线矛盾
    GrowthArcConflict,
}

/// 严重程度
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]

/// 图统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStatistics {
    /// 角色数量
    pub character_count: usize,
    /// 互动关系数量
    pub interaction_count: usize,
    /// 成长弧线数量
    pub growth_arc_count: usize,
    /// 连接最多的角色
    pub most_connected_character: Option<String>,
    /// 平均互动强度
    pub average_interaction_strength: f32,
}

// ============================================================================
// 测试模块
// ============================================================================

use neotrix_types::shared::Severity;
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_character_interaction_graph() {
        let mut graph = CharacterInteractionGraph::new();
        
        // 添加角色
        let char1 = Character {
            id: "char1".to_string(),
            name: "林墨".to_string(),
            tags: vec!["冷静毒舌".to_string(), "武力担当".to_string()],
            role: TeamRole::Protagonist,
            dynamic_params: CharacterDynamicParams {
                core_actions: vec!["摸红围巾".to_string()],
                micro_dynamics: vec!["眼神聚焦".to_string()],
                voice_details: VoiceDetails {
                    tone: "中低音".to_string(),
                    speed: "偏快".to_string(),
                    catchphrases: vec!["别废话".to_string()],
                },
            },
        };
        
        let char2 = Character {
            id: "char2".to_string(),
            name: "苏晚".to_string(),
            tags: vec!["天真勇敢".to_string(), "线索持有者".to_string()],
            role: TeamRole::Supporting,
            dynamic_params: CharacterDynamicParams {
                core_actions: vec!["攥项链".to_string()],
                micro_dynamics: vec!["双马尾晃动".to_string()],
                voice_details: VoiceDetails {
                    tone: "中音".to_string(),
                    speed: "偏快".to_string(),
                    catchphrases: vec!["林墨哥!".to_string()],
                },
            },
        };
        
        graph.add_character(char1);
        graph.add_character(char2);
        
        // 添加互动
        let interaction = Interaction {
            character_a: "char1".to_string(),
            character_b: "char2".to_string(),
            interaction_type: InteractionType::Protection,
            strength: 0.8,
            pattern: "表面嫌弃实际保护".to_string(),
            trigger_scenes: vec!["苏晚受伤".to_string()],
            dynamic_manifestation: "皱眉+毒舌，挡在身前".to_string(),
        };
        
        graph.add_interaction(interaction);
        
        // 验证
        let stats = graph.statistics();
        assert_eq!(stats.character_count, 2);
        assert_eq!(stats.interaction_count, 1);
        assert!(stats.average_interaction_strength > 0.0);
    }
    
    #[test]
    fn test_consistency_check() {
        let mut graph = CharacterInteractionGraph::new();
        
        // 添加一个孤立角色
        let char1 = Character {
            id: "char1".to_string(),
            name: "孤立角色".to_string(),
            tags: vec![],
            role: TeamRole::Supporting,
            dynamic_params: CharacterDynamicParams {
                core_actions: vec![],
                micro_dynamics: vec![],
                voice_details: VoiceDetails {
                    tone: String::new(),
                    speed: String::new(),
                    catchphrases: vec![],
                },
            },
        };
        
        graph.add_character(char1);
        
        let issues = graph.check_consistency();
        assert_eq!(issues.len(), 1);
        assert!(issues[0].description.contains("孤立角色"));
    }
}