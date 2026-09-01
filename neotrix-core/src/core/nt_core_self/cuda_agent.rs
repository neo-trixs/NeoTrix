use std::collections::HashMap;
use std::time::Instant;

/// CUDA Agent RL — 技能增强开发环境
///
/// 参考: arXiv:2602.24286 "CUDA Agent: Large-Scale Agentic RL for High-Performance CUDA Kernel Generation"
/// 核心思想: 技能增强的开发环境 + 自动验证/分析 → RL 奖励信号,
/// 支持代码优化、性能分析和技能学习。

/// 代码优化任务
#[derive(Debug, Clone)]
pub struct OptimizationTask {
    pub id: String,
    pub name: String,
    pub description: String,
    pub code: String,
    pub language: String,
    pub metrics: HashMap<String, f64>,
    pub constraints: Vec<OptimizationConstraint>,
    pub status: TaskStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

/// 优化约束
#[derive(Debug, Clone)]
pub struct OptimizationConstraint {
    pub name: String,
    pub constraint_type: ConstraintType,
    pub value: f64,
    pub unit: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintType {
    MaxLatency,
    MinThroughput,
    MaxMemoryUsage,
    MaxPowerConsumption,
    MinAccuracy,
}

/// 优化结果
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub task_id: String,
    pub original_metrics: HashMap<String, f64>,
    pub optimized_metrics: HashMap<String, f64>,
    pub improvement: HashMap<String, f64>,
    pub optimized_code: String,
    pub strategies_used: Vec<String>,
    pub reward: f64,
}

/// 优化策略
#[derive(Debug, Clone)]
pub struct OptimizationStrategy {
    pub id: String,
    pub name: String,
    pub description: String,
    pub applicable_languages: Vec<String>,
    pub expected_improvement: f64,
    pub success_rate: f64,
    pub usage_count: u32,
}

/// 性能分析器
pub struct PerformanceAnalyzer {
    benchmarks: HashMap<String, BenchmarkResult>,
    history: Vec<OptimizationResult>,
}

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub metrics: HashMap<String, f64>,
    pub timestamp: Instant,
}

impl PerformanceAnalyzer {
    pub fn new() -> Self {
        Self {
            benchmarks: HashMap::new(),
            history: Vec::new(),
        }
    }

    /// 运行基准测试
    pub fn run_benchmark(&mut self, name: &str, _code: &str, _language: &str) -> BenchmarkResult {
        // 模拟性能分析
        let mut metrics = HashMap::new();
        metrics.insert("execution_time_improvement".to_string(), 0.1);
        metrics.insert("memory_improvement".to_string(), 0.1);
        metrics.insert("throughput_improvement".to_string(), 0.1);
        metrics.insert("accuracy_maintained".to_string(), 0.95);

        let result = BenchmarkResult {
            name: name.to_string(),
            metrics,
            timestamp: Instant::now(),
        };

        self.benchmarks.insert(name.to_string(), result.clone());
        result
    }

    /// 比较两个版本的性能
    pub fn compare(&self, before: &str, after: &str) -> HashMap<String, f64> {
        let mut improvement = HashMap::new();

        if let (Some(b), Some(a)) = (self.benchmarks.get(before), self.benchmarks.get(after)) {
            for (key, &a_val) in &a.metrics {
                if let Some(&b_val) = b.metrics.get(key) {
                    if b_val > 0.0 {
                        let imp = (a_val - b_val) / b_val;
                        improvement.insert(key.clone(), imp);
                    }
                }
            }
        }

        improvement
    }

    /// 记录优化结果
    pub fn record_result(&mut self, result: OptimizationResult) {
        self.history.push(result);
    }

    /// 获取历史结果
    pub fn get_history(&self) -> &[OptimizationResult] {
        &self.history
    }

    /// Performance monitoring 集成: 将基准测试结果转换为性能指标
    ///
    /// 参考: GenAI_Agents "Trace-Based Agent Evaluation"
    /// 将基准测试结果转换为性能监控系统可消费的指标格式。
    pub fn to_performance_metrics(&self, benchmark: &BenchmarkResult) -> PerformanceMetrics {
        let mut metrics = HashMap::new();

        // 提取基准测试指标
        for (key, value) in &benchmark.metrics {
            metrics.insert(key.clone(), *value);
        }

        // 添加时间戳
        metrics.insert(
            "timestamp".to_string(),
            benchmark.timestamp.elapsed().as_secs_f64(),
        );

        // 添加历史统计
        if !self.history.is_empty() {
            let avg_improvement: f64 = self.history.iter()
                .map(|r| r.improvement.values().sum::<f64>() / r.improvement.len() as f64)
                .sum::<f64>() / self.history.len() as f64;
            metrics.insert("avg_improvement".to_string(), avg_improvement);
        }

        PerformanceMetrics {
            benchmark_name: benchmark.name.clone(),
            metrics,
            recommendations: self.generate_recommendations(benchmark),
        }
    }

    /// 生成性能优化建议
    fn generate_recommendations(&self, benchmark: &BenchmarkResult) -> Vec<String> {
        let mut recommendations = Vec::new();

        if let Some(&execution_time) = benchmark.metrics.get("execution_time_improvement") {
            if execution_time < 0.0 {
                recommendations.push("Consider optimizing execution time".to_string());
            }
        }

        if let Some(&memory) = benchmark.metrics.get("memory_improvement") {
            if memory < 0.0 {
                recommendations.push("Consider optimizing memory usage".to_string());
            }
        }

        if let Some(&throughput) = benchmark.metrics.get("throughput_improvement") {
            if throughput < 0.0 {
                recommendations.push("Consider optimizing throughput".to_string());
            }
        }

        recommendations
    }
}

/// 性能指标 (用于性能监控系统集成)
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub benchmark_name: String,
    pub metrics: HashMap<String, f64>,
    pub recommendations: Vec<String>,
}

/// 策略管理器
pub struct StrategyManager {
    strategies: Vec<OptimizationStrategy>,
    effectiveness: HashMap<String, f64>,
}

impl StrategyManager {
    pub fn new() -> Self {
        Self {
            strategies: Vec::new(),
            effectiveness: HashMap::new(),
        }
    }

    /// 注册策略
    pub fn register(&mut self, strategy: OptimizationStrategy) {
        self.strategies.push(strategy);
    }

    /// 选择最佳策略
    pub fn select_strategy(&self, language: &str, _target_metric: &str) -> Option<&OptimizationStrategy> {
        self.strategies.iter()
            .filter(|s| s.applicable_languages.contains(&language.to_string()))
            .max_by(|a, b| {
                let a_score = a.expected_improvement * a.success_rate;
                let b_score = b.expected_improvement * b.success_rate;
                a_score.partial_cmp(&b_score).unwrap()
            })
    }

    /// 更新策略有效性
    pub fn update_effectiveness(&mut self, strategy_id: &str, reward: f64) {
        let entry = self.effectiveness.entry(strategy_id.to_string()).or_insert(0.0);
        *entry = (*entry * 0.9) + (reward * 0.1); // 指数移动平均
    }

    /// 获取策略排名
    pub fn rank_strategies(&self) -> Vec<(&str, f64)> {
        let mut ranked: Vec<_> = self.effectiveness.iter()
            .map(|(k, v)| (k.as_str(), *v))
            .collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        ranked
    }
}

/// RL 奖励计算器
pub struct RewardCalculator {
    weights: HashMap<String, f64>,
}

impl RewardCalculator {
    pub fn new() -> Self {
        let mut weights = HashMap::new();
        weights.insert("execution_time_improvement".to_string(), 0.3);
        weights.insert("memory_improvement".to_string(), 0.2);
        weights.insert("throughput_improvement".to_string(), 0.3);
        weights.insert("accuracy_maintained".to_string(), 0.2);

        Self { weights }
    }

    /// 计算奖励
    pub fn calculate(&self, improvement: &HashMap<String, f64>) -> f64 {
        let mut total_reward = 0.0;

        for (key, &weight) in &self.weights {
            if let Some(&imp) = improvement.get(key) {
                total_reward += imp * weight;
            }
        }

        total_reward.max(-1.0).min(1.0) // 限制在 [-1, 1]
    }
}

/// CUDA Agent 开发环境
pub struct CudaAgentEnvironment {
    analyzer: PerformanceAnalyzer,
    strategies: StrategyManager,
    reward_calculator: RewardCalculator,
    tasks: Vec<OptimizationTask>,
}

impl CudaAgentEnvironment {
    pub fn new() -> Self {
        Self {
            analyzer: PerformanceAnalyzer::new(),
            strategies: StrategyManager::new(),
            reward_calculator: RewardCalculator::new(),
            tasks: Vec::new(),
        }
    }

    /// 提交优化任务
    pub fn submit_task(&mut self, task: OptimizationTask) -> String {
        let id = task.id.clone();
        self.tasks.push(task);
        id
    }

    /// 执行优化
    pub fn optimize(&mut self, task_id: &str) -> Option<OptimizationResult> {
        let task = self.tasks.iter_mut().find(|t| t.id == task_id)?;
        task.status = TaskStatus::InProgress;

        // 运行基准测试
        let original = self.analyzer.run_benchmark(
            &format!("{}_original", task_id),
            &task.code,
            &task.language,
        );

        // 选择策略
        let strategy_name = self.strategies.select_strategy(&task.language, "execution_time")
            .map(|s| s.name.clone());

        // 模拟优化 (实际应执行策略)
        let mut optimized_metrics = original.metrics.clone();
        if strategy_name.is_some() {
            for value in optimized_metrics.values_mut() {
                *value *= 1.1; // 模拟 10% 改进
            }
        }

        // 计算改进
        let mut improvement = HashMap::new();
        for (key, &opt_val) in &optimized_metrics {
            if let Some(&orig_val) = original.metrics.get(key) {
                if orig_val > 0.0 {
                    improvement.insert(key.clone(), (opt_val - orig_val) / orig_val);
                }
            }
        }

        // 计算奖励
        let reward = self.reward_calculator.calculate(&improvement);

        let result = OptimizationResult {
            task_id: task_id.to_string(),
            original_metrics: original.metrics.clone(),
            optimized_metrics,
            improvement,
            optimized_code: task.code.clone(), // 模拟
            strategies_used: strategy_name.map(|n| vec![n]).unwrap_or_default(),
            reward,
        };

        // 记录结果
        self.analyzer.record_result(result.clone());
        task.status = TaskStatus::Completed;

        Some(result)
    }

    /// 获取任务列表
    pub fn get_tasks(&self) -> &[OptimizationTask] {
        &self.tasks
    }

    /// 获取性能分析器
    pub fn get_analyzer(&self) -> &PerformanceAnalyzer {
        &self.analyzer
    }

    /// 获取策略管理器
    pub fn get_strategies(&self) -> &StrategyManager {
        &self.strategies
    }

    /// 获取策略管理器 (可变)
    pub fn get_strategies_mut(&mut self) -> &mut StrategyManager {
        &mut self.strategies
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_task(id: &str) -> OptimizationTask {
        OptimizationTask {
            id: id.to_string(),
            name: format!("Task {}", id),
            description: "Test optimization".to_string(),
            code: "def optimize(): pass".to_string(),
            language: "python".to_string(),
            metrics: HashMap::new(),
            constraints: vec![],
            status: TaskStatus::Pending,
        }
    }

    fn make_strategy(id: &str) -> OptimizationStrategy {
        OptimizationStrategy {
            id: id.to_string(),
            name: format!("Strategy {}", id),
            description: "Test strategy".to_string(),
            applicable_languages: vec!["python".to_string()],
            expected_improvement: 0.15,
            success_rate: 0.8,
            usage_count: 0,
        }
    }

    #[test]
    fn test_benchmark() {
        let mut analyzer = PerformanceAnalyzer::new();
        let result = analyzer.run_benchmark("test", "code", "python");
        assert!(result.metrics.contains_key("execution_time_improvement"));
    }

    #[test]
    fn test_strategy_selection() {
        let mut manager = StrategyManager::new();
        manager.register(make_strategy("s1"));
        manager.register(make_strategy("s2"));

        let selected = manager.select_strategy("python", "execution_time");
        assert!(selected.is_some());
    }

    #[test]
    fn test_reward_calculation() {
        let calculator = RewardCalculator::new();
        let mut improvement = HashMap::new();
        improvement.insert("execution_time_improvement".to_string(), 0.2);
        improvement.insert("throughput_improvement".to_string(), 0.3);

        let reward = calculator.calculate(&improvement);
        assert!(reward > 0.0);
        assert!(reward <= 1.0);
    }

    #[test]
    fn test_optimization_cycle() {
        let mut env = CudaAgentEnvironment::new();
        // 注册策略
        env.get_strategies_mut().register(make_strategy("s1"));
        let task_id = env.submit_task(make_task("t1"));
        let result = env.optimize(&task_id);
        assert!(result.is_some());
        assert!(result.unwrap().reward > 0.0);
    }
}
