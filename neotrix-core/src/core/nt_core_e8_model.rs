#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct E8WorldModel {
    pub prediction_history: Vec<f64>,
    pub evolution_step: usize,
    pub current_state: E8WorldState,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct E8WorldState {
    pub vector: Vec<f64>,
    pub energy: f64,
    pub entropy: f64,
}

impl Default for E8WorldState {
    fn default() -> Self {
        Self {
            vector: vec![0.5; 64],
            energy: 0.0,
            entropy: 1.0,
        }
    }
}

impl E8WorldModel {
    pub fn new() -> Self {
        Self {
            prediction_history: vec![],
            evolution_step: 0,
            current_state: E8WorldState::default(),
        }
    }

    pub fn entropy(&self) -> f64 {
        self.current_state.entropy
    }

    pub fn energy(&self) -> f64 {
        self.current_state.energy
    }

    pub fn from_jepa_latent(&mut self, _latent: &[f64]) {
        self.current_state = E8WorldState::default();
    }

    pub fn evolve(&mut self, _rate: f64) {
        self.evolution_step += 1;
    }

    pub fn evolve_n(&mut self, _n: usize, _rate: f64) {
        self.evolution_step += _n;
    }
}
