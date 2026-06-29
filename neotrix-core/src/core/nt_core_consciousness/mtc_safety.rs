#[derive(Debug, Clone)]
pub struct MtcSafety {
    pub safety_gate_active: bool,
    pub check_results: Vec<String>,
}

impl MtcSafety {
    pub fn new() -> Self {
        Self {
            safety_gate_active: true,
            check_results: vec![],
        }
    }
    pub fn check(&mut self, action: &str) -> bool {
        let safe = !action.contains("dangerous");
        self.check_results.push(format!(
            "{}: {}",
            action,
            if safe { "PASS" } else { "FAIL" }
        ));
        safe
    }
}
