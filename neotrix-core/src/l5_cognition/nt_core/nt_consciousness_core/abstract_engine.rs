//! 抽象引擎 (AbstractEngine)
//! 
//! 概念抽象、类比推理、泛化推广、迁移应用

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 抽象引擎
pub struct AbstractEngine {
    /// 概念库
    pub concepts: HashMap<String, Concept>,
    /// 抽象历史
    pub history: Vec<AbstractRecord>,
}

/// 概念
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concept {
    pub id: String,
    pub name: String,
    pub abstraction_level: AbstractionLevel,
    pub features: Vec<String>,
    pub examples: Vec<String>,
    pub relations: Vec<ConceptRelation>,
}

/// 抽象层次
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AbstractionLevel {
    Concrete,
    SubCategory,
    Category,
    SuperCategory,
    Abstract,
}

/// 概念关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptRelation {
    pub relation_type: String,
    pub target_concept: String,
    pub strength: f64,
}

/// 抽象记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbstractRecord {
    pub id: String,
    pub cycle: u32,
    pub input: String,
    pub abstracted: String,
    pub level: AbstractionLevel,
    pub timestamp: String,
}

impl AbstractEngine {
    pub fn new() -> Self {
        Self {
            concepts: HashMap::new(),
            history: Vec::new(),
        }
    }

    /// 抽象概念
    pub fn abstract_concept(&mut self, cycle: u32, input: &str) -> String {
        let abstracted = format!("Abstract({})", input);
        
        let record = AbstractRecord {
            id: format!("abs_{}", uuid::Uuid::new_v4()),
            cycle,
            input: input.to_string(),
            abstracted: abstracted.clone(),
            level: AbstractionLevel::Category,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        self.history.push(record);
        abstracted
    }

    /// 类比推理
    pub fn analogical_reasoning(&self, source: &str, target: &str) -> f64 {
        // 简单的类比评分
        let common_features = source.chars().filter(|c| target.contains(*c)).count();
        common_features as f64 / source.len().max(1) as f64
    }

    /// 泛化推广
    pub fn generalize(&self, examples: &[String]) -> String {
        if examples.is_empty() {
            return "Empty".to_string();
        }
        
        // 找到公共前缀
        let first = &examples[0];
        let prefix: String = first.chars()
            .take_while(|&c| examples.iter().all(|e| e.starts_with(&first[..first.find(c).unwrap_or(0) + 1])))
            .collect();
        
        if prefix.is_empty() {
            format!("General({})", examples.len())
        } else {
            format!("General({})", prefix)
        }
    }

    /// 获取统计
    pub fn stats(&self) -> AbstractStats {
        AbstractStats {
            total_concepts: self.concepts.len(),
            total_abstractions: self.history.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbstractStats {
    pub total_concepts: usize,
    pub total_abstractions: usize,
}

impl std::fmt::Display for AbstractStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "AbstractEngine: {} concepts, {} abstractions",
            self.total_concepts, self.total_abstractions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abstract_engine() {
        let engine = AbstractEngine::new();
        assert_eq!(engine.concepts.len(), 0);
    }
}
