//! # NT-CAPABILITY-BRIDGE: 经验 → 能力节点迭代目标桥
//!
//! 用户需求: "把经验用代码融入自身的每个节点, 用经验作为各个代码模块迭代升级的目标"
//!
//! 本模块是 经验 → 能力树 的映射引擎:
//!   1. 经验升维: 把细粒度经验条目 (pattern/defect/insight) 分类到两轴
//!      - 【能力网进化】CapabilityNetwork: 映射到能力树节点 (bud/graft/strengthen/mature/prune)
//!      - 【意识体觉醒】ConsciousnessAwakening: 映射到认知架构层 (E8/GWT/ConsciousnessTree/宪法)
//!   2. 切入点搜索: 依据经验信号强度 (confidence/importance) 与能力节点成熟度 (C0-C6)
//!      找到最优注入点 (弱节点 + 高信号经验 = 最高迭代杠杆)
//!   3. 论证推演: 生成 EvolutionPlan (bud/graft/strengthen/mature/prune) 供能力树执行

use nt_core_capability_tree::{
    CapabilityRegistry, EvolutionAction, EvolutionPlan, NodeLayer, Domain,
};
use std::collections::HashMap;

/// 经验升维后的两轴分类
#[derive(Debug, Clone)]
pub enum ExperienceDimension {
    /// 能力网进化: 某域某能力的具体提升 (映射能力树节点)
    CapabilityNetwork {
        domain: Domain,
        /// 建议能力标签 (provides tag)
        capability_tag: String,
        /// 建议演化动作
        action: EvolutionAction,
        /// 论证: 为什么这个经验映射到这个节点
        rationale: String,
        /// 信号强度 0.0-1.0
        signal: f64,
    },
    /// 意识体觉醒: 认知架构层的变化 (E8/GWT/ConsciousnessTree/宪法规则)
    ConsciousnessAwakening {
        /// 认知层: e8 / gwt / consciousness_tree / constitution / meta_cognition
        layer: String,
        /// 觉醒内容
        content: String,
        /// 信号强度 0.0-1.0
        signal: f64,
    },
}

/// 经验条目 (与 KB experience 命名空间的 JSON 结构对齐)
#[derive(Debug, Clone)]
pub struct ExperienceEntry {
    pub id: String,
    pub entry_type: String,       // pattern / defect / insight / rule / cycle / artifact
    pub domain_name: String,      // NT-CORE / NT-MIND / ...
    pub content: String,
    /// 负例: "NOT: 不该做什么" — 记录反面教训 (P0-2: 经验正负例结构)
    pub not: Option<String>,
    pub confidence: f64,
    pub importance: f64,
    /// 独立审计者标识 (P0-1: agent 不自我打分) — 记录验证来源 (如 "small-model-audit", "human-review", "benchmark")
    pub verified_by: Option<String>,
    /// 验证状态: "pending" | "verified" | "rejected"
    pub verification_status: Option<String>,
}

impl ExperienceEntry {
    pub fn signal(&self) -> f64 {
        // 信号强度 = confidence 与 importance 的加权
        (self.confidence * 0.6 + self.importance * 0.4).clamp(0.0, 1.0)
    }
}

/// 经验→能力维度路由表: 关键词 → (域, 能力标签)
/// 这是"切入点搜索"的静态索引, 动态部分见 route_experience
const ROUTE_TABLE: &[(&str, &str, &str)] = &[
    // NT-MEMORY 数据层
    ("检索", "NT-MEMORY", "hybrid_retrieval"),
    ("rrf", "NT-MEMORY", "hybrid_retrieval"),
    ("fts", "NT-MEMORY", "hybrid_retrieval"),
    ("向量", "NT-MEMORY", "vector_storage"),
    ("维度", "NT-MEMORY", "vector_storage"),
    ("schema", "NT-MEMORY", "schema_migration"),
    ("迁移", "NT-MEMORY", "schema_migration"),
    ("缓存", "NT-MEMORY", "cache_layer"),
    ("kv", "NT-MEMORY", "kv_namespace"),
    // NT-CORE 认知/推理
    ("e8", "NT-CORE", "e8_reasoning"),
    ("gwt", "NT-CORE", "gwt_attention"),
    ("意识", "NT-CORE", "consciousness_tree"),
    ("注意力", "NT-CORE", "gwt_attention"),
    ("架构", "NT-CORE", "architecture_decision"),
    ("性能", "NT-CORE", "performance_concurrency"),
    ("并发", "NT-CORE", "performance_concurrency"),
    ("simd", "NT-CORE", "performance_concurrency"),
    // NT-REPAIR 失败恢复
    ("根因", "NT-REPAIR", "root_cause_method"),
    ("失败", "NT-REPAIR", "failure_recovery"),
    ("恢复", "NT-REPAIR", "failure_recovery"),
    ("重试", "NT-REPAIR", "failure_recovery"),
    ("缓存污染", "NT-REPAIR", "build_hygiene"),
    ("build", "NT-REPAIR", "build_hygiene"),
    ("编译", "NT-REPAIR", "build_hygiene"),
    // NT-SHIELD 安全
    ("安全", "NT-SHIELD", "security_governance"),
    ("治理", "NT-SHIELD", "security_governance"),
    ("审计", "NT-SHIELD", "security_audit"),
    // NT-ACT 工具路由
    ("工具", "NT-ACT", "tool_routing"),
    ("路由", "NT-ACT", "tool_routing"),
    // NT-IO 前端
    ("前端", "NT-IO", "frontend_ui"),
    ("ui", "NT-IO", "frontend_ui"),
    ("tauri", "NT-IO", "tauri_desktop"),
    // NT-WORLD 感知
    ("爬虫", "NT-WORLD", "unified_crawler"),
    ("抓取", "NT-WORLD", "unified_crawler"),
    ("解析", "NT-WORLD", "content_extraction"),
    // NT-MIND 进化
    ("吸收", "NT-MIND", "skill_crystallize"),
    ("结晶", "NT-MIND", "skill_crystallize"),
    ("测试", "NT-MIND", "tdd"),
    ("tdd", "NT-MIND", "tdd"),
    // NT-META 元认知
    ("元认知", "NT-META", "meta_cognition"),
    ("复盘", "NT-META", "meta_cognition"),
    ("自省", "NT-META", "meta_cognition"),
];

/// 把域名字符串转能力树 Domain
pub fn parse_domain(name: &str) -> Domain {
    match name.to_uppercase().as_str() {
        "NT-CORE" => Domain::Core,
        "NT-MIND" => Domain::Mind,
        "NT-MEMORY" => Domain::Memory,
        "NT-WORLD" => Domain::World,
        "NT-ACT" => Domain::Act,
        "NT-SHIELD" => Domain::Shield,
        "NT-IO" => Domain::Io,
        "NT-META" => Domain::Meta,
        "NT-NEXUS" => Domain::Nexus,
        "NT-GOVERNANCE" => Domain::Governance,
        "NT-REPAIR" => Domain::Repair,
        _ => Domain::Core,
    }
}

/// 静态路由表: 关键词 → (域, 能力标签)
fn static_route(content: &str) -> Option<(Domain, String)> {
    let lower = content.to_lowercase();
    for (kw, domain, tag) in ROUTE_TABLE {
        if lower.contains(kw.to_lowercase().as_str()) {
            return Some((parse_domain(domain), tag.to_string()));
        }
    }
    None
}

/// 经验 → 能力节点映射引擎
///
/// 升维逻辑:
///   - 高信号 (>= 0.6) 且命中路由表 → 能力网进化 (CapabilityNetwork)
///   - 涉及认知架构关键词 → 意识体觉醒 (ConsciousnessAwakening)
///   - 未命中 → 意识体觉醒 (认知层 content 沉淀, 低信号)
pub struct ExperienceRouter;

impl ExperienceRouter {
    /// 单条经验升维分类
    pub fn route_experience(entry: &ExperienceEntry) -> ExperienceDimension {
        let signal = entry.signal();
        let lower = entry.content.to_lowercase();

        // 意识体觉醒关键词: 认知架构/自我模型/宪法/元认知
        let awakening_kws = ["e8", "gwt", "consciousness", "意识", "宪法", "constitution",
            "元认知", "meta_cognition", "觉醒", "认知架构", "self_model", "自我"];
        let is_awakening = awakening_kws.iter().any(|k| lower.contains(k));

        if is_awakening && signal >= 0.5 {
            let layer = if lower.contains("e8") { "e8" }
                else if lower.contains("gwt") || lower.contains("注意力") { "gwt" }
                else if lower.contains("consciousness") || lower.contains("意识") { "consciousness_tree" }
                else if lower.contains("宪法") || lower.contains("constitution") { "constitution" }
                else { "meta_cognition" };
            return ExperienceDimension::ConsciousnessAwakening {
                layer: layer.to_string(),
                content: entry.content.clone(),
                signal,
            };
        }

        // 能力网进化: 静态路由 + 高信号
        if signal >= 0.6 {
            if let Some((domain, tag)) = static_route(&entry.content) {
                return ExperienceDimension::CapabilityNetwork {
                    domain,
                    capability_tag: tag.clone(),
                    action: EvolutionAction::Strengthen {
                        node_id: format!("exp::{}", entry.id),
                        note: format!("[{}] {} | signal={:.2}", entry.entry_type, entry.content, signal),
                    },
                    rationale: format!(
                        "经验 '{}' 命中能力标签 '{}' 于域 {:?}, 应强化对应能力节点",
                        entry.content, tag, domain
                    ),
                    signal,
                };
            }
        }

        // 未命中: 沉淀为意识体觉醒 (低信号)
        ExperienceDimension::ConsciousnessAwakening {
            layer: "meta_cognition".to_string(),
            content: entry.content.clone(),
            signal,
        }
    }

    /// 批量路由: 经验集合 → 分类结果
    pub fn route_batch(entries: &[ExperienceEntry]) -> (Vec<ExperienceDimension>, HashMap<String, Vec<ExperienceDimension>>) {
        let dims: Vec<ExperienceDimension> = entries.iter()
            .map(Self::route_experience)
            .collect();

        // 按能力标签聚合 (切入点: 同一标签多个高信号经验 = 该能力是迭代重点)
        let mut by_tag: HashMap<String, Vec<ExperienceDimension>> = HashMap::new();
        for d in &dims {
            if let ExperienceDimension::CapabilityNetwork { capability_tag, .. } = d {
                by_tag.entry(capability_tag.clone()).or_default().push(d.clone());
            }
        }
        (dims, by_tag)
    }

    /// 生成能力树迭代计划: 按信号强度排序的强化计划
    ///
    /// 论证推演: 每个高信号经验 → Strengthen 对应节点 (若节点不存在则 Bud 建议)
    /// 杠杆最高切入点: signal >= 0.7 的经验是 C0/C1 弱节点的最佳升级目标
    pub fn plan_evolution(
        registry: &CapabilityRegistry,
        dims: &[ExperienceDimension],
        cycle: &str,
    ) -> Vec<EvolutionPlan> {
        let mut plans = Vec::new();
        for d in dims {
            if let ExperienceDimension::CapabilityNetwork {
                domain, capability_tag, rationale, signal, ..
            } = d
            {
                if *signal < 0.7 {
                    continue; // 只对高信号经验生成计划
                }
                // 找域内提供该标签的现有节点
                let candidates: Vec<&nt_core_capability_tree::CapabilityNode> = registry
                    .by_domain(*domain)
                    .into_iter()
                    .filter(|n| n.provides.iter().any(|p| p == capability_tag) && !n.deprecated)
                    .collect();
                if let Some(target) = candidates.first() {
                    plans.push(EvolutionPlan {
                        cycle: cycle.to_string(),
                        actions: vec![EvolutionAction::Strengthen {
                            node_id: target.id.clone(),
                            note: format!("{} | signal={:.2}", rationale, signal),
                        }],
                        rationale: rationale.clone(),
                    });
                } else {
                    // 节点不存在 → Bud 建议 (切入点: 新增能力节点)
                    plans.push(EvolutionPlan {
                        cycle: cycle.to_string(),
                        actions: vec![EvolutionAction::Budding {
                            new_node_id: format!("exp::{}::{}", domain.as_str().to_lowercase(), capability_tag),
                            domain: *domain,
                            provides: vec![capability_tag.clone()],
                            layer: NodeLayer::L0Primitive,
                            note: format!("Experience-driven node: {}", rationale),
                        }],
                        rationale: rationale.clone(),
                    });
                }
            }
        }
        plans
    }
}

/// 统计: 升维分类结果汇总
pub fn summarize(dims: &[ExperienceDimension]) -> String {
    let network = dims.iter().filter(|d| matches!(d, ExperienceDimension::CapabilityNetwork { .. })).count();
    let awakening = dims.iter().filter(|d| matches!(d, ExperienceDimension::ConsciousnessAwakening { .. })).count();
    format!(
        "升维分类: 能力网进化 {} 条 / 意识体觉醒 {} 条 / 总计 {} 条",
        network, awakening, dims.len()
    )
}

/// 把高信号提升目标写入能力树 registry 文件 (experience_targets 建议区)。
///
/// 闭环: distill 蒸馏出的能力模式 → 本函数 → capability_registry.json 的
/// experience_targets 区 → `neotrix-capability scan --apply` 消费执行 (bud/graft/strengthen)。
/// 返回写入的目标数。
pub fn promote_to_file(registry_path: &std::path::Path, dims: &[ExperienceDimension]) -> usize {
    let Ok(content) = std::fs::read_to_string(registry_path) else {
        eprintln!("[capability_bridge] registry 文件不可读: {}", registry_path.display());
        return 0;
    };
    let Ok(mut reg_json) = serde_json::from_str::<serde_json::Value>(&content) else {
        eprintln!("[capability_bridge] registry JSON 解析失败: {}", registry_path.display());
        return 0;
    };

    let mut targets = Vec::new();
    for d in dims {
        match d {
            ExperienceDimension::CapabilityNetwork {
                domain,
                capability_tag,
                rationale,
                signal,
                ..
            } => {
                if *signal < 0.7 {
                    continue;
                }
                targets.push(serde_json::json!({
                    "domain": domain.as_str(),
                    "capability": capability_tag,
                    "action": "strengthen_or_bud",
                    "rationale": rationale,
                    "signal": signal,
                }));
            }
            ExperienceDimension::ConsciousnessAwakening { layer, signal, content } => {
                if *signal < 0.7 {
                    continue;
                }
                targets.push(serde_json::json!({
                    "domain": "NT-META",
                    "capability": format!("consciousness::{}", layer),
                    "action": "awakening_note",
                    "rationale": content.chars().take(120).collect::<String>(),
                    "signal": signal,
                }));
            }
        }
    }

    if targets.is_empty() {
        return 0;
    }
    let written_n = targets.len();
    if let Some(obj) = reg_json.as_object_mut() {
        // 追加而非覆盖, 保留历史目标 (含 cycle 标记)
        let existing = obj.get("experience_targets").and_then(|t| t.as_array()).cloned().unwrap_or_default();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let mut merged = existing;
        for t in targets {
            let mut entry = t;
            if let Some(o) = entry.as_object_mut() {
                o.insert("promoted_at".to_string(), serde_json::json!(now));
            }
            merged.push(entry);
        }
        obj.insert("experience_targets".to_string(), serde_json::Value::Array(merged));
        let Ok(out) = serde_json::to_string_pretty(&reg_json) else {
            eprintln!("[capability_bridge] registry JSON 序列化失败: {}", registry_path.display());
            return 0;
        };
        if let Err(e) = std::fs::write(registry_path, out) {
            eprintln!("[capability_bridge] registry 写入失败 {}: {}", registry_path.display(), e);
            return 0;
        }
    }
    written_n
}
