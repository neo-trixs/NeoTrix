//! Implements core::nt_core_traits::ConsciousnessHandle for ConsciousnessIntegration.
//! This file exists in the neotrix layer to break the circular core↔neotrix import.

use super::ConsciousnessIntegration;
use crate::core::nt_core_traits::ConsciousnessHandle;

impl ConsciousnessHandle for ConsciousnessIntegration {
    fn apply_ne_edit(&mut self, target: &str, value: f64) -> String {
        self.apply_ne_edit(target, value)
    }

    fn stats_c_score(&self) -> f64 {
        self.stats().c_score
    }

    fn cognitive_load(&self) -> f64 {
        self.cognitive_load
    }

    fn self_evolution_best_score(&self) -> f64 {
        match self.self_evolution {
            Some(ref evo) => evo.archive.best_score,
            None => 0.0,
        }
    }

    fn eval_ne_string(&mut self, expr: &str) -> Result<String, String> {
        match self.ne_evaluator {
            Some(ref mut ev) => ev.eval_string(expr).map(|v| v.to_string()),
            None => Err("no ne_evaluator available".to_string()),
        }
    }

    fn set_self_evolution_archive(&mut self, best_score: f64) {
        if let Some(ref mut evo) = self.self_evolution {
            evo.archive.best_score = best_score;
        }
    }
}
