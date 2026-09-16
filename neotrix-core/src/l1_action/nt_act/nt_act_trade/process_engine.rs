//! 流程引擎 — 通用流程编排与执行框架
//!
//! 提供流程定义、实例化、步骤执行、状态流转等能力。
//! 支持线性/条件/并行分支等多种流程模式。
//!
//! 核心概念:
//! - `ProcessDefinition`: 流程蓝图，描述步骤与流转规则
//! - `ProcessInstance`: 运行时实例，追踪当前状态与上下文
//! - `StepHandler`: 步骤执行器 trait，业务方实现具体逻辑

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

// ============================================================
// 1. 状态枚举
// ============================================================

/// 流程实例状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessStatus {
    /// 已创建，尚未开始
    Pending,
    /// 执行中
    Running,
    /// 已暂停
    Paused,
    /// 已完成
    Completed,
    /// 执行失败
    Failed,
    /// 已取消
    Cancelled,
}

impl ProcessStatus {
    /// 是否为终态
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            ProcessStatus::Completed | ProcessStatus::Failed | ProcessStatus::Cancelled
        )
    }
}

impl std::fmt::Display for ProcessStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessStatus::Pending => write!(f, "Pending"),
            ProcessStatus::Running => write!(f, "Running"),
            ProcessStatus::Paused => write!(f, "Paused"),
            ProcessStatus::Completed => write!(f, "Completed"),
            ProcessStatus::Failed => write!(f, "Failed"),
            ProcessStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

/// 流程步骤状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    /// 等待执行
    Pending,
    /// 执行中
    Running,
    /// 已完成
    Completed,
    /// 执行失败
    Failed,
    /// 已跳过 (条件分支)
    Skipped,
}

impl StepStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            StepStatus::Completed | StepStatus::Failed | StepStatus::Skipped
        )
    }
}

// ============================================================
// 2. 流程定义
// ============================================================

/// 步骤条件表达式 (简化版: 基于上下文 key 的谓词)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum StepCondition {
    /// 无条件，始终执行
    #[default]
    Always,
    /// 上下文中某 key 等于指定值
    Equals { key: String, value: String },
    /// 上下文中某 key 存在
    KeyExists { key: String },
    /// 自定义条件名 (由 StepHandler 在 validate 中解释)
    Custom(String),
}

/// 流程步骤定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessStep {
    /// 步骤唯一标识
    pub id: String,
    /// 步骤名称
    pub name: String,
    /// 步骤描述
    pub description: String,
    /// 步骤处理器名称 (对应 StepHandler 注册名)
    pub handler: String,
    /// 执行条件
    #[serde(default)]
    pub condition: StepCondition,
    /// 步骤超时 (毫秒), 0 表示无超时
    #[serde(default)]
    pub timeout_ms: u64,
    /// 失败后最大重试次数
    #[serde(default)]
    pub max_retries: u32,
    /// 步骤参数 (传递给 handler 的静态配置)
    #[serde(default)]
    pub params: HashMap<String, String>,
}

/// 流程定义 — 流程蓝图
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessDefinition {
    /// 流程唯一标识
    pub id: String,
    /// 流程名称
    pub name: String,
    /// 流程版本
    pub version: String,
    /// 流程描述
    pub description: String,
    /// 有序步骤列表
    pub steps: Vec<ProcessStep>,
}

impl ProcessDefinition {
    /// 查找步骤定义
    pub fn find_step(&self, step_id: &str) -> Option<&ProcessStep> {
        self.steps.iter().find(|s| s.id == step_id)
    }

    /// 获取步骤在流程中的索引
    pub fn step_index(&self, step_id: &str) -> Option<usize> {
        self.steps.iter().position(|s| s.id == step_id)
    }
}

// ============================================================
// 3. 流程实例
// ============================================================

/// 单个步骤的运行时结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// 步骤 ID
    pub step_id: String,
    /// 步骤状态
    pub status: StepStatus,
    /// 执行耗时 (毫秒)
    pub duration_ms: u64,
    /// 步骤输出 (写入上下文)
    pub output: HashMap<String, String>,
    /// 错误信息 (仅 Failed 状态)
    pub error: Option<String>,
    /// 已重试次数
    pub retries: u32,
}

/// 流程实例 — 运行时状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInstance {
    /// 实例唯一 ID
    pub id: String,
    /// 关联的流程定义 ID
    pub definition_id: String,
    /// 当前状态
    pub status: ProcessStatus,
    /// 当前正在执行的步骤索引
    pub current_step_index: usize,
    /// 流程上下文 (全局共享, 步骤间传递数据)
    pub context: HashMap<String, String>,
    /// 各步骤执行结果
    pub step_results: Vec<StepResult>,
    /// 创建时间 (Unix 毫秒)
    pub created_at: u64,
    /// 最后更新时间 (Unix 毫秒)
    pub updated_at: u64,
}

impl ProcessInstance {
    /// 获取当前步骤结果 (如果已执行)
    pub fn current_step_result(&self) -> Option<&StepResult> {
        self.step_results.last()
    }

    /// 是否所有步骤已完成
    pub fn all_steps_completed(&self, total_steps: usize) -> bool {
        self.step_results
            .iter()
            .filter(|r| r.status != StepStatus::Skipped)
            .all(|r| r.status == StepStatus::Completed)
            && self.step_results.len() >= total_steps
    }
}

// ============================================================
// 4. StepHandler trait
// ============================================================

/// 步骤执行器 trait — 业务方实现此 trait 提供具体步骤逻辑
pub trait StepHandler: Send + Sync {
    /// 执行步骤
    ///
    /// # 参数
    /// - `instance`: 当前流程实例 (只读引用)
    /// - `step`: 当前步骤定义
    /// - `context`: 流程上下文 (可读写, 步骤间数据传递)
    ///
    /// # 返回
    /// - `Ok(output)`: 步骤成功, output 会合并到 context
    /// - `Err(msg)`: 步骤失败
    fn execute(
        &self,
        instance: &ProcessInstance,
        step: &ProcessStep,
        context: &mut HashMap<String, String>,
    ) -> Result<HashMap<String, String>, String>;

    /// 验证步骤前置条件
    ///
    /// 在 execute 之前调用, 返回 false 则跳过该步骤
    fn validate(
        &self,
        instance: &ProcessInstance,
        step: &ProcessStep,
        context: &HashMap<String, String>,
    ) -> bool {
        let _ = (instance, step, context);
        true
    }
}

// ============================================================
// 5. 事件系统
// ============================================================

/// 流程事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessEvent {
    /// 流程已创建
    ProcessCreated {
        instance_id: String,
        definition_id: String,
    },
    /// 流程已启动
    ProcessStarted { instance_id: String },
    /// 步骤开始执行
    StepStarted {
        instance_id: String,
        step_id: String,
    },
    /// 步骤执行完成
    StepCompleted {
        instance_id: String,
        step_id: String,
        success: bool,
    },
    /// 步骤已跳过
    StepSkipped {
        instance_id: String,
        step_id: String,
    },
    /// 流程已完成
    ProcessCompleted { instance_id: String },
    /// 流程失败
    ProcessFailed { instance_id: String, error: String },
    /// 流程已取消
    ProcessCancelled { instance_id: String },
}

/// 事件处理器 trait
pub trait EventHandler: Send + Sync {
    fn handle(&self, event: &ProcessEvent);
}

/// 事件总线 — 简单的发布-订阅机制
pub struct EventBus {
    handlers: Vec<Arc<dyn EventHandler>>,
}

impl Default for EventBus {
    fn default() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }
}

impl EventBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn subscribe(&mut self, handler: Arc<dyn EventHandler>) {
        self.handlers.push(handler);
    }

    pub fn publish(&self, event: &ProcessEvent) {
        for h in &self.handlers {
            h.handle(event);
        }
    }
}

// ============================================================
// 6. ProcessEngine — 流程引擎核心
// ============================================================

/// 流程引擎 — 管理流程定义、实例化与执行
pub struct ProcessEngine {
    /// 已注册的流程定义 (id → definition)
    process_definitions: HashMap<String, ProcessDefinition>,
    /// 运行中的流程实例 (id → instance)
    process_instances: HashMap<String, ProcessInstance>,
    /// 事件总线
    event_bus: EventBus,
    /// 已注册的步骤处理器 (handler_name → handler)
    handlers: HashMap<String, Arc<dyn StepHandler>>,
}

impl Default for ProcessEngine {
    fn default() -> Self {
        Self {
            process_definitions: HashMap::new(),
            process_instances: HashMap::new(),
            event_bus: EventBus::new(),
            handlers: HashMap::new(),
        }
    }
}

impl ProcessEngine {
    pub fn new() -> Self {
        Self::default()
    }

    // ── 定义管理 ──────────────────────────────────────────

    /// 注册流程定义
    pub fn register_definition(&mut self, definition: ProcessDefinition) {
        self.process_definitions
            .insert(definition.id.clone(), definition);
    }

    /// 获取流程定义
    pub fn get_definition(&self, id: &str) -> Option<&ProcessDefinition> {
        self.process_definitions.get(id)
    }

    /// 注册步骤处理器
    pub fn register_handler(&mut self, name: &str, handler: Arc<dyn StepHandler>) {
        self.handlers.insert(name.to_string(), handler);
    }

    // ── 事件订阅 ──────────────────────────────────────────

    /// 订阅流程事件
    pub fn subscribe(&mut self, handler: Arc<dyn EventHandler>) {
        self.event_bus.subscribe(handler);
    }

    // ── 实例管理 ──────────────────────────────────────────

    /// 创建流程实例
    pub fn create_instance(
        &mut self,
        definition_id: &str,
        context: HashMap<String, String>,
    ) -> Result<String, String> {
        let _definition = self
            .process_definitions
            .get(definition_id)
            .ok_or_else(|| format!("Process definition not found: {}", definition_id))?;

        let now = now_millis();
        let instance_id = format!("inst_{}_{}", definition_id, now);

        let instance = ProcessInstance {
            id: instance_id.clone(),
            definition_id: definition_id.to_string(),
            status: ProcessStatus::Pending,
            current_step_index: 0,
            context,
            step_results: Vec::new(),
            created_at: now,
            updated_at: now,
        };

        self.process_instances.insert(instance_id.clone(), instance);

        self.event_bus.publish(&ProcessEvent::ProcessCreated {
            instance_id: instance_id.clone(),
            definition_id: definition_id.to_string(),
        });

        Ok(instance_id)
    }

    /// 获取流程实例
    pub fn get_instance(&self, id: &str) -> Option<&ProcessInstance> {
        self.process_instances.get(id)
    }

    /// 获取流程实例 (可变引用)
    pub fn get_instance_mut(&mut self, id: &str) -> Option<&mut ProcessInstance> {
        self.process_instances.get_mut(id)
    }

    /// 列出所有流程实例
    pub fn list_instances(&self) -> Vec<&ProcessInstance> {
        self.process_instances.values().collect()
    }

    /// 按状态筛选实例
    pub fn list_instances_by_status(&self, status: ProcessStatus) -> Vec<&ProcessInstance> {
        self.process_instances
            .values()
            .filter(|i| i.status == status)
            .collect()
    }

    /// 取消流程实例
    pub fn cancel_instance(&mut self, instance_id: &str) -> Result<(), String> {
        let instance = self
            .process_instances
            .get_mut(instance_id)
            .ok_or_else(|| format!("Instance not found: {}", instance_id))?;

        if instance.status.is_terminal() {
            return Err(format!(
                "Cannot cancel instance in terminal state: {}",
                instance.status
            ));
        }

        instance.status = ProcessStatus::Cancelled;
        instance.updated_at = now_millis();

        self.event_bus.publish(&ProcessEvent::ProcessCancelled {
            instance_id: instance_id.to_string(),
        });

        Ok(())
    }

    /// 删除流程实例
    pub fn delete_instance(&mut self, instance_id: &str) -> Result<ProcessInstance, String> {
        let instance = self
            .process_instances
            .remove(instance_id)
            .ok_or_else(|| format!("Instance not found: {}", instance_id))?;

        if !instance.status.is_terminal() {
            self.process_instances
                .insert(instance_id.to_string(), instance.clone());
            return Err(format!(
                "Cannot delete instance in non-terminal state: {}",
                instance.status
            ));
        }

        Ok(instance)
    }

    // ── 流程执行 ──────────────────────────────────────────

    /// 启动流程实例 (从第一步开始执行)
    pub fn start_instance(&mut self, instance_id: &str) -> Result<(), String> {
        let instance = self
            .process_instances
            .get(instance_id)
            .ok_or_else(|| format!("Instance not found: {}", instance_id))?;

        if instance.status != ProcessStatus::Pending {
            return Err(format!(
                "Instance must be in Pending state to start, got: {}",
                instance.status
            ));
        }

        self.event_bus.publish(&ProcessEvent::ProcessStarted {
            instance_id: instance_id.to_string(),
        });

        // 设置为 Running 并开始执行
        {
            let inst = self.process_instances.get_mut(instance_id).unwrap();
            inst.status = ProcessStatus::Running;
            inst.updated_at = now_millis();
        }

        self.execute_next_step(instance_id)
    }

    /// 执行下一个步骤
    fn execute_next_step(&mut self, instance_id: &str) -> Result<(), String> {
        // 获取定义信息 (先 clone 避免借用冲突)
        let (def_id, step_index, instance_status) = {
            let inst = self
                .process_instances
                .get(instance_id)
                .ok_or_else(|| format!("Instance not found: {}", instance_id))?;
            (
                inst.definition_id.clone(),
                inst.current_step_index,
                inst.status,
            )
        };

        if instance_status != ProcessStatus::Running {
            return Ok(());
        }

        let definition = self
            .process_definitions
            .get(&def_id)
            .ok_or_else(|| format!("Definition not found: {}", def_id))?;

        // 检查是否已完成所有步骤
        if step_index >= definition.steps.len() {
            let inst = self.process_instances.get_mut(instance_id).unwrap();
            inst.status = ProcessStatus::Completed;
            inst.updated_at = now_millis();

            self.event_bus.publish(&ProcessEvent::ProcessCompleted {
                instance_id: instance_id.to_string(),
            });
            return Ok(());
        }

        let step = definition.steps[step_index].clone();

        // 检查条件
        let should_execute = {
            let inst = self.process_instances.get(instance_id).unwrap();
            evaluate_condition(&step.condition, &inst.context)
        };

        if !should_execute {
            // 跳过步骤
            let result = StepResult {
                step_id: step.id.clone(),
                status: StepStatus::Skipped,
                duration_ms: 0,
                output: HashMap::new(),
                error: None,
                retries: 0,
            };

            let inst = self.process_instances.get_mut(instance_id).unwrap();
            inst.step_results.push(result);
            inst.current_step_index += 1;
            inst.updated_at = now_millis();

            self.event_bus.publish(&ProcessEvent::StepSkipped {
                instance_id: instance_id.to_string(),
                step_id: step.id,
            });

            return self.execute_next_step(instance_id);
        }

        // 获取处理器
        let handler = self
            .handlers
            .get(&step.handler)
            .ok_or_else(|| format!("Handler not found: {}", step.handler))?;

        // 验证
        let (instance_snapshot, context_snapshot) = {
            let inst = self.process_instances.get(instance_id).unwrap();
            (inst.clone(), inst.context.clone())
        };

        if !handler.validate(&instance_snapshot, &step, &context_snapshot) {
            let result = StepResult {
                step_id: step.id.clone(),
                status: StepStatus::Skipped,
                duration_ms: 0,
                output: HashMap::new(),
                error: Some("Validation failed".into()),
                retries: 0,
            };

            let inst = self.process_instances.get_mut(instance_id).unwrap();
            inst.step_results.push(result);
            inst.current_step_index += 1;
            inst.updated_at = now_millis();

            return self.execute_next_step(instance_id);
        }

        // 执行步骤 (带重试)
        self.event_bus.publish(&ProcessEvent::StepStarted {
            instance_id: instance_id.to_string(),
            step_id: step.id.clone(),
        });

        let start = now_millis();
        let max_retries = step.max_retries;
        let handler_name = step.handler.clone();
        let step_id = step.id.clone();

        let mut last_error = None;
        let mut succeeded = false;
        let mut output = HashMap::new();

        for attempt in 0..=max_retries {
            let handler = self.handlers.get(&handler_name).unwrap();
            let inst = self.process_instances.get(instance_id).unwrap();
            let mut ctx = inst.context.clone();

            match handler.execute(inst, &step, &mut ctx) {
                Ok(out) => {
                    output = out;
                    succeeded = true;
                    // 将 output 合并到上下文
                    let inst = self.process_instances.get_mut(instance_id).unwrap();
                    for (k, v) in &output {
                        inst.context.insert(k.clone(), v.clone());
                    }
                    break;
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_retries {
                        // 简单重试，不做 backoff
                        continue;
                    }
                }
            }
        }

        let duration = now_millis().saturating_sub(start);
        let result = StepResult {
            step_id: step_id.clone(),
            status: if succeeded {
                StepStatus::Completed
            } else {
                StepStatus::Failed
            },
            duration_ms: duration,
            output,
            error: last_error,
            retries: max_retries,
        };

        let success = succeeded;
        {
            let inst = self.process_instances.get_mut(instance_id).unwrap();
            inst.step_results.push(result);
            inst.updated_at = now_millis();

            if succeeded {
                inst.current_step_index += 1;
            } else {
                inst.status = ProcessStatus::Failed;
            }
        }

        self.event_bus.publish(&ProcessEvent::StepCompleted {
            instance_id: instance_id.to_string(),
            step_id,
            success,
        });

        if !success {
            self.event_bus.publish(&ProcessEvent::ProcessFailed {
                instance_id: instance_id.to_string(),
                error: "Step execution failed".into(),
            });
            return Ok(());
        }

        // 继续下一步
        self.execute_next_step(instance_id)
    }

    /// 暂停流程
    pub fn pause_instance(&mut self, instance_id: &str) -> Result<(), String> {
        let inst = self
            .process_instances
            .get_mut(instance_id)
            .ok_or_else(|| format!("Instance not found: {}", instance_id))?;

        if inst.status != ProcessStatus::Running {
            return Err(format!(
                "Instance must be Running to pause, got: {}",
                inst.status
            ));
        }

        inst.status = ProcessStatus::Paused;
        inst.updated_at = now_millis();
        Ok(())
    }

    /// 恢复流程
    pub fn resume_instance(&mut self, instance_id: &str) -> Result<(), String> {
        let status = {
            let inst = self
                .process_instances
                .get(instance_id)
                .ok_or_else(|| format!("Instance not found: {}", instance_id))?;
            inst.status
        };

        if status != ProcessStatus::Paused {
            return Err(format!(
                "Instance must be Paused to resume, got: {}",
                status
            ));
        }

        {
            let inst = self.process_instances.get_mut(instance_id).unwrap();
            inst.status = ProcessStatus::Running;
            inst.updated_at = now_millis();
        }

        self.execute_next_step(instance_id)
    }

    /// 从指定步骤重试流程
    pub fn retry_from_step(&mut self, instance_id: &str, step_index: usize) -> Result<(), String> {
        let def_id = {
            let inst = self
                .process_instances
                .get(instance_id)
                .ok_or_else(|| format!("Instance not found: {}", instance_id))?;
            if inst.status != ProcessStatus::Failed {
                return Err("Can only retry from Failed state".into());
            }
            inst.definition_id.clone()
        };

        let definition = self
            .process_definitions
            .get(&def_id)
            .ok_or_else(|| format!("Definition not found: {}", def_id))?;

        if step_index >= definition.steps.len() {
            return Err("Step index out of range".into());
        }

        {
            let inst = self.process_instances.get_mut(instance_id).unwrap();
            inst.status = ProcessStatus::Running;
            inst.current_step_index = step_index;
            // 保留已通过的步骤结果，清除后续结果
            inst.step_results.truncate(step_index);
            inst.updated_at = now_millis();
        }

        self.execute_next_step(instance_id)
    }

    // ── 查询方法 ──────────────────────────────────────────

    /// 获取流程执行进度 (0.0 ~ 1.0)
    pub fn get_progress(&self, instance_id: &str) -> Option<f64> {
        let inst = self.process_instances.get(instance_id)?;
        let def = self.process_definitions.get(&inst.definition_id)?;
        if def.steps.is_empty() {
            return Some(1.0);
        }
        Some(inst.current_step_index as f64 / def.steps.len() as f64)
    }

    /// 获取流程总耗时 (毫秒)
    pub fn get_total_duration_ms(&self, instance_id: &str) -> Option<u64> {
        let inst = self.process_instances.get(instance_id)?;
        Some(inst.step_results.iter().map(|r| r.duration_ms).sum())
    }
}

// ============================================================
// 7. 辅助函数
// ============================================================

/// 评估步骤条件
pub(crate) fn evaluate_condition(condition: &StepCondition, context: &HashMap<String, String>) -> bool {
    match condition {
        StepCondition::Always => true,
        StepCondition::Equals { key, value } => context.get(key.as_str()) == Some(value),
        StepCondition::KeyExists { key } => context.contains_key(key.as_str()),
        StepCondition::Custom(_) => true, // Custom 条件由 handler.validate 处理
    }
}

/// 获取当前时间戳 (毫秒)
fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

// ============================================================
// 8. 内置示例处理器
// ============================================================

/// 无操作处理器 (用于测试或占位步骤)
pub struct NoOpHandler;

impl StepHandler for NoOpHandler {
    fn execute(
        &self,
        _instance: &ProcessInstance,
        _step: &ProcessStep,
        _context: &mut HashMap<String, String>,
    ) -> Result<HashMap<String, String>, String> {
        Ok(HashMap::new())
    }
}

// ============================================================
// 9. 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    struct LogHandler;

    impl StepHandler for LogHandler {
        fn execute(
            &self,
            _instance: &ProcessInstance,
            step: &ProcessStep,
            context: &mut HashMap<String, String>,
        ) -> Result<HashMap<String, String>, String> {
            let mut output = HashMap::new();
            output.insert("executed".into(), step.id.clone());
            context.insert(format!("{}_done", step.id), "true".into());
            Ok(output)
        }
    }

    struct FailOnceHandler {
        attempts: std::sync::atomic::AtomicU32,
    }

    impl StepHandler for FailOnceHandler {
        fn execute(
            &self,
            _instance: &ProcessInstance,
            _step: &ProcessStep,
            _context: &mut HashMap<String, String>,
        ) -> Result<HashMap<String, String>, String> {
            let count = self
                .attempts
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if count == 0 {
                Err("Transient failure".into())
            } else {
                Ok(HashMap::new())
            }
        }
    }

    struct CollectEventsHandler {
        events: std::sync::Mutex<Vec<ProcessEvent>>,
    }

    impl CollectEventsHandler {
        fn new() -> Arc<Self> {
            Arc::new(Self {
                events: std::sync::Mutex::new(Vec::new()),
            })
        }

        fn events(&self) -> Vec<ProcessEvent> {
            self.events.lock().unwrap().clone()
        }
    }

    impl EventHandler for CollectEventsHandler {
        fn handle(&self, event: &ProcessEvent) {
            self.events.lock().unwrap().push(event.clone());
        }
    }

    fn make_definition() -> ProcessDefinition {
        ProcessDefinition {
            id: "test_proc".into(),
            name: "Test Process".into(),
            version: "1.0".into(),
            description: "A test process".into(),
            steps: vec![
                ProcessStep {
                    id: "step1".into(),
                    name: "Step 1".into(),
                    description: "First step".into(),
                    handler: "log".into(),
                    condition: StepCondition::Always,
                    timeout_ms: 0,
                    max_retries: 0,
                    params: HashMap::new(),
                },
                ProcessStep {
                    id: "step2".into(),
                    name: "Step 2".into(),
                    description: "Second step".into(),
                    handler: "log".into(),
                    condition: StepCondition::Always,
                    timeout_ms: 0,
                    max_retries: 0,
                    params: HashMap::new(),
                },
            ],
        }
    }

    #[test]
    fn test_process_status_terminal() {
        assert!(ProcessStatus::Completed.is_terminal());
        assert!(ProcessStatus::Failed.is_terminal());
        assert!(ProcessStatus::Cancelled.is_terminal());
        assert!(!ProcessStatus::Pending.is_terminal());
        assert!(!ProcessStatus::Running.is_terminal());
        assert!(!ProcessStatus::Paused.is_terminal());
    }

    #[test]
    fn test_step_status_terminal() {
        assert!(StepStatus::Completed.is_terminal());
        assert!(StepStatus::Failed.is_terminal());
        assert!(StepStatus::Skipped.is_terminal());
        assert!(!StepStatus::Pending.is_terminal());
        assert!(!StepStatus::Running.is_terminal());
    }

    #[test]
    fn test_definition_lookup() {
        let def = make_definition();
        assert!(def.find_step("step1").is_some());
        assert!(def.find_step("nonexistent").is_none());
        assert_eq!(def.step_index("step1"), Some(0));
        assert_eq!(def.step_index("step2"), Some(1));
    }

    #[test]
    fn test_evaluate_condition() {
        let mut ctx = HashMap::new();
        ctx.insert("status".into(), "active".into());

        assert!(evaluate_condition(&StepCondition::Always, &ctx));
        assert!(evaluate_condition(
            &StepCondition::Equals {
                key: "status".into(),
                value: "active".into(),
            },
            &ctx
        ));
        assert!(!evaluate_condition(
            &StepCondition::Equals {
                key: "status".into(),
                value: "inactive".into(),
            },
            &ctx
        ));
        assert!(evaluate_condition(
            &StepCondition::KeyExists {
                key: "status".into(),
            },
            &ctx
        ));
        assert!(!evaluate_condition(
            &StepCondition::KeyExists {
                key: "missing".into(),
            },
            &ctx
        ));
    }

    #[test]
    fn test_full_lifecycle() {
        let mut engine = ProcessEngine::new();
        engine.register_definition(make_definition());
        engine.register_handler("log", Arc::new(LogHandler));

        let ctx = HashMap::new();
        let inst_id = engine.create_instance("test_proc", ctx).unwrap();

        assert_eq!(
            engine.get_instance(&inst_id).unwrap().status,
            ProcessStatus::Pending
        );

        engine.start_instance(&inst_id).unwrap();

        let inst = engine.get_instance(&inst_id).unwrap();
        assert_eq!(inst.status, ProcessStatus::Completed);
        assert_eq!(inst.step_results.len(), 2);
        assert!(inst
            .step_results
            .iter()
            .all(|r| r.status == StepStatus::Completed));

        assert_eq!(engine.get_progress(&inst_id), Some(1.0));
    }

    #[test]
    fn test_pause_resume() {
        let mut engine = ProcessEngine::new();
        engine.register_definition(make_definition());
        engine.register_handler("log", Arc::new(LogHandler));

        let inst_id = engine.create_instance("test_proc", HashMap::new()).unwrap();
        engine.start_instance(&inst_id).unwrap();
        engine.pause_instance(&inst_id).unwrap();
        assert_eq!(
            engine.get_instance(&inst_id).unwrap().status,
            ProcessStatus::Paused
        );

        engine.resume_instance(&inst_id).unwrap();
        assert_eq!(
            engine.get_instance(&inst_id).unwrap().status,
            ProcessStatus::Completed
        );
    }

    #[test]
    fn test_cancel() {
        let mut engine = ProcessEngine::new();
        engine.register_definition(make_definition());
        engine.register_handler("log", Arc::new(LogHandler));

        let inst_id = engine.create_instance("test_proc", HashMap::new()).unwrap();
        engine.start_instance(&inst_id).unwrap();
        engine.cancel_instance(&inst_id).unwrap();
        assert_eq!(
            engine.get_instance(&inst_id).unwrap().status,
            ProcessStatus::Cancelled
        );
    }

    #[test]
    fn test_cancel_terminal_fails() {
        let mut engine = ProcessEngine::new();
        engine.register_definition(make_definition());
        engine.register_handler("log", Arc::new(LogHandler));

        let inst_id = engine.create_instance("test_proc", HashMap::new()).unwrap();
        engine.start_instance(&inst_id).unwrap();
        assert!(engine.cancel_instance(&inst_id).is_err());
    }

    #[test]
    fn test_conditional_step_skip() {
        let def = ProcessDefinition {
            id: "cond_proc".into(),
            name: "Conditional".into(),
            version: "1.0".into(),
            description: "".into(),
            steps: vec![
                ProcessStep {
                    id: "s1".into(),
                    name: "S1".into(),
                    description: "".into(),
                    handler: "log".into(),
                    condition: StepCondition::Always,
                    timeout_ms: 0,
                    max_retries: 0,
                    params: HashMap::new(),
                },
                ProcessStep {
                    id: "s2".into(),
                    name: "S2".into(),
                    description: "".into(),
                    handler: "log".into(),
                    condition: StepCondition::Equals {
                        key: "run_s2".into(),
                        value: "true".into(),
                    },
                    timeout_ms: 0,
                    max_retries: 0,
                    params: HashMap::new(),
                },
            ],
        };

        let mut engine = ProcessEngine::new();
        engine.register_definition(def);
        engine.register_handler("log", Arc::new(LogHandler));

        let mut ctx = HashMap::new();
        ctx.insert("run_s2".into(), "false".into());

        let inst_id = engine.create_instance("cond_proc", ctx).unwrap();
        engine.start_instance(&inst_id).unwrap();

        let inst = engine.get_instance(&inst_id).unwrap();
        assert_eq!(inst.status, ProcessStatus::Completed);
        assert_eq!(inst.step_results.len(), 2);
        assert_eq!(inst.step_results[0].status, StepStatus::Completed);
        assert_eq!(inst.step_results[1].status, StepStatus::Skipped);
    }

    #[test]
    fn test_retry_from_step() {
        let mut engine = ProcessEngine::new();
        engine.register_definition(make_definition());
        engine.register_handler("log", Arc::new(LogHandler));

        let inst_id = engine.create_instance("test_proc", HashMap::new()).unwrap();
        engine.start_instance(&inst_id).unwrap();

        // Simulate failure by manually setting status
        {
            let inst = engine.get_instance_mut(&inst_id).unwrap();
            inst.status = ProcessStatus::Failed;
        }

        engine.retry_from_step(&inst_id, 0).unwrap();
        assert_eq!(
            engine.get_instance(&inst_id).unwrap().status,
            ProcessStatus::Completed
        );
    }

    #[test]
    fn test_retry_requires_failure() {
        let mut engine = ProcessEngine::new();
        engine.register_definition(make_definition());
        engine.register_handler("log", Arc::new(LogHandler));

        let inst_id = engine.create_instance("test_proc", HashMap::new()).unwrap();
        engine.start_instance(&inst_id).unwrap();

        assert!(engine.retry_from_step(&inst_id, 0).is_err());
    }

    #[test]
    fn test_delete_requires_terminal() {
        let mut engine = ProcessEngine::new();
        engine.register_definition(make_definition());
        engine.register_handler("log", Arc::new(LogHandler));

        let inst_id = engine.create_instance("test_proc", HashMap::new()).unwrap();
        assert!(engine.delete_instance(&inst_id).is_err());
    }

    #[test]
    fn test_events_published() {
        let handler = CollectEventsHandler::new();
        let handler_clone = handler.clone();

        let mut engine = ProcessEngine::new();
        engine.register_definition(make_definition());
        engine.register_handler("log", Arc::new(LogHandler));
        engine.subscribe(handler_clone);

        let inst_id = engine.create_instance("test_proc", HashMap::new()).unwrap();
        engine.start_instance(&inst_id).unwrap();

        let events = handler.events();
        assert!(events.len() >= 5); // Created, Started, StepStarted x2, StepCompleted x2, Completed
    }

    #[test]
    fn test_unknown_definition() {
        let mut engine = ProcessEngine::new();
        assert!(engine
            .create_instance("nonexistent", HashMap::new())
            .is_err());
    }

    #[test]
    fn test_unknown_handler() {
        let mut engine = ProcessEngine::new();
        engine.register_definition(make_definition());
        // No handler registered

        let inst_id = engine.create_instance("test_proc", HashMap::new()).unwrap();
        assert!(engine.start_instance(&inst_id).is_err());
    }

    #[test]
    fn test_list_instances_by_status() {
        let mut engine = ProcessEngine::new();
        engine.register_definition(make_definition());
        engine.register_handler("log", Arc::new(LogHandler));

        engine.create_instance("test_proc", HashMap::new()).unwrap();
        engine.create_instance("test_proc", HashMap::new()).unwrap();

        assert_eq!(
            engine
                .list_instances_by_status(ProcessStatus::Pending)
                .len(),
            2
        );
        assert_eq!(
            engine
                .list_instances_by_status(ProcessStatus::Running)
                .len(),
            0
        );
    }
}
