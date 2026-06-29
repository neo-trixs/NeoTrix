#[derive(Debug, Clone)]
pub struct DgmhEdit {
    pub target: String,
    pub old_value: f64,
    pub new_value: f64,
    pub gate: String,
    pub reason: String,
}
impl DgmhEdit {
    pub fn new(target: &str, old_value: f64, new_value: f64, gate: &str, reason: &str) -> Self {
        Self {
            target: target.to_string(),
            old_value,
            new_value,
            gate: gate.to_string(),
            reason: reason.to_string(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MutationRecord {
    pub handler: String,
    pub action: String,
    pub cycle: u64,
    pub pre_success_rate: f64,
    pub post_success_rate: Option<f64>,
    pub outcome: String,
}
