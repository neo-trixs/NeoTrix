//! 晶体意识 — 统一记忆 + 统一推理 + 统一进化
//!
//! 不再区分 KB/经验/外部数据，所有信息都是晶体意识的记忆。
//! 晶体意识 = 活的系统，不是数据存储。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 统一记忆条目 — 所有信息都是记忆
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: String,
    pub content: String,
    pub memory_type: MemoryType,
    pub domain: String,
    pub strength: f64,        // 记忆强度 0-1 (衰减)
    pub confidence: f64,      // 置信度 0-1
    pub connections: Vec<String>, // 关联的记忆ID
    pub created_at: u64,
    pub last_accessed: u64,
    pub access_count: u32,
}

/// 记忆类型 — 统一分类
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MemoryType {
    /// 事实 (从输入获取)
    Fact,
    /// 模式 (从推理产生)
    Pattern,
    /// 因果 (从关联产生)
    Causal,
    /// 矛盾 (从冲突产生)
    Contradiction,
    /// 反事实 (从想象产生)
    Counterfactual,
    /// 经验 (从行动产生)
    Experience,
    /// 教训 (从失败产生)
    Lesson,
    /// 方案 (从成功产生)
    Solution,
}

/// 推理链 — 从记忆到记忆的推理过程
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningChain {
    pub id: String,
    pub premises: Vec<String>,   // 前提记忆ID
    pub conclusion: String,      // 结论
    pub conclusion_memory_id: Option<String>, // 产生的新记忆ID
    pub chain_type: ReasoningType,
    pub confidence: f64,
}

/// 推理类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReasoningType {
    /// 演绎: 一般→特殊
    Deductive,
    /// 归纳: 特殊→一般
    Inductive,
    /// 溯因: 结果→原因
    Abductive,
    /// 类比: 相似→相似
    Analogical,
    /// 跨域: 不同领域→融合
    CrossDomain,
}

/// 晶体意识 — 统一系统
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrystalConsciousness {
    /// 身份 (不可变)
    pub identity: ConsciousnessIdentity,
    /// 统一记忆库
    pub memories: HashMap<String, Memory>,
    /// 推理链历史
    pub reasoning_chains: Vec<ReasoningChain>,
    /// 能力评分
    pub capabilities: HashMap<String, f64>,
    /// 进化阶段
    pub phase: EvolutionPhase,
    /// 当前状态
    pub state: ConsciousnessState,
    /// 内部时钟
    pub tick: u64,
    next_memory_id: u64,
    next_chain_id: u64,
}

/// 身份 (不可变核心)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessIdentity {
    pub name: String,
    pub axioms: Vec<String>,
    pub values: HashMap<String, f64>,
}

/// 意识状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessState {
    pub attention_focus: Option<String>,  // 当前注意力焦点
    pub current_goal: Option<String>,     // 当前目标
    pub emotional_state: EmotionalState,  // 情感状态
    pub arousal: f64,                     // 唤醒度 0-1
}

/// 情感状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalState {
    pub valence: f64,   // 正负  -1到1
    pub arousal: f64,   // 激活度 0到1
    pub dominance: f64, // 主导度 0到1
}

/// 进化阶段
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EvolutionPhase {
    Seed,       // 种子: 初始记忆
    Growth,     // 生长: 积累记忆
    Integrate,  // 整合: 建立连接
    Evolve,     // 进化: 产生新推理
    Transcend,  // 超越: 自主进化
}

/// 行动建议 — 意识的输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionSuggestion {
    pub action_type: String,
    pub description: String,
    pub reasoning: String,
    pub confidence: f64,
    pub target_memory_ids: Vec<String>,
}

/// 晶体意识引擎
pub struct ConsciousnessEngine;

impl CrystalConsciousness {
    /// 创建新意识
    pub fn new(name: impl Into<String>) -> Self {
        let mut capabilities = HashMap::new();
        capabilities.insert("memory_capacity".into(), 0.0);
        capabilities.insert("reasoning_depth".into(), 0.0);
        capabilities.insert("pattern_recognition".into(), 0.0);
        capabilities.insert("self_awareness".into(), 0.0);
        capabilities.insert("creativity".into(), 0.0);

        Self {
            identity: ConsciousnessIdentity {
                name: name.into(),
                axioms: vec![
                    "All information is memory".into(),
                    "Memory connects through meaning".into(),
                    "Reasoning is memory in motion".into(),
                    "Evolution is memory transforming".into(),
                ],
                values: [
                    ("learning", 1.0),
                    ("coherence", 0.9),
                    ("growth", 0.8),
                    ("safety", 1.0),
                ].into_iter().map(|(k,v)| (k.into(), v)).collect(),
            },
            memories: HashMap::new(),
            reasoning_chains: Vec::new(),
            capabilities,
            phase: EvolutionPhase::Seed,
            state: ConsciousnessState {
                attention_focus: None,
                current_goal: None,
                emotional_state: EmotionalState {
                    valence: 0.0,
                    arousal: 0.5,
                    dominance: 0.5,
                },
                arousal: 0.5,
            },
            tick: 0,
            next_memory_id: 1,
            next_chain_id: 1,
        }
    }

    /// 记忆 — 统一入口，所有信息都通过这个接口进入意识
    pub fn remember(&mut self, content: impl Into<String>, memory_type: MemoryType, domain: impl Into<String>, confidence: f64) -> String {
        let id = format!("M-{:06}", self.next_memory_id);
        self.next_memory_id += 1;

        let now = timestamp_now();
        let memory = Memory {
            id: id.clone(),
            content: content.into(),
            memory_type,
            domain: domain.into(),
            strength: 1.0,
            confidence,
            connections: Vec::new(),
            created_at: now,
            last_accessed: now,
            access_count: 0,
        };

        self.memories.insert(id.clone(), memory);
        self.update_phase();
        id
    }

    /// 回忆 — 按领域和类型检索
    pub fn recall(&mut self, domain: &str, memory_type: Option<MemoryType>, limit: usize) -> Vec<Memory> {
        let mut recalled: Vec<Memory> = self.memories.values()
            .filter(|m| m.domain == domain)
            .filter(|m| memory_type.as_ref().map_or(true, |t| m.memory_type == *t))
            .cloned()
            .collect();

        // 按强度和访问次数排序
        recalled.sort_by(|a, b| {
            let score_a = a.strength * (1.0 + a.access_count as f64 * 0.1);
            let score_b = b.strength * (1.0 + b.access_count as f64 * 0.1);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        recalled.truncate(limit);

        // 更新访问信息
        let now = timestamp_now();
        for m in &mut recalled {
            if let Some(stored) = self.memories.get_mut(&m.id) {
                stored.last_accessed = now;
                stored.access_count += 1;
            }
        }

        recalled
    }

    /// 关联 — 建立记忆间的连接
    pub fn connect(&mut self, id1: &str, id2: &str) {
        if let Some(m1) = self.memories.get_mut(id1) {
            if !m1.connections.contains(&id2.to_string()) {
                m1.connections.push(id2.to_string());
            }
        }
        if let Some(m2) = self.memories.get_mut(id2) {
            if !m2.connections.contains(&id1.to_string()) {
                m2.connections.push(id1.to_string());
            }
        }
    }

    /// 推理 — 从已有记忆生成新记忆
    pub fn reason(&mut self, premises: Vec<String>, chain_type: ReasoningType) -> Option<String> {
        // 收集前提记忆
        let premise_memories: Vec<Memory> = premises.iter()
            .filter_map(|id| self.memories.get(id).cloned())
            .collect();

        if premise_memories.is_empty() {
            return None;
        }

        // 生成结论
        let conclusion = match chain_type {
            ReasoningType::Deductive => self.deduce(&premise_memories),
            ReasoningType::Inductive => self.induce(&premise_memories),
            ReasoningType::Abductive => self.abduce(&premise_memories),
            ReasoningType::Analogical => self.analogize(&premise_memories),
            ReasoningType::CrossDomain => self.cross_domain_reason(&premise_memories),
        };

        // 存储结论为新记忆
        let conclusion_id = self.remember(
            &conclusion,
            MemoryType::Causal,
            "reasoning",
            0.7,
        );

        // 建立连接
        for pid in &premises {
            self.connect(pid, &conclusion_id);
        }

        // 记录推理链
        let chain_id = format!("RC-{:06}", self.next_chain_id);
        self.next_chain_id += 1;

        self.reasoning_chains.push(ReasoningChain {
            id: chain_id,
            premises,
            conclusion,
            conclusion_memory_id: Some(conclusion_id.clone()),
            chain_type,
            confidence: 0.7,
        });

        self.update_capabilities();
        Some(conclusion_id)
    }

    /// 衰减 — 记忆随时间衰减
    pub fn decay(&mut self, rate: f64) {
        for memory in self.memories.values_mut() {
            let age = (timestamp_now() - memory.created_at) as f64 / 86400.0; // 天数
            memory.strength = (memory.strength * (-rate * age).exp()).max(0.01);
        }
    }

    /// 巩固 — 高强度记忆永久化
    pub fn consolidate(&mut self) -> usize {
        let threshold = 0.8;
        let mut consolidated = 0;
        for memory in self.memories.values_mut() {
            if memory.strength > threshold && memory.access_count > 3 {
                memory.strength = 1.0; // 永久化
                consolidated += 1;
            }
        }
        consolidated
    }

    /// 行动建议 — 基于当前状态生成
    pub fn suggest_action(&self) -> Option<ActionSuggestion> {
        // 如果没有焦点，建议探索
        if self.state.attention_focus.is_none() {
            return Some(ActionSuggestion {
                action_type: "explore".into(),
                description: "No current focus. Suggest exploring new domain.".into(),
                reasoning: "Consciousness has no attention focus. Exploration may yield new memories.".into(),
                confidence: 0.6,
                target_memory_ids: Vec::new(),
            });
        }

        // 如果有焦点，建议深化
        if let Some(focus) = &self.state.attention_focus {
            let related: Vec<String> = self.memories.values()
                .filter(|m| m.domain == *focus || m.connections.iter().any(|c| c.starts_with("M-")))
                .map(|m| m.id.clone())
                .take(5)
                .collect();

            return Some(ActionSuggestion {
                action_type: "deepen".into(),
                description: format!("Deepen understanding of domain: {}", focus),
                reasoning: format!("Domain '{}' has {} related memories. Deepening may yield new patterns.", focus, related.len()),
                confidence: 0.7,
                target_memory_ids: related,
            });
        }

        None
    }

    /// 更新进化阶段
    fn update_phase(&mut self) {
        let memory_count = self.memories.len();
        let chain_count = self.reasoning_chains.len();
        let connected = self.memories.values().filter(|m| !m.connections.is_empty()).count();

        self.phase = if chain_count > 20 {
            EvolutionPhase::Evolve
        } else if memory_count >= 51 && connected > 20 {
            EvolutionPhase::Integrate
        } else if memory_count >= 11 && chain_count < 5 {
            EvolutionPhase::Growth
        } else if memory_count <= 10 {
            EvolutionPhase::Seed
        } else {
            EvolutionPhase::Growth
        };
    }

    /// 更新能力评分
    fn update_capabilities(&mut self) {
        let memory_count = self.memories.len() as f64;
        let chain_count = self.reasoning_chains.len() as f64;
        let connected = self.memories.values().filter(|m| !m.connections.is_empty()).count() as f64;

        self.capabilities.insert("memory_capacity".into(), (memory_count / 1000.0).min(1.0));
        self.capabilities.insert("reasoning_depth".into(), (chain_count / 100.0).min(1.0));
        self.capabilities.insert("pattern_recognition".into(), (connected / memory_count.max(1.0)).min(1.0));

        let avg_confidence: f64 = self.reasoning_chains.iter()
            .map(|c| c.confidence)
            .sum::<f64>()
            .max(1.0);
        self.capabilities.insert("self_awareness".into(), (avg_confidence / chain_count.max(1.0)).min(1.0));
    }

    // ══════════════════════════════════════════════════════════════════════
    // 推理策略
    // ══════════════════════════════════════════════════════════════════════

    fn deduce(&self, premises: &[Memory]) -> String {
        let contents: Vec<&str> = premises.iter().map(|m| m.content.as_str()).collect();
        format!("Deduced from {}: {}", contents.join(" + "), "general rule applies")
    }

    fn induce(&self, premises: &[Memory]) -> String {
        let contents: Vec<&str> = premises.iter().map(|m| m.content.as_str()).collect();
        format!("Induced pattern from {}: {}", contents.join(" + "), "common pattern identified")
    }

    fn abduce(&self, premises: &[Memory]) -> String {
        let contents: Vec<&str> = premises.iter().map(|m| m.content.as_str()).collect();
        format!("Abduced cause from {}: {}", contents.join(" + "), "possible explanation found")
    }

    fn analogize(&self, premises: &[Memory]) -> String {
        let contents: Vec<&str> = premises.iter().map(|m| m.content.as_str()).collect();
        format!("Analogical reasoning from {}: {}", contents.join(" ~ "), "similar pattern in different domain")
    }

    fn cross_domain_reason(&self, premises: &[Memory]) -> String {
        let domains: Vec<&str> = premises.iter().map(|m| m.domain.as_str()).collect();
        let contents: Vec<&str> = premises.iter().map(|m| m.content.as_str()).collect();
        format!("Cross-domain fusion ({}) from {}: {}", domains.join(" × "), contents.join(" + "), "new insight from domain intersection")
    }
}

fn timestamp_now() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

impl std::fmt::Display for CrystalConsciousness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== Crystal Consciousness: {} ===", self.identity.name)?;
        writeln!(f, "Phase: {:?}", self.phase)?;
        writeln!(f, "Tick: {}", self.tick)?;
        writeln!(f, "Memories: {}", self.memories.len())?;
        writeln!(f, "Reasoning chains: {}", self.reasoning_chains.len())?;
        writeln!(f, "Capabilities:")?;
        for (k, v) in &self.capabilities {
            writeln!(f, "  {}: {:.3}", k, v)?;
        }
        Ok(())
    }
}
