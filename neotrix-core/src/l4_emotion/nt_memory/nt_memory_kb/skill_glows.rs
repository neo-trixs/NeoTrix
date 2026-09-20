#[derive(Debug, Clone)]
pub struct SkillGlow {
    pub skill_name: String,
    pub trigger_pattern: String,
    pub glow_strength: f64,
    pub activation_count: u32,
    pub last_activated: u64,
}

pub struct SkillGlowManager {
    skills: Vec<SkillGlow>,
    decay_rate: f64,
}

impl SkillGlowManager {
    pub fn new(decay_rate: f64) -> Self {
        Self {
            skills: Vec::new(),
            decay_rate,
        }
    }

    pub fn register_skill(&mut self, name: &str, pattern: &str) {
        self.skills.push(SkillGlow {
            skill_name: name.to_string(),
            trigger_pattern: pattern.to_string(),
            glow_strength: 0.5,
            activation_count: 0,
            last_activated: 0,
        });
    }

    pub fn activate(&mut self, name: &str) -> bool {
        if let Some(s) = self.skills.iter_mut().find(|s| s.skill_name == name) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            s.glow_strength = (s.glow_strength + 0.1).min(1.0);
            s.activation_count += 1;
            s.last_activated = now;
            true
        } else {
            false
        }
    }

    pub fn decay(&mut self) {
        for s in &mut self.skills {
            s.glow_strength = (s.glow_strength - self.decay_rate).max(0.0);
        }
    }

    pub fn query(&self, pattern: &str) -> Vec<&SkillGlow> {
        self.skills
            .iter()
            .filter(|s| s.trigger_pattern.contains(pattern) || pattern.contains(&s.trigger_pattern))
            .collect()
    }

    pub fn strongest(&self) -> Option<&SkillGlow> {
        self.skills.iter().max_by(|a, b| {
            a.glow_strength
                .partial_cmp(&b.glow_strength)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }
    pub fn skills(&self) -> &[SkillGlow] {
        &self.skills
    }
}
impl Default for SkillGlowManager {
    fn default() -> Self {
        Self::new(0.01)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_register_and_activate() {
        let mut m = SkillGlowManager::new(0.01);
        m.register_skill("rust", "ownership");
        assert!(m.activate("rust"));
        assert_eq!(m.skills[0].activation_count, 1);
    }
    #[test]
    fn test_decay() {
        let mut m = SkillGlowManager::new(0.5);
        m.register_skill("s", "p");
        m.activate("s");
        m.decay();
        assert!(m.skills[0].glow_strength < 0.6);
    }
    #[test]
    fn test_query() {
        let mut m = SkillGlowManager::new(0.01);
        m.register_skill("rust", "ownership");
        m.register_skill("python", "gil");
        let r = m.query("ownership");
        assert_eq!(r.len(), 1);
    }
}
