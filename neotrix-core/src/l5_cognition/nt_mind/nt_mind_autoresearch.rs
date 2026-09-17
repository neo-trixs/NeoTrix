#![forbid(unsafe_code)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutoresearchPhase { Propose, Change, Experiment, Inspect, Decide }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoresearchState {
    pub phase: AutoresearchPhase,
    pub iteration: u64,
    pub hypothesis: String,
}

pub struct AutoresearchLoop { state: AutoresearchState }

impl AutoresearchLoop {
    pub fn new() -> Self { Self { state: AutoresearchState { phase: AutoresearchPhase::Propose, iteration: 0, hypothesis: String::new() } } }
    pub fn advance(&mut self) -> &AutoresearchPhase {
        self.state.phase = match self.state.phase {
            AutoresearchPhase::Propose => AutoresearchPhase::Change,
            AutoresearchPhase::Change => AutoresearchPhase::Experiment,
            AutoresearchPhase::Experiment => AutoresearchPhase::Inspect,
            AutoresearchPhase::Inspect => AutoresearchPhase::Decide,
            AutoresearchPhase::Decide => { self.state.iteration += 1; AutoresearchPhase::Propose }
        };
        &self.state.phase
    }
    pub fn state(&self) -> &AutoresearchState { &self.state }
}
