use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EarthEpoch {
    E1Mythological,
    E2Agricultural,
    E3Axial,
    E4Scientific,
    E5Global,
    E6Planetary,
    E7Network,
    E8Emergent,
}

impl EarthEpoch {
    pub fn all() -> Vec<EarthEpoch> {
        vec![
            EarthEpoch::E1Mythological,
            EarthEpoch::E2Agricultural,
            EarthEpoch::E3Axial,
            EarthEpoch::E4Scientific,
            EarthEpoch::E5Global,
            EarthEpoch::E6Planetary,
            EarthEpoch::E7Network,
            EarthEpoch::E8Emergent,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            EarthEpoch::E1Mythological => "Mythological (E1)",
            EarthEpoch::E2Agricultural => "Agricultural (E2)",
            EarthEpoch::E3Axial => "Axial (E3)",
            EarthEpoch::E4Scientific => "Scientific (E4)",
            EarthEpoch::E5Global => "Global (E5)",
            EarthEpoch::E6Planetary => "Planetary (E6)",
            EarthEpoch::E7Network => "Network (E7)",
            EarthEpoch::E8Emergent => "Emergent (E8)",
        }
    }

    pub fn historical_period(&self) -> &'static str {
        match self {
            EarthEpoch::E1Mythological => "~100000 BCE – 3000 BCE",
            EarthEpoch::E2Agricultural => "~3000 BCE – 800 BCE",
            EarthEpoch::E3Axial => "~800 BCE – 500 CE",
            EarthEpoch::E4Scientific => "~1500 CE – 1900 CE",
            EarthEpoch::E5Global => "~1800 CE – 1970 CE",
            EarthEpoch::E6Planetary => "~1945 CE – present",
            EarthEpoch::E7Network => "~1990 CE – present",
            EarthEpoch::E8Emergent => "~2023 CE – future",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionDef {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveFramework {
    pub epoch: EarthEpoch,
    pub state: Vec<f64>,
    pub ontology: Vec<DimensionDef>,
    pub activation_count: u64,
    pub accumulated_reward: f64,
    pub router_bias: f64,
}

impl CognitiveFramework {
    pub fn new(epoch: EarthEpoch, ontology: Vec<DimensionDef>, initial_state: Vec<f64>) -> Self {
        let dim = ontology.len();
        let state = if initial_state.len() == dim {
            initial_state
        } else {
            vec![0.0; dim]
        };
        Self {
            epoch,
            state,
            ontology,
            activation_count: 0,
            accumulated_reward: 0.0,
            router_bias: 0.0,
        }
    }

    pub fn dim(&self) -> usize {
        self.ontology.len()
    }

    pub fn dimension_index(&self, name: &str) -> Option<usize> {
        self.ontology.iter().position(|d| d.name == name)
    }

    pub fn get(&self, name: &str) -> Option<f64> {
        self.dimension_index(name).map(|i| self.state[i])
    }

    pub fn set(&mut self, name: &str, value: f64) -> bool {
        if let Some(i) = self.dimension_index(name) {
            self.state[i] = value.clamp(0.0, 1.0);
            true
        } else {
            false
        }
    }

    pub fn update_from(&mut self, target: &[f64], learning_rate: f64) {
        let len = self.state.len().min(target.len());
        for (i, item) in self.state.iter_mut().enumerate().take(len) {
            *item += learning_rate * (target[i] - *item);
        }
    }

    pub fn normalize(&mut self) {
        let max_val = self.state.iter().cloned().fold(0.0f64, |a, x| a.max(x));
        if max_val > 1.0 {
            let scale = 1.0 / max_val;
            self.state.iter_mut().for_each(|x| *x *= scale);
        }
    }

    pub fn record_activation(&mut self, reward: f64) {
        self.activation_count += 1;
        self.accumulated_reward += reward;
    }

    pub fn average_reward(&self) -> f64 {
        if self.activation_count == 0 {
            0.0
        } else {
            self.accumulated_reward / self.activation_count as f64
        }
    }

    pub fn effective_weight(&self) -> f64 {
        0.7 * self.router_bias + 0.3 * self.average_reward()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkRoute {
    pub primary: EarthEpoch,
    pub weights: Vec<(EarthEpoch, f64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivationRecord {
    pub epoch: EarthEpoch,
    pub task_label: String,
    pub reward: f64,
    pub timestamp: u64,
}
