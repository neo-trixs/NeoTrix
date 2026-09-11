//! Trinity Memory — 基于 TencentDB Agent Memory 模式
//!
//! 统一向量 + 图 + 关系存储。

use std::collections::HashMap;

use super::shared_utils::{cosine_similarity, now_ts};

/// 记忆条目

#[derive(Clone, Debug)]
pub struct MemoryEntry {
    pub id: String,
    pub content: String,
    pub memory_type: MemoryType,
    pub embedding: Vec<f32>,
    pub relations: Vec<Relation>,
    pub metadata: HashMap<String, String>,
    pub created_at: i64,
}

/// 记忆类型

#[derive(Clone, Debug)]
pub enum MemoryType {
    /// 向量记忆（语义搜索）
    Vector,
    /// 图记忆（实体关系）
    Graph,
    /// 关系记忆（结构化数据）
    Relational,
}

/// 关系

#[derive(Clone, Debug)]
pub struct Relation {
    pub from: String,
    pub to: String,
    pub relation_type: String,
    pub weight: f64,
}

/// Trinity 记忆存储

#[derive(Debug)]
pub struct TrinityMemory {
    entries: Vec<MemoryEntry>,
    vector_index: HashMap<String, usize>,
    graph_index: HashMap<String, Vec<String>>,
    next_id: u32,
}


impl TrinityMemory {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            vector_index: HashMap::new(),
            graph_index: HashMap::new(),
            next_id: 0,
        }
    }

    /// 添加记忆
    pub fn add(&mut self, content: &str, memory_type: MemoryType, embedding: Vec<f32>) -> String {
        self.next_id += 1;
        let id = format!("mem_{}", self.next_id);

        self.entries.push(MemoryEntry {
            id: id.clone(),
            content: content.to_string(),
            memory_type,
            embedding,
            relations: Vec::new(),
            metadata: HashMap::new(),
            created_at: now_ts(),
        });

        self.vector_index.insert(id.clone(), self.entries.len() - 1);
        id
    }

    /// 添加关系
    pub fn add_relation(&mut self, from: &str, to: &str, relation_type: &str) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.id == from) {
            entry.relations.push(Relation {
                from: from.to_string(),
                to: to.to_string(),
                relation_type: relation_type.to_string(),
                weight: 1.0,
            });
        }
        self.graph_index
            .entry(from.to_string())
            .or_default()
            .push(to.to_string());
    }

    /// 向量搜索（简化：余弦相似度）
    pub fn search_vector(&self, query_embedding: &[f32], top_k: usize) -> Vec<(&str, f64)> {
        let mut scores: Vec<(&str, f64)> = self
            .entries
            .iter()
            .filter(|e| matches!(e.memory_type, MemoryType::Vector))
            .map(|e| {
                let sim = cosine_similarity(&e.embedding, query_embedding);
                (e.id.as_str(), sim)
            })
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.into_iter().take(top_k).collect()
    }

    /// 图搜索（BFS）
    pub fn search_graph(&self, start_id: &str, depth: usize) -> Vec<String> {
        let mut visited = std::collections::HashSet::new();
        let mut queue = vec![(start_id.to_string(), 0)];
        let mut results = Vec::new();

        while let Some((id, d)) = queue.pop() {
            if d > depth || visited.contains(&id) {
                continue;
            }
            visited.insert(id.clone());
            results.push(id.clone());

            if let Some(related) = self.graph_index.get(&id) {
                for rel_id in related {
                    queue.push((rel_id.clone(), d + 1));
                }
            }
        }

        results
    }

    /// 关系搜索
    pub fn search_relations(&self, entity: &str) -> Vec<(&str, &str)> {
        self.entries
            .iter()
            .filter(|e| e.id == entity)
            .flat_map(|e| {
                e.relations
                    .iter()
                    .map(|r| (r.to.as_str(), r.relation_type.as_str()))
            })
            .collect()
    }

    /// 统计
    pub fn stats(&self) -> (usize, usize, usize) {
        let vector = self
            .entries
            .iter()
            .filter(|e| matches!(e.memory_type, MemoryType::Vector))
            .count();
        let graph = self
            .entries
            .iter()
            .filter(|e| matches!(e.memory_type, MemoryType::Graph))
            .count();
        let relational = self
            .entries
            .iter()
            .filter(|e| matches!(e.memory_type, MemoryType::Relational))
            .count();
        (vector, graph, relational)
    }
}
