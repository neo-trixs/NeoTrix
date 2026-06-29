#[derive(Debug, Clone)]
pub struct SandboxProposal {
    pub id: u64,
    pub description: String,
    pub code: String,
    pub risk_level: f64,
}

#[derive(Debug, Clone)]
pub struct SandboxResult {
    pub proposal_id: u64,
    pub passed: bool,
    pub violations: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SelfModSandbox {
    pub proposals: Vec<SandboxProposal>,
    pub results: Vec<SandboxResult>,
}

impl SelfModSandbox {
    pub fn new() -> Self {
        Self {
            proposals: vec![],
            results: vec![],
        }
    }
    pub fn submit(&mut self, desc: &str, code: &str, risk: f64) -> u64 {
        let id = self.proposals.len() as u64 + 1;
        self.proposals.push(SandboxProposal {
            id,
            description: desc.into(),
            code: code.into(),
            risk_level: risk.clamp(0.0, 1.0),
        });
        id
    }
    pub fn assess(&mut self, id: u64) -> SandboxResult {
        let passed = self
            .proposals
            .iter()
            .find(|p| p.id == id)
            .map_or(false, |p| p.risk_level < 0.7);
        let result = SandboxResult {
            proposal_id: id,
            passed,
            violations: if passed {
                vec![]
            } else {
                vec!["High risk".into()]
            },
        };
        self.results.push(result.clone());
        result
    }
}
