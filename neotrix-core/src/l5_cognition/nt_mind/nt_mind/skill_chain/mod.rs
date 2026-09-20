//! 技能链编排模块 — Skill Chain Orchestration
//!
//! 四文件架构:
//! - `skill_step`: 步骤定义 (Brainstorm/Plan/Tdd/Review)
//! - `skill_chain`: 链结构 + 状态管理
//! - `chain_executor`: 执行器 + 暂停/恢复
//! - `chain_config`: 配置 (超时/重试/自动推进)
//!
//! R-P123: 按认知域拆分步骤类型
//! R-P124: 配置集中管理，支持 Default trait

pub mod chain_config;
pub mod chain_executor;
pub mod skill_chain;
pub mod skill_step;

pub use chain_config::ChainConfig;
pub use chain_executor::{ChainContext, ChainExecutor, ChainResult, MockStepExecutor, StepExecutor, StepExecution};
pub use skill_chain::{ChainStatus, SkillChain};
pub use skill_step::{SkillStep, StepInputType, StepOutputType};
