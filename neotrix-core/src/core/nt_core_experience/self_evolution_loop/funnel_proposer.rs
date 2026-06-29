// G413 meta-wiring: PopulationFunnel as SelfEvolutionLoop proposer strategy.
// Replaces single Thompson-sample with parallel N-candidate funnel competition.
use super::{MutationOp, SelfEvolutionStep};
use crate::core::nt_core_experience::population_funnel::{
    FunnelConfig, PopulationFunnel, ProposalVariant,
};
use crate::core::nt_core_hcube::VsaVector;

/// Predict gain for a mutation before execution (smoke-screen scoring).
/// Higher = more likely to improve fitness.
fn predict_mutation_gain(op: &MutationOp) -> f64 {
    match op {
        MutationOp::TuneParam { delta, .. } => {
            let abs_delta = delta.abs();
            if abs_delta < 0.01 {
                0.1
            } else if abs_delta < 0.1 {
                0.4
            } else {
                0.6
            }
        }
        MutationOp::RewriteHandler { .. } => 0.5,
        MutationOp::SwapPolicy { .. } => 0.3,
        MutationOp::RewritePrimitive { .. } => 0.4,
        MutationOp::RewriteMeta { .. } => 0.7,
        MutationOp::SelfModifyProposal { .. } => 0.2,
        MutationOp::AddHandler { .. } => 0.45,
    }
}

/// Describe a mutation op for human-readable proposal labels.
fn describe_mutation(op: &MutationOp) -> String {
    match op {
        MutationOp::TuneParam { target, delta } => {
            format!("Tune {} by {:.4}", target, delta)
        }
        MutationOp::RewriteHandler { name, .. } => {
            format!("Rewrite handler {}", name)
        }
        MutationOp::SwapPolicy { gates } => {
            format!("Swap policy to {:?}", gates)
        }
        MutationOp::RewritePrimitive { name, .. } => {
            format!("Rewrite primitive {}", name)
        }
        MutationOp::RewriteMeta { .. } => "Rewrite meta-strategy".into(),
        MutationOp::SelfModifyProposal { target, .. } => {
            format!("Self-modify {}", target)
        }
        MutationOp::AddHandler { position, .. } => {
            format!("Add handler at {}", position)
        }
    }
}

/// Generate N candidate mutations by sampling from the same distribution
/// that `propose_via_rust` uses, but with varying RNG seeds.
pub fn generate_funnel_candidates(
    best_steps: &[SelfEvolutionStep],
    n_candidates: usize,
    rng_seed: u64,
) -> Vec<MutationOp> {
    let mut candidates = Vec::with_capacity(n_candidates);

    // Strategy pool mirroring propose_via_rust arm distribution
    let strategies = [
        "explore",
        "exploit",
        "repair",
        "innovate",
        "harden",
        "prune",
        "socialize",
    ];

    for i in 0..n_candidates {
        let seed = rng_seed.wrapping_add(i as u64);
        let strategy = strategies[i % strategies.len()];
        let op = sample_mutation_for_strategy(strategy, best_steps, seed);
        candidates.push(op);
    }
    candidates
}

/// Sample one mutation op for a given drive strategy (simplified from core.rs).
fn sample_mutation_for_strategy(
    strategy: &str,
    best_steps: &[SelfEvolutionStep],
    seed: u64,
) -> MutationOp {
    use super::core::{crossover_ops, mutate_op_from, random_tune_param};
    use rand::Rng;
    use rand::SeedableRng;
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

    match strategy {
        "explore" => {
            if rng.gen_bool(0.6) {
                random_tune_param(rng.gen(), rng.gen())
            } else if !best_steps.is_empty() {
                let idx = (rng.gen::<f64>() * best_steps.len() as f64) as usize;
                mutate_op_from(&best_steps[idx].mutation, rng.gen(), rng.gen())
            } else {
                random_tune_param(rng.gen(), rng.gen())
            }
        }
        "exploit" => {
            if !best_steps.is_empty() && rng.gen_bool(0.7) {
                let idx = (rng.gen::<f64>() * best_steps.len() as f64) as usize;
                mutate_op_from(&best_steps[idx].mutation, rng.gen(), rng.gen())
            } else if best_steps.len() >= 2 {
                let a_idx = (rng.gen::<f64>() * best_steps.len() as f64) as usize;
                let mut b_idx = (rng.gen::<f64>() * best_steps.len() as f64) as usize;
                while b_idx == a_idx && best_steps.len() > 1 {
                    b_idx = (rng.gen::<f64>() * best_steps.len() as f64) as usize;
                }
                crossover_ops(
                    &best_steps[a_idx].mutation,
                    &best_steps[b_idx].mutation,
                    rng.gen(),
                )
            } else if !best_steps.is_empty() {
                let idx = (rng.gen::<f64>() * best_steps.len() as f64) as usize;
                mutate_op_from(&best_steps[idx].mutation, rng.gen(), rng.gen())
            } else {
                random_tune_param(rng.gen(), rng.gen())
            }
        }
        "repair" => {
            if rng.gen_bool(0.8) && !best_steps.is_empty() {
                let idx = (rng.gen::<f64>() * best_steps.len() as f64) as usize;
                match &best_steps[idx].mutation {
                    MutationOp::RewriteHandler { name, code } => MutationOp::RewriteHandler {
                        name: name.clone(),
                        code: format!("{} // repaired", code),
                    },
                    other => mutate_op_from(other, rng.gen(), rng.gen()),
                }
            } else {
                random_tune_param(rng.gen(), rng.gen())
            }
        }
        "innovate" => {
            if best_steps.len() >= 2 && rng.gen_bool(0.5) {
                let a_idx = (rng.gen::<f64>() * best_steps.len() as f64) as usize;
                let mut b_idx = (rng.gen::<f64>() * best_steps.len() as f64) as usize;
                while b_idx == a_idx && best_steps.len() > 1 {
                    b_idx = (rng.gen::<f64>() * best_steps.len() as f64) as usize;
                }
                crossover_ops(
                    &best_steps[a_idx].mutation,
                    &best_steps[b_idx].mutation,
                    rng.gen(),
                )
            } else if !best_steps.is_empty() && rng.gen_bool(0.5) {
                let idx = (rng.gen::<f64>() * best_steps.len() as f64) as usize;
                mutate_op_from(&best_steps[idx].mutation, rng.gen(), rng.gen())
            } else {
                random_tune_param(rng.gen(), rng.gen())
            }
        }
        "harden" => {
            if rng.gen_bool(0.7) {
                let delta = (rng.gen::<f64>() - 0.5) * 0.05;
                let targets = [
                    "cognitive_load.thinking_budget",
                    "emergent_reasoning.exploration_rate",
                    "inner_critic.relevance_threshold",
                ];
                let idx = (rng.gen::<f64>() * targets.len() as f64) as usize;
                MutationOp::TuneParam {
                    target: targets[idx].to_string(),
                    delta,
                }
            } else if !best_steps.is_empty() {
                let idx = (rng.gen::<f64>() * best_steps.len() as f64) as usize;
                mutate_op_from(&best_steps[idx].mutation, rng.gen(), rng.gen())
            } else {
                random_tune_param(rng.gen(), rng.gen())
            }
        }
        _ => random_tune_param(rng.gen(), rng.gen()),
    }
}

/// Propose a mutation via PopulationFunnel: generate N candidates, score,
/// iterate survivors, return best.
pub fn propose_via_funnel(best_steps: &[SelfEvolutionStep], rng_seed: u64) -> MutationOp {
    let mut funnel = PopulationFunnel::new(FunnelConfig {
        initial_population: 6,
        top_k_survivors: 3,
        max_iterations: 3,
        plateau_window: 2,
        plateau_threshold: 0.03,
        smoke_screen_gates: vec!["syntax".into(), "semantic".into()],
    });

    // Round 0: generate N candidates
    let candidates = generate_funnel_candidates(best_steps, 6, rng_seed);

    if candidates.is_empty() {
        // Fallback: random tune param
        return MutationOp::TuneParam {
            target: "curiosity_drive.learning_rate".into(),
            delta: (rng_seed as f64 / u64::MAX as f64 - 0.5) * 0.1,
        };
    }

    // Seed funnel with initial variants (score via predict_mutation_gain)
    let variants: Vec<ProposalVariant> = candidates
        .iter()
        .enumerate()
        .map(|(i, op)| ProposalVariant {
            id: i as u64,
            description: describe_mutation(op),
            score: predict_mutation_gain(op),
            gate_results: Vec::new(),
            vsa_signature: VsaVector::random(rng_seed.wrapping_add(i as u64)),
            iteration: 0,
            parent_id: None,
        })
        .collect();

    funnel.seed_population(variants);

    // Run funnel iterations
    let mut round = 0;
    while !funnel.should_stop() {
        let passed = funnel.run_smoke_screen(round);
        let survivors = funnel.select_survivors(&passed);
        funnel.survivors = survivors;
        funnel.record_round(round, funnel.survivors.clone());
        round += 1;
    }

    // Return best candidate's mutation
    let best_idx = funnel.best_variant().map(|v| v.id as usize).unwrap_or(0);
    let total = candidates.len().saturating_sub(1);

    candidates
        .into_iter()
        .nth(best_idx.min(total))
        .unwrap_or_else(|| MutationOp::TuneParam {
            target: "curiosity_drive.learning_rate".into(),
            delta: 0.01,
        })
}
