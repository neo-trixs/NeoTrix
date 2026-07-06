//! # L0 — 基底层 (Substrate)
//!
//! 物理硬件承载。所有上层运行的基础。
//! 科幻映射: Matrix 锡安 / GitS 义体硬件
//!
//! ## 规则
//! - L0 是所有上层运行的基础
//! - L0 的变更影响全局 — 必须经过完整的大过滤器验证
//! - L0 不包含任何推理逻辑

pub use crate::core::nt_core_deploy as deploy;
pub use crate::core::nt_core_deploy_cache as deploy_cache;

pub use crate::core::nt_core_deploy::{
    EdgeDeployPipeline, Quantizer, HardwareDetector, AotCompiler,
    Quantization, OsType, AotTarget, HardwareProfile, LoraAdapter,
    QuantizedModel, AotResult, DeployReport,
    AWQQuantization, AWQConfig, GGUFLevel, GGUFQuantization, GGUFConfig,
    QuantizationPipeline,
    PowerState, PowerProfile, HardwarePowerProfile, PowerThermalModel,
    AneProgramCache, CacheEntry, CachePolicy,
};
