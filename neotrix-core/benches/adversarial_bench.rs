use criterion::{criterion_group, criterion_main, Criterion};

use neotrix::core::nt_core_experience::{AdversarialArena, ArenaConfig};

fn bench_adversarial_evolution(c: &mut Criterion) {
    c.bench_function("adversarial_evolution_100_generations", |b| {
        b.iter(|| {
            let mut arena = AdversarialArena::new(ArenaConfig {
                population_size: 20,
                tournament_size: 3,
                mutation_rate: 0.2,
                crossover_rate: 0.3,
                mutation_sigma: 0.1,
                elite_count: 2,
            });
            arena.seed_population(
                &[
                    "reasoning_depth",
                    "exploration_rate",
                    "risk_tolerance",
                    "memory_retention",
                    "curiosity_strength",
                ],
                0.3,
            );
            for round in 1..=100 {
                let _gen = arena.run_round("bench", &mut |agent| {
                    agent.traits.values().sum::<f64>() / agent.traits.len() as f64
                });
                arena.evolve(round);
            }
        })
    });
}

criterion_group!(benches, bench_adversarial_evolution);
criterion_main!(benches);
