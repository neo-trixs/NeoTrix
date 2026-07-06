use serde::{Deserialize, Serialize};

/// The 8 cognitive epochs of human civilization's Earth-perception.
///
/// Each epoch represents a distinct ontological framework — a way of
/// experiencing, modeling, and interacting with the world. The system
/// evolves by switching between these frameworks, not by optimizing
/// within a single one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EarthEpoch {
    /// E1: Mythological — 万物有灵, 循环时间, 仪式因果
    /// Era of animism, cyclical time, causal reasoning through narrative
    E1Mythological,
    /// E2: Agricultural — 中心化宇宙, 天圆地方, 等级秩序
    /// Centralized cosmos, celestial-earth correspondence, hierarchical order
    E2Agricultural,
    /// E3: Axial — 三大轴心文明独立认知框架并存
    /// Three axial-age civilizations' independent cognitive frameworks coexisting
    E3Axial,
    /// E4: Scientific — 客观测量, 数学定律, 还原论
    /// Objective measurement, mathematical laws, reductionism
    E4Scientific,
    /// E5: Global/Industrial — 系统思维, 全球尺度, 资源优化
    /// Systems thinking, global scale, resource optimization
    E5Global,
    /// E6: Planetary — 从外部看地球, 封闭系统, 自调节
    /// Viewing Earth from outside, closed system, self-regulation (Gaia)
    E6Planetary,
    /// E7: Network/Informational — 数据流, 拓扑连通, 涌现
    /// Data flow, network topology, emergent computation
    E7Network,
    /// E8: Emergent/AI — 自我修改, 元认知, 碳硅共生
    /// Self-modification, meta-cognition, carbon-silicon symbiosis
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

    /// The human-era epoch this framework emerged from.
    /// Used to set default activation precedence.
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

/// Describes a single dimension within a cognitive framework's ontology.
/// Each epoch has its own set of dimensions — they are NOT shared.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionDef {
    pub name: String,
    pub description: String,
}

/// A complete cognitive framework tied to one EarthEpoch.
///
/// Each framework has:
/// - An **ontology** (named dimensions that define what is "real" in this epoch)
/// - A **state vector** over those dimensions (the system's capability in this mode)
/// - A **router weight** distribution (when to switch to this framework)
/// - An activation history for reinforcement-based absorption
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

    /// Update state vector toward `target` at `learning_rate`.
    /// This is the epoch-specific version of `CapabilityVector::update_from_other`.
    pub fn update_from(&mut self, target: &[f64], learning_rate: f64) {
        let len = self.state.len().min(target.len());
        for (i, item) in self.state.iter_mut().enumerate().take(len) {
            *item += learning_rate * (target[i] - *item);
        }
    }

    /// Normalize state so max value is at most 1.0
    pub fn normalize(&mut self) {
        let max_val = self.state.iter().cloned().fold(0.0f64, |a, x| a.max(x));
        if max_val > 1.0 {
            let scale = 1.0 / max_val;
            self.state.iter_mut().for_each(|x| *x *= scale);
        }
    }

    /// Record one activation and accumulate reward signal
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

    /// The effective "weight" of this framework for routing decisions.
    /// Combines static router_bias with dynamic average_reward.
    pub fn effective_weight(&self) -> f64 {
        0.7 * self.router_bias + 0.3 * self.average_reward()
    }
}

/// Describes the routing decision for a task: which framework(s) to use.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkRoute {
    pub primary: EarthEpoch,
    pub weights: Vec<(EarthEpoch, f64)>,
}

/// Records one activation event for tracking framework usage patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivationRecord {
    pub epoch: EarthEpoch,
    pub task_label: String,
    pub reward: f64,
    pub timestamp: u64,
}
