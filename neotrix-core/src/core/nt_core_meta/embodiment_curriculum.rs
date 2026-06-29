#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EmbodimentTier {
    Video,
    Human,
    Robot,
}

#[derive(Debug, Clone)]
pub struct EmbodimentCurriculum {
    pub tier: EmbodimentTier,
    pub progress: f64,
    pub transfer_rate: f64,
}

impl EmbodimentCurriculum {
    pub fn new(tier: EmbodimentTier) -> Self {
        Self {
            tier,
            progress: 0.0,
            transfer_rate: 0.5,
        }
    }
    pub fn advance(&mut self, amount: f64) {
        self.progress = (self.progress + amount).min(1.0);
    }
    pub fn transfer_to(&self, new_tier: EmbodimentTier) -> EmbodimentCurriculum {
        EmbodimentCurriculum {
            tier: new_tier,
            progress: self.progress * self.transfer_rate,
            transfer_rate: self.transfer_rate,
        }
    }
}
