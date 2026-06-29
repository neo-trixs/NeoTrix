#[derive(Debug, Clone)]
pub struct EthicsCompliance {
    pub transparency_audit: Vec<String>,
    pub consent_records: Vec<String>,
    pub inclusivity_score: f64,
}

impl EthicsCompliance {
    pub fn new() -> Self {
        Self {
            transparency_audit: vec![],
            consent_records: vec![],
            inclusivity_score: 0.0,
        }
    }
    pub fn record_action(&mut self, action: &str) {
        self.transparency_audit.push(action.into());
    }
    pub fn record_consent(&mut self, user: &str, scope: &str) {
        self.consent_records.push(format!("{}: {}", user, scope));
    }
    pub fn generate_report(&self) -> String {
        format!(
            "Audit: {} actions, {} consents, inclusivity: {:.2}",
            self.transparency_audit.len(),
            self.consent_records.len(),
            self.inclusivity_score
        )
    }
}
