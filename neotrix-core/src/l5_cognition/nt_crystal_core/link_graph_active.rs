//! LinkGraph 激活模块 — 链接形成规则 + 无意识通信执行器
//!
//! 启发来源: CTM-AI (链接记忆), CRP (协作响应)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::ctm::{Chunk, ChunkType, CTMModule, LinkGraph};

/// 链接形成规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkFormationRules {
    /// 域亲和表: 同组模块自动建立强链接
    pub domain_groups: Vec<Vec<String>>,
    /// 协作频率阈值: 超过此值形成新链接
    pub collaboration_threshold: f64,
    /// 链接衰减率: 每步衰减比例
    pub decay_rate: f64,
    /// 最小链接强度: 低于此值删除
    pub min_strength: f64,
}

impl LinkFormationRules {
    pub fn default_rules() -> Self {
        let domain_groups = vec![
            // NT-MEMORY + NT-PERCEPTION: 高亲和 (感知→记忆流)
            vec!["memory".into(), "perception".into()],
            // NT-ACTION + NT-SHIELD: 高亲和 (行动→安全流)
            vec!["action".into(), "safety".into()],
            // NT-EMOTION + NT-META: 高亲和 (情感→元认知流)
            vec!["emotion".into(), "meta".into()],
        ];

        Self {
            domain_groups,
            collaboration_threshold: 0.6,
            decay_rate: 0.05,
            min_strength: 0.1,
        }
    }

    /// 根据模块域自动形成链接
    pub fn auto_form_links(
        &self,
        links: &mut LinkGraph,
        module_domains: &[(String, String)],
    ) {
        let name_to_domain: HashMap<&str, &str> = module_domains
            .iter()
            .map(|(n, d)| (n.as_str(), d.as_str()))
            .collect();

        for group in &self.domain_groups {
            let members: Vec<&str> = group
                .iter()
                .filter(|name| name_to_domain.contains_key(name.as_str()))
                .map(|name| name.as_str())
                .collect();

            // 同域成员两两建立链接
            for i in 0..members.len() {
                for j in (i + 1)..members.len() {
                    if !links.has_link(members[i], members[j]) {
                        links.form_link(members[i], members[j], 0.5);
                    }
                }
            }
        }

        // 跨域协作检测: 两个不同域的模块如果都出现在 module_domains 中
        // 且它们的域在 domain_groups 中有交叉，增强链接
        for (name_a, domain_a) in module_domains {
            for (name_b, domain_b) in module_domains {
                if name_a == name_b {
                    continue;
                }
                // 检查是否共享某个 domain group
                for group in &self.domain_groups {
                    let has_a = group.iter().any(|g| g == domain_a);
                    let has_b = group.iter().any(|g| g == domain_b);
                    if has_a && has_b && !links.has_link(name_a, name_b) {
                        links.form_link(name_a, name_b, 0.3);
                    }
                }
            }
        }
    }

    /// 链接衰减: 所有链接强度 *= (1 - decay_rate)
    pub fn decay_links(&self, links: &mut LinkGraph) -> Vec<String> {
        let mut removed = Vec::new();
        // 收集所有链接
        let all_pairs: Vec<(String, String, f64)> = {
            let mut pairs = Vec::new();
            let mut seen = std::collections::HashSet::new();
            for name in links.all_modules() {
                for (other, strength) in links.get_links(&name) {
                    let key = if name < other {
                        (name.clone(), other.clone())
                    } else {
                        (other.clone(), name.clone())
                    };
                    if seen.insert(key.clone()) {
                        pairs.push((key.0, key.1, strength));
                    }
                }
            }
            pairs
        };

        for (name_a, name_b, old_strength) in all_pairs {
            let new_strength = old_strength * (1.0 - self.decay_rate);
            if new_strength < self.min_strength {
                links.remove_link(&name_a, &name_b);
                removed.push(format!("{} <-> {}", name_a, name_b));
            } else {
                links.form_link(&name_a, &name_b, new_strength);
            }
        }
        removed
    }
}

/// 通信事件记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationEvent {
    pub from: String,
    pub to: String,
    pub content_preview: String,
    pub strength: f64,
    pub timestamp: u64,
}

/// 通信统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationStats {
    pub total_events: usize,
    pub avg_strength: f64,
    pub unique_pairs: usize,
    pub events_by_module: HashMap<String, usize>,
}

/// 无意识通信执行器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnconsciousCommunicator {
    pub rules: LinkFormationRules,
    pub communication_log: Vec<CommunicationEvent>,
    #[serde(skip)]
    counter: u64,
}

impl Default for UnconsciousCommunicator {
    fn default() -> Self {
        Self::new()
    }
}

impl UnconsciousCommunicator {
    pub fn new() -> Self {
        Self {
            rules: LinkFormationRules::default_rules(),
            communication_log: Vec::new(),
            counter: 0,
        }
    }

    pub fn with_rules(rules: LinkFormationRules) -> Self {
        Self {
            rules,
            communication_log: Vec::new(),
            counter: 0,
        }
    }

    /// 执行无意识通信: 强链接模块间直接传递信息
    pub fn communicate(
        &mut self,
        links: &LinkGraph,
        modules: &mut [Box<dyn CTMModule>],
        min_strength: f64,
    ) -> Vec<CommunicationEvent> {
        let mut events = Vec::new();
        self.counter += 1;
        let now = self.counter;

        // 建立模块名→索引映射
        let name_to_idx: HashMap<String, usize> = modules
            .iter()
            .enumerate()
            .map(|(i, m)| (m.name().to_string(), i))
            .collect();

        // 遍历所有强链接
        for name in links.all_modules() {
            for (other, strength) in links.get_links(&name) {
                if strength < min_strength {
                    continue;
                }
                // 找到发送方和接收方
                if let Some(&to_idx) = name_to_idx.get(&other) {
                    // 从发送方的最新输出构建通信内容
                    let source_chunk = Chunk {
                        content: format!("[unconscious] {} -> {}", name, other),
                        score: strength,
                        source_module: name.clone(),
                        chunk_type: ChunkType::Memory,
                        metadata: HashMap::new(),
                    };

                    let content_preview = format!(
                        "link({:.2}): {} -> {}",
                        strength, name, other
                    );

                    // 接收方写入信息
                    modules[to_idx].write(source_chunk);

                    let event = CommunicationEvent {
                        from: name.clone(),
                        to: other.clone(),
                        content_preview,
                        strength,
                        timestamp: now,
                    };
                    events.push(event.clone());
                    self.communication_log.push(event);
                }
            }
        }

        events
    }

    /// 获取通信统计
    pub fn stats(&self) -> CommunicationStats {
        let mut events_by_module: HashMap<String, usize> = HashMap::new();
        let mut pairs = std::collections::HashSet::new();
        let mut total_strength = 0.0;

        for event in &self.communication_log {
            *events_by_module
                .entry(event.from.clone())
                .or_insert(0) += 1;
            *events_by_module
                .entry(event.to.clone())
                .or_insert(0) += 1;

            let key = if event.from < event.to {
                (event.from.clone(), event.to.clone())
            } else {
                (event.to.clone(), event.from.clone())
            };
            pairs.insert(key);
            total_strength += event.strength;
        }

        let count = self.communication_log.len();
        CommunicationStats {
            total_events: count,
            avg_strength: if count > 0 {
                total_strength / count as f64
            } else {
                0.0
            },
            unique_pairs: pairs.len(),
            events_by_module,
        }
    }
}
