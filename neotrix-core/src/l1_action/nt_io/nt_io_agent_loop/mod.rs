//! # NT-IO AgentLoop — NeoTrix 作为主体的对话驱动循环
//!
//! 架构目标（CORE 反转关系）：
//!   - **NeoTrix 系统** 是和你对话的主体（持有状态、工具、决策逻辑）
//!   - **LLM** 是它调用的一个后端能力（"推理生成函数"）
//!
//! AgentLoop 不依赖具体 provider 类型，只依赖 `LlmProvider` trait，
//! 生产环境注入 `GatewayV2`（含路由/熔断/限流），测试注入 mock。
//!
//! 循环契约：
//!   1. 追加用户消息到会话历史
//!   2. 构建 `LlmRequest`（携带工具定义 + 完整历史）
//!   3. 调用 LLM 后端
//!   4. 若响应带 tool_calls → 执行每个工具 → 结果以 Role::Tool 消息回填 → 回到 2
//!   5. 无工具调用 → 返回最终回答，追加 Assistant 消息
//!
//! 安全上限：`max_tool_rounds` 防止工具循环死循环；`max_history` 防止上下文无限膨胀。
//!
//! 单文件转目录门面（纯搬移，行为零变更），外部路径不变：
//! `crate::l1_action::nt_io::nt_io_agent_loop::{AgentLoop, ToolInvocation}`。
//! 按循环阶段切分：`nt_loop_types`（类型/状态）/`nt_loop_handle`（句柄/控制）/
//! `nt_loop_core`（主循环）/`nt_loop_step`（单步）+ `nt_loop_tests`（单测）。

pub mod nt_loop_core;
pub mod nt_loop_handle;
pub mod nt_loop_step;
pub mod nt_loop_types;
#[cfg(test)]
pub mod nt_loop_canary_tests;
#[cfg(test)]
pub mod nt_loop_tests;

pub use nt_loop_types::*;
