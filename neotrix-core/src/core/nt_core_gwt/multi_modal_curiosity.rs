#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CuriosityModality {
    Sensory,
    Motor,
    Cognitive,
    Social,
}

#[derive(Debug, Clone)]
pub struct ModalityCuriosity {
    pub modality: CuriosityModality,
    pub signal: f64,
    pub decay_rate: f64,
    pub baseline: f64,
}

impl ModalityCuriosity {
    pub fn new(modality: CuriosityModality, baseline: f64) -> Self {
        Self {
            modality,
            signal: baseline,
            decay_rate: 0.05,
            baseline,
        }
    }

    pub fn tick(&mut self) {
        self.signal += self.decay_rate * (self.baseline - self.signal);
    }

    pub fn boost(&mut self, amount: f64) {
        self.signal = (self.signal + amount).clamp(0.0, 1.0);
    }
}

#[derive(Debug, Clone)]
pub struct MultiModalCuriosity {
    pub modalities: Vec<ModalityCuriosity>,
    pub cross_synergy: f64,
}

impl MultiModalCuriosity {
    pub fn new() -> Self {
        Self {
            modalities: vec![
                ModalityCuriosity::new(CuriosityModality::Sensory, 0.3),
                ModalityCuriosity::new(CuriosityModality::Motor, 0.2),
                ModalityCuriosity::new(CuriosityModality::Cognitive, 0.5),
                ModalityCuriosity::new(CuriosityModality::Social, 0.4),
            ],
            cross_synergy: 0.1,
        }
    }

    pub fn tick_all(&mut self) {
        for m in &mut self.modalities {
            m.tick();
        }
    }

    pub fn boost_modality(&mut self, modality: CuriosityModality, amount: f64) {
        for m in &mut self.modalities {
            if m.modality == modality {
                m.boost(amount);
            } else {
                m.boost(amount * self.cross_synergy);
            }
        }
    }

    pub fn dominant(&self) -> CuriosityModality {
        self.modalities
            .iter()
            .max_by(|a, b| a.signal.partial_cmp(&b.signal).unwrap_or(std::cmp::Ordering::Equal))
            .map(|m| m.modality)
            .unwrap_or(CuriosityModality::Cognitive)
    }

    pub fn total_curiosity(&self) -> f64 {
        self.modalities.iter().map(|m| m.signal).sum::<f64>() / self.modalities.len() as f64
    }
}

impl Default for MultiModalCuriosity {
    fn default() -> Self {
        Self::new()
    }
}
