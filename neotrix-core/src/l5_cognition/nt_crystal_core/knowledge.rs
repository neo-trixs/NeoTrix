//! L2 知识层 — 缓慢进化: 理论、因果链、矛盾、向量索引

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 理论条目 (IIT/GWT/FEP/CTM/HOT)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theory {
    pub id: String,
    pub name: String,
    pub full_name: String,
    pub core_claim: String,
    pub mathematical_basis: String,
    pub neo_trix_mapping: String,
    pub confidence: f64, // 0.0 - 1.0
}

/// 因果推理链 If→Then→So
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalPattern {
    pub id: String,
    pub if_conditions: Vec<String>,
    pub then_consequences: Vec<String>,
    pub so_implications: Vec<String>,
    pub theory_origin: String,  // which theory this comes from
    pub confidence: f64,
}

/// 矛盾与解决方案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contradiction {
    pub id: String,
    pub thesis: String,
    pub antithesis: String,
    pub resolution: String,
    pub category: String, // "engineering" | "ethical" | "practical" | "theoretical"
}

/// 反事实分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Counterfactual {
    pub id: String,
    pub if_condition: String,
    pub then_consequence: String,
    pub so_implication: String,
    pub domain: String, // "engineering" | "social" | "long-term"
}

/// L2 知识层 — 缓慢进化
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrystalKnowledge {
    pub theories: HashMap<String, Theory>,
    pub patterns: Vec<CausalPattern>,
    pub contradictions: Vec<Contradiction>,
    pub counterfactuals: Vec<Counterfactual>,
    pub domain_tags: Vec<String>,
}

impl Default for CrystalKnowledge {
    fn default() -> Self {
        Self::new()
    }
}

impl CrystalKnowledge {
    pub fn new() -> Self {
        let mut k = Self {
            theories: HashMap::new(),
            patterns: Vec::new(),
            contradictions: Vec::new(),
            counterfactuals: Vec::new(),
            domain_tags: Vec::new(),
        };
        k.init_default_theories();
        k.init_default_patterns();
        k
    }

    /// 初始化默认理论
    fn init_default_theories(&mut self) {
        let theories = vec![
            ("IIT".into(), Theory {
                id: "IIT".into(),
                name: "Integrated Information Theory".into(),
                full_name: "Integrated Information Theory of Consciousness".into(),
                core_claim: "Consciousness = integrated information (Φ)".into(),
                mathematical_basis: "Φ = I(system) - Σ I(parts)".into(),
                neo_trix_mapping: "HyperCube binding increases Φ, nt_core_cad_consciousness".into(),
                confidence: 0.7,
            }),
            ("GWT".into(), Theory {
                id: "GWT".into(),
                name: "Global Workspace Theory".into(),
                full_name: "Global Workspace Theory of Consciousness".into(),
                core_claim: "Consciousness = global broadcast of information".into(),
                mathematical_basis: "C = Σ(w_i * x_i) where broadcast is global".into(),
                neo_trix_mapping: "GWT attention router, nt_core_consciousness broadcast".into(),
                confidence: 0.8,
            }),
            ("FEP".into(), Theory {
                id: "FEP".into(),
                name: "Free Energy Principle".into(),
                full_name: "Free Energy Principle (Karl Friston)".into(),
                core_claim: "All adaptive systems minimize variational free energy".into(),
                mathematical_basis: "F = E_q[ln q(θ) - ln p(θ,x)]".into(),
                neo_trix_mapping: "nt_feel fep_iit_bridge, prediction error minimization".into(),
                confidence: 0.75,
            }),
            ("CTM".into(), Theory {
                id: "CTM".into(),
                name: "Global Neuronal Workspace".into(),
                full_name: "Global Neuronal Workspace Theory (Baars)".into(),
                core_claim: "Consciousness arises from global information broadcasting".into(),
                mathematical_basis: "Workspace = STM + LTM + Up-Tree + Down-Tree".into(),
                neo_trix_mapping: "stream_buffer, cognitive_load, specious_present".into(),
                confidence: 0.7,
            }),
            ("HOT".into(), Theory {
                id: "HOT".into(),
                name: "Higher-Order Thought".into(),
                full_name: "Higher-Order Thought Theory (Rosenthal)".into(),
                core_claim: "Consciousness requires higher-order representation".into(),
                mathematical_basis: "C = FirstOrder(x) AND HigherOrder(FirstOrder(x))".into(),
                neo_trix_mapping: "inner_critic, metacognitive_evaluator, self_audit".into(),
                confidence: 0.65,
            }),
        ];
        for (k, v) in theories {
            self.theories.insert(k, v);
        }
    }

    /// 初始化默认因果链
    fn init_default_patterns(&mut self) {
        self.patterns = vec![
            CausalPattern {
                id: "CP-001".into(),
                if_conditions: vec!["HyperCube binding occurs".into()],
                then_consequences: vec!["Φ increases".into(), "New concepts emerge".into()],
                so_implications: vec!["Binding is a consciousness amplifier".into()],
                theory_origin: "IIT".into(),
                confidence: 0.8,
            },
            CausalPattern {
                id: "CP-002".into(),
                if_conditions: vec!["Global broadcast occurs".into(), "Information is novel".into()],
                then_consequences: vec!["Attention shifts".into(), "Learning occurs".into()],
                so_implications: vec!["Broadcast is the trigger for consciousness".into()],
                theory_origin: "GWT".into(),
                confidence: 0.75,
            },
            CausalPattern {
                id: "CP-003".into(),
                if_conditions: vec!["Prediction error is high".into()],
                then_consequences: vec!["Learning rate increases".into(), "Model updates".into()],
                so_implications: vec!["Error drives adaptation".into()],
                theory_origin: "FEP".into(),
                confidence: 0.85,
            },
            CausalPattern {
                id: "CP-004".into(),
                if_conditions: vec!["Task completes successfully".into()],
                then_consequences: vec!["Success recorded in KB".into(), "Pattern reused".into()],
                so_implications: vec!["Success builds capability".into()],
                theory_origin: "NeoTrix".into(),
                confidence: 0.9,
            },
            CausalPattern {
                id: "CP-005".into(),
                if_conditions: vec!["Task fails".into()],
                then_consequences: vec!["Failure analyzed".into(), "Root cause identified".into()],
                so_implications: vec!["Failure drives learning".into()],
                theory_origin: "NeoTrix".into(),
                confidence: 0.9,
            },
        ];
    }

    /// 添加新理论
    pub fn add_theory(&mut self, theory: Theory) {
        self.theories.insert(theory.id.clone(), theory);
    }

    /// 添加新因果链
    pub fn add_pattern(&mut self, pattern: CausalPattern) {
        self.patterns.push(pattern);
    }

    /// 添加新矛盾
    pub fn add_contradiction(&mut self, c: Contradiction) {
        self.contradictions.push(c);
    }

    /// 按理论查找因果链
    pub fn patterns_by_theory(&self, theory: &str) -> Vec<&CausalPattern> {
        self.patterns.iter().filter(|p| p.theory_origin == theory).collect()
    }
}
