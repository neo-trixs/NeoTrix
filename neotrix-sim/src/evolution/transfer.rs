use std::collections::HashMap;

pub struct Skill {
    pub name: String,
    pub effectiveness: f32,
}

pub struct TransferLearning {
    pub skill_library: Vec<Skill>,
    pub transferability: HashMap<(String, String), f32>,
}

impl TransferLearning {
    pub fn new() -> Self {
        TransferLearning {
            skill_library: Vec::new(),
            transferability: HashMap::new(),
        }
    }

    pub fn add_skill(&mut self, skill: Skill) {
        self.skill_library.push(skill);
    }

    pub fn set_transferability(&mut self, from: &str, to: &str, score: f32) {
        self.transferability.insert((from.into(), to.into()), score);
    }

    pub fn transfer_score(&self, from: &str, to: &str) -> f32 {
        self.transferability
            .get(&(from.into(), to.into()))
            .copied()
            .unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_skill_increases_library() {
        let mut tl = TransferLearning::new();
        tl.add_skill(Skill { name: "forage".into(), effectiveness: 0.8 });
        assert_eq!(tl.skill_library.len(), 1);
    }

    #[test]
    fn transfer_score_returns_stored_value() {
        let mut tl = TransferLearning::new();
        tl.set_transferability("forage", "hunt", 0.7);
        assert!((tl.transfer_score("forage", "hunt") - 0.7).abs() < 0.001);
    }

    #[test]
    fn transfer_score_defaults_to_zero() {
        let tl = TransferLearning::new();
        assert_eq!(tl.transfer_score("a", "b"), 0.0);
    }
}
