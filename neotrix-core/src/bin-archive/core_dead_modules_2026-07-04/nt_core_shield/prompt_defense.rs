pub struct DefenseConfig {
    pub max_risk_score: f64,
    pub enabled_layers: Vec<DefenseLayer>,
}

pub enum DefenseLayer {
    Static,
    Semantic,
    Encoding,
    Dynamic,
}

pub struct DefenseReport {
    pub threat: Option<ThreatSignal>,
    pub score: f64,
    pub layers_triggered: Vec<DefenseLayer>,
}

pub struct DynamicSafetyInjector {
    pub patterns: Vec<String>,
}

pub struct EncodingAnomalyDetector {
    pub threshold: f64,
}

pub struct PromptDefenseOrchestrator {
    pub config: DefenseConfig,
}

pub struct SafetyTemplate {
    pub name: String,
    pub content: String,
}

pub struct SemanticInjectionAnalyzer {
    pub sensitivity: f64,
}

pub struct StaticInjectionDetector {
    pub patterns: Vec<String>,
}

pub struct ThreatSignal {
    pub category: String,
    pub confidence: f64,
}

pub struct TrigramAnalyzer {
    pub window_size: usize,
}

pub enum Verdict {
    Safe,
    Suspicious(f64),
    Blocked(String),
}
