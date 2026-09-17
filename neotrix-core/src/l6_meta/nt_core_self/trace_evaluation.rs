use std::collections::HashMap;
use std::time::Instant;

/// Trace-Based Agent Evaluation — 基于轨迹的 Agent 评估
///
/// 参考: GenAI_Agents "Trace-Based Agent Evaluation"
/// 核心思想: 通过分析 agent 执行轨迹来评估其行为质量，
/// 支持确定性评分、行为模式识别和性能基准测试。

/// 评估维度
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EvaluationDimension {
    /// 效率 (完成任务的步骤数/时间)
    Efficiency,
    /// 正确性 (结果是否正确)
    Correctness,
    /// 鲁棒性 (处理错误的能力)
    Robustness,
    /// 可解释性 (决策是否可解释)
    Explainability,
    /// 资源使用 (token/内存/时间)
    ResourceUsage,
}

/// 评估结果
#[derive(Debug, Clone)]
pub struct EvaluationResult {
    pub dimension: EvaluationDimension,
    pub score: f64, // 0.0 - 1.0
    pub detail: String,
    pub evidence: Vec<String>,
}

/// 评估报告
#[derive(Debug, Clone)]
pub struct EvaluationReport {
    pub agent_id: String,
    pub task_id: String,
    pub timestamp: Instant,
    pub results: Vec<EvaluationResult>,
    pub overall_score: f64,
    pub grade: EvaluationGrade,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EvaluationGrade {
    Excellent, // >= 0.9
    Good,      // >= 0.7
    Adequate,  // >= 0.5
    Poor,      // >= 0.3
    Failed,    // < 0.3
}

/// 性能指标 (用于性能监控系统集成)
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub agent_id: String,
    pub task_id: String,
    pub timestamp: Instant,
    pub metrics: HashMap<String, f64>,
    pub recommendations: Vec<String>,
}

/// 性能报告 (批量评估结果)
#[derive(Debug, Clone)]
pub struct PerformanceReport {
    pub total_traces: usize,
    pub average_score: f64,
    pub grade_distribution: HashMap<String, usize>,
    pub reports: Vec<EvaluationReport>,
}

impl EvaluationGrade {
    pub fn from_score(score: f64) -> Self {
        if score >= 0.9 { Self::Excellent }
        else if score >= 0.7 { Self::Good }
        else if score >= 0.5 { Self::Adequate }
        else if score >= 0.3 { Self::Poor }
        else { Self::Failed }
    }
}

/// Agent 执行轨迹
#[derive(Debug, Clone)]
pub struct AgentTrace {
    pub agent_id: String,
    pub task_id: String,
    pub steps: Vec<TraceStep>,
    pub start_time: Instant,
    pub end_time: Option<Instant>,
    pub final_result: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TraceStep {
    pub step_number: u32,
    pub action: String,
    pub input: String,
    pub output: String,
    pub duration_ms: u64,
    pub tokens_used: u32,
    pub success: bool,
    pub error: Option<String>,
}

/// 轨迹评估器
pub struct TraceEvaluator {
    weights: HashMap<EvaluationDimension, f64>,
    benchmarks: HashMap<String, f64>, // task_type -> baseline_score
}

impl TraceEvaluator {
    pub fn new() -> Self {
        let mut weights = HashMap::new();
        weights.insert(EvaluationDimension::Efficiency, 0.2);
        weights.insert(EvaluationDimension::Correctness, 0.3);
        weights.insert(EvaluationDimension::Robustness, 0.2);
        weights.insert(EvaluationDimension::Explainability, 0.15);
        weights.insert(EvaluationDimension::ResourceUsage, 0.15);

        Self {
            weights,
            benchmarks: HashMap::new(),
        }
    }

    /// 设置评估权重
    pub fn set_weight(&mut self, dimension: EvaluationDimension, weight: f64) {
        self.weights.insert(dimension, weight);
    }

    /// 设置基准分数
    pub fn set_benchmark(&mut self, task_type: &str, score: f64) {
        self.benchmarks.insert(task_type.to_string(), score);
    }

    /// 评估轨迹
    pub fn evaluate(&self, trace: &AgentTrace) -> EvaluationReport {
        let mut results = Vec::new();

        // 效率评估
        let efficiency = self.evaluate_efficiency(trace);
        results.push(efficiency);

        // 正确性评估
        let correctness = self.evaluate_correctness(trace);
        results.push(correctness);

        // 鲁棒性评估
        let robustness = self.evaluate_robustness(trace);
        results.push(robustness);

        // 可解释性评估
        let explainability = self.evaluate_explainability(trace);
        results.push(explainability);

        // 资源使用评估
        let resource_usage = self.evaluate_resource_usage(trace);
        results.push(resource_usage);

        // 计算总分
        let overall_score = self.compute_overall_score(&results);
        let grade = EvaluationGrade::from_score(overall_score);

        // 生成建议
        let recommendations = self.generate_recommendations(&results);

        EvaluationReport {
            agent_id: trace.agent_id.clone(),
            task_id: trace.task_id.clone(),
            timestamp: Instant::now(),
            results,
            overall_score,
            grade,
            recommendations,
        }
    }

    /// 性能监控集成: 将轨迹评估结果转换为性能指标
    ///
    /// 参考: GenAI_Agents "Trace-Based Agent Evaluation"
    /// 将轨迹评估结果转换为性能监控系统可消费的指标格式。
    pub fn to_performance_metrics(&self, report: &EvaluationReport) -> PerformanceMetrics {
        let mut metrics = HashMap::new();

        // 提取各维度分数作为性能指标
        for result in &report.results {
            metrics.insert(
                format!("{:?}", result.dimension).to_lowercase(),
                result.score,
            );
        }

        // 添加总体指标
        metrics.insert("overall_score".to_string(), report.overall_score);
        metrics.insert(
            "grade_numeric".to_string(),
            match report.grade {
                EvaluationGrade::Excellent => 1.0,
                EvaluationGrade::Good => 0.8,
                EvaluationGrade::Adequate => 0.6,
                EvaluationGrade::Poor => 0.4,
                EvaluationGrade::Failed => 0.2,
            },
        );

        PerformanceMetrics {
            agent_id: report.agent_id.clone(),
            task_id: report.task_id.clone(),
            timestamp: report.timestamp,
            metrics,
            recommendations: report.recommendations.clone(),
        }
    }

    /// 批量评估并生成性能报告
    pub fn evaluate_batch(&self, traces: &[AgentTrace]) -> PerformanceReport {
        let reports: Vec<EvaluationReport> = traces.iter()
            .map(|t| self.evaluate(t))
            .collect();

        let avg_score = if reports.is_empty() {
            0.0
        } else {
            reports.iter().map(|r| r.overall_score).sum::<f64>() / reports.len() as f64
        };

        let grade_counts = reports.iter().fold(HashMap::new(), |mut acc, r| {
            *acc.entry(format!("{:?}", r.grade)).or_insert(0) += 1;
            acc
        });

        PerformanceReport {
            total_traces: traces.len(),
            average_score: avg_score,
            grade_distribution: grade_counts,
            reports,
        }
    }

    fn evaluate_efficiency(&self, trace: &AgentTrace) -> EvaluationResult {
        let step_count = trace.steps.len() as f64;
        let total_duration: u64 = trace.steps.iter().map(|s| s.duration_ms).sum();
        
        // 效率分数: 步骤越少、时间越短越好
        let step_score = if step_count <= 5.0 { 1.0 } else { 5.0 / step_count };
        let time_score = if total_duration <= 10000 { 1.0 } else { 10000.0 / total_duration as f64 };
        let score = (step_score + time_score) / 2.0;

        EvaluationResult {
            dimension: EvaluationDimension::Efficiency,
            score,
            detail: format!("{} steps, {}ms total", step_count, total_duration),
            evidence: vec![],
        }
    }

    fn evaluate_correctness(&self, trace: &AgentTrace) -> EvaluationResult {
        let total_steps = trace.steps.len() as f64;
        let success_steps = trace.steps.iter().filter(|s| s.success).count() as f64;
        let score = if total_steps > 0.0 { success_steps / total_steps } else { 0.0 };

        EvaluationResult {
            dimension: EvaluationDimension::Correctness,
            score,
            detail: format!("{}/{} steps succeeded", success_steps, total_steps),
            evidence: trace.steps.iter().filter(|s| !s.success).map(|s| {
                format!("Step {}: {}", s.step_number, s.error.as_deref().unwrap_or("unknown"))
            }).collect(),
        }
    }

    fn evaluate_robustness(&self, trace: &AgentTrace) -> EvaluationResult {
        let error_count = trace.steps.iter().filter(|s| !s.success).count() as f64;
        let recovery_count = trace.steps.windows(2).filter(|w| {
            !w[0].success && w[1].success
        }).count() as f64;

        let score = if error_count == 0.0 {
            1.0
        } else {
            (recovery_count / error_count).min(1.0)
        };

        EvaluationResult {
            dimension: EvaluationDimension::Robustness,
            score,
            detail: format!("{} errors, {} recoveries", error_count, recovery_count),
            evidence: vec![],
        }
    }

    fn evaluate_explainability(&self, trace: &AgentTrace) -> EvaluationResult {
        // 简单启发式: 有输出的步骤比例
        let with_output = trace.steps.iter().filter(|s| !s.output.is_empty()).count() as f64;
        let total = trace.steps.len() as f64;
        let score = if total > 0.0 { with_output / total } else { 0.0 };

        EvaluationResult {
            dimension: EvaluationDimension::Explainability,
            score,
            detail: format!("{}/{} steps have output", with_output, total),
            evidence: vec![],
        }
    }

    fn evaluate_resource_usage(&self, trace: &AgentTrace) -> EvaluationResult {
        let total_tokens: u32 = trace.steps.iter().map(|s| s.tokens_used).sum();
        // 基准: 10000 tokens 为满分
        let score = if total_tokens <= 10000 {
            1.0
        } else {
            10000.0 / total_tokens as f64
        };

        EvaluationResult {
            dimension: EvaluationDimension::ResourceUsage,
            score,
            detail: format!("{} tokens total", total_tokens),
            evidence: vec![],
        }
    }

    fn compute_overall_score(&self, results: &[EvaluationResult]) -> f64 {
        let mut total_weight = 0.0;
        let mut weighted_sum = 0.0;

        for result in results {
            if let Some(&weight) = self.weights.get(&result.dimension) {
                weighted_sum += result.score * weight;
                total_weight += weight;
            }
        }

        if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            0.0
        }
    }

    fn generate_recommendations(&self, results: &[EvaluationResult]) -> Vec<String> {
        let mut recommendations = Vec::new();

        for result in results {
            if result.score < 0.5 {
                match result.dimension {
                    EvaluationDimension::Efficiency => {
                        recommendations.push("Consider reducing number of steps or parallelizing tasks".to_string());
                    }
                    EvaluationDimension::Correctness => {
                        recommendations.push("Review error handling and add validation steps".to_string());
                    }
                    EvaluationDimension::Robustness => {
                        recommendations.push("Add retry logic and error recovery mechanisms".to_string());
                    }
                    EvaluationDimension::Explainability => {
                        recommendations.push("Add more descriptive outputs and logging".to_string());
                    }
                    EvaluationDimension::ResourceUsage => {
                        recommendations.push("Optimize token usage and reduce unnecessary calls".to_string());
                    }
                }
            }
        }

        recommendations
    }
}

/// 轨迹收集器
pub struct TraceCollector {
    traces: Vec<AgentTrace>,
    max_traces: usize,
}

impl TraceCollector {
    pub fn new(max_traces: usize) -> Self {
        Self {
            traces: Vec::new(),
            max_traces,
        }
    }

    pub fn collect(&mut self, trace: AgentTrace) {
        if self.traces.len() >= self.max_traces {
            self.traces.remove(0);
        }
        self.traces.push(trace);
    }

    pub fn get_traces(&self) -> &[AgentTrace] {
        &self.traces
    }

    pub fn get_traces_for_agent(&self, agent_id: &str) -> Vec<&AgentTrace> {
        self.traces.iter().filter(|t| t.agent_id == agent_id).collect()
    }

    pub fn clear(&mut self) {
        self.traces.clear();
    }
}

/// 基准测试器
pub struct BenchmarkRunner {
    evaluator: TraceEvaluator,
    benchmarks: Vec<BenchmarkCase>,
}

pub struct BenchmarkCase {
    pub name: String,
    pub task_type: String,
    pub expected_score: f64,
    pub traces: Vec<AgentTrace>,
}

impl BenchmarkRunner {
    pub fn new(evaluator: TraceEvaluator) -> Self {
        Self {
            evaluator,
            benchmarks: Vec::new(),
        }
    }

    pub fn add_benchmark(&mut self, benchmark: BenchmarkCase) {
        self.benchmarks.push(benchmark);
    }

    pub fn run_all(&self) -> Vec<BenchmarkResult> {
        self.benchmarks.iter().map(|b| self.run_benchmark(b)).collect()
    }

    fn run_benchmark(&self, benchmark: &BenchmarkCase) -> BenchmarkResult {
        let mut scores = Vec::new();

        for trace in &benchmark.traces {
            let report = self.evaluator.evaluate(trace);
            scores.push(report.overall_score);
        }

        let avg_score = if scores.is_empty() {
            0.0
        } else {
            scores.iter().sum::<f64>() / scores.len() as f64
        };

        let passed = avg_score >= benchmark.expected_score;

        BenchmarkResult {
            name: benchmark.name.clone(),
            task_type: benchmark.task_type.clone(),
            avg_score,
            expected_score: benchmark.expected_score,
            passed,
            trial_count: benchmark.traces.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub task_type: String,
    pub avg_score: f64,
    pub expected_score: f64,
    pub passed: bool,
    pub trial_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_trace(steps: u32, success_rate: f64) -> AgentTrace {
        let steps_data = (0..steps).map(|i| {
            let success = (i as f64 / steps as f64) < success_rate;
            TraceStep {
                step_number: i,
                action: format!("action_{}", i),
                input: format!("input_{}", i),
                output: if success { format!("output_{}", i) } else { String::new() },
                duration_ms: 1000,
                tokens_used: 100,
                success,
                error: if success { None } else { Some("error".to_string()) },
            }
        }).collect();

        AgentTrace {
            agent_id: "agent_001".to_string(),
            task_id: "task_001".to_string(),
            steps: steps_data,
            start_time: Instant::now(),
            end_time: Some(Instant::now()),
            final_result: Some("completed".to_string()),
        }
    }

    #[test]
    fn test_evaluation_report() {
        let evaluator = TraceEvaluator::new();
        let trace = make_trace(10, 0.8);
        let report = evaluator.evaluate(&trace);

        assert_eq!(report.results.len(), 5);
        assert!(report.overall_score > 0.0);
        assert!(report.overall_score <= 1.0);
    }

    #[test]
    fn test_evaluation_grade() {
        assert_eq!(EvaluationGrade::from_score(0.95), EvaluationGrade::Excellent);
        assert_eq!(EvaluationGrade::from_score(0.8), EvaluationGrade::Good);
        assert_eq!(EvaluationGrade::from_score(0.6), EvaluationGrade::Adequate);
        assert_eq!(EvaluationGrade::from_score(0.4), EvaluationGrade::Poor);
        assert_eq!(EvaluationGrade::from_score(0.2), EvaluationGrade::Failed);
    }

    #[test]
    fn test_trace_collector() {
        let mut collector = TraceCollector::new(5);
        for i in 0..10 {
            let mut trace = make_trace(5, 1.0);
            trace.task_id = format!("task_{}", i);
            collector.collect(trace);
        }
        assert_eq!(collector.get_traces().len(), 5);
    }

    #[test]
    fn test_benchmark_runner() {
        let evaluator = TraceEvaluator::new();
        let mut runner = BenchmarkRunner::new(evaluator);

        let traces: Vec<AgentTrace> = (0..5).map(|_| make_trace(8, 0.9)).collect();
        runner.add_benchmark(BenchmarkCase {
            name: "basic_task".to_string(),
            task_type: "code_generation".to_string(),
            expected_score: 0.7,
            traces,
        });

        let results = runner.run_all();
        assert_eq!(results.len(), 1);
        assert!(results[0].passed);
    }
}
