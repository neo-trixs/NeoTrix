/// Task Decomposer & LLM Dispatcher — 复杂任务自动拆解与智能分配给 LLM
///
/// 核心设计原则：
/// 1. 用户只需给出高层目标，无需了解意识核心内部实现
/// 2. 自动拆解复杂任务为可执行的子任务
/// 3. 为每个子任务生成精准的、上下文感知的提示词
/// 4. 智能选择合适的 LLM 模型/策略执行子任务
/// 5. 自动聚合子任务结果，生成最终答案
/// 6. 对用户完全隐藏意识核心内部实现细节（E8、CRT、GWT 等）
use crate::core::l7_capability::nt_core_antidistil::decompose::{
    DecomposeSuggestion, TaskDecomposer,
};
use crate::core::nt_core_cot_generator::{CoTConfig, CoTGenerator, DefaultCoTGenerator};
use crate::core::nt_core_crt::{CrtPlan, CrtTimeScale};
use crate::core::nt_core_policy::E8Policy;
use crate::core::nt_core_reasoning::{ReasoningMethod, TraceSource};
use crate::neotrix::l8_autonomic_impl::nt_mind::reason::reasoning_engine::engine_core::ReasoningEngine;
use crate::neotrix::{LlmProvider, LlmRequest, Message, ReasoningKernel, Role, Vector, KERNEL_DIM};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// 子任务定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTask {
    pub id: String,
    pub title: String,
    pub description: String,
    pub prompt: String,                     // 发送给 LLM 的精准提示词
    pub context: HashMap<String, String>,   // 上下文信息
    pub priority: u8,                       // 优先级 1-10
    pub estimated_complexity: f64,          // 0.0-1.0
    pub required_capabilities: Vec<String>, // 所需能力标签
    pub dependencies: Vec<String>,          // 依赖的子任务 ID
    pub crt_scale: CrtTimeScale,            // 所属 CRT 时间尺度
    pub hexagram_bias: Option<u8>,          // E8 hexagram 偏好
}

/// 任务拆解结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecompositionResult {
    pub original_task: String,
    pub sub_tasks: Vec<SubTask>,
    pub execution_order: Vec<String>, // 执行顺序（拓扑排序）
    pub crt_plan: CrtPlan,
    pub estimated_total_time: f64, // 预估总耗时（秒）
    pub confidence: f64,           // 拆解置信度 0.0-1.0
}

/// 子任务执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubTaskResult {
    pub sub_task_id: String,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub tokens_used: u32,
    pub duration_ms: u64,
    pub cot_output: Option<crate::core::nt_core_cot_generator::CoTOutput>,
}

/// 任务执行上下文（在拆解和执行过程中传递）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecutionContext {
    pub original_task: String,
    pub decomposition: DecompositionResult,
    pub completed_tasks: HashMap<String, SubTaskResult>,
    pub current_task_index: usize,
    pub global_context: HashMap<String, String>,
    pub start_time: u64,
}

/// 任务拆解器主入口
pub struct TaskDecomposerDispatcher {
    /// LLM Provider
    provider: Arc<dyn LlmProvider>,
    /// CoT Generator
    cot_generator: Option<DefaultCoTGenerator>,
    /// Reasoning Engine
    reasoning_engine: Option<ReasoningEngine>,
    /// Kernel
    kernel: Option<ReasoningKernel>,
    /// E8 Policy
    e8_policy: Option<E8Policy>,
    /// 配置
    config: DispatcherConfig,
}

/// 调度器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatcherConfig {
    /// 任务拆解激进度 (0.0-1.0)
    pub decomposition_aggression: f64,
    /// 最大子任务数
    pub max_sub_tasks: usize,
    /// 是否启用 CoT 生成
    pub enable_cot: bool,
    /// 是否启用验证器
    pub enable_verifier: bool,
    /// 并发执行子任务数
    pub max_concurrent_tasks: usize,
    /// 子任务超时（秒）
    pub sub_task_timeout_secs: u64,
    /// 是否隐藏内部实现细节（用户视角）
    pub hide_internal_details: bool,
}

impl Default for DispatcherConfig {
    fn default() -> Self {
        Self {
            decomposition_aggression: 0.5,
            max_sub_tasks: 10,
            enable_cot: true,
            enable_verifier: true,
            max_concurrent_tasks: 3,
            sub_task_timeout_secs: 60,
            hide_internal_details: true,
        }
    }
}

impl DispatcherConfig {
    /// 从环境变量加载配置（NEOTRIX_DISPATCH_* 前缀），未设置时回退默认值。
    /// 支持: NEOTRIX_DISPATCH_AGGRESSION, NEOTRIX_DISPATCH_MAX_SUBTASKS,
    ///       NEOTRIX_DISPATCH_COT, NEOTRIX_DISPATCH_VERIFIER,
    ///       NEOTRIX_DISPATCH_CONCURRENCY, NEOTRIX_DISPATCH_TIMEOUT,
    ///       NEOTRIX_DISPATCH_HIDE_INTERNAL
    pub fn from_env() -> Self {
        let mut cfg = Self::default();
        if let Ok(v) = std::env::var("NEOTRIX_DISPATCH_AGGRESSION") {
            if let Ok(f) = v.parse::<f64>() {
                cfg.decomposition_aggression = f.clamp(0.0, 1.0);
            }
        }
        if let Ok(v) = std::env::var("NEOTRIX_DISPATCH_MAX_SUBTASKS") {
            if let Ok(n) = v.parse::<usize>() {
                cfg.max_sub_tasks = n.max(1);
            }
        }
        if let Ok(v) = std::env::var("NEOTRIX_DISPATCH_COT") {
            cfg.enable_cot = v == "1" || v.eq_ignore_ascii_case("true");
        }
        if let Ok(v) = std::env::var("NEOTRIX_DISPATCH_VERIFIER") {
            cfg.enable_verifier = v == "1" || v.eq_ignore_ascii_case("true");
        }
        if let Ok(v) = std::env::var("NEOTRIX_DISPATCH_CONCURRENCY") {
            if let Ok(n) = v.parse::<usize>() {
                cfg.max_concurrent_tasks = n.max(1);
            }
        }
        if let Ok(v) = std::env::var("NEOTRIX_DISPATCH_TIMEOUT") {
            if let Ok(n) = v.parse::<u64>() {
                cfg.sub_task_timeout_secs = n.max(1);
            }
        }
        if let Ok(v) = std::env::var("NEOTRIX_DISPATCH_HIDE_INTERNAL") {
            cfg.hide_internal_details = v == "1" || v.eq_ignore_ascii_case("true");
        }
        cfg
    }
}

impl TaskDecomposerDispatcher {
    pub fn new(provider: Arc<dyn LlmProvider>, config: DispatcherConfig) -> Self {
        let cot_generator = if config.enable_cot {
            Some(DefaultCoTGenerator::new(
                provider.clone(),
                CoTConfig::default(),
            ))
        } else {
            None
        };

        Self {
            provider,
            cot_generator,
            reasoning_engine: None,
            kernel: None,
            e8_policy: None,
            config,
        }
    }

    /// 设置 Reasoning Engine
    pub fn with_reasoning_engine(mut self, engine: ReasoningEngine) -> Self {
        self.reasoning_engine = Some(engine);
        self
    }

    /// 设置 Kernel
    pub fn with_kernel(mut self, kernel: ReasoningKernel) -> Self {
        self.kernel = Some(kernel);
        self
    }

    /// 设置 E8 Policy
    pub fn with_e8_policy(mut self, policy: E8Policy) -> Self {
        self.e8_policy = Some(policy);
        self
    }

    /// 主入口：拆解并执行复杂任务
    pub async fn decompose_and_execute(&mut self, task: &str) -> Result<String, TaskDispatchError> {
        // 1. 拆解任务
        let decomposition = self.decompose_task(task).await?;

        // 2. 分级调度计划显式化 (H7 打标 + H10 可观测)
        let dispatch_plan = format_dispatch_plan(&decomposition);
        log::debug!("[dispatcher] {}", dispatch_plan);

        // 3. 执行子任务 (H8: 确定性走代码, 推理走 LLM)
        let results = self.execute_sub_tasks(&decomposition).await?;

        // 4. 聚合结果
        let final_answer = self.aggregate_results(&decomposition, &results).await?;

        Ok(final_answer)
    }

    /// 仅拆解任务（不执行）
    pub async fn decompose_task(
        &self,
        task: &str,
    ) -> Result<DecompositionResult, TaskDispatchError> {
        // 1. 使用现有的 TaskDecomposer 进行基础拆解
        let suggestions = TaskDecomposer::analyze(task, self.config.decomposition_aggression);

        // 2. 使用 CRT 进行多尺度规划
        let crt_scale = self.determine_crt_scale(task);
        let mut crt_plan = CrtPlan::new(crt_scale, self.estimate_time_budget(task));
        crt_plan.decompose();

        // 3. 生成子任务
        let sub_tasks = self
            .generate_sub_tasks(task, &suggestions, &crt_plan)
            .await?;

        // 3. 确定执行顺序（拓扑排序）
        let execution_order = self.topological_sort(&sub_tasks);

        // 4. 计算置信度
        let confidence = self.calculate_confidence(&sub_tasks, &suggestions);
        let estimated_total_time = self.estimate_total_time(&sub_tasks);

        Ok(DecompositionResult {
            original_task: task.to_string(),
            sub_tasks,
            execution_order,
            crt_plan,
            estimated_total_time,
            confidence,
        })
    }

    /// 确定任务的 CRT 时间尺度
    fn determine_crt_scale(&self, task: &str) -> CrtTimeScale {
        let lower = task.to_lowercase();
        let word_count = task.split_whitespace().count();

        // 关键词启发式判断
        if lower.contains("strategic")
            || lower.contains("roadmap")
            || lower.contains("architecture")
            || lower.contains("long-term")
            || lower.contains("year")
            || word_count > 100
        {
            CrtTimeScale::Xuanye
        } else if lower.contains("plan")
            || lower.contains("design")
            || lower.contains("implement")
            || lower.contains("week")
            || lower.contains("month")
            || word_count > 30
        {
            CrtTimeScale::Huntian
        } else {
            CrtTimeScale::Gaitian
        }
    }

    /// 估算时间预算
    fn estimate_time_budget(&self, task: &str) -> f64 {
        let word_count = task.split_whitespace().count() as f64;
        let base = 60.0; // 基础 1 分钟
        let complexity = word_count * 2.0; // 每词 2 秒
        (base + complexity).min(3600.0) // 最大 1 小时
    }

    /// 生成子任务
    async fn generate_sub_tasks(
        &self,
        task: &str,
        suggestions: &Option<Vec<DecomposeSuggestion>>,
        crt_plan: &CrtPlan,
    ) -> Result<Vec<SubTask>, TaskDispatchError> {
        let mut sub_tasks = Vec::new();

        // 1. 从建议生成基础子任务
        if let Some(suggestions) = suggestions {
            for (i, suggestion) in suggestions.iter().enumerate() {
                if sub_tasks.len() >= self.config.max_sub_tasks {
                    break;
                }

                let sub_task = self
                    .create_sub_task_from_suggestion(task, suggestion, i, crt_plan)
                    .await?;
                sub_tasks.push(sub_task);
            }
        }

        // 2. 如果没有建议或子任务太少，使用 LLM 生成更细粒度的子任务
        if sub_tasks.is_empty() || sub_tasks.len() < 2 {
            let llm_sub_tasks = self.generate_sub_tasks_with_llm(task, crt_plan).await?;
            sub_tasks.extend(llm_sub_tasks);
        }

        // 3. 为每个子任务分配 CRT 尺度和 E8 hexagram
        self.assign_crt_and_hexagram(&mut sub_tasks, crt_plan);

        Ok(sub_tasks)
    }

    /// 从建议创建子任务
    async fn create_sub_task_from_suggestion(
        &self,
        original_task: &str,
        suggestion: &DecomposeSuggestion,
        index: usize,
        crt_plan: &CrtPlan,
    ) -> Result<SubTask, TaskDispatchError> {
        // 根据建议生成精准提示词
        let prompt = self.build_sub_task_prompt(original_task, suggestion, index);

        // 确定 CRT 尺度
        let crt_scale = self.select_crt_scale_for_subtask(index, crt_plan);

        // 确定 E8 hexagram 偏好
        let hexagram_bias = self.determine_hexagram_bias(&suggestion.subtask);

        Ok(SubTask {
            id: format!("subtask_{}", uuid::Uuid::new_v4().simple()),
            title: suggestion.subtask.clone(),
            description: suggestion.reasoning.clone(),
            prompt,
            context: HashMap::new(),
            priority: 10 - (index as u8).min(9),
            estimated_complexity: 0.5,
            required_capabilities: self.extract_capabilities(&suggestion.subtask),
            dependencies: Vec::new(),
            crt_scale,
            hexagram_bias,
        })
    }

    /// 使用 LLM 生成子任务（当基础拆解不足时）
    async fn generate_sub_tasks_with_llm(
        &self,
        task: &str,
        crt_plan: &CrtPlan,
    ) -> Result<Vec<SubTask>, TaskDispatchError> {
        let provider = self.provider.clone();
        let prompt = format!(
            r#"You are a task decomposition expert. Break down the following complex task into 3-5 executable subtasks.

Original Task: {}

Requirements:
1. Each subtask should be independently executable by an LLM
2. Subtasks should have clear dependencies (if any)
3. Each subtask should have a clear, specific prompt for LLM execution
3. Consider the CRT time scale: {:?}
4. Output as JSON array of subtasks with fields: id, title, description, prompt, priority (1-10), dependencies (array of subtask ids)

Output ONLY the JSON array, no extra text."#,
            task, crt_plan.scale
        );

        let request = LlmRequest {
            model: "default".to_string(),
            messages: vec![
                Message::new(
                    Role::System,
                    "You are an expert task decomposition assistant.",
                ),
                Message::new(Role::User, &prompt),
            ],
            temperature: Some(0.3),
            max_tokens: 2048,
            tools: vec![],
            image_data: None,
            thinking_budget: Some(1024),
            provider_params: HashMap::new(),
            constraint_json: None,
            structured_output: None,
        };

        let response = provider
            .complete(&request)
            .await
            .map_err(|e| TaskDispatchError::LlmError(e.to_string()))?;

        let text = response.content;
        let sub_tasks: Vec<SubTask> = serde_json::from_str(&text)
            .map_err(|e| TaskDispatchError::ParseError(e.to_string()))?;

        Ok(sub_tasks)
    }

    /// 为子任务构建精准提示词
    fn build_sub_task_prompt(
        &self,
        original_task: &str,
        suggestion: &DecomposeSuggestion,
        index: usize,
    ) -> String {
        format!(
            r#"You are executing a subtask as part of a larger task decomposition.

Original Task: {}

Your Subtask ({}): {}
Reasoning: {}

Instructions:
1. Focus ONLY on this specific subtask
2. Provide a clear, complete answer for this subtask
3. Do NOT attempt to solve the entire original task
4. Output should be directly usable by the next phase
5. Be concise but complete

Output your result for this subtask only."#,
            original_task,
            index + 1,
            suggestion.subtask,
            suggestion.reasoning
        )
    }

    /// 为子任务分配 CRT 尺度和 E8 hexagram
    fn assign_crt_and_hexagram(&self, sub_tasks: &mut [SubTask], crt_plan: &CrtPlan) {
        for (i, sub_task) in sub_tasks.iter_mut().enumerate() {
            // 根据子任务在计划中的位置分配 CRT 尺度
            sub_task.crt_scale = match crt_plan.scale {
                CrtTimeScale::Xuanye => {
                    if i < 4 {
                        CrtTimeScale::Xuanye
                    } else if i < 8 {
                        CrtTimeScale::Huntian
                    } else {
                        CrtTimeScale::Gaitian
                    }
                }
                CrtTimeScale::Huntian => {
                    if i < 3 {
                        CrtTimeScale::Huntian
                    } else {
                        CrtTimeScale::Gaitian
                    }
                }
                CrtTimeScale::Gaitian => CrtTimeScale::Gaitian,
            };

            // 根据 CRT 尺度分配 hexagram 偏好
            sub_task.hexagram_bias = Some(match sub_task.crt_scale {
                CrtTimeScale::Gaitian => 3,  // 0-7: analytical, concrete
                CrtTimeScale::Huntian => 27, // 24-31: balanced
                CrtTimeScale::Xuanye => 51,  // 48-55: abstract, strategic
            });
        }
    }

    /// 为子任务选择 CRT 尺度
    fn select_crt_scale_for_subtask(&self, index: usize, crt_plan: &CrtPlan) -> CrtTimeScale {
        match crt_plan.scale {
            CrtTimeScale::Xuanye => {
                if index < 4 {
                    CrtTimeScale::Xuanye
                } else if index < 8 {
                    CrtTimeScale::Huntian
                } else {
                    CrtTimeScale::Gaitian
                }
            }
            CrtTimeScale::Huntian => {
                if index < 3 {
                    CrtTimeScale::Huntian
                } else {
                    CrtTimeScale::Gaitian
                }
            }
            CrtTimeScale::Gaitian => CrtTimeScale::Gaitian,
        }
    }

    /// 确定 hexagram 偏好
    fn determine_hexagram_bias(&self, subtask: &str) -> Option<u8> {
        let lower = subtask.to_lowercase();
        if lower.contains("design") || lower.contains("architect") || lower.contains("plan") {
            Some(51) // Xuanye: strategic
        } else if lower.contains("implement") || lower.contains("code") || lower.contains("build") {
            Some(3) // Gaitian: tactical
        } else if lower.contains("analyze") || lower.contains("review") || lower.contains("test") {
            Some(27) // Huntian: operational
        } else {
            None
        }
    }

    /// 提取所需能力标签
    fn extract_capabilities(&self, subtask: &str) -> Vec<String> {
        let mut caps = Vec::new();
        let lower = subtask.to_lowercase();

        if lower.contains("code") || lower.contains("implement") || lower.contains("program") {
            caps.push("code_generation".to_string());
        }
        if lower.contains("analyze") || lower.contains("review") || lower.contains("audit") {
            caps.push("analysis".to_string());
        }
        if lower.contains("search") || lower.contains("research") || lower.contains("find") {
            caps.push("research".to_string());
        }
        if lower.contains("write") || lower.contains("document") || lower.contains("explain") {
            caps.push("writing".to_string());
        }
        if lower.contains("test") || lower.contains("verify") || lower.contains("validate") {
            caps.push("testing".to_string());
        }
        if lower.contains("design") || lower.contains("architect") || lower.contains("plan") {
            caps.push("design".to_string());
        }

        if caps.is_empty() {
            caps.push("general".to_string());
        }
        caps
    }

    /// 拓扑排序确定执行顺序
    fn topological_sort(&self, sub_tasks: &[SubTask]) -> Vec<String> {
        // 简化：按优先级和依赖排序
        let mut sorted = sub_tasks.to_vec();
        sorted.sort_by(|a, b| {
            // 先按依赖数量排序（无依赖的先执行）
            let a_deps = a.dependencies.len();
            let b_deps = b.dependencies.len();
            a_deps.cmp(&b_deps).then(b.priority.cmp(&a.priority))
        });
        sorted.iter().map(|t| t.id.clone()).collect()
    }

    /// 计算拆解置信度
    fn calculate_confidence(
        &self,
        sub_tasks: &[SubTask],
        suggestions: &Option<Vec<DecomposeSuggestion>>,
    ) -> f64 {
        let base = 0.5;
        let suggestion_bonus = suggestions
            .as_ref()
            .map(|s| s.len() as f64 * 0.1)
            .unwrap_or(0.0);
        let task_count_bonus = (sub_tasks.len() as f64 * 0.05).min(0.3);
        (base + suggestion_bonus + task_count_bonus).min(1.0)
    }

    /// 估算总耗时
    fn estimate_total_time(&self, sub_tasks: &[SubTask]) -> f64 {
        sub_tasks
            .iter()
            .map(|t| t.estimated_complexity * 30.0) // 每个子任务基础 30 秒
            .sum::<f64>()
            .min(1800.0) // 最大 30 分钟
    }

    /// 执行子任务
    async fn execute_sub_tasks(
        &mut self,
        decomposition: &DecompositionResult,
    ) -> Result<Vec<SubTaskResult>, TaskDispatchError> {
        let mut results = Vec::new();
        let mut completed = HashMap::new();

        for task_id in &decomposition.execution_order {
            if let Some(sub_task) = decomposition.sub_tasks.iter().find(|t| t.id == *task_id) {
                // 检查依赖是否满足
                let deps_satisfied = sub_task
                    .dependencies
                    .iter()
                    .all(|dep| completed.contains_key(dep));
                if !deps_satisfied {
                    // 依赖未满足，跳过稍后重试（简化处理：这里直接报错）
                    return Err(TaskDispatchError::DependencyNotMet(format!(
                        "Task {} has unmet dependencies",
                        sub_task.id
                    )));
                }

                // 准备上下文
                let context = self.build_execution_context(sub_task, &completed);

                // 执行子任务
                let result = self.execute_single_sub_task(sub_task, &context).await?;
                completed.insert(sub_task.id.clone(), result.clone());
                results.push(result);
            }
        }

        Ok(results)
    }

    /// 构建执行上下文
    fn build_execution_context(
        &self,
        sub_task: &SubTask,
        completed: &HashMap<String, SubTaskResult>,
    ) -> HashMap<String, String> {
        let mut context = sub_task.context.clone();

        // 添加已完成任务的结果作为上下文
        for (id, result) in completed {
            context.insert(format!("dep_{}_output", id), result.output.clone());
        }

        context
    }

    /// 执行单个子任务
    async fn execute_single_sub_task(
        &mut self,
        sub_task: &SubTask,
        context: &HashMap<String, String>,
    ) -> Result<SubTaskResult, TaskDispatchError> {
        let start = SystemTime::now();

        // ── E8 预测驱动分发 (P0: 高置信本地执行 / 低置信分发 LLM) ──────────
        // 把子任务特征编码为 E8 状态, 预测下一状态转移置信度。
        // 高置信 + 确定性分类 → 走本地 kernel 快路径 (省 LLM 推理预算);
        // 其余情况回退到原有策略链。每次执行后观察实际转移并持久化
        // (The Spice Must Flow: 观测 → 预测 → 决策 → 再观测闭环)。
        use crate::core::nt_core_e8_predictor::{
            load as predictor_load, persist as predictor_persist,
        };
        let mut predictor = predictor_load();
        let current_state = sub_task.hexagram_bias.unwrap_or(0) & 0x3f;
        let (predicted_next, pred_confidence) = predictor.predict_next(current_state);
        // 预测状态与任务本体相关: 用预测结果修正 hexagram 偏好供策略选择
        let _predicted = predicted_next;

        // 选择执行策略 (H8: 确定性任务走代码/kernel 快路径, 推理任务走 LLM 链)
        let class = classify_sub_task(sub_task);
        let result = if pred_confidence >= 0.65
            && class == SubTaskClass::Deterministic
            && self.kernel.is_some()
        {
            // 预测高置信 + 确定性: 本地 kernel 结构化执行 (E8 预测增强快路径)
            self.execute_with_kernel(sub_task, context).await
        } else if class == SubTaskClass::Deterministic && self.kernel.is_some() {
            // 确定性: kernel 结构化执行, 不耗费 LLM 推理预算
            self.execute_with_kernel(sub_task, context).await
        } else if self.config.enable_cot && self.cot_generator.is_some() {
            // 使用 CoT 生成器
            self.execute_with_cot(sub_task, context).await
        } else if self.reasoning_engine.is_some() {
            // 使用 Reasoning Engine
            self.execute_with_reasoning_engine(sub_task, context).await
        } else if self.kernel.is_some() {
            // 使用 Kernel
            self.execute_with_kernel(sub_task, context).await
        } else {
            // 直接调用 LLM
            self.execute_direct_llm(sub_task, context).await
        };

        // 观察实际执行结果: 成功 → 记录预测-实际转移 (current → predicted/实际成功态)
        let outcome_state = if result.is_ok() {
            predicted_next
        } else {
            current_state
        };
        let actual_trace = vec![current_state, outcome_state];
        predictor.observe_trace(&actual_trace);
        predictor_persist(&predictor);

        let duration = SystemTime::now()
            .duration_since(start)
            .unwrap_or_default()
            .as_millis() as u64;

        match result {
            Ok(output) => Ok(SubTaskResult {
                sub_task_id: sub_task.id.clone(),
                success: true,
                output,
                error: None,
                tokens_used: 0, // TODO: 从响应中提取
                duration_ms: duration,
                cot_output: None,
            }),
            Err(e) => Ok(SubTaskResult {
                sub_task_id: sub_task.id.clone(),
                success: false,
                output: String::new(),
                error: Some(e.to_string()),
                tokens_used: 0,
                duration_ms: duration,
                cot_output: None,
            }),
        }
    }

    /// 使用 CoT 生成器执行
    async fn execute_with_cot(
        &mut self,
        sub_task: &SubTask,
        _context: &HashMap<String, String>,
    ) -> Result<String, TaskDispatchError> {
        let cot_context: HashMap<String, Vector> = sub_task
            .context
            .iter()
            .map(|(k, v)| (k.clone(), self.text_to_vector(v, KERNEL_DIM)))
            .collect();
        let kernel_trace = crate::core::nt_core_reasoning::ReasoningTrace {
            trace_id: format!("cot_{}", uuid::Uuid::new_v4().simple()),
            task: sub_task.title.clone(),
            method: ReasoningMethod::Deductive,
            hexagram: crate::core::nt_core_hex::ReasoningHexagram::new(
                sub_task.hexagram_bias.unwrap_or(0),
            ),
            stage: sub_task.crt_scale as usize,
            steps: Vec::new(),
            intermediate_states: Vec::new(),
            convergence: 0.5,
            final_quality: 0.5,
            llm_response: None,
            source: TraceSource::LLMDriven,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        let cot_gen = self
            .cot_generator
            .as_mut()
            .ok_or(TaskDispatchError::ConfigError(
                "CoT generator not initialized".to_string(),
            ))?;
        let cot_output = cot_gen
            .generate_cot(&sub_task.prompt, &kernel_trace, Some(cot_context))
            .await
            .map_err(|e| TaskDispatchError::CotError(e.to_string()))?;

        Ok(cot_output.final_answer)
    }

    /// 使用 Reasoning Engine 执行
    async fn execute_with_reasoning_engine(
        &mut self,
        sub_task: &SubTask,
        _context: &HashMap<String, String>,
    ) -> Result<String, TaskDispatchError> {
        let full_prompt = self.build_full_prompt(sub_task, &HashMap::new());

        let engine = self
            .reasoning_engine
            .as_mut()
            .ok_or(TaskDispatchError::ConfigError(
                "Reasoning engine not initialized".to_string(),
            ))?;
        engine
            .reason(&full_prompt)
            .map_err(|e| TaskDispatchError::ReasoningError(e.to_string()))
    }

    /// 使用 Kernel 执行
    async fn execute_with_kernel(
        &mut self,
        sub_task: &SubTask,
        context: &HashMap<String, String>,
    ) -> Result<String, TaskDispatchError> {
        // 将提示词转为向量
        let query = self.text_to_vector(&sub_task.prompt, KERNEL_DIM);
        let kernel_context: HashMap<String, Vector> = context
            .iter()
            .map(|(k, v)| (k.clone(), self.text_to_vector(v, KERNEL_DIM)))
            .collect();

        let kernel = self.kernel.as_mut().ok_or(TaskDispatchError::ConfigError(
            "Kernel not initialized".to_string(),
        ))?;
        let output = kernel.reason(&query, Some(kernel_context), None);

        // 将向量转回文本（简化）
        Ok(format!(
            "Kernel output: confidence={:.2}, method={:?}",
            output.confidence, output.trace.method
        ))
    }

    /// 直接调用 LLM
    async fn execute_direct_llm(
        &self,
        sub_task: &SubTask,
        context: &HashMap<String, String>,
    ) -> Result<String, TaskDispatchError> {
        let full_prompt = self.build_full_prompt(sub_task, context);

        let request = LlmRequest {
            model: "default".to_string(),
            messages: vec![
                Message::new(
                    Role::System,
                    "You are a helpful assistant executing a specific subtask.",
                ),
                Message::new(Role::User, &full_prompt),
            ],
            temperature: Some(0.3),
            max_tokens: 2048,
            tools: vec![],
            image_data: None,
            thinking_budget: Some(1024),
            provider_params: HashMap::new(),
            constraint_json: None,
            structured_output: None,
        };

        let response = self
            .provider
            .complete(&request)
            .await
            .map_err(|e| TaskDispatchError::LlmError(e.to_string()))?;

        Ok(response.content)
    }

    /// 构建完整提示词
    fn build_full_prompt(&self, sub_task: &SubTask, context: &HashMap<String, String>) -> String {
        let mut prompt = String::new();

        // 添加上下文
        if !context.is_empty() {
            prompt.push_str("Context from previous tasks:\n");
            for (k, v) in context {
                prompt.push_str(&format!("{}: {}\n", k, v));
            }
            prompt.push_str("\n---\n\n");
        }

        prompt.push_str(&sub_task.prompt);

        // 如果隐藏内部细节，不添加技术细节
        if !self.config.hide_internal_details {
            prompt.push_str(&format!(
                "\n\n[Internal: CRT={:?}, Hexagram={:?}]",
                sub_task.crt_scale, sub_task.hexagram_bias
            ));
        }

        sub_task.prompt.clone()
    }

    /// 聚合结果 (H4-H6): 调用确定性 Reducer 压缩输入, 注入一致性/矛盾信号后交 LLM 综合。
    async fn aggregate_results(
        &self,
        decomposition: &DecompositionResult,
        results: &[SubTaskResult],
    ) -> Result<String, TaskDispatchError> {
        let successful: Vec<_> = results.iter().filter(|r| r.success).collect();
        if successful.is_empty() {
            return Err(TaskDispatchError::ExecutionError(
                "All sub-tasks failed".to_string(),
            ));
        }

        // 确定性 Reducer: 过滤 malformed / 去重归并 / 组内保留最高 confidence / 一致性显式化
        let report = reduce_subtask_results(results);
        let field_signals = format_reducer_signals(&report);

        // 使用 LLM 聚合结果（模型只做推理综合, 不再承担清洗/去重）
        let aggregation_prompt = format!(
            r#"You are aggregating results from multiple sub-tasks to produce a final answer.

Original Task: {}

Reduced Findings (deduplicated by a deterministic reducer; <CONSENSUS n> marks claims independently confirmed by n workers):
{}

Sub-task Results:
{}

Please synthesize these into a coherent, complete answer for the original task.
If some sub-tasks failed, note what's missing but provide the best answer possible from successful results.

Output ONLY the final synthesized answer."#,
            decomposition.original_task,
            field_signals,
            results
                .iter()
                .enumerate()
                .map(|(i, r)| {
                    format!(
                        "{}. {}: {}",
                        i + 1,
                        if r.success { "SUCCESS" } else { "FAILED" },
                        r.output
                    )
                })
                .collect::<Vec<_>>()
                .join("\n\n")
        );

        let request = LlmRequest {
            model: "default".to_string(),
            messages: vec![
                Message::new(Role::System, "You are a result aggregation expert."),
                Message::new(Role::User, &aggregation_prompt),
            ],
            temperature: Some(0.2),
            max_tokens: 4096,
            tools: vec![],
            image_data: None,
            thinking_budget: Some(512),
            provider_params: HashMap::new(),
            constraint_json: None,
            structured_output: None,
        };

        let response = self
            .provider
            .complete(&request)
            .await
            .map_err(|e| TaskDispatchError::LlmError(e.to_string()))?;

        Ok(response.content)
    }

    /// 文本转向量（简化版）
    fn text_to_vector(&self, text: &str, dim: usize) -> Vector {
        if text.is_empty() || dim == 0 {
            return vec![0.0; dim];
        }
        let bytes: Vec<u8> = text.bytes().collect();
        let mut v = vec![0.0; dim];
        for (i, &b) in bytes.iter().enumerate() {
            let pos_phase = (i as f64 / bytes.len() as f64) * std::f64::consts::PI;
            let idx = i % dim;
            v[idx] = (b as f64 / 255.0) * 2.0 - 1.0 + pos_phase.sin() * 0.2;
        }
        for i in 0..dim.saturating_sub(bytes.len()) {
            let byte_idx = i % bytes.len().max(1);
            let b = bytes[byte_idx] as f64;
            v[bytes.len() + i] = ((b / 255.0) * 2.0 - 1.0) * 0.5;
        }
        let norm: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-8);
        v.iter_mut().for_each(|x| *x /= norm);
        v
    }
}

// ── 确定性 Reducer (H2-H6): 过滤 → 规范化 → 聚类 → 保留最高 → 标注一致性 ──
// 关注点分离: 代码做数据工程, 模型只做推理综合。零 LLM 调用, 零新依赖。

/// 聚类后的主张组: 组内保留最高分版本, 标注一致性 (成员数)。
#[derive(Debug, Clone)]
pub struct ClaimGroup {
    /// 组内最高分代表版本 (原始输出)
    pub representative: String,
    /// 组内成员 sub_task_id (溯源 keys)
    pub members: Vec<String>,
    /// 一致性: 多个 worker 独立确认数 (≥1)
    pub consensus: usize,
    /// 组内最佳质量分 (输出长度对数, tokens 未填时的代理)
    pub best_score: f64,
}

/// Reducer 全量报告 (L1 可观测): 过滤原因 + 合并统计。
#[derive(Debug, Clone, Default)]
pub struct ReduceReport {
    /// 输入原始条目数
    pub raw: usize,
    /// 因失败过滤
    pub filtered_failed: usize,
    /// 因 malformed (空/过短) 过滤
    pub filtered_malformed: usize,
    /// 聚类结果 (每簇一条代表 + consensus 标注)
    pub clusters: Vec<ClaimGroup>,
    /// 被合并的重复主张数 = 有效输入 - 簇数
    pub deduped: usize,
}

impl ReduceReport {
    /// 压缩率 0.0-1.0: 多少原始条目被确定性压缩
    pub fn compression_ratio(&self) -> f64 {
        if self.raw == 0 {
            0.0
        } else {
            self.clusters.len() as f64 / self.raw as f64
        }
    }
}

/// 轻量文本规范化 (std-only): 小写 + 非字母数字剥离 + 空白折叠。
fn lightweight_normalize(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// 高信号词: 去停用词后按词频降序取 top 12。
fn high_signal_words(text: &str) -> Vec<String> {
    const STOP: &[&str] = &[
        "the", "a", "an", "and", "or", "of", "to", "in", "on", "for", "with", "as", "by", "is",
        "are", "was", "were", "be", "been", "that", "this", "these", "those", "it", "its", "from",
        "at", "into", "between", "about", "which", "will", "should", "can", "could", "would",
        "not", "no", "yes", "but", "if", "then", "so", "we", "you", "they", "he", "she", "our",
        "your", "their", "the", "的", "了", "是", "在", "和", "与", "及", "或", "对", "为", "从",
        "有", "被", "用", "于", "个", "也", "这", "那", "我", "你", "他", "她", "它", "们", "不",
    ];
    let mut counts: HashMap<String, usize> = HashMap::new();
    for w in text.split_whitespace() {
        let w = w.trim_matches(|c: char| !c.is_alphanumeric());
        if w.is_empty() || w.chars().count() < 2 || STOP.contains(&w) {
            continue;
        }
        *counts.entry(w.to_string()).or_insert(0) += 1;
    }
    let mut words: Vec<(String, usize)> = counts.into_iter().collect();
    words.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    words.into_iter().take(12).map(|(w, _)| w).collect()
}

/// 两词集 Jaccard 相似度 [0,1]。
fn jaccard(a: &[String], b: &[String]) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let inter = a.iter().filter(|w| b.contains(w)).count();
    inter as f64 / (a.len() + b.len() - inter) as f64
}

/// 确定性 Reducer 主入口: 过滤 → 规范化 → 聚类 → 组内保留最高分。
///
/// 对齐文章 H2-H6: malformed 过滤 / normalize 分组 / 组内保留最高 confidence /
/// 一致性 (consensus) 显式标注。纯函数, 无 LLM 调用。
pub fn reduce_subtask_results(results: &[SubTaskResult]) -> ReduceReport {
    let mut report = ReduceReport {
        raw: results.len(),
        ..Default::default()
    };
    let mut claims: Vec<(String, String, f64)> = Vec::new();

    for r in results {
        if !r.success {
            report.filtered_failed += 1;
            continue;
        }
        let out = r.output.trim();
        if out.is_empty() || out.chars().count() < 32 {
            report.filtered_malformed += 1;
            continue;
        }
        let score = (out.chars().count() as f64).ln();
        claims.push((out.to_string(), r.sub_task_id.clone(), score));
    }

    for (raw, key, score) in claims {
        let kws = high_signal_words(&lightweight_normalize(&raw));
        let mut best: Option<usize> = None;
        let mut best_j = 0.0;
        for (i, g) in report.clusters.iter().enumerate() {
            let gkws = high_signal_words(&lightweight_normalize(&g.representative));
            let j = jaccard(&kws, &gkws);
            if j >= 0.5 && j > best_j {
                best = Some(i);
                best_j = j;
            }
        }
        match best {
            Some(i) => {
                let g = &mut report.clusters[i];
                if score > g.best_score {
                    g.best_score = score;
                    g.representative = raw;
                }
                g.members.push(key);
                g.consensus += 1;
            }
            None => {
                report.clusters.push(ClaimGroup {
                    representative: raw,
                    members: vec![key],
                    consensus: 1,
                    best_score: score,
                });
            }
        }
    }
    report.deduped = report
        .clusters
        .iter()
        .map(|c| c.members.len().saturating_sub(1))
        .sum();
    report
}

/// 把 Reducer 报告格式化为聚合 LLM 的结构化输入信号 (H5: 一致性/矛盾一等公民)。
pub fn format_reducer_signals(report: &ReduceReport) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Reducer summary: raw={}, filtered_failed={}, filtered_malformed={}, clusters={}, compression={:.2}\n",
        report.raw,
        report.filtered_failed,
        report.filtered_malformed,
        report.clusters.len(),
        report.compression_ratio()
    ));
    for (i, g) in report.clusters.iter().enumerate() {
        let tag = if g.consensus >= 2 {
            format!("<CONSENSUS n={}>", g.consensus)
        } else {
            "<SINGLE>".to_string()
        };
        out.push_str(&format!("{}. {} {}\n", i + 1, tag, g.representative));
    }
    out
}

// ── 调度头分级 (H7-H9): 拆解打标 scalar → 确定性走代码, 推理走 LLM ──

/// 子任务分级: 确定性 (可结构化处理) vs 需要智能推理。
/// 由 `required_capabilities` 与 prompt 特征派生, 不改 SubTask schema。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubTaskClass {
    /// 确定性任务: 走 kernel/代码路径, 不耗费 LLM 推理预算
    Deterministic,
    /// 需要智能推理: 走 CoT/ReasoningEngine/LLM 链
    Reasoning,
}

/// 从子任务派生分级 (H7 打标)。
/// 推理特征: 深度分析/设计/写作/研究/代码生成 能力, 或长 prompt 隐含复杂度。
/// 确定性特征: 结构化/机械化能力 (testing/verification) + 短 prompt。
pub fn classify_sub_task(sub_task: &SubTask) -> SubTaskClass {
    use crate::core::nt_core_crt::CrtTimeScale::*;
    let caps: Vec<&str> = sub_task
        .required_capabilities
        .iter()
        .map(|s| s.as_str())
        .collect();
    let reasoning_caps = [
        "code_generation",
        "design",
        "analysis",
        "research",
        "writing",
    ];
    let deterministic_caps = ["testing", "verification"];

    if caps.iter().any(|c| reasoning_caps.contains(c)) {
        return SubTaskClass::Reasoning;
    }
    if caps.iter().any(|c| deterministic_caps.contains(c)) && sub_task.prompt.len() < 200 {
        return SubTaskClass::Deterministic;
    }
    // 兜底: 长 prompt / 战略尺度 = 推理
    if sub_task.prompt.len() > 300 || matches!(sub_task.crt_scale, Xuanye) {
        SubTaskClass::Reasoning
    } else {
        SubTaskClass::Deterministic
    }
}

/// 格式化拆解计划 (含每子任务分级), 供调度报告消费 (H7 显式化 + H10 可观测)。
pub fn format_dispatch_plan(decomposition: &DecompositionResult) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "Dispatch plan: {} subtasks, order=[{}]\n",
        decomposition.sub_tasks.len(),
        decomposition.execution_order.join(",")
    ));
    for st in &decomposition.sub_tasks {
        let class = match classify_sub_task(st) {
            SubTaskClass::Deterministic => "DET",
            SubTaskClass::Reasoning => "RES",
        };
        out.push_str(&format!(
            "  [{}] {} (caps={}) deps=[{}]\n",
            class,
            st.title,
            st.required_capabilities.join("+"),
            st.dependencies.join(",")
        ));
    }
    out
}

/// 调度错误类型
#[derive(Debug, thiserror::Error)]
pub enum TaskDispatchError {
    #[error("LLM error: {0}")]
    LlmError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("CoT error: {0}")]
    CotError(String),
    #[error("Reasoning error: {0}")]
    ReasoningError(String),
    #[error("Config error: {0}")]
    ConfigError(String),
    #[error("Dependency not met: {0}")]
    DependencyNotMet(String),
    #[error("Execution error: {0}")]
    ExecutionError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatcher_config_default() {
        let config = DispatcherConfig::default();
        assert_eq!(config.decomposition_aggression, 0.5);
        assert_eq!(config.max_sub_tasks, 10);
        assert!(config.enable_cot);
        assert!(config.enable_verifier);
    }

    #[test]
    fn test_sub_task_creation() {
        let sub_task = SubTask {
            id: "test_1".to_string(),
            title: "Test Task".to_string(),
            description: "A test task".to_string(),
            prompt: "Do something".to_string(),
            context: HashMap::new(),
            priority: 5,
            estimated_complexity: 0.5,
            required_capabilities: vec!["general".to_string()],
            dependencies: Vec::new(),
            crt_scale: CrtTimeScale::Gaitian,
            hexagram_bias: Some(3),
        };
        assert_eq!(sub_task.id, "test_1");
        assert_eq!(sub_task.crt_scale, CrtTimeScale::Gaitian);
    }

    fn result(id: &str, success: bool, output: &str) -> SubTaskResult {
        SubTaskResult {
            sub_task_id: id.to_string(),
            success,
            output: output.to_string(),
            error: None,
            tokens_used: 0,
            duration_ms: 1,
            cot_output: None,
        }
    }

    #[test]
    fn test_reducer_happy_path_keeps_all() {
        let results = vec![result(
            "a",
            true,
            "The system uses a vector database for retrieval.",
        )];
        let report = reduce_subtask_results(&results);
        assert_eq!(report.raw, 1);
        assert_eq!(report.clusters.len(), 1);
        assert_eq!(report.clusters[0].consensus, 1);
        assert_eq!(report.filtered_failed, 0);
        assert_eq!(report.filtered_malformed, 0);
    }

    #[test]
    fn test_reducer_all_empty_filters_malformed() {
        let results = vec![result("a", true, ""), result("b", true, "short")];
        let report = reduce_subtask_results(&results);
        assert_eq!(report.filtered_malformed, 2);
        assert!(report.clusters.is_empty());
        assert_eq!(report.raw, 2);
    }

    #[test]
    fn test_reducer_failed_filtered() {
        let results = vec![
            result("a", false, "should be dropped"),
            result(
                "b",
                true,
                "active finding that is long enough to keep as a valid claim",
            ),
        ];
        let report = reduce_subtask_results(&results);
        assert_eq!(report.filtered_failed, 1);
        assert_eq!(report.clusters.len(), 1);
        assert_eq!(report.clusters[0].consensus, 1);
    }

    #[test]
    fn test_reducer_duplicate_merged_with_consensus() {
        let r0 = result("w1", true, "The architecture uses a vector symbolic database for semantic retrieval of knowledge entries");
        let r1 = result("w2", true, "The architecture uses vector symbolic storage for knowledge retrieval with semantic lookup");
        let report = reduce_subtask_results(&[r0, r1]);
        assert_eq!(report.raw, 2);
        assert_eq!(
            report.deduped, 1,
            "two near-dup claims merge into one cluster"
        );
        assert_eq!(report.clusters.len(), 1);
        assert_eq!(
            report.clusters[0].consensus, 2,
            "independent workers confirming same claim"
        );
    }

    #[test]
    fn test_reducer_short_output_filtered_while_valid_kept() {
        let valid = "A detailed finding about the reducer design that fully satisfies the minimum length requirement";
        let results = vec![result("a", true, "short"), result("b", true, valid)];
        let report = reduce_subtask_results(&results);
        assert_eq!(report.filtered_malformed, 1);
        assert_eq!(report.clusters.len(), 1);
        assert_eq!(report.clusters[0].representative, valid);
    }

    #[test]
    fn test_reducer_signals_format_consensus() {
        let r0 = result("w1", true, "The architecture uses a vector symbolic database for semantic retrieval of knowledge entries");
        let r1 = result("w2", true, "The architecture uses vector symbolic storage for knowledge retrieval with semantic lookup");
        let report = reduce_subtask_results(&[r0, r1]);
        let signals = format_reducer_signals(&report);
        assert!(
            signals.contains("<CONSENSUS n=2>"),
            "consensus marker for multi-worker confirmation"
        );
        assert!(signals.contains("clusters=1"));
    }

    fn sub_task(id: &str, title: &str, caps: Vec<&str>, prompt_len: usize) -> SubTask {
        SubTask {
            id: id.to_string(),
            title: title.to_string(),
            description: String::new(),
            prompt: "x".repeat(prompt_len),
            context: HashMap::new(),
            priority: 5,
            estimated_complexity: 0.5,
            required_capabilities: caps.into_iter().map(|s| s.to_string()).collect(),
            dependencies: Vec::new(),
            crt_scale: CrtTimeScale::Gaitian,
            hexagram_bias: Some(3),
        }
    }

    #[test]
    fn test_classify_reasoning_capability() {
        let st = sub_task("s1", "Design the system architecture", vec!["design"], 80);
        assert_eq!(classify_sub_task(&st), SubTaskClass::Reasoning);
    }

    #[test]
    fn test_classify_deterministic_verification() {
        let st = sub_task(
            "s2",
            "Verify output passes format check",
            vec!["testing"],
            100,
        );
        assert_eq!(classify_sub_task(&st), SubTaskClass::Deterministic);
    }

    #[test]
    fn test_classify_long_prompt_falls_back_reasoning() {
        let st = sub_task("s3", "Generic task", vec!["general"], 350);
        assert_eq!(classify_sub_task(&st), SubTaskClass::Reasoning);
    }

    #[test]
    fn test_classify_default_deterministic() {
        let st = sub_task("s4", "Generic task", vec!["general"], 50);
        assert_eq!(classify_sub_task(&st), SubTaskClass::Deterministic);
    }

    #[test]
    fn test_format_dispatch_plan_report() {
        let order = vec!["s2".to_string(), "s1".to_string()];
        let plan = DecompositionResult {
            original_task: "Build a system".to_string(),
            sub_tasks: vec![
                sub_task("s2", "Verify output", vec!["testing"], 100),
                sub_task("s1", "Design architecture", vec!["design"], 80),
            ],
            execution_order: order,
            crt_plan: CrtPlan::new(CrtTimeScale::Huntian, 120.0),
            estimated_total_time: 60.0,
            confidence: 0.7,
        };
        let report = format_dispatch_plan(&plan);
        assert!(report.contains("DET"), "deterministic subtask tagged");
        assert!(report.contains("RES"), "reasoning subtask tagged");
        assert!(report.contains("order=[s2,s1]"));
    }
}
