use log;
use std::time::{Duration, Instant};

use super::core::BrainMutView;
use super::memory::ReasoningBank;
use super::reasoning_engine::ReasoningEngine;
use super::self_iterating::ReasoningBrain;
use crate::core::default_specialist_states;
use crate::core::nt_core_self::SiliconSelfModel;
use crate::neotrix::nt_expert_routing::workspace::GlobalWorkspace;

#[derive(Debug, Clone)]
pub struct BenchPhase {
    pub label: String,
    pub durations: Vec<Duration>,
}

impl BenchPhase {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            durations: Vec::new(),
        }
    }

    pub fn record(&mut self, d: Duration) {
        self.durations.push(d);
    }

    pub fn avg_ms(&self) -> f64 {
        if self.durations.is_empty() {
            return 0.0;
        }
        let total: Duration = self.durations.iter().cloned().sum();
        total.as_secs_f64() * 1000.0 / self.durations.len() as f64
    }

    pub fn min_ms(&self) -> f64 {
        self.durations
            .iter()
            .min()
            .copied()
            .unwrap_or(Duration::ZERO)
            .as_secs_f64()
            * 1000.0
    }

    pub fn max_ms(&self) -> f64 {
        self.durations
            .iter()
            .max()
            .copied()
            .unwrap_or(Duration::ZERO)
            .as_secs_f64()
            * 1000.0
    }

    pub fn calls(&self) -> usize {
        self.durations.len()
    }
}

fn build_engine() -> ReasoningEngine {
    let brain: Box<dyn BrainMutView> = Box::new(ReasoningBrain::new());
    let bank = ReasoningBank::new(100);
    ReasoningEngine::from_env(brain, bank)
}

fn build_full_engine() -> ReasoningEngine {
    let mut engine = build_engine();
    let mut gwt = GlobalWorkspace::new(0.3);
    gwt.register_default_specialists();
    engine.gwt = Some(gwt);
    engine.silicon_self = Some(SiliconSelfModel::new());
    engine
}

pub fn bench_plan_reasoning(iterations: usize) -> Vec<BenchPhase> {
    log::warn!("TODO(GWT-MIGRATION): replace GlobalWorkspace with GlobalLatentWorkspace equivalent — calls register_default_specialists(), broadcast(), resonant_broadcast()");
    let prompts = [
        "review the code for error handling gaps in the network layer",
        "design an event-driven architecture with Kafka and microservices",
        "analyze the regression in test suite performance",
        "refactor the authentication module to support OAuth2",
        "plan the migration from REST to GraphQL API",
    ];

    let mut e8_phase = BenchPhase::new("Reasoning mode selection");
    let mut gwt_phase = BenchPhase::new("Attention broadcast");
    let mut total_phase = BenchPhase::new("plan_reasoning (total)");

    for i in 0..iterations {
        let prompt = prompts[i % prompts.len()];

        {
            let mut engine = build_engine();
            let start = Instant::now();
            let _mode = engine.select_mode(prompt);
            e8_phase.record(start.elapsed());
        }

        {
            let mut gwt = GlobalWorkspace::new(0.3);
            gwt.register_default_specialists();
            gwt.broadcast(&format!("task: {}", prompt));
            let states = default_specialist_states();
            let start = Instant::now();
            let _report = gwt.resonant_broadcast(&format!("task analysis: {}", prompt), &states);
            gwt_phase.record(start.elapsed());
        }

        {
            let mut engine = build_full_engine();
            let start = Instant::now();
            let _plan = engine.plan_reasoning(prompt);
            total_phase.record(start.elapsed());
        }
    }

    vec![e8_phase, gwt_phase, total_phase]
}

pub fn print_benchmark_table(phases: &[BenchPhase]) {
    log::info!("");
    log::info!("╭───── E8 → GWT → SelfIteration Pipeline Benchmark ────────────────╮");
    log::info!(
        "│ {:<34} {:>10} {:>10} {:>10} {:>6} │",
        "Phase",
        "Avg (ms)",
        "Min (ms)",
        "Max (ms)",
        "Calls"
    );
    log::info!("├──────────────────────────────────────────────────────────────────┤");
    for p in phases {
        log::info!(
            "│ {:<34} {:>10.3} {:>10.3} {:>10.3} {:>6} │",
            p.label,
            p.avg_ms(),
            p.min_ms(),
            p.max_ms(),
            p.calls()
        );
    }
    log::info!("╰──────────────────────────────────────────────────────────────────╯");
    log::info!("");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bench_plan_reasoning_simple() {
        let phases = bench_plan_reasoning(3);
        assert!(!phases.is_empty(), "Should produce phase results");
        let total = phases
            .iter()
            .find(|p| p.label.contains("total"))
            .expect("Should have total phase");
        let avg = total.avg_ms();
        assert!(
            avg < 5000.0,
            "Simple plan_reasoning avg {:.1}ms exceeds 5s limit",
            avg
        );
    }

    #[test]
    fn test_bench_display_results() {
        let phases = bench_plan_reasoning(5);
        print_benchmark_table(&phases);
        let total = phases
            .iter()
            .find(|p| p.label.contains("total"))
            .expect("Should have total phase");
        assert!(
            total.avg_ms() < 5000.0,
            "Total avg {:.1}ms exceeds 5s limit",
            total.avg_ms()
        );
    }

    #[test]
    fn test_bench_plan_reasoning_compound() {
        let compound_prompts = [
            "design a distributed event-sourcing system with CQRS pattern \
             using Kafka for event streaming, PostgreSQL for read models, \
             and implement sagas for distributed transaction coordination",
            "analyze the CI/CD pipeline for bottlenecks, propose a migration \
             from Jenkins to GitHub Actions with matrix builds, cache optimization, \
             and parallel job orchestration",
            "review the nt_shield architecture: identify OWASP Top 10 vulnerabilities \
             in the authentication flow, propose remediation for each, \
             and design a rate-limiting strategy with Redis",
        ];

        let mut phase = BenchPhase::new("plan_reasoning (compound)");
        for prompt in &compound_prompts {
            let mut engine = build_full_engine();
            let start = Instant::now();
            let _plan = engine.plan_reasoning(prompt);
            phase.record(start.elapsed());
        }

        let avg = phase.avg_ms();
        assert!(
            avg < 5000.0,
            "Compound plan_reasoning avg {:.1}ms exceeds 5s limit",
            avg
        );
    }
}
