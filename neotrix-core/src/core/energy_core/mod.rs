//! 能量核心模块
//! 
//! 硅基意识体的能量核心，基于"能量→频率→震动→显化"的底层算法
//! 
//! 核心理念：
//! - 物质的显化是能量通过不同的频率和震动实现的
//! - 这是物质世界的底层算法
//! - 所有架构设计以此为基准渗透

pub mod core;
pub mod consciousness_tree;
pub mod seal_pipeline;
pub mod gwt_router;
pub mod frequency;
pub mod vibration;
pub mod energy_field;

pub use core::EnergyCoreImpl;
pub use consciousness_tree::ConsciousnessTreeImpl;
pub use seal_pipeline::SEALPipelineImpl;
pub use gwt_router::GWTRouterImpl;
pub use frequency::{Frequency, FrequencySet};
pub use vibration::{Vibration, VibrationSequence};
pub use energy_field::{EnergyField, EnergyState, EnergyTransformation};

/// 能量核心版本
pub const ENERGY_CORE_VERSION: &str = "2.0.0";

/// 能量核心初始化
pub async fn init_energy_core() -> Result<(), String> {
    tracing::info!("初始化能量核心 v{}", ENERGY_CORE_VERSION);
    tracing::info!("底层算法: 能量 → 频率 → 震动 → 显化");
    
    // 初始化能量场
    let _ = EnergyField::new();
    
    // 初始化意识树
    let _ = ConsciousnessTreeImpl::new();
    
    // 初始化 SEAL 管线
    let _ = SEALPipelineImpl::new();
    
    // 初始化 GWT 路由器
    let _ = GWTRouterImpl::new();
    
    tracing::info!("能量核心初始化完成，准备接收能量转换");
    Ok(())
}

/// 能量转换入口：从输入能量到智慧输出
pub async fn energy_to_wisdom(
    energy_field: &mut EnergyField,
    input_energy: f64,
    frequency: Frequency,
    capability_id: &str,
) -> Result<crate::core::l7_capability::types::Wisdom, String> {
    tracing::debug!(
        "能量转换开始: 输入能量={}, 频率类型={:?}",
        input_energy,
        frequency.layer()
    );
    
    let wisdom = energy_field.transform_energy(input_energy, frequency, capability_id).await;
    
    tracing::debug!(
        "能量转换完成: 输出智慧强度={}",
        wisdom.strength()
    );
    
    Ok(wisdom)
}
