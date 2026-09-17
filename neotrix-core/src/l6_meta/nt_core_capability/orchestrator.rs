//! 能力编排引擎
//!
//! 支持复杂的多能力编排流程

use crate::core::nt_core_capability::*;
use std::sync::Arc;

/// 编排模式
#[derive(Debug, Clone)]
pub enum OrchestrationMode {
    /// 顺序执行
    Sequential,
    /// 并行执行
    Parallel,
    /// 条件执行
    Conditional,
    /// 循环执行
    Loop,
    /// 分支执行
    Branching,
}

/// 编排步骤
#[derive(Debug, Clone)]
pub struct OrchestrationStep {
    /// 步骤ID
    pub id: String,
    /// 步骤名称
    pub name: String,
    /// 能力ID
    pub capability_id: String,
    /// 输入映射
    pub input_mapping: String,
    /// 输出映射
    pub output_mapping: String,
    /// 条件表达式
    pub condition: Option<String>,
    /// 重试次数
    pub max_retries: u32,
    /// 超时时间
    pub timeout_ms: u64,
}

/// 编排流程
#[derive(Debug, Clone)]
pub struct OrchestrationFlow {
    /// 流程ID
    pub id: String,
    /// 流程名称
    pub name: String,
    /// 编排模式
    pub mode: OrchestrationMode,
    /// 步骤列表
    pub steps: Vec<OrchestrationStep>,
    /// 全局超时
    pub global_timeout_ms: u64,
    /// 最大并行度
    pub max_parallelism: u32,
}

/// 编排上下文
#[derive(Debug, Clone)]
pub struct OrchestrationContext {
    /// 上下文ID
    pub id: String,
    /// 流程ID
    pub flow_id: String,
    /// 当前步骤索引
    pub current_step: usize,
    /// 步骤结果
    pub step_results: Vec<StepResult>,
    /// 共享状态
    pub shared_state: std::collections::HashMap<String, String>,
    /// 开始时间
    pub started_at: std::time::Instant,
}

/// 步骤结果
#[derive(Debug, Clone)]
pub struct StepResult {
    /// 步骤ID
    pub step_id: String,
    /// 状态
    pub status: StepStatus,
    /// 输出
    pub output: Option<CapabilityOutput>,
    /// 错误信息
    pub error: Option<String>,
    /// 执行时间
    pub execution_time_ms: u64,
}

/// 步骤状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
    Timeout,
}

/// 编排引擎
pub struct OrchestrationEngine {
    /// 能力注册中心
    registry: Arc<CapabilityRegistry>,
    /// 活跃流程
    active_flows: Vec<OrchestrationContext>,
    /// 最大活跃流程数
    max_active_flows: usize,
}

impl OrchestrationEngine {
    /// 创建新的编排引擎
    pub fn new(registry: Arc<CapabilityRegistry>) -> Self {
        Self {
            registry,
            active_flows: Vec::new(),
            max_active_flows: 10,
        }
    }

    /// 执行编排流程
    pub fn execute_flow(
        &mut self,
        flow: &OrchestrationFlow,
        initial_input: CapabilityInput,
    ) -> Result<OrchestrationContext, CapabilityError> {
        // 检查活跃流程数
        if self.active_flows.len() >= self.max_active_flows {
            return Err(CapabilityError::ExecutionFailed(
                "达到最大活跃流程数".into(),
            ));
        }

        // 创建上下文
        let mut context = OrchestrationContext {
            id: format!("ctx_{}", chrono::Utc::now().timestamp()),
            flow_id: flow.id.clone(),
            current_step: 0,
            step_results: Vec::new(),
            shared_state: std::collections::HashMap::new(),
            started_at: std::time::Instant::now(),
        };

        // 根据模式执行
        match flow.mode {
            OrchestrationMode::Sequential => {
                self.execute_sequential(flow, &mut context, initial_input)?;
            }
            OrchestrationMode::Parallel => {
                self.execute_parallel(flow, &mut context, initial_input)?;
            }
            OrchestrationMode::Conditional => {
                self.execute_conditional(flow, &mut context, initial_input)?;
            }
            OrchestrationMode::Loop => {
                self.execute_loop(flow, &mut context, initial_input)?;
            }
            OrchestrationMode::Branching => {
                self.execute_branching(flow, &mut context, initial_input)?;
            }
        }

        // 保存上下文
        self.active_flows.push(context.clone());

        Ok(context)
    }

    /// 顺序执行
    fn execute_sequential(
        &self,
        flow: &OrchestrationFlow,
        context: &mut OrchestrationContext,
        mut input: CapabilityInput,
    ) -> Result<(), CapabilityError> {
        for (i, step) in flow.steps.iter().enumerate() {
            context.current_step = i;

            // 执行步骤
            let result = self.execute_step(step, &input)?;
            context.step_results.push(result.clone());

            // 更新输入
            if let Some(output) = &result.output {
                input = self.map_output_to_input(output, &step.output_mapping)?;
            }
        }
        Ok(())
    }

    /// 并行执行
    fn execute_parallel(
        &self,
        flow: &OrchestrationFlow,
        context: &mut OrchestrationContext,
        input: CapabilityInput,
    ) -> Result<(), CapabilityError> {
        let mut handles = Vec::new();

        for step in &flow.steps {
            let step = step.clone();
            let input = input.clone();
            let registry = self.registry.clone();

            let handle = std::thread::spawn(move || {
                let cap = registry.get(&step.capability_id);
                if let Some(cap) = cap {
                    let start = std::time::Instant::now();
                    let result = cap.execute(input);
                    let duration = start.elapsed().as_millis() as u64;

                    match result {
                        Ok(output) => StepResult {
                            step_id: step.id,
                            status: StepStatus::Completed,
                            output: Some(output),
                            error: None,
                            execution_time_ms: duration,
                        },
                        Err(e) => StepResult {
                            step_id: step.id,
                            status: StepStatus::Failed,
                            output: None,
                            error: Some(e.to_string()),
                            execution_time_ms: duration,
                        },
                    }
                } else {
                    StepResult {
                        step_id: step.id,
                        status: StepStatus::Failed,
                        output: None,
                        error: Some("能力未注册".into()),
                        execution_time_ms: 0,
                    }
                }
            });

            handles.push(handle);
        }

        // 等待所有步骤完成
        for handle in handles {
            if let Ok(result) = handle.join() {
                context.step_results.push(result);
            }
        }

        Ok(())
    }

    /// 条件执行
    fn execute_conditional(
        &self,
        flow: &OrchestrationFlow,
        context: &mut OrchestrationContext,
        input: CapabilityInput,
    ) -> Result<(), CapabilityError> {
        for (i, step) in flow.steps.iter().enumerate() {
            context.current_step = i;

            // 检查条件
            if let Some(condition) = &step.condition {
                if !self.evaluate_condition(condition, context) {
                    context.step_results.push(StepResult {
                        step_id: step.id.clone(),
                        status: StepStatus::Skipped,
                        output: None,
                        error: None,
                        execution_time_ms: 0,
                    });
                    continue;
                }
            }

            // 执行步骤
            let result = self.execute_step(step, &input)?;
            context.step_results.push(result);
        }
        Ok(())
    }

    /// 循环执行
    fn execute_loop(
        &self,
        flow: &OrchestrationFlow,
        context: &mut OrchestrationContext,
        input: CapabilityInput,
    ) -> Result<(), CapabilityError> {
        let max_iterations = 10; // 安全限制
        let mut iteration = 0;

        loop {
            if iteration >= max_iterations {
                break;
            }

            for (i, step) in flow.steps.iter().enumerate() {
                context.current_step = i;

                // 执行步骤
                let result = self.execute_step(step, &input)?;
                context.step_results.push(result);

                // 检查退出条件
                if let Some(condition) = &step.condition {
                    if self.evaluate_condition(condition, context) {
                        return Ok(());
                    }
                }
            }

            iteration += 1;
        }

        Ok(())
    }

    /// 分支执行
    fn execute_branching(
        &self,
        flow: &OrchestrationFlow,
        context: &mut OrchestrationContext,
        input: CapabilityInput,
    ) -> Result<(), CapabilityError> {
        // 简化的分支执行
        for (i, step) in flow.steps.iter().enumerate() {
            context.current_step = i;

            // 执行步骤
            let result = self.execute_step(step, &input)?;
            context.step_results.push(result);
        }
        Ok(())
    }

    /// 执行单个步骤
    fn execute_step(
        &self,
        step: &OrchestrationStep,
        input: &CapabilityInput,
    ) -> Result<StepResult, CapabilityError> {
        let cap = self.registry.get(&step.capability_id).ok_or_else(|| {
            CapabilityError::ExecutionFailed(format!("能力未注册: {}", step.capability_id))
        })?;

        let start = std::time::Instant::now();
        let result = cap.execute(input.clone());
        let duration = start.elapsed().as_millis() as u64;

        match result {
            Ok(output) => Ok(StepResult {
                step_id: step.id.clone(),
                status: StepStatus::Completed,
                output: Some(output),
                error: None,
                execution_time_ms: duration,
            }),
            Err(e) => Ok(StepResult {
                step_id: step.id.clone(),
                status: StepStatus::Failed,
                output: None,
                error: Some(e.to_string()),
                execution_time_ms: duration,
            }),
        }
    }

    /// 映射输出到输入
    fn map_output_to_input(
        &self,
        output: &CapabilityOutput,
        mapping: &str,
    ) -> Result<CapabilityInput, CapabilityError> {
        match mapping {
            "passthrough" => match output {
                CapabilityOutput::Text(text) => Ok(CapabilityInput::Text(text.clone())),
                _ => Ok(CapabilityInput::Text("".into())),
            },
            "text_to_nlp" => match output {
                CapabilityOutput::Text(text) => Ok(CapabilityInput::Nlp(NlpInput {
                    task: NlpTask::Tokenize,
                    text: text.clone(),
                    language: None,
                })),
                _ => Err(CapabilityError::ExecutionFailed("类型不匹配".into())),
            },
            _ => Err(CapabilityError::ExecutionFailed(format!(
                "未知映射: {}",
                mapping
            ))),
        }
    }

    /// 评估条件
    fn evaluate_condition(&self, condition: &str, context: &OrchestrationContext) -> bool {
        // 简化的条件评估
        match condition {
            "always" => true,
            "never" => false,
            "has_results" => !context.step_results.is_empty(),
            "last_success" => context
                .step_results
                .last()
                .map(|r| r.status == StepStatus::Completed)
                .unwrap_or(false),
            _ => false,
        }
    }

    /// 获取活跃流程
    pub fn get_active_flows(&self) -> &[OrchestrationContext] {
        &self.active_flows
    }

    /// 清理完成的流程
    pub fn cleanup_completed(&mut self) {
        self.active_flows.retain(|ctx| {
            let elapsed = ctx.started_at.elapsed();
            elapsed.as_secs() < 300 // 5分钟超时
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_creation() {
        let registry = Arc::new(CapabilityRegistry::new());
        let engine = OrchestrationEngine::new(registry);
        assert!(engine.get_active_flows().is_empty());
    }

    #[test]
    fn sequential_flow() {
        let mut registry = CapabilityRegistry::new();
        // 注册测试能力
        let cap = crate::l2_perception::nt_world::nt_nlp_capability::create_nlp_capability();
        registry.register(cap);

        let mut engine = OrchestrationEngine::new(Arc::new(registry));

        let flow = OrchestrationFlow {
            id: "test_flow".into(),
            name: "测试流程".into(),
            mode: OrchestrationMode::Sequential,
            steps: vec![OrchestrationStep {
                id: "step1".into(),
                name: "步骤1".into(),
                capability_id: "nt-world-nlp".into(),
                input_mapping: "passthrough".into(),
                output_mapping: "passthrough".into(),
                condition: None,
                max_retries: 3,
                timeout_ms: 5000,
            }],
            global_timeout_ms: 30000,
            max_parallelism: 1,
        };

        let input = CapabilityInput::Text("测试".into());
        let result = engine.execute_flow(&flow, input);
        assert!(result.is_ok());
    }
}
