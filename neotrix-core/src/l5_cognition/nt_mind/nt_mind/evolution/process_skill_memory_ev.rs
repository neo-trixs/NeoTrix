#[derive(Debug, Clone)]
pub struct SkillRecord {
    pub name: String,
    pub success: bool,
    pub duration_ms: u64,
}

pub struct ProcessSkillMemoryEv {
    records: Vec<SkillRecord>,
}

impl ProcessSkillMemoryEv {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn record(&mut self, name: &str, success: bool, duration_ms: u64) {
        self.records.push(SkillRecord {
            name: name.to_string(),
            success,
            duration_ms,
        });
    }

    pub fn success_rate(&self, name: &str) -> f64 {
        let relevant: Vec<_> = self.records.iter().filter(|r| r.name == name).collect();
        if relevant.is_empty() {
            0.0
        } else {
            relevant.iter().filter(|r| r.success).count() as f64 / relevant.len() as f64
        }
    }

    pub fn avg_duration(&self, name: &str) -> u64 {
        let relevant: Vec<_> = self.records.iter().filter(|r| r.name == name).collect();
        if relevant.is_empty() {
            0
        } else {
            relevant.iter().map(|r| r.duration_ms).sum::<u64>() / relevant.len() as u64
        }
    }

    pub fn count(&self) -> usize {
        self.records.len()
    }
}

impl Default for ProcessSkillMemoryEv {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record() {
        let mut m = ProcessSkillMemoryEv::new();
        m.record("deploy", true, 1000);
        m.record("deploy", true, 2000);
        assert_eq!(m.success_rate("deploy"), 1.0);
        assert_eq!(m.avg_duration("deploy"), 1500);
    }

    #[test]
    fn test_failure() {
        let mut m = ProcessSkillMemoryEv::new();
        m.record("s", true, 100);
        m.record("s", false, 200);
        assert_eq!(m.success_rate("s"), 0.5);
    }
}
