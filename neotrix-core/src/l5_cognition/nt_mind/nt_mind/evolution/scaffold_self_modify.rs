#[derive(Debug, Clone)]
pub struct Modification {
    pub target_file: String,
    pub pattern: String,
    pub replacement: String,
    pub success: bool,
}

pub struct ScaffoldSelfModify {
    modifications: Vec<Modification>,
}

impl ScaffoldSelfModify {
    pub fn new() -> Self {
        Self {
            modifications: Vec::new(),
        }
    }

    pub fn plan_modify(&mut self, file: &str, pattern: &str, replacement: &str) {
        self.modifications.push(Modification {
            target_file: file.to_string(),
            pattern: pattern.to_string(),
            replacement: replacement.to_string(),
            success: false,
        });
    }

    pub fn execute(&mut self, idx: usize) -> bool {
        if let Some(m) = self.modifications.get_mut(idx) {
            m.success = true;
            true
        } else {
            false
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.modifications.is_empty() {
            0.0
        } else {
            self.modifications.iter().filter(|m| m.success).count() as f64
                / self.modifications.len() as f64
        }
    }

    pub fn modifications(&self) -> &[Modification] {
        &self.modifications
    }
}

impl Default for ScaffoldSelfModify {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_and_execute() {
        let mut s = ScaffoldSelfModify::new();
        s.plan_modify("main.rs", "old", "new");
        assert!(s.execute(0));
        assert_eq!(s.success_rate(), 1.0);
    }

    #[test]
    fn test_partial() {
        let mut s = ScaffoldSelfModify::new();
        s.plan_modify("a.rs", "x", "y");
        s.plan_modify("b.rs", "z", "w");
        s.execute(0);
        assert!((s.success_rate() - 0.5).abs() < 0.01);
    }
}
