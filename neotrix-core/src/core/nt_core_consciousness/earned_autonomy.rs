#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum AutonomyLevel {
    L1,
    L2,
    L3,
    L4,
    L5,
    L6,
    L7,
}

#[derive(Debug, Clone)]
pub struct EarnedAutonomy {
    pub level: AutonomyLevel,
    pub competence: f64,
    pub history: Vec<String>,
}

impl EarnedAutonomy {
    pub fn new() -> Self {
        Self {
            level: AutonomyLevel::L1,
            competence: 0.0,
            history: vec![],
        }
    }
    pub fn record_success(&mut self, task: &str) {
        self.competence = (self.competence + 0.1).min(1.0);
        self.history.push(format!("+{}", task));
        self.update_level();
    }
    fn update_level(&mut self) {
        self.level = if self.competence > 0.9 {
            AutonomyLevel::L7
        } else if self.competence > 0.75 {
            AutonomyLevel::L6
        } else if self.competence > 0.6 {
            AutonomyLevel::L5
        } else if self.competence > 0.45 {
            AutonomyLevel::L4
        } else if self.competence > 0.3 {
            AutonomyLevel::L3
        } else if self.competence > 0.15 {
            AutonomyLevel::L2
        } else {
            AutonomyLevel::L1
        };
    }
}
