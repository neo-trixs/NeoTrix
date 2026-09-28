//! 晶体意识 — 统一记忆 + 统一推理 + 统一进化
//!
//! 不再区分 KB/经验/外部数据，所有信息都是晶体意识的记忆。
//! 晶体意识 = 活的系统，不是数据存储。

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// reflect 去重阈值（缺陷 #4 修复）：结论与既有 Pattern 的关键词
/// Jaccard ≥ 此值则视为重复并丢弃，防模式库注水。
pub const REFLECT_DUP_JACCARD: f64 = 0.8;

/// 统一记忆条目 — 所有信息都是记忆
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: String,
    pub content: String,
    pub memory_type: MemoryType,
    pub domain: String,
    pub strength: f64,        // 记忆强度 0-1 (衰减)
    pub confidence: f64,      // 置信度 0-1
    /// 重要性 0-1（Generative Agents 映射：retrieval 三信号之一；
    /// 旧快照缺省 0.5，cocoons 303M 兼容）
    #[serde(default = "default_importance")]
    pub importance: f64,
    pub connections: Vec<String>, // 关联的记忆ID
    pub created_at: u64,
    pub last_accessed: u64,
    pub access_count: u32,
}

/// 旧记忆反序列化缺省重要性
fn default_importance() -> f64 {
    0.5
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
    /// 有连接的记忆数（增量维护，全量炼 O(1) 相位判定；旧快照缺省 0，
    /// 由 `recount_connections` 校准）
    #[serde(default)]
    connected_count: usize,
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
            connected_count: 0,
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
            // 出生重要性 = 置信度（Generative Agents 映射：重要事项更可信）；
            // 后续被反复访问的记忆在 recall 排序中自然上浮。
            importance: confidence.clamp(0.0, 1.0),
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

        // 三信号排序（Generative Agents 映射）：强度（新近）× 重要性 × 访问（相关热度）
        recalled.sort_by(|a, b| {
            let score_a =
                a.strength * (0.5 + a.importance) * (1.0 + a.access_count as f64 * 0.1);
            let score_b =
                b.strength * (0.5 + b.importance) * (1.0 + b.access_count as f64 * 0.1);
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

    /// 关联 — 建立记忆间的连接（两侧 0→1 时递增连接计数）
    pub fn connect(&mut self, id1: &str, id2: &str) {
        if let Some(m1) = self.memories.get_mut(id1) {
            if m1.connections.is_empty() {
                self.connected_count += 1;
            }
            if !m1.connections.contains(&id2.to_string()) {
                m1.connections.push(id2.to_string());
            }
        }
        if let Some(m2) = self.memories.get_mut(id2) {
            if m2.connections.is_empty() {
                self.connected_count += 1;
            }
            if !m2.connections.contains(&id1.to_string()) {
                m2.connections.push(id1.to_string());
            }
        }
    }

    /// 重算连接计数（外部批量灌入后校准，如茧恢复）
    pub fn recount_connections(&mut self) {
        self.connected_count = self
            .memories
            .values()
            .filter(|m| !m.connections.is_empty())
            .count();
        self.update_phase();
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

        // 存储结论为新记忆（置信度继承前提均值，可溯源）
        let conclusion_conf = Self::premise_confidence(&premise_memories);
        let conclusion_id = self.remember(
            &conclusion,
            MemoryType::Causal,
            "reasoning",
            conclusion_conf,
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
            confidence: conclusion_conf,
        });

        self.update_capabilities();
        // R-REFINE-1：链入账后再定相位，否则门判定永远少看最后一条链
        self.update_phase();
        Some(conclusion_id)
    }

    /// 定时反思（Generative Agents 映射）：对最近访问的记忆做归纳，
    /// 产出高层 Pattern 记忆（reason() 产 Causal，反思产 Pattern，
    /// 层级有别）。由觉醒循环按节奏调用，无需外部 LLM。
    pub fn reflect(&mut self, recent_n: usize) -> Option<String> {
        let mut recent: Vec<&Memory> = self.memories.values().collect();
        recent.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));
        let premises: Vec<String> = recent
            .into_iter()
            .take(recent_n.max(2))
            .map(|m| m.id.clone())
            .collect();
        let id = self.reason(premises, ReasoningType::Inductive)?;
        // 去重（缺陷 #4 修复）：结论与既有 Pattern 高度重叠则丢弃，
        // 防定时 reflect 在重叠 recent 集上批量铸造近重复模式、注水模式库。
        if self.is_duplicate_pattern(&id) {
            self.memories.remove(&id);
            return None;
        }
        if let Some(m) = self.memories.get_mut(&id) {
            m.memory_type = MemoryType::Pattern;
        }
        Some(id)
    }

    /// 结论是否与既有 Pattern 重复（Jaccard ≥ 阈值）。
    /// 公开供外部预检（orchestrator 入库前调用，避免垃圾进记忆）。
    pub fn is_duplicate_pattern(&self, conclusion_id: &str) -> bool {
        let concl = match self.memories.get(conclusion_id) {
            Some(m) => m,
            None => return false,
        };
        let query: HashSet<String> = Self::keywords(&concl.content).into_iter().collect();
        if query.is_empty() {
            return false;
        }
        self.memories.values().any(|m| {
            if m.id == conclusion_id || m.memory_type != MemoryType::Pattern {
                return false;
            }
            let cand: HashSet<String> = Self::keywords(&m.content).into_iter().collect();
            if cand.is_empty() {
                return false;
            }
            let inter = query.intersection(&cand).count() as f64;
            let union = query.union(&cand).count().max(1) as f64;
            inter / union >= REFLECT_DUP_JACCARD
        })
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

    /// 更新进化阶段：Transcend 要求链>50、连接比>0.6、跨域链≥5、记忆≥200
    /// 四门全过（单轮默认预算只到 Evolve，须多轮持续炼化）。
    fn update_phase(&mut self) {
        let memory_count = self.memories.len();
        let chain_count = self.reasoning_chains.len();
        // 读增量计数器 O(1)，全量炼 46万次 remember 不卡
        let connected = self.connected_count;
        let cross_domain = self
            .reasoning_chains
            .iter()
            .filter(|c| c.chain_type == ReasoningType::CrossDomain)
            .count();
        let connected_ratio = connected as f64 / memory_count.max(1) as f64;

        self.phase = if chain_count > 50
            && connected_ratio > 0.6
            && cross_domain >= 5
            && memory_count >= 200
        {
            EvolutionPhase::Transcend
        } else if chain_count > 20 {
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

    /// 更新能力评分（读增量计数器，O(1)）
    fn update_capabilities(&mut self) {
        let memory_count = self.memories.len() as f64;
        let chain_count = self.reasoning_chains.len() as f64;
        let connected = self.connected_count as f64;

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
    // 推理策略（规则式真实现 R-P0-1）
    //
    // 约束：结论必须包含前提的实际内容片段（可溯源），按字符截断
    // （绝不按字节切分，RUST-STANDARDS / clippy::string_slice 合规），
    // 无 unwrap / expect / panic，全路径返回 Option 或默认值。
    // ══════════════════════════════════════════════════════════════════════

    /// 结论片段最大字符数
    const SNIPPET_LEN: usize = 60;

    /// 按字符截断并加省略号（短文本原样返回）
    fn snippet(s: &str) -> String {
        if s.chars().count() > Self::SNIPPET_LEN {
            let taken: String = s.chars().take(Self::SNIPPET_LEN).collect();
            format!("{taken}…")
        } else {
            s.to_string()
        }
    }

    /// 中英停用词（关键词抽取用）
    fn is_stop(word: &str) -> bool {
        matches!(
            word,
            "的" | "了" | "在" | "是" | "和" | "与" | "及" | "或" | "将" | "被" | "对"
                | "等" | "中" | "上" | "下" | "这" | "那" | "它" | "其" | "不" | "没"
                | "很" | "都" | "也" | "就" | "还" | "个" | "有" | "我" | "你"
                | "the" | "a" | "an" | "of" | "to" | "in" | "is" | "and" | "or"
                | "for" | "with" | "on" | "by" | "as" | "at" | "from" | "that"
                | "this" | "it" | "are" | "was" | "be"
        )
    }

    /// 去掉回灌来源前缀 `[src:…]`，避免 `:` 切分把 tag 拆成
    /// 2 个 token 拉低 Jaccard（同体 5/7=0.71 < 0.8 漏拒；
    /// 去 tag 后 5/5=1.0 ≥ 0.8 正确拒绝）。
    fn strip_src_tag(text: &str) -> &str {
        if let Some(rest) = text.strip_prefix("[src:") {
            if let Some(end) = rest.find(']') {
                return rest[end + 1..].trim_start();
            }
        }
        text
    }

    /// 关键词抽取：按空白与中英标点切分，去停用词与单字符 token
    /// （crate 内共享：觉醒循环的新颖度验证复用同一分词口径）
    pub(crate) fn keywords(text: &str) -> Vec<String> {
        Self::strip_src_tag(text).split(|c: char| {
            c.is_whitespace() || "，。、；：？！…—·,. ;:?!()（）「」『』\"'【】《》".contains(c)
        })
        .filter(|w| w.chars().count() > 1 && !Self::is_stop(w))
        .map(|w| w.to_string())
        .collect()
    }

    /// 跨前提共享词：按出现频次排序取前 5（归纳/演绎的证据核心）
    fn shared_terms(premises: &[Memory]) -> Vec<String> {
        let mut freq: HashMap<String, usize> = HashMap::new();
        for m in premises {
            for k in Self::keywords(&m.content) {
                *freq.entry(k).or_insert(0) += 1;
            }
        }
        let mut ranked: Vec<(String, usize)> = freq.into_iter().collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1));
        ranked.into_iter().take(5).map(|(w, _)| w).collect()
    }

    /// 前提置信度均值（结论置信度来源，钳制到 [0.1, 1.0]）
    fn premise_confidence(premises: &[Memory]) -> f64 {
        if premises.is_empty() {
            return 0.5;
        }
        let sum: f64 = premises.iter().map(|m| m.confidence).sum();
        (sum / premises.len() as f64).clamp(0.1, 1.0)
    }

    fn deduce(&self, premises: &[Memory]) -> String {
        let first_domain = premises
            .first()
            .map(|m| m.domain.as_str())
            .unwrap_or("general");
        let same_domain = premises.iter().all(|m| m.domain == first_domain);
        let shared = Self::shared_terms(premises);
        let shared_txt = if shared.is_empty() {
            "共同前提".to_string()
        } else {
            shared.join("、")
        };
        let evidence: Vec<String> =
            premises.iter().map(|m| Self::snippet(&m.content)).collect();
        if same_domain {
            format!(
                "演绎({first_domain})：由{}可得，{shared_txt}成立",
                evidence.join(" ＋ ")
            )
        } else {
            format!(
                "演绎(跨域)：由{}可得，{shared_txt}成立",
                evidence.join(" ＋ ")
            )
        }
    }

    fn induce(&self, premises: &[Memory]) -> String {
        let mut by_domain: HashMap<&str, usize> = HashMap::new();
        for m in premises {
            *by_domain.entry(m.domain.as_str()).or_insert(0) += 1;
        }
        let (top_domain, top_n) = by_domain
            .into_iter()
            .max_by_key(|(_, n)| *n)
            .map(|(d, n)| (d.to_string(), n))
            .unwrap_or(("general".to_string(), 0));
        let shared = Self::shared_terms(premises);
        let shared_txt = if shared.is_empty() {
            "未命名共性".to_string()
        } else {
            shared.join("、")
        };
        format!("归纳({top_domain})：{top_n}条同域现象共享“{shared_txt}”，提炼为共性模式")
    }

    fn abduce(&self, premises: &[Memory]) -> String {
        let best = premises.iter().max_by(|a, b| {
            a.confidence
                .partial_cmp(&b.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let effect = premises
            .last()
            .map(|m| Self::snippet(&m.content))
            .unwrap_or_default();
        match best {
            Some(b) => format!(
                "溯因：观察到“{effect}”，最可能的解释是“{}”（置信{:.2}）",
                Self::snippet(&b.content),
                b.confidence.clamp(0.0, 1.0)
            ),
            None => "溯因：前提为空，无解释".to_string(),
        }
    }

    fn analogize(&self, premises: &[Memory]) -> String {
        let first_snip = premises
            .first()
            .map(|m| Self::snippet(&m.content))
            .unwrap_or_default();
        let first_domain = premises
            .first()
            .map(|m| m.domain.as_str())
            .unwrap_or("general");
        let second = premises.get(1);
        let shared = Self::shared_terms(premises);
        let shared_txt = if shared.is_empty() {
            "隐含结构".to_string()
        } else {
            shared.join("、")
        };
        match second {
            Some(s) => format!(
                "类比：{first_domain}中的“{first_snip}”与{}中的“{}”结构相似，共享{shared_txt}",
                s.domain,
                Self::snippet(&s.content)
            ),
            None => format!("类比：{first_domain}中的“{first_snip}”暂无跨域映射对象"),
        }
    }

    fn cross_domain_reason(&self, premises: &[Memory]) -> String {
        let mut domains: Vec<&str> =
            premises.iter().map(|m| m.domain.as_str()).collect();
        domains.sort_unstable();
        domains.dedup();
        let shared = Self::shared_terms(premises);
        let shared_txt = if shared.is_empty() {
            "待发现的交叉点".to_string()
        } else {
            shared.join("、")
        };
        let evidence: Vec<String> =
            premises.iter().map(|m| Self::snippet(&m.content)).collect();
        format!(
            "跨域融合({})：由{}交汇，得到域交叉新知——{shared_txt}",
            domains.join(" × "),
            evidence.join(" ＋ ")
        )
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
