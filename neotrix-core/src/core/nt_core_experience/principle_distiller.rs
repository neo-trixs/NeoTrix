/// PrincipleDistiller — 从执行迹 + 档案中蒸馏设计原则
///
/// 扫描以下数据源，提取重复模式并转换为 KnowledgeNode:
/// - EvolutionTaskSystem: 任务完成/失败模式
/// - DecisionChain: 决策上下文 + 结果
/// - TokenRegistry: 健康度趋势
///
/// 这是 GEPA 反射式进化的知识蒸馏阶段：迹 → 模式 → 原则。

use std::collections::HashMap;

use super::decision_chain::DecisionChain;
use super::design_token::{PrimitiveDomain, PrimitiveToken, TokenRegistry};
use super::knowledge_node::{KnowledgeGraph, NodeType};

/// 蒸馏配置。
#[derive(Debug, Clone)]
pub struct DistillerConfig {
    /// 每次扫描最大生成的知识节点数
    pub max_principles_per_scan: usize,
    /// 视为"重复模式"的最小出现次数
    pub min_evidence_for_pattern: u64,
    /// 低健康的健康阈值
    pub low_health_threshold: f64,
}

impl Default for DistillerConfig {
    fn default() -> Self {
        Self {
            max_principles_per_scan: 5,
            min_evidence_for_pattern: 2,
            low_health_threshold: 0.4,
        }
    }
}

/// 原则蒸馏器。从运行时数据中提取设计原则/模式/决策/反模式。
#[derive(Debug, Clone)]
pub struct PrincipleDistiller {
    pub config: DistillerConfig,
    pub scan_count: u64,
    /// 跟踪已生成的知识节点标题，防止重复
    known_titles: HashMap<String, u64>,
}

impl PrincipleDistiller {
    pub fn new(config: DistillerConfig) -> Self {
        Self {
            config,
            scan_count: 0,
            known_titles: HashMap::new(),
        }
    }

    /// 执行一次完整的蒸馏扫描。
    ///
    /// 返回本次新增的知识节点 ID 列表。
    pub fn scan(
        &mut self,
        cycle: u64,
        tokens: &mut TokenRegistry,
        knowledge: &mut KnowledgeGraph,
        decisions: &DecisionChain,
    ) -> Vec<u64> {
        self.scan_count += 1;
        let mut new_ids: Vec<u64> = Vec::new();

        // 1. 从低健康度基元蒸馏反模式
        for sem in &tokens.semantics {
            if sem.health < self.config.low_health_threshold && sem.health > 0.0 {
                let title = format!("low_{}", sem.primitive.name());
                if self.known_titles.contains_key(&title) {
                    let known_id = self.known_titles[&title];
                    knowledge.add_evidence(known_id, cycle);
                } else {
                    let id = knowledge.add_node(
                        NodeType::Antipattern,
                        format!("{}: {}", sem.primitive.name(), sem.intent),
                        format!("Health {:.3} is below threshold {:.1}. Primitive: {}.",
                            sem.health, self.config.low_health_threshold, sem.primitive.name()),
                        sem.confidence,
                        vec![],
                        vec![sem.primitive.name().to_string()],
                        cycle,
                    );
                    self.known_titles.insert(title, id);
                    new_ids.push(id);
                }
            }
        }

        // 2. 从决策链提取原则
        if decisions.total_entries() >= 2 {
            let recent_decisions = decisions.recent_decisions(10, false);
            // 按提案类型分组计算成功率
            let mut type_stats: HashMap<String, (usize, usize)> = HashMap::new();
            for d in &recent_decisions {
                let entry = type_stats.entry(d.proposal_type.clone()).or_insert((0, 0));
                entry.0 += 1;
                if d.success { entry.1 += 1; }
            }

            for (ptype, (total, success)) in &type_stats {
                if *total >= self.config.min_evidence_for_pattern as usize {
                    let rate = *success as f64 / *total as f64;
                    let title = format!("decision_success_{}", ptype);

                    if rate > 0.7 {
                        // 成功的模式 → Principle
                        if let Some(&known_id) = self.known_titles.get(&title) {
                            knowledge.add_evidence(known_id, cycle);
                        } else if new_ids.len() < self.config.max_principles_per_scan {
                            let id = knowledge.add_node(
                                NodeType::Principle,
                                format!("{} 决策模式: {}/{} 成功 ({:.0}%)",
                                    ptype, success, total, rate * 100.0),
                                format!("Proposal type '{}' has {:.0}% success rate across {} attempts.",
                                    ptype, rate * 100.0, total),
                                rate * 0.9,
                                vec![],
                                vec![],
                                cycle,
                            );
                            self.known_titles.insert(title, id);
                            new_ids.push(id);
                        }
                    } else if rate < 0.3 && *total >= 3 {
                        // 失败的模式 → Antipattern
                        let antip_title = format!("antipattern_{}", ptype);
                        if !self.known_titles.contains_key(&antip_title)
                            && new_ids.len() < self.config.max_principles_per_scan
                        {
                            let id = knowledge.add_node(
                                NodeType::Antipattern,
                                format!("{} 高风险: {}/{} 失败 ({:.0}%)",
                                    ptype, total - success, total, (1.0 - rate) * 100.0),
                                format!("Proposal type '{}' has high failure rate.", ptype),
                                1.0 - rate,
                                vec![],
                                vec![],
                                cycle,
                            );
                            self.known_titles.insert(antip_title, id);
                            new_ids.push(id);
                        }
                    }
                }
            }
        }

        // 3. 从一致的健康趋势中提取模式
        let mut improving_sensors: Vec<String> = Vec::new();
        let mut declining_sensors: Vec<String> = Vec::new();
        for sem in &tokens.semantics {
            if sem.trend > 0.02 && sem.confidence > 0.6 {
                improving_sensors.push(sem.primitive.name().to_string());
            } else if sem.trend < -0.02 && sem.confidence > 0.6 {
                declining_sensors.push(sem.primitive.name().to_string());
            }
        }

        if !improving_sensors.is_empty() && new_ids.len() < self.config.max_principles_per_scan {
            let title = format!("trend_improving_{}", improving_sensors.len());
            if !self.known_titles.contains_key(&title) {
                let desc = format!("Improving primitives: {}", improving_sensors.join(", "));
                let id = knowledge.add_node(
                    NodeType::Pattern,
                    format!("{} 个基元正在改善", improving_sensors.len()),
                    desc,
                    0.6,
                    vec![],
                    improving_sensors,
                    cycle,
                );
                self.known_titles.insert(title, id);
                new_ids.push(id);
            }
        }

        if !declining_sensors.is_empty() && new_ids.len() < self.config.max_principles_per_scan {
            let title = format!("trend_declining_{}", declining_sensors.len());
            if !self.known_titles.contains_key(&title) {
                let desc = format!("Declining primitives: {}", declining_sensors.join(", "));
                let id = knowledge.add_node(
                    NodeType::Pattern,
                    format!("{} 个基元正在恶化", declining_sensors.len()),
                    desc,
                    0.6,
                    vec![],
                    declining_sensors,
                    cycle,
                );
                self.known_titles.insert(title, id);
                new_ids.push(id);
            }
        }

        new_ids
    }

    /// 设计感知扫描：从 DESIGN.md 意识 + 设计令牌传感器提取设计节点。
    ///
    /// 检测以下状况并生成对应类型节点：
    /// - 低 TokenCoverage → DesignToken 缺失反模式
    /// - 布局模式(重复成功) → LayoutPattern
    /// - 组件描述(循环中一致) → ComponentSpec
    /// - 设计系统深度变化 → Principle
    pub fn scan_design(
        &mut self,
        cycle: u64,
        tokens: &mut TokenRegistry,
        knowledge: &mut KnowledgeGraph,
    ) -> Vec<u64> {
        let mut new_ids: Vec<u64> = Vec::new();

        // 收集所有设计域语义的健康度
        let design_sems: Vec<_> = tokens.semantics.iter()
            .filter(|s| matches!(s.primitive.domain(), PrimitiveDomain::Design))
            .cloned()
            .collect();

        // --- 低令牌覆盖率反模式 ---
        let coverage = design_sems.iter()
            .find(|s| matches!(s.primitive, PrimitiveToken::TokenCoverage));
        if let Some(cov) = coverage {
            if cov.health < self.config.low_health_threshold && cov.health > 0.0 {
                let title = "low_token_coverage".to_string();
                if let Some(&known_id) = self.known_titles.get(&title) {
                    knowledge.add_evidence(known_id, cycle);
                } else if new_ids.len() < self.config.max_principles_per_scan {
                    let id = knowledge.add_node(
                        NodeType::Antipattern,
                        "令牌覆盖率低: 设计系统未定义".to_string(),
                        format!("TokenCoverage {:.3} — 设计系统定义不足，新增 UI 将使用硬编码值。", cov.health),
                        cov.confidence,
                        vec![],
                        vec!["design_token".to_string()],
                        cycle,
                    );
                    self.known_titles.insert(title, id);
                    new_ids.push(id);
                }
            }
        }

        // --- 间距一致性好的 LayoutPattern ---
        let spacing = design_sems.iter()
            .find(|s| matches!(s.primitive, PrimitiveToken::SpacingConsistency));
        if let Some(s) = spacing {
            if s.health > 0.7 && s.trend > 0.01 {
                let title = "spacing_layer_pattern".to_string();
                if !self.known_titles.contains_key(&title)
                    && new_ids.len() < self.config.max_principles_per_scan
                {
                    let id = knowledge.add_node(
                        NodeType::LayoutPattern,
                        "间距层系统已稳定".to_string(),
                        format!("间距一致性 {:.2} — 使用设计令牌布局而非硬编码值的模式已确立。", s.health),
                        0.6,
                        vec![],
                        vec!["design_token".to_string(), "spacing".to_string()],
                        cycle,
                    );
                    self.known_titles.insert(title, id);
                    new_ids.push(id);
                }
            }
        }

        // --- 色彩与排版一致性好的 ComponentSpec ---
        let color = design_sems.iter()
            .find(|s| matches!(s.primitive, PrimitiveToken::ColorConsistency));
        let typo = design_sems.iter()
            .find(|s| matches!(s.primitive, PrimitiveToken::TypographyConsistency));
        if let (Some(c), Some(t)) = (color, typo) {
            if c.health > 0.7 && t.health > 0.7 {
                let title = "component_spec_mature".to_string();
                if !self.known_titles.contains_key(&title)
                    && new_ids.len() < self.config.max_principles_per_scan
                {
                    let id = knowledge.add_node(
                        NodeType::ComponentSpec,
                        "色彩+排版系统达到成熟".to_string(),
                        format!("色彩 {:.2} / 排版 {:.2} — 组件可使用设计令牌系统定义规范变体。", c.health, t.health),
                        0.65,
                        vec![],
                        vec!["color".to_string(), "typography".to_string()],
                        cycle,
                    );
                    self.known_titles.insert(title, id);
                    new_ids.push(id);
                }
            }
        }

        // --- 设计系统深度趋势产生 Principle ---
        let depth = design_sems.iter()
            .find(|s| matches!(s.primitive, PrimitiveToken::DesignSystemDepth));
        if let Some(d) = depth {
            if d.trend > 0.05 && d.confidence > 0.6 {
                let title = "design_depth_improving".to_string();
                if !self.known_titles.contains_key(&title)
                    && new_ids.len() < self.config.max_principles_per_scan
                {
                    let id = knowledge.add_node(
                        NodeType::Principle,
                        "设计系统深度持续增加".to_string(),
                        format!("DesignSystemDepth趋势 {:.3} — 更多设计层次被代码引用。", d.trend),
                        0.55,
                        vec![],
                        vec!["design_system".to_string()],
                        cycle,
                    );
                    self.known_titles.insert(title, id);
                    new_ids.push(id);
                }
            }
        }

        new_ids
    }

    pub fn stats(&self) -> String {
        format!(
            "principle_distiller: scans={}, known_titles={}",
            self.scan_count,
            self.known_titles.len(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_experience::{DecisionContext, PrimitiveToken};

    #[test]
    fn test_scan_generates_antipatterns_for_low_health() {
        let mut tokens = TokenRegistry::new();
        let mut knowledge = KnowledgeGraph::new();
        let decisions = DecisionChain::new();

        // Set a sensor very low
        tokens.update_sensor(PrimitiveToken::Phi, 0.05, 0.0, 0.5);

        let mut distiller = PrincipleDistiller::new(DistillerConfig::default());
        let new_ids = distiller.scan(1, &mut tokens, &mut knowledge, &decisions);
        assert!(!new_ids.is_empty(), "should generate at least one antipattern");

        let aps = knowledge.find_by_type(NodeType::Antipattern);
        assert!(!aps.is_empty(), "should have antipattern nodes");
    }

    #[test]
    fn test_scan_reuses_known_titles() {
        let mut tokens = TokenRegistry::new();
        let mut knowledge = KnowledgeGraph::new();
        let decisions = DecisionChain::new();
        tokens.update_sensor(PrimitiveToken::Phi, 0.05, 0.0, 0.5);

        let mut distiller = PrincipleDistiller::new(DistillerConfig::default());
        let first = distiller.scan(1, &mut tokens, &mut knowledge, &decisions);
        let first_count = first.len();

        let second = distiller.scan(2, &mut tokens, &mut knowledge, &decisions);
        // Second scan should add evidence not new nodes
        assert!(second.len() < first_count || second.is_empty());
    }

    #[test]
    fn test_scan_from_decision_chain() {
        let mut tokens = TokenRegistry::new();
        let mut knowledge = KnowledgeGraph::new();
        let mut decisions = DecisionChain::new();

        // Create some decision history
        for i in 0..5 {
            let ctx = DecisionContext {
                ece: 0.1, meta_accuracy: 0.8, composite_loss: 0.3, arousal: 0.5, cycle: i,
            };
            let id = decisions.begin_decision(
                format!("p{}", i), "ModuleWiring".into(), ctx.clone(),
                vec![], "test".into(), "gain".into(),
            );
            decisions.complete_decision(id, ctx.clone(), 0.05, true);
        }

        let mut distiller = PrincipleDistiller::new(DistillerConfig::default());
        let new_ids = distiller.scan(10, &mut tokens, &mut knowledge, &decisions);
        // Should find ModuleWiring has high success → generate principle
        let principles = knowledge.find_by_type(NodeType::Principle);
        let has_wiring = principles.iter().any(|p| p.title.contains("ModuleWiring"));
        assert!(has_wiring || !new_ids.is_empty());
    }

    #[test]
    fn test_improving_declining_patterns() {
        let mut tokens = TokenRegistry::new();
        let mut knowledge = KnowledgeGraph::new();
        let decisions = DecisionChain::new();

        // Simulate improving ECE
        tokens.update_sensor(PrimitiveToken::Ece, 0.05, 0.03, 0.8);
        // Simulate declining meta-accuracy
        tokens.update_sensor(PrimitiveToken::MetaAccuracy, 0.6, -0.05, 0.8);

        let mut distiller = PrincipleDistiller::new(DistillerConfig::default());
        let new_ids = distiller.scan(1, &mut tokens, &mut knowledge, &decisions);

        let patterns = knowledge.find_by_type(NodeType::Pattern);
        let has_improving = patterns.iter().any(|p| p.title.contains("改善"));
        let has_declining = patterns.iter().any(|p| p.title.contains("恶化"));
        assert!(has_improving || has_declining || !new_ids.is_empty());
    }

    #[test]
    fn test_empty_scan_noop() {
        let mut tokens = TokenRegistry::new();
        let mut knowledge = KnowledgeGraph::new();
        let decisions = DecisionChain::new();

        let mut distiller = PrincipleDistiller::new(DistillerConfig::default());
        let new_ids = distiller.scan(1, &mut tokens, &mut knowledge, &decisions);
        // With default health (all > 0.4) and empty decisions, should produce little
        assert!(new_ids.is_empty() || new_ids.len() <= 2);
    }

    #[test]
    fn test_stats() {
        let distiller = PrincipleDistiller::new(DistillerConfig::default());
        assert!(distiller.stats().contains("principle_distiller:"));
    }
}
