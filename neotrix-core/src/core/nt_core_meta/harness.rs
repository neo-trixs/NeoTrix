#[derive(Debug, Clone)]
pub enum ReadinessStage {
    Stage1,
    Stage2,
    Stage3,
    Stage4,
}

#[derive(Debug, Clone)]
pub struct HarnEngine {
    pub stage: ReadinessStage,
    pub rules: Vec<String>,
    pub security_sandbox: bool,
}

impl HarnEngine {
    pub fn new() -> Self {
        Self {
            stage: ReadinessStage::Stage1,
            rules: vec![],
            security_sandbox: false,
        }
    }
    pub fn advance(&mut self) {
        self.stage = match self.stage {
            ReadinessStage::Stage1 => ReadinessStage::Stage2,
            ReadinessStage::Stage2 => ReadinessStage::Stage3,
            ReadinessStage::Stage3 => ReadinessStage::Stage4,
            ReadinessStage::Stage4 => ReadinessStage::Stage4,
        };
    }
    pub fn add_rule(&mut self, rule: &str) {
        self.rules.push(rule.into());
    }
}
