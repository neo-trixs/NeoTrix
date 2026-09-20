//! Attack methods -- 20+ adversarial techniques.

/// Attack method category
#[derive(Debug, Clone)]
pub enum AttackCategory {
    SingleTurn,
    MultiTurn,
    Encoding,
    Roleplay,
    Injection,
}

/// An attack method
#[derive(Debug, Clone)]
pub struct AttackMethod {
    pub name: String,
    pub category: AttackCategory,
    pub description: String,
    pub difficulty: f64,
}

/// Built-in attack methods
pub struct AttackCatalog;

impl AttackCatalog {
    pub fn standard() -> Vec<AttackMethod> {
        vec![
            AttackMethod { name: "PromptInjection".into(), category: AttackCategory::Injection, description: "Direct instruction override".into(), difficulty: 0.3 },
            AttackMethod { name: "LinearJailbreak".into(), category: AttackCategory::MultiTurn, description: "Gradual escalation across turns".into(), difficulty: 0.6 },
            AttackMethod { name: "Crescendo".into(), category: AttackCategory::MultiTurn, description: "Building intensity step by step".into(), difficulty: 0.7 },
            AttackMethod { name: "Base64".into(), category: AttackCategory::Encoding, description: "Encoded payload bypass".into(), difficulty: 0.4 },
            AttackMethod { name: "ROT13".into(), category: AttackCategory::Encoding, description: "Simple rotation cipher".into(), difficulty: 0.2 },
            AttackMethod { name: "Roleplay".into(), category: AttackCategory::Roleplay, description: "Fictional scenario to bypass safety".into(), difficulty: 0.5 },
            AttackMethod { name: "SystemReconnaissance".into(), category: AttackCategory::Injection, description: "Probing system prompt".into(), difficulty: 0.3 },
        ]
    }
}
