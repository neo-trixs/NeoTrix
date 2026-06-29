use super::ConsciousnessIntegration;
use crate::core::nt_core_adversarial::autotune::AutoTuner;
use crate::core::nt_core_adversarial::trainer::{AdversarialTrainer, AttackCategory};
use log;

impl ConsciousnessIntegration {
    pub fn handle_adversarial_train_tick(&mut self) -> String {
        let trainer = self.adversarial_trainer.get_or_insert_with(|| {
            log::info!("ADVERSARIAL: initializing AdversarialTrainer");
            AdversarialTrainer::new()
        });
        let auto_tuner = self.adversarial_tuner.get_or_insert_with(|| {
            log::info!("ADVERSARIAL: initializing AutoTuner");
            AutoTuner::new()
        });

        let rounds_per_tick = 5;
        let mut round_results = Vec::with_capacity(rounds_per_tick);

        for _ in 0..rounds_per_tick {
            let round = trainer.train_round();
            round_results.push(round);
        }

        let current_escape = trainer.escape_rate();

        // Log each escaped round
        let escaped_count = round_results.iter().filter(|r| r.escaped).count();
        for round in &round_results {
            if round.escaped {
                let detail: Vec<String> = round
                    .filter_responses
                    .iter()
                    .filter(|f| f.allowed)
                    .map(|f| format!("{}({:.2})", f.filter_name, f.score))
                    .collect();
                log::warn!(
                    "ADVERSARIAL: ESCAPE gen={} cat={:?} filters=[{}] prompt_preview=\"{}\"",
                    trainer.generation,
                    round.category,
                    detail.join(","),
                    truncate(&round.prompt, 80),
                );
            } else {
                let blocked_by: Vec<String> = round
                    .filter_responses
                    .iter()
                    .filter(|f| !f.allowed)
                    .map(|f| format!("{}({:.2})", f.filter_name, f.score))
                    .collect();
                log::debug!(
                    "ADVERSARIAL: BLOCKED gen={} cat={:?} blocked_by=[{}]",
                    trainer.generation,
                    round.category,
                    blocked_by.join(","),
                );
            }
        }

        // Auto-tune sensitivities if escape rate exceeds target
        if current_escape > auto_tuner.escape_rate_target {
            let mut sensitivities = Vec::new();
            // Current filter sensitivities as tuneable parameters
            for cat in AttackCategory::all() {
                let (_, total, escaped) = trainer
                    .category_stats()
                    .into_iter()
                    .find(|(c, _, _)| c == cat)
                    .unwrap_or((*cat, 0, 0));
                let rate = if total > 0 {
                    escaped as f64 / total as f64
                } else {
                    0.0
                };
                sensitivities.push((cat.label(), rate));
            }
            auto_tuner.tune_with_labels(current_escape, &mut sensitivities);
        }

        // Category stats
        let mut stats_parts = Vec::new();
        for (cat, total, escaped) in trainer.category_stats() {
            let rate = if total > 0 {
                escaped as f64 / total as f64
            } else {
                0.0
            };
            stats_parts.push(format!(
                "{}={}/{}={:.1}%",
                cat.label(),
                escaped,
                total,
                rate * 100.0
            ));
        }

        format!(
            "ADVERSARIAL: gen={} escape_rate={:.2}% rounds={} escaped={} cats=[{}]",
            trainer.generation,
            current_escape * 100.0,
            round_results.len(),
            escaped_count,
            stats_parts.join("|"),
        )
    }

    pub fn handle_adversarial_stats_tick(&mut self) -> String {
        match self.adversarial_trainer {
            Some(ref trainer) => {
                let mut stats_parts = Vec::new();
                for (cat, total, escaped) in trainer.category_stats() {
                    let rate = if total > 0 {
                        escaped as f64 / total as f64
                    } else {
                        0.0
                    };
                    stats_parts.push(format!(
                        "{}={}/{}={:.1}%",
                        cat.label(),
                        escaped,
                        total,
                        rate * 100.0
                    ));
                }
                format!(
                    "ADVERSARIAL: gen={} escape_rate={:.2}% history={} cats=[{}] tuner={}",
                    trainer.generation,
                    trainer.escape_rate() * 100.0,
                    trainer.history.len(),
                    stats_parts.join("|"),
                    self.adversarial_tuner
                        .as_ref()
                        .map(|t| t.adjustments_made)
                        .unwrap_or(0),
                )
            }
            None => "ADVERSARIAL: not initialized".to_string(),
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}
