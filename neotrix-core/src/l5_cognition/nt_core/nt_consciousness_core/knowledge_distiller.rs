//! 知识蒸馏器 (KnowledgeDistiller)
//! 
//! 从吸收的信息中提取核心知识，压缩冗余，建立关联

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 知识蒸馏器
pub struct KnowledgeDistiller {
    /// 蒸馏历史
    pub distillation_history: Vec<_DistillationRecord>,
    /// 知识图谱
    pub knowledge_graph: KnowledgeGraph,
    /// 蒸馏配置
    pub config: _DistillerConfig,
}

/// 蒸馏配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _DistillerConfig {
    /// 最大历史记录
    pub max_history: usize,
    /// 最小置信度阈值
    pub min_confidence: f64,
    /// 最大知识节点数
    pub max_knowledge_nodes: usize,
    /// 压缩率
    pub compression_ratio: f64,
}

impl Default for _DistillerConfig {
    fn default() -> Self {
        Self {
            max_history: 500,
            min_confidence: 0.7,
            max_knowledge_nodes: 10000,
            compression_ratio: 0.3,
        }
    }
}

/// 蒸馏记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _DistillationRecord {
    /// 记录ID
    pub id: String,
    /// 周期
    pub cycle: u32,
    /// 输入内容
    pub input: String,
    /// 输出知识
    pub output: _KnowledgeUnit,
    /// 压缩率
    pub compression_rate: f64,
    /// 时间戳
    pub timestamp: String,
}

/// 知识单元
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _KnowledgeUnit {
    /// 知识ID
    pub id: String,
    /// 知识内容
    pub content: String,
    /// 知识类型
    pub knowledge_type: _KnowledgeType,
    /// 置信度
    pub confidence: f64,
    /// 来源
    pub source: String,
    /// 关联知识
    pub related: Vec<String>,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

/// 知识类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum _KnowledgeType {
    /// 概念知识
    Concept,
    /// 事实知识
    Fact,
    /// 过程知识
    Procedure,
    /// 策略知识
    Strategy,
    /// 启发式知识
    Heuristic,
}

/// 知识图谱
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnowledgeGraph {
    /// 知识节点
    pub nodes: HashMap<String, KnowledgeNode>,
    /// 知识边
    pub edges: Vec<KnowledgeEdge>,
    /// 知识簇
    pub clusters: Vec<KnowledgeCluster>,
}

/// 知识节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeNode {
    /// 节点ID
    pub id: String,
    /// 知识内容
    pub knowledge: _KnowledgeUnit,
    /// 重要性分数
    pub importance: f64,
    /// 访问次数
    pub access_count: u32,
    /// 最后访问时间
    pub last_accessed: Option<String>,
}

/// 知识边
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEdge {
    /// 边ID
    pub id: String,
    /// 源节点
    pub source: String,
    /// 目标节点
    pub target: String,
    /// 关系类型
    pub relation: String,
    /// 强度
    pub strength: f64,
}

/// 知识簇
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeCluster {
    /// 簇ID
    pub id: String,
    /// 簇名称
    pub name: String,
    /// 包含的节点
    pub nodes: Vec<String>,
    /// 簇中心
    pub center: String,
    /// 簇密度
    pub density: f64,
}

impl KnowledgeDistiller {
    /// 创建新的知识蒸馏器
    pub fn new(config: _DistillerConfig) -> Self {
        Self {
            distillation_history: Vec::new(),
            knowledge_graph: KnowledgeGraph::default(),
            config,
        }
    }

    /// 蒸馏知识
    pub fn distill(&mut self, cycle: u32, input: &str, source: &str) -> _KnowledgeUnit {
        // 提取核心知识
        let core_content = self.extract_core(input);
        
        // 确定知识类型
        let knowledge_type = self.classify_knowledge(&core_content);
        
        // 计算置信度
        let confidence = self.calculate_confidence(&core_content);
        
        // 创建知识单元
        let knowledge = _KnowledgeUnit {
            id: format!("k_{}", uuid::Uuid::new_v4()),
            content: core_content.clone(),
            knowledge_type,
            confidence,
            source: source.to_string(),
            related: Vec::new(),
            metadata: HashMap::from([
                ("cycle".to_string(), cycle.to_string()),
                ("input_length".to_string(), input.len().to_string()),
                ("output_length".to_string(), core_content.len().to_string()),
            ]),
        };

        // 计算压缩率
        let compression_rate = core_content.len() as f64 / input.len().max(1) as f64;

        // 记录
        let record = _DistillationRecord {
            id: format!("dist_{}", uuid::Uuid::new_v4()),
            cycle,
            input: input.to_string(),
            output: knowledge.clone(),
            compression_rate,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        self.distillation_history.push(record);
        self.trim_history();

        // 添加到知识图谱
        self.add_to_graph(knowledge.clone());

        knowledge
    }

    /// 提取核心内容
    fn extract_core(&self, input: &str) -> String {
        // 简单的核心提取：取前20%或关键句
        let words: Vec<&str> = input.split_whitespace().collect();
        let core_count = (words.len() as f64 * self.config.compression_ratio).max(1.0) as usize;
        
        words.iter()
            .take(core_count)
            .cloned()
            .collect::<Vec<&str>>()
            .join(" ")
    }

    /// 分类知识类型
    fn classify_knowledge(&self, content: &str) -> _KnowledgeType {
        let content_lower = content.to_lowercase();
        
        if content_lower.contains("定义") || content_lower.contains("概念") || content_lower.contains("是什么") {
            _KnowledgeType::Concept
        } else if content_lower.contains("事实") || content_lower.contains("数据") || content_lower.contains("统计") {
            _KnowledgeType::Fact
        } else if content_lower.contains("步骤") || content_lower.contains("如何") || content_lower.contains("流程") {
            _KnowledgeType::Procedure
        } else if content_lower.contains("策略") || content_lower.contains("方法") || content_lower.contains("技巧") {
            _KnowledgeType::Strategy
        } else if content_lower.contains("建议") || content_lower.contains("启发") || content_lower.contains("经验") {
            _KnowledgeType::Heuristic
        } else {
            _KnowledgeType::Concept
        }
    }

    /// 计算置信度
    fn calculate_confidence(&self, content: &str) -> f64 {
        let mut confidence: f64 = 0.5;
        
        // 基于内容长度
        if content.len() > 50 {
            confidence += 0.1;
        }
        if content.len() > 200 {
            confidence += 0.1;
        }
        
        // 基于关键词
        let keywords = ["重要", "关键", "核心", "必须", "注意"];
        for keyword in keywords {
            if content.contains(keyword) {
                confidence += 0.05;
            }
        }
        
        confidence.min(1.0)
    }

    /// 添加到知识图谱
    fn add_to_graph(&mut self, knowledge: _KnowledgeUnit) {
        let node = KnowledgeNode {
            id: knowledge.id.clone(),
            knowledge: knowledge.clone(),
            importance: knowledge.confidence,
            access_count: 1,
            last_accessed: Some(chrono::Utc::now().to_rfc3339()),
        };

        self.knowledge_graph.nodes.insert(knowledge.id.clone(), node);

        // 裁剪节点
        if self.knowledge_graph.nodes.len() > self.config.max_knowledge_nodes {
            // 按重要性排序，移除最不重要的
            let mut nodes: Vec<_> = self.knowledge_graph.nodes.iter().collect();
            nodes.sort_by(|a, b| a.1.importance.partial_cmp(&b.1.importance).unwrap());
            
            let to_remove: Vec<String> = nodes.iter()
                .take(nodes.len() - self.config.max_knowledge_nodes)
                .map(|(id, _)| id.to_string())
                .collect();
            
            for id in to_remove {
                self.knowledge_graph.nodes.remove(&id);
            }
        }
    }

    /// 获取蒸馏统计
    pub fn stats(&self) -> _DistillerStats {
        let total_distillations = self.distillation_history.len();
        let avg_compression = if total_distillations > 0 {
            self.distillation_history.iter()
                .map(|r| r.compression_rate)
                .sum::<f64>() / total_distillations as f64
        } else {
            0.0
        };

        let avg_confidence = if total_distillations > 0 {
            self.distillation_history.iter()
                .map(|r| r.output.confidence)
                .sum::<f64>() / total_distillations as f64
        } else {
            0.0
        };

        let mut type_counts = HashMap::new();
        for record in &self.distillation_history {
            *type_counts.entry(format!("{:?}", record.output.knowledge_type)).or_insert(0) += 1;
        }

        _DistillerStats {
            total_distillations,
            avg_compression_rate: avg_compression,
            avg_confidence,
            knowledge_nodes: self.knowledge_graph.nodes.len(),
            knowledge_edges: self.knowledge_graph.edges.len(),
            type_distribution: type_counts,
        }
    }

    /// 裁剪历史
    fn trim_history(&mut self) {
        while self.distillation_history.len() > self.config.max_history {
            self.distillation_history.remove(0);
        }
    }
}

/// 蒸馏统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _DistillerStats {
    pub total_distillations: usize,
    pub avg_compression_rate: f64,
    pub avg_confidence: f64,
    pub knowledge_nodes: usize,
    pub knowledge_edges: usize,
    pub type_distribution: HashMap<String, u32>,
}

impl std::fmt::Display for _DistillerStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "        KnowledgeDistiller 统计")?;
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "总蒸馏次数:      {}", self.total_distillations)?;
        writeln!(f, "平均压缩率:      {:.2}%", self.avg_compression_rate * 100.0)?;
        writeln!(f, "平均置信度:      {:.4}", self.avg_confidence)?;
        writeln!(f, "知识节点:        {}", self.knowledge_nodes)?;
        writeln!(f, "知识边:          {}", self.knowledge_edges)?;
        writeln!(f, "───────────────────────────────────────────────")?;
        writeln!(f, "知识类型分布:")?;
        for (k, v) in &self.type_distribution {
            writeln!(f, "  {}: {}", k, v)?;
        }
        writeln!(f, "═══════════════════════════════════════════════")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distiller_creation() {
        let distiller = KnowledgeDistiller::new(_DistillerConfig::default());
        assert_eq!(distiller.distillation_history.len(), 0);
        assert_eq!(distiller.knowledge_graph.nodes.len(), 0);
    }

    #[test]
    fn test_distill_knowledge() {
        let mut distiller = KnowledgeDistiller::new(_DistillerConfig::default());
        let knowledge = distiller.distill(0, "这是一个重要的概念定义", "test_source");
        
        assert_eq!(distiller.distillation_history.len(), 1);
        assert!(knowledge.confidence > 0.0);
    }
}
