#[derive(Debug, Clone)]
pub struct ProcessSkill {
    pub name: String,
    pub steps: Vec<String>,
    pub success_count: u32,
    pub fail_count: u32,
    pub avg_duration_ms: u64,
}

pub struct ProcessSkillMemory {
    skills: Vec<ProcessSkill>,
}

impl ProcessSkillMemory {
    pub fn new() -> Self {
        Self { skills: Vec::new() }
    }

    pub fn register(&mut self, name: &str, steps: Vec<String>) {
        self.skills.push(ProcessSkill {
            name: name.to_string(),
            steps,
            success_count: 0,
            fail_count: 0,
            avg_duration_ms: 0,
        });
    }

    pub fn record_success(&mut self, name: &str, duration_ms: u64) {
        if let Some(s) = self.skills.iter_mut().find(|s| s.name == name) {
            s.success_count += 1;
            let n = s.success_count as f64;
            s.avg_duration_ms =
                (s.avg_duration_ms as f64 * ((n - 1.0) / n) + duration_ms as f64 / n) as u64;
        }
    }

    pub fn record_failure(&mut self, name: &str) {
        if let Some(s) = self.skills.iter_mut().find(|s| s.name == name) {
            s.fail_count += 1;
        }
    }

    pub fn success_rate(&self, name: &str) -> f64 {
        self.skills
            .iter()
            .find(|s| s.name == name)
            .map_or(0.0, |s| {
                let total = s.success_count + s.fail_count;
                if total == 0 {
                    0.0
                } else {
                    s.success_count as f64 / total as f64
                }
            })
    }

    pub fn best_skill(&self) -> Option<&ProcessSkill> {
        self.skills.iter().max_by(|a, b| {
            a.success_rate()
                .partial_cmp(&b.success_rate())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    pub fn skills(&self) -> &[ProcessSkill] {
        &self.skills
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_record() {
        let mut m = ProcessSkillMemory::new();
        m.register("deploy", vec!["build".into(), "test".into()]);
        m.record_success("deploy", 5000);
        m.record_success("deploy", 3000);
        assert_eq!(m.success_rate("deploy"), 1.0);
    }

    #[test]
    fn test_failure() {
        let mut m = ProcessSkillMemory::new();
        m.register("s", vec![]);
        m.record_success("s", 100);
        m.record_failure("s");
        assert_eq!(m.success_rate("s"), 0.5);
    }
}
