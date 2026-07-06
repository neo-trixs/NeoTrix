use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_mind::SelfIteratingBrain;
use crate::neotrix::nt_mind::benchmark::{bench_plan_reasoning, print_benchmark_table};

pub struct BenchmarkCmd;
impl CliCommand for BenchmarkCmd {
    fn name(&self) -> &str {
        "/benchmark"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["/bench"]
    }

    fn description(&self) -> &str {
        "运行 E8→GWT→SelfIteration 管线基准测试"
    }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let iterations = args.iter()
            .position(|a| a == "--iterations" || a == "-n")
            .and_then(|i| args.get(i + 1))
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(10);

        let results = bench_plan_reasoning(iterations);
        print_benchmark_table(&results);

        CommandOutput::ok(&format!("Benchmark complete: {} iterations", iterations))
    }
}
