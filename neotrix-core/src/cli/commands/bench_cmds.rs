use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::nt_mind::SelfIteratingBrain;
use crate::neotrix::nt_mind::benchmark::{bench_plan_reasoning, print_benchmark_table};
use crate::neotrix::nt_io_provider::{LlmProviderType, create_provider_from_type};
use crate::neotrix::l9_transcendent_impl::nt_mind_eval_harness::{
    EvalHarness, ModelSpec, DatasetSpec, EvalQuery, DEFAULT_BUDGET_GRID,
};

pub struct BenchmarkCmd;
impl CliCommand for BenchmarkCmd {
    fn name(&self) -> &str {
        "/benchmark"
    }
    fn is_primary(&self) -> bool { false }

    fn aliases(&self) -> Vec<&str> {
        vec!["/bench"]
    }

    fn description(&self) -> &str {
        "运行 E8→GWT→SelfIteration 管线基准测试 (子命令: eval)"
    }

    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        // /bench eval [--provider NAME] [--model ID] [--budgets n,n,n] [--queries n]
        if args.iter().any(|a| a == "eval") {
            return run_eval(args);
        }

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

fn arg_value(args: &[String], flag: &str) -> Option<String> {
    args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).cloned()
}

/// /bench eval — R2-Bench 式质量-成本评测 (F5 接线, R-P79)
fn run_eval(args: &[String]) -> CommandOutput {
    let provider_name = arg_value(args, "--provider").unwrap_or_else(|| "ollama".into());
    let model_id = arg_value(args, "--model").unwrap_or_else(|| "qwen2.5:7b".into());
    let _judge_model = arg_value(args, "--judge").unwrap_or_else(|| model_id.clone());
    let queries_n = arg_value(args, "--queries").and_then(|s| s.parse::<usize>().ok()).unwrap_or(2);
    let budget_grid: Vec<u32> = arg_value(args, "--budgets")
        .map(|s| s.split(',').filter_map(|x| x.trim().parse::<u32>().ok()).collect())
        .unwrap_or_else(|| vec![DEFAULT_BUDGET_GRID[0], DEFAULT_BUDGET_GRID[2], DEFAULT_BUDGET_GRID[4]]);

    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => return CommandOutput::err(&format!("tokio runtime init failed: {e}")),
    };

    let model = ModelSpec {
        name: model_id.clone(),
        provider_type: provider_name.clone(),
        model_id: model_id.clone(),
        base_url: None,
        api_key_env: None,
        pricing_per_1m_in: 0.0,
        pricing_per_1m_out: 0.0,
    };
    let queries: Vec<EvalQuery> = (0..queries_n)
        .map(|i| EvalQuery {
            id: format!("q{i}"),
            prompt: format!("Please reason step-by-step about problem #{i}."),
            category: "reasoning".into(),
            difficulty: 0.6,
            expected_tokens: 512,
        })
        .collect();
    let dataset = DatasetSpec {
        name: "cli-dryrun".into(),
        queries,
        judge_model: model_id.clone(),
        judge_base_url: None,
        judge_api_key_env: None,
        golden_answers: None,
    };

    // 用同一 provider 作 judge (最小化依赖: 无 key 的本地端点)
    let judge_provider = create_provider_from_type(
        LlmProviderType::from_name(&provider_name).unwrap_or(LlmProviderType::Ollama),
        None,
    );

    let harness = EvalHarness::new_default(vec![model], vec![dataset], Arc::from(judge_provider), model_id.clone())
        .with_budget_grid(budget_grid);

    match rt.block_on(harness.run()) {
        Ok(reports) if !reports.is_empty() => {
            let r = &reports[0];
            let mut lines = vec![format!("Eval complete: dataset={} models={}", r.dataset_name, r.curves.len())];
            for curve in &r.curves {
                let audc = r.audc_scores.get(&curve.model_name).unwrap_or(&0.0);
                let peak = r.peak_quality.get(&curve.model_name).unwrap_or(&0.0);
                lines.push(format!("  {}: AUDC={:.3} Peak={:.3} points={}", curve.model_name, audc, peak, curve.points.len()));
            }
            lines.push(r.summary.clone());
            CommandOutput::ok(&lines.join("\n"))
        }
        Ok(_) => CommandOutput::ok("eval: no reports generated"),
        Err(e) => CommandOutput::err(&format!("eval failed: {e}")),
    }
}
