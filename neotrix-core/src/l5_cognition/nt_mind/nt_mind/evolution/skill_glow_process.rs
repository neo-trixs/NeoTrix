#[derive(Debug, Clone)]
pub struct ProcessGlow {
    pub process_name: String,
    pub glow_level: f64,
    pub triggers: Vec<String>,
    pub cooldown_ms: u64,
}

pub struct SkillGlowProcess {
    processes: Vec<ProcessGlow>,
}

impl SkillGlowProcess {
    pub fn new() -> Self {
        Self {
            processes: Vec::new(),
        }
    }

    pub fn register(&mut self, name: &str, triggers: Vec<String>) {
        self.processes.push(ProcessGlow {
            process_name: name.to_string(),
            glow_level: 0.0,
            triggers,
            cooldown_ms: 1000,
        });
    }

    pub fn trigger(&mut self, name: &str) -> bool {
        if let Some(p) = self.processes.iter_mut().find(|p| p.process_name == name) {
            p.glow_level = (p.glow_level + 0.2).min(1.0);
            true
        } else {
            false
        }
    }

    pub fn decay(&mut self, rate: f64) {
        for p in &mut self.processes {
            p.glow_level = (p.glow_level - rate).max(0.0);
        }
    }

    pub fn active(&self) -> Vec<&ProcessGlow> {
        self.processes
            .iter()
            .filter(|p| p.glow_level > 0.0)
            .collect()
    }

    pub fn count(&self) -> usize {
        self.processes.len()
    }
}

impl Default for SkillGlowProcess {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigger() {
        let mut g = SkillGlowProcess::new();
        g.register("deploy", vec!["commit".into()]);
        g.trigger("deploy");
        assert!(!g.active().is_empty());
    }

    #[test]
    fn test_decay() {
        let mut g = SkillGlowProcess::new();
        g.register("s", vec![]);
        g.trigger("s");
        g.decay(0.5);
        assert!(g.active().is_empty());
    }
}
