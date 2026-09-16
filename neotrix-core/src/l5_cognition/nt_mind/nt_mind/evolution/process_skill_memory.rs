#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub steps: Vec<String>,
    pub success_rate: f64,
    pub uses: u32,
}

pub struct ProcessSkillMemory {
    skills: Vec<Skill>,
}

impl ProcessSkillMemory {
    pub fn new() -> Self {
        Self { skills: Vec::new() }
    }

    pub fn register(&mut self, name: &str, steps: Vec<String>) {
        self.skills.push(Skill {
            name: name.to_string(),
            steps,
            success_rate: 0.5,
            uses: 0,
        });
    }

    pub fn record(&mut self, name: &str, success: bool) {
        if let Some(s) = self.skills.iter_mut().find(|s| s.name == name) {
            s.uses += 1;
            let n = s.uses as f64;
            s.success_rate = s.success_rate * ((n - 1.0) / n) + if success { 1.0 / n } else { 0.0 };
        }
    }

    pub fn best(&self) -> Option<&Skill> {
        self.skills.iter().max_by(|a, b| {
            a.success_rate
                .partial_cmp(&b.success_rate)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn skills(&self) -> &[Skill] {
        &self.skills
    }
}

impl Default for ProcessSkillMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record() {
        let mut m = ProcessSkillMemory::new();
        m.register("deploy", vec!["build".into()]);
        m.record("deploy", true);
        m.record("deploy", true);
        assert!(m.best().unwrap().success_rate > 0.5);
    }

    #[test]
    fn test_failure() {
        let mut m = ProcessSkillMemory::new();
        m.register("s", vec![]);
        m.record("s", true);
        m.record("s", false);
        assert!((m.best().unwrap().success_rate - 0.5).abs() < 0.01);
    }
}
