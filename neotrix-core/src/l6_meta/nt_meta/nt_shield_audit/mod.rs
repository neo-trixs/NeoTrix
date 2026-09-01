//! Shield Audit — NT-SHIELD 审计层
//! 
//! 吸收的能力模块 (cloudflare/security-audit-skill):
//! - recon_phase: Phase 1 三并行研究代理
//! - hunt_phase: Phase 2 并行狩猎 + 12 角度 + 域伴侣路由
//! - validate_phase: Phase 3 合并重复 + 对抗性反证 + 双路径验证 (R-P106)
//! - attack_class_registry: 8 攻击类别 + 4 域伴侣路由

pub mod recon_phase;
pub mod hunt_phase;
pub mod validate_phase;
pub mod attack_class_registry;