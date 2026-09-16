//! L4 / NT-CORE — design-extract (github.com/Manavarya09/design-extract) 吸收节点 (C0)。
//!
//! 源: design-extract — 从工件 (源代码 / 架构图 / 文档) 中抽取设计模式, 并映射为
//! 可复用的设计知识。NeoTrix 视角: 模式提取 → 能力节点 (capability node) 映射,
//! 强化 NT-CORE 的架构仲裁与模式识别能力。
//!
//! 本模块提供 trait: 从代码/图提取模式 → 产出能力节点映射。C0 (编译通过 + 基础
//! 逻辑 + SelfTest T1)。真实 AST/图解析留待 C2+。

use crate::core::nt_core_self_test::SelfTest;
use std::collections::HashMap;

/// 一个被识别的设计模式实例。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct _DesignPattern {
    pub name: String,
    /// 模式类别 (creational/structural/behavioral/idiom)。
    pub category: String,
    /// 信号来源 (code/diagram/doc)。
    pub source: String,
    /// 置信度 0..1。
    pub confidence: f32,
}

impl _DesignPattern {
    pub fn new(name: impl Into<String>, category: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            category: category.into(),
            source: "code".into(),
            confidence: 0.5,
        }
    }

    pub fn with_source(mut self, src: impl Into<String>) -> Self {
        self.source = src.into();
        self
    }

    pub fn with_confidence(mut self, c: f32) -> Self {
        self.confidence = c.clamp(0.0, 1.0);
        self
    }
}

/// 设计模式映射出的能力节点 (强化 NT-CORE 能力树)。
#[derive(Debug, Clone, PartialEq)]
pub struct CapabilityNode {
    pub node_id: String,
    pub pattern: String,
    /// 映射到的 NeoTrix 能力域 (如 nt_core_* )。
    pub target_domain: String,
    pub weight: f32,
}

/// C0 基础提取器 — 关键词启发式, 无需 AST 依赖即可编译运行。
#[derive(Default)]
pub struct _DesignExtractor {
    /// 模式关键词 → 目标 NeoTrix 域 的查找表。
    rules: HashMap<String, String>,
}

impl _DesignExtractor {
    pub fn new() -> Self {
        let mut rules = HashMap::new();
        rules.insert("factory".into(), "nt_core_capability_tree".into());
        rules.insert("builder".into(), "nt_core_capability_tree".into());
        rules.insert("observer".into(), "nt_core_event_bus".into());
        rules.insert("adapter".into(), "nt_core_capability_tree".into());
        rules.insert("strategy".into(), "nt_core_kernel".into());
        Self { rules }
    }

    pub fn extract(&self, source: &str) -> Vec<_DesignPattern> {
        let lower = source.to_lowercase();
        let mut out = Vec::new();
        for kw in ["factory", "builder", "observer", "adapter", "strategy"] {
            if lower.contains(kw) {
                let count = lower.matches(kw).count() as f32;
                let conf = (0.4 + count * 0.1).min(0.95);
                out.push(
                    _DesignPattern::new(kw, "structural")
                        .with_source("code")
                        .with_confidence(conf),
                );
            }
        }
        out
    }

    pub fn map_to_capability(&self, pattern: &_DesignPattern) -> CapabilityNode {
        let domain = self
            .rules
            .get(&pattern.name)
            .cloned()
            .unwrap_or_else(|| "nt_core_capability_tree".into());
        CapabilityNode {
            node_id: format!("cap_{}", pattern.name),
            pattern: pattern.name.clone(),
            target_domain: domain,
            weight: pattern.confidence,
        }
    }

    pub fn build_mapping(&self, patterns: &[_DesignPattern]) -> Vec<CapabilityNode> {
        let mut best: HashMap<String, _DesignPattern> = HashMap::new();
        for p in patterns {
            best.entry(p.name.clone())
                .and_modify(|e| {
                    if p.confidence > e.confidence {
                        *e = p.clone();
                    }
                })
                .or_insert_with(|| p.clone());
        }
        best.values().map(|p| self.map_to_capability(p)).collect()
    }
}

/// T1 SelfTest: 提取与映射基础不变量在 C0 可用。
#[derive(Default)]
pub struct _DesignExtractSelfTest;

impl SelfTest for _DesignExtractSelfTest {
    fn name(&self) -> &str {
        "nt_core_design_extract"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let ex = _DesignExtractor::new();
        let src = "class Factory { build() { return new Builder().observe(new Observer()) } }";
        let patterns = ex.extract(src);
        let mut errs = Vec::new();
        if patterns.is_empty() {
            errs.push("design_extract: failed to extract known patterns".into());
        }
        let mapping = ex.build_mapping(&patterns);
        if mapping.is_empty() {
            errs.push("design_extract: mapping produced no capability nodes".into());
        }
        for n in &mapping {
            if n.weight < 0.0 || n.weight > 1.0 {
                errs.push(format!("design_extract: bad weight for {}", n.node_id));
            }
        }
        if errs.is_empty() {
            Ok(())
        } else {
            Err(errs)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_finds_keyword_patterns() {
        let ex = _DesignExtractor::new();
        let ps = ex.extract("use Factory and Builder pattern");
        let names: Vec<&str> = ps.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"factory"));
        assert!(names.contains(&"builder"));
    }

    #[test]
    fn test_map_to_capability_domain_routing() {
        let ex = _DesignExtractor::new();
        let p = _DesignPattern::new("observer", "behavioral").with_confidence(0.8);
        let node = ex.map_to_capability(&p);
        assert_eq!(node.target_domain, "nt_core_event_bus");
        assert_eq!(node.weight, 0.8);
        assert_eq!(node.node_id, "cap_observer");
    }

    #[test]
    fn test_build_mapping_dedups_keeps_max_confidence() {
        let ex = _DesignExtractor::new();
        let ps = vec![
            _DesignPattern::new("factory", "structural").with_confidence(0.4),
            _DesignPattern::new("factory", "structural").with_confidence(0.9),
            _DesignPattern::new("adapter", "structural").with_confidence(0.6),
        ];
        let mapping = ex.build_mapping(&ps);
        assert_eq!(mapping.len(), 2);
        let factory = mapping.iter().find(|n| n.node_id == "cap_factory").unwrap();
        assert_eq!(factory.weight, 0.9);
    }

    #[test]
    fn test_extract_empty_when_no_pattern() {
        let ex = _DesignExtractor::new();
        let ps = ex.extract("just some plain logic with no structure");
        assert!(ps.is_empty());
    }

    #[test]
    fn test_confidence_clamped() {
        let p = _DesignPattern::new("x", "y").with_confidence(2.0);
        assert_eq!(p.confidence, 1.0);
    }
}
