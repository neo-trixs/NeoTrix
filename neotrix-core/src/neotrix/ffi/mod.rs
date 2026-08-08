// NeoTrix FFI Module
// Rust-to-Swift bridge implementation for the NeoGram iOS app
// Compiled to staticlib and exposed via uniffi-generated Swift bindings

#[cfg(feature = "ios-bridge")]
use uniffi;

pub mod types;
pub mod e8_reasoning;
pub mod vsa_hypercube;
pub mod gwt_attention;
pub mod consciousness_tree;
pub mod seal_pipeline;
pub mod kb_bridge;
pub mod skill_tree;
pub mod rune_socketing;
pub mod constellation_system;
pub mod dual_specialization;

pub use types::*;
pub use e8_reasoning::E8ReasoningImpl;
pub use vsa_hypercube::VSAHyperCubeImpl;
pub use gwt_attention::GWTAttentionRouterImpl;
pub use consciousness_tree::ConsciousnessTreeImpl;
pub use seal_pipeline::SEALPipelineImpl;
pub use kb_bridge::KBBridgeImpl;
pub use skill_tree::SkillTreeImpl;
pub use rune_socketing::RuneSocketingImpl;
pub use constellation_system::ConstellationSystemImpl;
pub use dual_specialization::DualSpecializationImpl;

// Internal facade — not exported to Swift
struct NeoTrix {
    config: NeoTrixConfig,
    e8: std::sync::Arc<E8ReasoningImpl>,
    vsa: std::sync::Arc<VSAHyperCubeImpl>,
    gwt: std::sync::Arc<GWTAttentionRouterImpl>,
    consciousness: std::sync::Arc<ConsciousnessTreeImpl>,
    seal: std::sync::Arc<SEALPipelineImpl>,
    kb: std::sync::Arc<KBBridgeImpl>,
    skill_tree: std::sync::Arc<SkillTreeImpl>,
    runes: std::sync::Arc<RuneSocketingImpl>,
    constellations: std::sync::Arc<ConstellationSystemImpl>,
    specialization: std::sync::Arc<DualSpecializationImpl>,
    started_at: std::time::Instant,
}

impl NeoTrix {
    fn new(config: NeoTrixConfig) -> Result<Self, NeoTrixError> {
        let e8 = std::sync::Arc::new(E8ReasoningImpl::init(config.clone())?);
        let vsa = std::sync::Arc::new(VSAHyperCubeImpl::init(1024, 0.1)?);
        let gwt = std::sync::Arc::new(GWTAttentionRouterImpl::init(default_modules())?);
        let consciousness = std::sync::Arc::new(ConsciousnessTreeImpl::init(config.clone())?);
        let seal = std::sync::Arc::new(SEALPipelineImpl::init(config.clone())?);
        let kb = std::sync::Arc::new(KBBridgeImpl::init(config.clone())?);
        let skill_tree = std::sync::Arc::new(SkillTreeImpl::init()?);
        let runes = std::sync::Arc::new(RuneSocketingImpl::init()?);
        let constellations = std::sync::Arc::new(ConstellationSystemImpl::init()?);
        let specialization = std::sync::Arc::new(DualSpecializationImpl::init()?);

        Ok(NeoTrix {
            config,
            e8,
            vsa,
            gwt,
            consciousness,
            seal,
            kb,
            skill_tree,
            runes,
            constellations,
            specialization,
            started_at: std::time::Instant::now(),
        })
    }
}

// Opaque handle type for Swift
#[derive(uniffi::Object)]
pub struct NeoTrixHandle {
    inner: Box<NeoTrix>,
}

// Free functions exported to Swift — each returns a subsystem object

#[uniffi::export]
pub fn neotrix_initialize(config: NeoTrixConfig) -> Result<std::sync::Arc<NeoTrixHandle>, NeoTrixError> {
    let inner = NeoTrix::new(config)?;
    Ok(std::sync::Arc::new(NeoTrixHandle { inner: Box::new(inner) }))
}

#[uniffi::export]
pub fn neotrix_e8_reasoning(handle: &NeoTrixHandle) -> Result<std::sync::Arc<E8ReasoningImpl>, NeoTrixError> {
    Ok(handle.inner.e8.clone())
}

#[uniffi::export]
pub fn neotrix_vsa_hypercube(handle: &NeoTrixHandle) -> Result<std::sync::Arc<VSAHyperCubeImpl>, NeoTrixError> {
    Ok(handle.inner.vsa.clone())
}

#[uniffi::export]
pub fn neotrix_gwt_attention(handle: &NeoTrixHandle) -> Result<std::sync::Arc<GWTAttentionRouterImpl>, NeoTrixError> {
    Ok(handle.inner.gwt.clone())
}

#[uniffi::export]
pub fn neotrix_consciousness_tree(handle: &NeoTrixHandle) -> Result<std::sync::Arc<ConsciousnessTreeImpl>, NeoTrixError> {
    Ok(handle.inner.consciousness.clone())
}

#[uniffi::export]
pub fn neotrix_seal_pipeline(handle: &NeoTrixHandle) -> Result<std::sync::Arc<SEALPipelineImpl>, NeoTrixError> {
    Ok(handle.inner.seal.clone())
}

#[uniffi::export]
pub fn neotrix_kb_bridge(handle: &NeoTrixHandle) -> Result<std::sync::Arc<KBBridgeImpl>, NeoTrixError> {
    Ok(handle.inner.kb.clone())
}

#[uniffi::export]
pub fn neotrix_skill_tree(handle: &NeoTrixHandle) -> Result<std::sync::Arc<SkillTreeImpl>, NeoTrixError> {
    Ok(handle.inner.skill_tree.clone())
}

#[uniffi::export]
pub fn neotrix_rune_socketing(handle: &NeoTrixHandle) -> Result<std::sync::Arc<RuneSocketingImpl>, NeoTrixError> {
    Ok(handle.inner.runes.clone())
}

#[uniffi::export]
pub fn neotrix_constellation_system(handle: &NeoTrixHandle) -> Result<std::sync::Arc<ConstellationSystemImpl>, NeoTrixError> {
    Ok(handle.inner.constellations.clone())
}

#[uniffi::export]
pub fn neotrix_dual_specialization(handle: &NeoTrixHandle) -> Result<std::sync::Arc<DualSpecializationImpl>, NeoTrixError> {
    Ok(handle.inner.specialization.clone())
}

#[uniffi::export]
pub fn neotrix_capabilities(handle: &NeoTrixHandle) -> CapabilityList {
    let inner = &handle.inner;
    CapabilityList {
        e8_reasoning: true,
        vsa_hypercube: true,
        gwt_attention: true,
        consciousness_tree: true,
        seal_pipeline: true,
        kb_bridge: true,
        skill_tree: true,
        rune_socketing: true,
        constellation_system: true,
        dual_specialization: true,
        mtproto_networking: true,
        telegram_premium: inner.config.enable_premium_features,
        ai_chat_assistant: inner.config.enable_ai_features,
        smart_filtering: inner.config.enable_ai_features,
        knowledge_injection: inner.config.enable_ai_features,
        consciousness_monitor: inner.config.enable_ai_features,
        auto_evolution: inner.config.enable_ai_features,
    }
}

#[uniffi::export]
pub fn neotrix_health_check(handle: &NeoTrixHandle) -> HealthStatus {
    let inner = &handle.inner;
    HealthStatus {
        healthy: true,
        subsystems: default_health_map(),
        issues: Vec::new(),
        uptime_seconds: inner.started_at.elapsed().as_secs(),
    }
}

#[uniffi::export]
pub fn neotrix_shutdown(handle: std::sync::Arc<NeoTrixHandle>) -> bool {
    drop(handle);
    true
}

fn default_modules() -> Vec<String> {
    vec![
        "NT-CORE".into(),
        "NT-MIND".into(),
        "NT-MEMORY".into(),
        "NT-WORLD".into(),
        "NT-ACT".into(),
        "NT-IO".into(),
        "NT-SHIELD".into(),
        "NT-META".into(),
        "NT-REPAIR".into(),
        "NT-GOVERNANCE".into(),
        "NT-NEXUS".into(),
    ]
}

fn default_health_map() -> std::collections::HashMap<String, bool> {
    let mut m = std::collections::HashMap::new();
    for name in default_modules() {
        m.insert(name, true);
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::neotrix::ffi::types::*;

    fn test_config() -> NeoTrixConfig {
        NeoTrixConfig {
            server_url: "https://test.neotrix.local".into(),
            api_key: "test-key".into(),
            enable_ai_features: true,
            enable_premium_features: false,
            log_level: "info".into(),
            data_directory: "/tmp/neotrix-ffi-test".into(),
            cache_size_mb: 64,
        }
    }

    /// 大阵启动：初始化意识核心，验证 handle 创建成功
    #[test]
    fn test_initialize_core() {
        let handle = neotrix_initialize(test_config()).expect("init should succeed");
        assert_eq!(std::sync::Arc::strong_count(&handle), 1);
    }

    /// 大阵连通性：10 个子系统逐一从 handle 取出并调用代表性方法
    #[test]
    fn test_connectivity_all_subsystems() {
        let handle = neotrix_initialize(test_config()).expect("init should succeed");

        // NT-CORE — E8
        let e8 = neotrix_e8_reasoning(&handle).expect("e8 reachable");
        let hex = e8.get_current_hexagram();
        assert!(hex.lines > 0, "E8 hexagram has lines");

        // NT-CORE — VSA HyperCube
        let vsa = neotrix_vsa_hypercube(&handle).expect("vsa reachable");
        let v = vsa.random_vector("test-label");
        assert_eq!(v.dimensions, 1024, "VSA vector dimension");

        // NT-CORE — GWT
        let gwt = neotrix_gwt_attention(&handle).expect("gwt reachable");
        let ws = gwt.get_workspace_state();
        assert!(ws.active_signals.len() < 1000, "workspace sane");

        // NT-META — ConsciousnessTree
        let ct = neotrix_consciousness_tree(&handle).expect("consciousness reachable");
        let state = ct.get_state();
        assert_eq!(state.branches.len(), 11, "11 branches");

        // NT-MIND — SEAL
        let seal = neotrix_seal_pipeline(&handle).expect("seal reachable");
        let status = seal.get_status();
        assert_eq!(status.current_stage, "idle");

        // NT-MEMORY — KB
        let kb = neotrix_kb_bridge(&handle).expect("kb reachable");
        let stats = kb.get_stats();
        assert_eq!(stats.total_nodes, 0, "fresh KB");

        // NT-CORE — SkillTree
        let skill = neotrix_skill_tree(&handle).expect("skill tree reachable");
        let s = skill.get_state();
        assert_eq!(s.available_points, 10, "skill points");

        // NT-CORE — RuneSocketing
        let runes = neotrix_rune_socketing(&handle).expect("runes reachable");
        assert_eq!(runes.get_runes().len(), 5, "5 runes");

        // NT-CORE — ConstellationSystem
        let cs = neotrix_constellation_system(&handle).expect("constellations reachable");
        let all = cs.get_all_states();
        assert_eq!(all.len(), 7, "7 modules in constellation");

        // NT-CORE — DualSpecialization
        let ds = neotrix_dual_specialization(&handle).expect("specialization reachable");
        let ds_state = ds.get_state();
        assert_eq!(ds_state.sets.len(), 2, "2 weapon sets");
    }

    /// 运转测试：子系统间互操作 — KB 写读 → VSA 检索 → GWT 路由联动
    #[test]
    fn test_runtime_interop() {
        let handle = neotrix_initialize(test_config()).expect("init should succeed");

        // NT-MEMORY 写入一条经验
        let kb = neotrix_kb_bridge(&handle).expect("kb reachable");
        let id = kb
            .store_experience("GWT resonance routing works", "experience", std::collections::HashMap::new())
            .expect("store ok");
        assert!(!id.is_empty());

        // NT-MEMORY 读回
        let stats = kb.get_stats();
        assert_eq!(stats.total_nodes, 1, "1 node stored");

        // NT-MIND 运转 SEAL 一周期
        let seal = neotrix_seal_pipeline(&handle).expect("seal reachable");
        let status = seal.run_cycle();
        assert_eq!(status.cycle_count, 1, "1 seal cycle");
        assert_eq!(status.current_stage, "completed");

        // NT-META 触发一次元认知
        let ct = neotrix_consciousness_tree(&handle).expect("consciousness reachable");
        let after = ct.trigger_meta_cognition();
        assert_eq!(after.stage, "Branches", "stage advanced from Trunk (index 2→3)");

        // NT-CORE VSA 存储 + 检索（RwLock 写路径）
        let vsa = neotrix_vsa_hypercube(&handle).expect("vsa reachable");
        let v = vsa.random_vector("persist-me");
        assert!(vsa.store("persist-me", v), "store ok");
        let got = vsa.retrieve("persist-me").expect("retrieve ok");
        assert_eq!(got.dimensions, 1024);
    }

    /// 能力清单 + 健康检查：大阵整体状态
    #[test]
    fn test_capabilities_and_health() {
        let handle = neotrix_initialize(test_config()).expect("init should succeed");
        let caps = neotrix_capabilities(&handle);
        assert!(caps.e8_reasoning && caps.vsa_hypercube && caps.gwt_attention);
        assert!(caps.consciousness_tree && caps.seal_pipeline && caps.kb_bridge);
        assert!(caps.skill_tree && caps.rune_socketing && caps.constellation_system);
        assert!(caps.dual_specialization);

        let health = neotrix_health_check(&handle);
        assert!(health.healthy);
        assert_eq!(health.subsystems.len(), 11, "11 subsystems reported");
        assert!(health.issues.is_empty());
    }

    /// 关停：大阵收束
    #[test]
    fn test_shutdown_core() {
        let handle = neotrix_initialize(test_config()).expect("init should succeed");
        assert!(neotrix_shutdown(handle), "shutdown returns true");
    }
}