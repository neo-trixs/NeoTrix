use crate::core::nt_core_hcube::e8_cortical::CORTICAL_NEURON_COUNT;

/// Resonance mode for cortical computation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResonanceMode {
    Theta,
    Gamma,
    SharpWave,
}

impl Default for ResonanceMode {
    fn default() -> Self {
        Self::Theta
    }
}

/// Cortical resonator: binds VSA vectors through oscillatory resonance
#[derive(Debug, Clone)]
pub struct CortexAdaptive {
    pub resonance: ResonanceMode,
    pub frequency: f64,
    pub phase: f64,
}

impl Default for CortexAdaptive {
    fn default() -> Self {
        Self {
            resonance: ResonanceMode::default(),
            frequency: 4.0,
            phase: 0.0,
        }
    }
}

/// Cerebellum-inspired resonator for temporal VSA binding
#[derive(Debug, Clone)]
pub struct CerebellumResonator {
    pub tick_rate: f64,
    pub memory: Vec<f64>,
}

impl Default for CerebellumResonator {
    fn default() -> Self {
        Self {
            tick_rate: 10.0,
            memory: vec![0.0; 8],
        }
    }
}

/// Cortical bidirectional RNN for sequence VSA processing
#[derive(Debug, Clone)]
pub struct CBRNN {
    pub hidden: Vec<f64>,
    pub forward: Vec<f64>,
    pub backward: Vec<f64>,
}

impl Default for CBRNN {
    fn default() -> Self {
        Self {
            hidden: vec![0.0; CORTICAL_NEURON_COUNT.min(64)],
            forward: vec![0.0; CORTICAL_NEURON_COUNT.min(64)],
            backward: vec![0.0; CORTICAL_NEURON_COUNT.min(64)],
        }
    }
}
