/// Task Decomposer & LLM Dispatcher — 复杂任务自动拆解与智能分配给 LLM
///
/// 核心设计原则：
/// 1. 用户只需给出高层目标，无需了解意识核心内部实现
/// 2. 自动拆解复杂任务为可执行的子任务
/// 3. 为每个子任务生成精准的、上下文感知的提示词
/// 4. 智能选择合适的 LLM 模型/策略执行子任务
/// 5. 自动聚合子任务结果，生成最终答案
/// 6. 对用户完全隐藏意识核心内部实现细节（E8、CRT、GWT 等）
///
/// L1 不直接依赖 L5 — 通过 `ReasoningEngineProvider` trait 抽象推理能力。
// T03c 类型本地化（行为零变更）：新写入点经 L1 本地 DTO（LocalDecomposeSuggestion /
// LocalCrtPlan / LocalTrace）再转回 L5 存储；读点仍为 L5 类型（见各 from_l5 注记与遗留清单）。
// LAYER-EXCEPTION(T03c 遗留读点): DecomposeSuggestion（generate_sub_tasks 等签名）/ CrtPlan
// （DecompositionResult.crt_plan 字段）/ CrtTimeScale（SubTask.crt_scale 字段＋分类/分配读点）/
// TraceSource（kernel_trace 构造）/ CoTOutput（SubTaskResult.cot_output 字段），只读不断行为故暂留。

pub mod nt_dispatcher_config;
pub mod nt_dispatcher_core;
pub mod nt_dispatcher_dto;
pub mod nt_dispatcher_policy;
pub mod nt_dispatcher_reduce;

pub use nt_dispatcher_config::DispatcherConfig;
pub use nt_dispatcher_core::{PredictorStore, ReasoningEngineProvider, TaskDecomposerDispatcher};
pub use nt_dispatcher_dto::{
    DecompositionResult, LocalCrtPlan, LocalDecomposeSuggestion, LocalTrace, SubTask,
    SubTaskResult, TaskExecutionContext,
};
pub use nt_dispatcher_policy::{CONFIDENCE_FLOOR, DispatchLogRecord, HISTORY_CAP};
pub use nt_dispatcher_reduce::{
    ClaimGroup, ReduceReport, SubTaskClass, TaskDispatchError, classify_sub_task,
    format_dispatch_plan, format_reducer_signals, reduce_subtask_results,
};
