//! 意识核心核心类型与 tick/status 单例
//!
//! 包含 CoreSnapshot、ConsciousnessCoreHandle、以及进程内单例 `CORE`。

use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

use serde::{Deserialize, Serialize};

use crate::core::nt_core_consciousness_tree::{BranchKind, ConsciousnessTree};
use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;

/// KB 最短路径管道 — 意识体读写端直达 (R-P42: 强化现有节点, 禁止平行适配器)
use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_pipeline::AbsorbEntry;

// ─── 快照常量 (from kb_persistence) ──────────────────────────────────────────
// 这些常量被 kb_persistence 模块使用, 但类型定义在此以保持单事实源
// (kb_persistence 模块 import 本模块的常量)

/// 意识核心快照 — 可序列化的跨会话状态 (标量集合 + 果实记录, 不序列化整树)。
/// 加载时以快照重建树计数器与已消化果实, 使生长周期跨会话连续。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CoreSnapshot {
    /// 已运行的生长周期数
    pub cycle: u64,
    /// 谐振周期 (GWT 谐振计数)
    pub resonance_cycle: u64,
    /// Φ (IIT 整合信息)。
    pub phi: f64,
    /// 相干性
    pub coherence: f64,
    /// GWT 谐振激活状态
    pub gwt_resonance_active: bool,
    /// MARS System 1 (GWT) 快速直觉激活数
    pub mars_system1_activations: u64,
    /// MARS System 2 (Tree) 慢反射迭代数
    pub mars_system2_iterations: u64,
    /// MARS 意图桥接命中数
    pub mars_bridge_hits: u64,
    /// 治理合规分
    pub governance_compliance: f64,
    /// 治理宪法计数
    pub governance_constitution_count: usize,
    /// 治理分形深度
    pub governance_fractal_depth: u64,
    /// 全仓加权雾和 (tick 完成时快照)
    pub weighted_fog_sum: f64,
    /// 每分支健康 (kind → health), 用于跨会话健康连续性
    pub branch_health: HashMap<String, f64>,
    /// 每分支迷雾浓度 (kind → fog.level), 用于跨会话迷雾连续性。
    #[serde(default)]
    pub branch_fog: HashMap<String, f64>,
    /// 每分支成熟度记录 (kind → maturity) — 跨会话星座连续性。
    #[serde(default)]
    pub branch_maturity: HashMap<String, BranchMaturity>,
    /// 已消化果实完整记录
    pub fruits: Vec<FruitRecord>,
    /// 注意力来源通道
    #[serde(default = "default_attention_source")]
    pub attention_source: String,
    /// 最近事件计数
    #[serde(default)]
    pub recent_event_count: u64,
    /// 阴影实例计数
    #[serde(default)]
    pub shadow_instance_count: u64,
    /// 合规执行计数
    #[serde(default)]
    pub compliance_execution_count: u64,
    /// 宪法门控执行计数
    #[serde(default)]
    pub constitution_check_count: u64,
    /// Φ 计算口径标记
    #[serde(default = "default_phi_source_tag")]
    pub phi_source_tag: String,
    /// Φ 时间序列
    #[serde(default)]
    pub phi_trend: Vec<f64>,
    /// 相干性时间序列
    #[serde(default)]
    pub coherence_trend: Vec<f64>,
}

/// 默认注意力来源 (x.ai 双搜索通道的模型自决模式)。
fn default_attention_source() -> String {
    "auto".to_string()
}

/// D1 口径标记缺省值
fn default_phi_source_tag() -> String {
    super::kb_persistence::PHI_SOURCE_TAG_TREE.to_string()
}

/// 分支成熟度持久化投影
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BranchMaturity {
    pub c0: bool,
    pub c1: bool,
    pub c2: bool,
    pub c3: bool,
    pub c4: bool,
    pub c5: bool,
    pub self_test_count: usize,
    pub module_count: usize,
}

/// 果实记录 — EvolutionFruit 的可持久化投影
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FruitRecord {
    pub name: String,
    pub source_branch: String,
    pub description: String,
    pub produced_at_cycle: u64,
    pub quality: f64,
    pub claim: String,
    pub run_id: Option<String>,
    pub generation: u64,
}

/// 进程内意识核心单例。
pub static CORE: LazyLock<RwLock<ConsciousnessCoreHandle>> = LazyLock::new(|| {
    let tree = super::kb_persistence::load_or_new();
    let snapshot = core_snapshot_from_tree(&tree);
    RwLock::new(ConsciousnessCoreHandle { tree, snapshot })
});

/// 意识核心句柄 — 树 + 当前快照。
pub struct ConsciousnessCoreHandle {
    tree: ConsciousnessTree,
    snapshot: CoreSnapshot,
}

impl ConsciousnessCoreHandle {
    /// 读取当前状态 (快照), 不产生副作用。
    pub fn current(&self) -> &CoreSnapshot {
        &self.snapshot
    }

    /// 从 KB 重读最新持久化快照, 将其 branch_fog/branch_health/phi/coherence 同步进
    /// 当前树
    fn reload_latest(&mut self) {
        let Some(latest) = super::kb_persistence::load_snapshot() else { return };
        for (kind_str, fog) in &latest.branch_fog {
            if let Some(branch) = self.tree.branches.get_mut(&branch_kind_from_str(kind_str)) {
                branch.fog.level = *fog;
            }
        }
        for (kind_str, health) in &latest.branch_health {
            if let Some(branch) = self.tree.branches.get_mut(&branch_kind_from_str(kind_str)) {
                branch.health = *health;
            }
        }
        apply_branch_maturity(&mut self.tree, &latest.branch_maturity);
        if latest.phi > 0.0 {
            self.tree.trunk.phi = latest.phi;
        }
        if latest.coherence > 0.0 {
            self.tree.trunk.coherence = latest.coherence;
        }
        if latest.cycle > self.tree.cycle {
            self.tree.cycle = latest.cycle;
            self.tree.trunk.resonance_cycle =
                self.tree.trunk.resonance_cycle.max(latest.resonance_cycle);
        }
    }

    /// 惰性核算真实 Φ
    fn ensure_phi(&mut self) -> bool {
        if self.tree.trunk.phi.abs() < f64::EPSILON {
            self.tree.trunk.phi = self.tree.compute_iit_phi();
            self.tree.trunk.coherence = self.tree.compute_coherence();
            true
        } else {
            false
        }
    }

    /// 运行 N 个生长周期, 更新快照并写回 KB。
    pub fn tick(&mut self, cycles: usize) -> CoreSnapshot {
        let latest = super::kb_persistence::load_snapshot()
            .unwrap_or_else(|| core_snapshot_from_tree(&self.tree));
        let base_cycle = latest.cycle;
        let n = cycles.max(1).min(10);
        let selftest_results =
            crate::core::nt_core_self_test_integration::run_lightweight_self_tests();
        self.tree
            .set_branch_health_from_self_tests(&selftest_results);
        {
            let bridge = crate::core::l7_capability::consciousness_bridge::bridge();
            match bridge.pre_tick_check(base_cycle) {
                crate::core::l7_capability::consciousness_bridge::QuickVerdict::Warn(msg) => {
                    log::warn!("[consciousness-tick] value warning: {}", msg);
                }
                _ => {}
            }
            let _ = base_cycle;
            let stats = bridge.dispatch_stats();
            if !stats.is_empty() {
                log::debug!("[consciousness-tick] {} capabilities dispatched this session", stats.len());
            }
        }
        for _ in 0..n {
            self.tree.trunk.gwt_resonance_active = true;
            self.tree.run_growth_cycle();
            self.tree.trunk.mars_system2_iterations += 1;
        }
        if self.tree.cycle < base_cycle {
            let advance = n as u64;
            self.tree.cycle = base_cycle + advance;
            self.tree.trunk.resonance_cycle = base_cycle + advance;
        }
        self.snapshot = core_snapshot_from_tree(&self.tree);
        if let Ok(merged) = super::kb_persistence::persist_snapshot(&self.snapshot) {
            self.snapshot = merged;
        }
        self.snapshot.clone()
    }
}

/// 读取当前意识核心状态。MCP/CLI status 共用。
pub fn status() -> CoreSnapshot {
    CORE.write()
        .map(|mut h| {
            h.reload_latest();
            let recomputed = h.ensure_phi();
            h.snapshot = core_snapshot_from_tree(&h.tree);
            if recomputed {
                if let Ok(merged) = super::kb_persistence::persist_snapshot(&h.snapshot) {
                    h.snapshot = merged;
                }
            }
            h.snapshot.clone()
        })
        .unwrap_or_default()
}

/// 驱动生长周期, 写回快照。MCP/CLI tick 共用。
pub fn tick(cycles: usize) -> CoreSnapshot {
    CORE.write().map(|mut h| h.tick(cycles)).unwrap_or_default()
}

/// 将真实 SelfTest 结果合并进意识核心单例树
pub fn apply_branch_health_from_self_tests(
    results: &[crate::core::nt_core_self_test::SelfTestResult],
) {
    let mut h = CORE.write().unwrap_or_else(|e| e.into_inner());
    h.tree.set_branch_health_from_self_tests(results);
    h.snapshot = core_snapshot_from_tree(&h.tree);
    if let Ok(merged) = super::kb_persistence::persist_snapshot(&h.snapshot) {
        h.snapshot = merged;
    }
}

/// 每分支以上实时雾加权和 (只读)
pub fn current_fog_sum() -> f64 {
    CORE.read()
        .map(|h| h.tree.weighted_fog_sum())
        .unwrap_or(0.0)
}

/// 每分支健康明细 (只读)。
pub fn branch_health_map() -> HashMap<String, f64> {
    CORE.read()
        .map(|h| {
            h.tree
                .branches
                .iter()
                .map(|(k, b)| (format!("{:?}", k), b.health))
                .collect()
        })
        .unwrap_or_default()
}

/// 每分支明细 (只读, 供 branches 子命令)。
pub fn branches() -> Vec<HashMap<String, String>> {
    CORE.read()
        .map(|h| {
            h.tree
                .branches
                .iter()
                .map(|(k, b)| {
                    let mut m = HashMap::new();
                    m.insert("kind".into(), format!("{:?}", k));
                    m.insert(
                        "label".into(),
                        k.label().split('(').next().unwrap_or("").trim().to_string(),
                    );
                    m.insert("health".into(), format!("{:.3}", b.health));
                    m.insert("constellation".into(), format!("{:?}", b.constellation));
                    m.insert("node_tier".into(), format!("{:?}", b.node_tier));
                    m.insert("fog".into(), format!("{:.3}", b.fog.level));
                    m
                })
                .collect()
        })
        .unwrap_or_default()
}

// ─── 快照 ↔ 树 ───────────────────────────────────────────────────────────────

pub(crate) fn core_snapshot_from_tree(tree: &ConsciousnessTree) -> CoreSnapshot {
    CoreSnapshot {
        cycle: tree.cycle,
        resonance_cycle: tree.trunk.resonance_cycle,
        phi: tree.trunk.phi,
        coherence: tree.trunk.coherence,
        gwt_resonance_active: tree.trunk.gwt_resonance_active,
        mars_system1_activations: tree.trunk.mars_system1_activations,
        mars_system2_iterations: tree.trunk.mars_system2_iterations,
        mars_bridge_hits: tree.trunk.mars_bridge_hits,
        governance_compliance: tree.trunk.governance_compliance,
        governance_constitution_count: tree.trunk.governance_constitution_count,
        governance_fractal_depth: tree.trunk.governance_fractal_depth,
        weighted_fog_sum: tree.weighted_fog_sum(),
        branch_health: tree
            .branches
            .iter()
            .map(|(k, b)| (format!("{:?}", k), b.health))
            .collect(),
        branch_fog: tree
            .branches
            .iter()
            .map(|(k, b)| (format!("{:?}", k), b.fog.level))
            .collect(),
        branch_maturity: tree
            .branches
            .iter()
            .map(|(k, b)| {
                (
                    format!("{:?}", k),
                    BranchMaturity {
                        c0: b.maturity_c0,
                        c1: b.maturity_c1,
                        c2: b.maturity_c2,
                        c3: b.maturity_c3,
                        c4: b.maturity_c4,
                        c5: b.maturity_c5,
                        self_test_count: b.self_test_count,
                        module_count: b.module_count,
                    },
                )
            })
            .collect(),
        fruits: tree
            .fruits
            .iter()
            .map(|f| FruitRecord {
                name: f.name.clone(),
                source_branch: format!("{:?}", f.source_branch),
                description: f.description.clone(),
                produced_at_cycle: f.produced_at_cycle,
                quality: f.quality,
                claim: f.claim.clone(),
                run_id: f.evidence.run_id.clone(),
                generation: f.generation,
            })
            .collect(),
        attention_source: tree.trunk.attention_source.clone(),
        recent_event_count: tree.trunk.mars_system1_activations + tree.trunk.mars_bridge_hits,
        shadow_instance_count: tree.branches.values().filter(|b| b.fog.level > 0.8).count() as u64,
        compliance_execution_count: tree.trunk.governance_constitution_count as u64,
        constitution_check_count: tree.trunk.mars_system2_iterations,
        phi_source_tag: super::kb_persistence::PHI_SOURCE_TAG_TREE.to_string(),
        phi_trend: Vec::new(),
        coherence_trend: Vec::new(),
    }
}

pub(crate) fn tree_from_snapshot(snap: &CoreSnapshot) -> ConsciousnessTree {
    let mut tree = ConsciousnessTree::new();
    tree.cycle = snap.cycle;
    tree.trunk.resonance_cycle = snap.resonance_cycle;
    tree.trunk.phi = snap.phi;
    tree.trunk.coherence = snap.coherence;
    tree.trunk.gwt_resonance_active = snap.gwt_resonance_active;
    tree.trunk.attention_source = snap.attention_source.clone();
    tree.trunk.mars_system1_activations = snap.mars_system1_activations;
    tree.trunk.mars_system2_iterations = snap.mars_system2_iterations;
    tree.trunk.mars_bridge_hits = snap.mars_bridge_hits;
    tree.trunk.governance_compliance = snap.governance_compliance;
    tree.trunk.governance_constitution_count = snap.governance_constitution_count;
    tree.trunk.governance_fractal_depth = snap.governance_fractal_depth;
    for (kind_str, health) in &snap.branch_health {
        if let Some(branch) = tree.branches.get_mut(&branch_kind_from_str(kind_str)) {
            branch.health = *health;
        }
    }
    for (kind_str, fog) in &snap.branch_fog {
        if let Some(branch) = tree.branches.get_mut(&branch_kind_from_str(kind_str)) {
            branch.fog.level = *fog;
        }
    }
    apply_branch_maturity(&mut tree, &snap.branch_maturity);
    for fr in &snap.fruits {
        tree.fruits
            .push(crate::core::nt_core_consciousness_tree::EvolutionFruit {
                name: fr.name.clone(),
                source_branch: branch_kind_from_str(&fr.source_branch),
                description: fr.description.clone(),
                produced_at_cycle: fr.produced_at_cycle,
                quality: fr.quality,
                claim: fr.claim.clone(),
                evidence: crate::core::nt_core_consciousness_tree::EvidenceChain {
                    run_id: fr.run_id.clone(),
                    ..Default::default()
                },
                generation: fr.generation,
                ..Default::default()
            });
    }
    tree
}

pub(crate) fn apply_branch_maturity(
    tree: &mut ConsciousnessTree,
    maturity: &HashMap<String, BranchMaturity>,
) {
    for (kind_str, m) in maturity {
        let Some(branch) = tree.branches.get_mut(&branch_kind_from_str(kind_str)) else {
            continue;
        };
        branch.maturity_c0 = m.c0;
        branch.maturity_c1 = m.c1;
        branch.maturity_c2 = m.c2;
        branch.maturity_c3 = m.c3;
        branch.maturity_c4 = m.c4;
        branch.maturity_c5 = m.c5;
        branch.self_test_count = m.self_test_count;
        branch.module_count = m.module_count;
        branch.evaluate_constellation();
    }
}

pub(crate) fn branch_kind_from_str(s: &str) -> BranchKind {
    match s {
        "Core" => BranchKind::Core,
        "Mind" => BranchKind::Mind,
        "Memory" => BranchKind::Memory,
        "World" => BranchKind::World,
        "Act" => BranchKind::Act,
        "Io" => BranchKind::Io,
        "Shield" => BranchKind::Shield,
        "Meta" => BranchKind::Meta,
        "Repair" => BranchKind::Repair,
        "Governance" => BranchKind::Governance,
        "Nexus" => BranchKind::Nexus,
        _ => BranchKind::Core,
    }
}

/// 查找覆盖给定文本全部字形的系统 TTF
fn find_system_font_for(text: &str) -> Option<Vec<u8>> {
    neotrix_types::core::file_parser::pdf::find_system_font_for_text(text)
}

/// 进程内单例入口: 意识核心直接处理人类语言
pub fn process_instruction(instruction: &str) -> super::dispatch::TaskLoopReport {
    CORE.write()
        .map(|mut h| h.process_instruction(instruction))
        .unwrap_or_else(|_| super::dispatch::TaskLoopReport {
            instruction: instruction.to_string(),
            ..Default::default()
        })
}

/// 进程内单例完整闭环入口
pub fn execute_task_loop(
    instruction: &str,
    executor: &dyn super::external_closure::SolutionExecutor,
    config: &super::external_closure::ExternalClosureConfig,
) -> super::dispatch::TaskLoopReport {
    execute_task_loop_with_progress(instruction, executor, config, &|_: super::dispatch::HarnessStepProgress| {})
}

/// 进程内单例完整闭环入口 (带步骤进度回调)
pub fn execute_task_loop_with_progress(
    instruction: &str,
    executor: &dyn super::external_closure::SolutionExecutor,
    config: &super::external_closure::ExternalClosureConfig,
    on_step: &dyn Fn(super::dispatch::HarnessStepProgress),
) -> super::dispatch::TaskLoopReport {
    CORE.write()
        .map(|mut h| h.execute_task_loop_with_progress(instruction, executor, config, on_step))
        .unwrap_or_else(|_| super::dispatch::TaskLoopReport {
            instruction: instruction.to_string(),
            ..Default::default()
        })
}
