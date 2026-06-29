#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EvolutionMode {
    Fix,
    Derived,
    Captured,
}

#[derive(Debug, Clone)]
pub struct SkillEvolutionModes {
    pub mode: EvolutionMode,
    pub token_reduction: f64,
}

impl SkillEvolutionModes {
    pub fn new(mode: EvolutionMode) -> Self {
        Self {
            mode,
            token_reduction: 0.0,
        }
    }
    pub fn evolve(&self, context: &str) -> String {
        match self.mode {
            EvolutionMode::Fix => format!("fix: {}", context),
            EvolutionMode::Derived => format!("derived from: {}", context),
            EvolutionMode::Captured => format!("captured: {}", context),
        }
    }
}
