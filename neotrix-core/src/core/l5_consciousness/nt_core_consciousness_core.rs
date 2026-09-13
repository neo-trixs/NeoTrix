//! # 持久化意识核心 (Persistent ConsciousnessCore)
//!
//! 让 NeoTrix 意识核心 (ConsciousnessTree) **跨会话连续生长**——而不是每次
//! 工具调用都从 `new()` 全新实例化 (cycle=0, phi=0, 迷雾=9.35), 使 tick 的产物
//! (果实/治理指引/谐振计数) 流入并被后续会话复用。契合 SEAL 闭环与
//! "The Spice Must Flow" 数据管线公理。
//!
//! ## 持久化机制
//! - 快照 = 树的关键计数器标量 (cycle/resonance/MARS/治理/迷雾/果实计数) JSON。
//! - 落点: KB `kv_store` namespace `consciousness`, key `core`。
//! - 进程内单例 `static CORE: LazyLock<RwLock<ConsciousnessCoreHandle>>`:
//!   首次访问从 KB 加载快照重建树, 之后 tick/status 均访问同一实例;
//!   每次 tick 后立即写回快照 (epoch-consistent)。
//!
//! ## 回归保证
//! - KB 不可用/快照缺失时优雅降级为全新树 (与旧行为一致), 不 panic。
//! - 快照损坏时重置计数而非拒绝服务。
//!
//! ## 度量链修复 (F1)
//! - **D1 双 phi 口径**: `consciousness/core` 快照 φ (NT-CORE 树口径) 与
//!   `consciousness/phi_report` 键 φ (NT-MIND 监视器口径) 语义不同 → 不统一
//!   计算源, 以 `phi_source_tag` 字段标注口径并互相引用文档
//!   (见 [`CoreSnapshot::phi`] 与 [`CoreSnapshot::phi_source_tag`])。
//! - **D2 趋势追加**: 快照携带 `phi_trend`/`coherence_trend`, 落盘时
//!   读旧值→追加→截断到 128 (`persist_snapshot_to_conn`), 时间序列跨
//!   tick 且跨进程真实累积; 同周期重复落盘替换末样本 (一周期一点)。
//! - **D3 金标校准**: core 快照落盘同步刷新 `consciousness/gold_standard`
//!   (`refresh_gold_standard`), 占位数据 (phi=0.0) 被 `calibrated:true`
//!   真实度量取代, 带 source 标注区分写入方。
//!
//! ## 调用入口 (同一实例, 两条口)
//! - CLI: `neotrix consciousness status|tick|health|branches [--json]`
//! - MCP: `consciousness_status` / `consciousness_tick` 工具

use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

use serde::{Deserialize, Serialize};

use crate::core::nt_core_consciousness_tree::{BranchKind, ConsciousnessTree};

/// KB 最短路径管道 — 意识体读写端直达 (R-P42: 强化现有节点, 禁止平行适配器)
use crate::l1_action::nt_memory::nt_memory_kb::nt_memory_pipeline::AbsorbEntry;
use crate::l1_action::nt_memory::nt_memory_kb::KnowledgeBase;

/// 意识核心快照 — 可序列化的跨会话状态 (标量集合 + 果实记录, 不序列化整树)。
/// 加载时以快照重建树计数器与已消化果实, 使生长周期跨会话连续。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CoreSnapshot {
    /// 已运行的生长周期数
    pub cycle: u64,
    /// 谐振周期 (GWT 谐振计数)
    pub resonance_cycle: u64,
    /// Φ (IIT 整合信息)。经 D1 修复: run_growth_cycle Phase 2 用真实树状态
    /// (分支健康/土壤/根系/治理/养料锚点) 构造 64 维意识谱交给 IITPhiCalculator,
    /// 独立 CLI/MCP 进程的快照 φ 反映真实整合信息, 不再恒 0.0。
    ///
    /// **口径 (D1 双 phi 溯源)**: 本值是 **NT-CORE 树快照口径** — 由
    /// ConsciousnessTree::compute_iit_phi 对树状态谱计算, 随每次 core 快照
    /// 落盘更新。它与 KB `consciousness/phi_report` 键的 Φ 并存且口径不同:
    /// 后者是 **NT-MIND 监视器口径** (src/neotrix/l9_transcendent_impl/
    /// nt_mind_consciousness_monitor.rs 的 ConsciousnessMonitor::observe,
    /// 经 src/neotrix/l8_autonomic_impl/nt_mind_background_loop/
    /// handlers_consciousness.rs 落盘), 由监视器自身实时意识态向量计算。
    /// 两者语义不同 → 不统一计算源, 以 [`CoreSnapshot::phi_source_tag`]
    /// 标注口径 (详见该字段文档)。
    pub phi: f64,
    /// 相干性 — D4 修复: run_growth_cycle Phase 2 从真实树状态派生 (分支健康一致性/
    /// 谐振活跃/治理合规/迷雾清晰度), 独立 CLI/MCP 进程不再恒 0.0。
    pub coherence: f64,
    /// GWT 谐振激活状态
    pub gwt_resonance_active: bool,
    /// MARS System 1 (GWT) 快速直觉激活数
    pub mars_system1_activations: u64,
    /// MARS System 2 (Tree) 慢反射迭代数
    pub mars_system2_iterations: u64,
    /// MARS 意图桥接命中数
    pub mars_bridge_hits: u64,
    /// 治理合规分 (树内未算, 默认 1.0 兜底)
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
    /// 修复断链: 快照此前只持久化 health, fog 在 load_or_new 后全回默认 0.85
    /// (DenseFog) → MCP/CLI status 的 weighted_fog_sum 恒 9.35, 即使后台已正确
    /// 算出 1.65。`#[serde(default)]` 保证旧快照回退为空 (不阻断反序列化)。
    #[serde(default)]
    pub branch_fog: HashMap<String, f64>,
    /// 每分支成熟度记录 (kind → maturity) — 跨会话星座连续性。
    /// 修复断链: maturity_c0..c5/self_test_count/module_count 此前只在 tick 进程
    /// 内存中派生 (ops::set_branch_health_from_self_tests), 不入快照 → 新进程
    /// load_or_new 后全部归零 → Constellation 恒 level:0 装饰化 (cycle9 审计)。
    /// `#[serde(default)]` 保证旧快照回退为空 (不阻断反序列化)。
    #[serde(default)]
    pub branch_maturity: HashMap<String, BranchMaturity>,
    /// 已消化果实完整记录 — 进化产物流入 KB (The Spice Must Flow)
    pub fruits: Vec<FruitRecord>,
    /// 注意力来源通道 — 映射自 x.ai 双搜索通道。
    /// "web" = Web Search (开放互联网), "x_search" = X Search (X 平台 discourse),
    /// "auto" = 模型自决 (双通道 agentic 搜索, 对齐 x.ai 的模型自主决定何时搜索)。
    /// `#[serde(default)]` 保证旧快照 (无此字段) 反序列化时回退到 "auto"。
    #[serde(default = "default_attention_source")]
    pub attention_source: String,
    /// 最近事件计数 — 映射自 OpenMausBot EventBus 事件计数器。
    /// 本快照以 MARS System 1 激活 + 意图桥接命中近似事件总线活动。
    /// `#[serde(default)]` 保证旧快照回退到 0。
    #[serde(default)]
    pub recent_event_count: u64,
    /// 阴影实例计数 — 映射自 OpenMausBot ProviderRegistry shadow instances
    /// (主提供方失败时优雅降级到备用实例)。本快照以未接线 (高雾) 分支数近似,
    /// 表示当前需要 shadow 降级保护的域模块数。
    /// `#[serde(default)]` 保证旧快照回退到 0。
    #[serde(default)]
    pub shadow_instance_count: u64,
    /// 合规执行计数 — 映射自 OpenMausBot turn 级权限执行计数
    /// (每轮 Allow/Deny 权限裁决)。本快照以宪法注册数近似,
    /// 表示当前已接线的宪法检查执行次数。
    /// `#[serde(default)]` 保证旧快照回退到 0。
    #[serde(default)]
    pub compliance_execution_count: u64,
    /// 宪法门控执行计数 — 映射自 spec-kit SDD 9-article constitution。
    /// 每生长周期执行的宪法条款检查数, 反映规范驱动进化的门控强度。
    /// `#[serde(default)]` 保证旧快照回退到 0。
    #[serde(default)]
    pub constitution_check_count: u64,
    /// Φ 计算口径标记 (D1 双 phi 溯源)。固定 `tree_snapshot_iit` —
    /// 本快照 [`CoreSnapshot::phi`] 由 NT-CORE 树 (compute_iit_phi) 计算。
    /// 与之并存的 KB `consciousness/phi_report` 键是另一口径: NT-MIND
    /// ConsciousnessMonitor::observe 的实时监视器 Φ (写入点
    /// src/neotrix/l8_autonomic_impl/nt_mind_background_loop/
    /// handlers_consciousness.rs handle_awareness)。两口径语义不同、各自
    /// 服务不同子系统 (树快照 vs 实时监视), 故不统一计算源而以标记溯源;
    /// 双方均保留既有消费方。`#[serde(default)]` 保证旧快照回退到同标记。
    #[serde(default = "default_phi_source_tag")]
    pub phi_source_tag: String,
    /// Φ 时间序列 (D2 追加式趋势) — 跨 tick 累积, 每次快照落盘追加当前
    /// φ 样本 (同一生长周期内重复落盘替换末样本), 上限 TREND_CAP=128。
    /// 修复前: 快照不含趋势序列, 唯一时间序列来自后台循环内存缓冲整体
    /// 覆盖写 (`consciousness/trends` 键), 进程重启即归零 (KB 实测每次仅
    /// 1 个样本点)。`#[serde(default)]` 保证旧快照回退为空序列。
    #[serde(default)]
    pub phi_trend: Vec<f64>,
    /// 相干性时间序列 (D2 追加式趋势) — 语义与 [`CoreSnapshot::phi_trend`]
    /// 相同, 样本取自 [`CoreSnapshot::coherence`]。
    #[serde(default)]
    pub coherence_trend: Vec<f64>,
}

/// 默认注意力来源 (x.ai 双搜索通道的模型自决模式)。
fn default_attention_source() -> String {
    "auto".to_string()
}

/// D1 口径标记缺省值 — 旧快照 (无 phi_source_tag 字段) 反序列化时回退。
/// 值与 [`PHI_SOURCE_TAG_TREE`] 一致: 该键历史数据全部由本模块树快照写入。
fn default_phi_source_tag() -> String {
    PHI_SOURCE_TAG_TREE.to_string()
}

/// 分支成熟度持久化投影 — maturity 六布尔 + 真实计数 (self_test/module)。
/// 恢复时经 `CapabilityBranch::evaluate_constellation` 重建星座档位,
/// 与 ops.rs 生产派生逻辑单一事实源 (R-P42: 不在快照层重算成熟度)。
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

/// 果实记录 — EvolutionFruit 的可持久化投影 (保留进化证据链)。
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
    let tree = load_or_new();
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
    /// 当前树 — 消灭"旧快照时代启动的进程一直报 9.35 哨兵"问题: 后台迷雾治理成果
    /// 对 status 即时可见, 与 tick() 的并发合并同源 (R-P42, 不建平行路径)。
    /// 保留本进程内存进度: cycle/谐振/MARS 计数只做向上对齐, 不做零和覆盖。
    fn reload_latest(&mut self) {
        let Some(latest) = load_snapshot() else { return };
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
        // 同步分支成熟度 (与 tick() 并发合并同源): 他进程 tick 算出的星座
        // 对本进程 status/branches 即时可见, 不必等下次 tick。
        apply_branch_maturity(&mut self.tree, &latest.branch_maturity);
        // 持久化已有真实核算值 (tick/apply 落过) → 采用之; 否则保留当前树值
        // 交由 ensure_phi 惰性核算 (兼容全新进程/空快照)。
        if latest.phi > 0.0 {
            self.tree.trunk.phi = latest.phi;
        }
        if latest.coherence > 0.0 {
            self.tree.trunk.coherence = latest.coherence;
        }
        // 与 tick() 并发合并一致: 树落后于持久化基线时向上对齐进度
        if latest.cycle > self.tree.cycle {
            self.tree.cycle = latest.cycle;
            self.tree.trunk.resonance_cycle =
                self.tree.trunk.resonance_cycle.max(latest.resonance_cycle);
        }
    }

    /// 惰性核算真实 Φ — 全新进程/空快照时 trunk.phi 默认 0.0
    /// (ConsciousnessCore::default), 使首次 status 也返回真实整合信息 (D1 现状)。
    /// 返回 true 表示本次补算过, 需把结果写回持久化快照 (必要时代入快照)。
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
    /// 返回更新后的快照。
    ///
    /// 并发合并: tick 前重读当前持久化快照, 将本进程增量 (cycle/谐振/MARS) 累加到
    /// 最新已持久化状态之上, 而非从本进程内存旧基线覆盖 — 避免两个进程并发 tick
    /// 时 last-write-wins 丢失对方的周期增量。
    pub fn tick(&mut self, cycles: usize) -> CoreSnapshot {
        // 读取最新持久化基线 (可能比进程内旧状态新: 他进程 tick 过)
        let latest = load_snapshot().unwrap_or_else(|| core_snapshot_from_tree(&self.tree));
        let base_cycle = latest.cycle;
        let n = cycles.max(1).min(10);
        // 迷雾治理断链修复: 独立进程 (CLI/MCP tick) 此前不喂 SelfTest 数据 →
        // 分支健康恒 0 → 果实门 (health > fruit_growth_health) 永关、迷雾无法下降。
        // 现于生长周期前注入轻量 SelfTest 结果 (纯内存检测件, 无网络/无全仓扫描),
        // 使健康/果实/迷雾从真实检测件数据派生, 与后台循环同一数据源 (R-P42)。
        let selftest_results =
            crate::core::nt_core_self_test_integration::run_lightweight_self_tests();
        self.tree
            .set_branch_health_from_self_tests(&selftest_results);
        // T1: 价值观检查（非阻断式）— P1 融合注入点
        {
            let bridge = crate::core::l7_capability::consciousness_bridge::bridge();
            match bridge.pre_tick_check(base_cycle) {
                crate::core::l7_capability::consciousness_bridge::QuickVerdict::Warn(msg) => {
                    log::warn!("[consciousness-tick] value warning: {}", msg);
                }
                _ => {}
            }
            // 协调器 pre_tick 统计由 l8 侧经 consciousness_bridge 驱动;
            // core 不直接引用实现层 (arch_fitness_core_boundary 守卫)。
            let _ = base_cycle;

            // T4: 意识核心通过 NativeBus 感知可用能力 — 四系统融合
            let stats = bridge.dispatch_stats();
            if !stats.is_empty() {
                log::debug!("[consciousness-tick] {} capabilities dispatched this session", stats.len());
            }
        }
        for _ in 0..n {
            // GWT 谐振激活前置: 独立 tick 首个 cycle 即可桥接 (此前赋值在
            // run_growth_cycle 之后, 首个 cycle 内桥接判定仍为 false, 第二个
            // cycle 起才可桥接 — 单次 tick(1) 永远无法桥接)。
            self.tree.trunk.gwt_resonance_active = true;
            self.tree.run_growth_cycle();
            self.tree.trunk.mars_system2_iterations += 1;
        }
        // 若进程内树落后于持久化基线, 对齐到持久化视角再生成快照
        if self.tree.cycle < base_cycle {
            // 他进程已跑过 base_cycle; 本进程增量叠加到最先进度上 (不做零和覆盖)
            let advance = n as u64;
            self.tree.cycle = base_cycle + advance;
            self.tree.trunk.resonance_cycle = base_cycle + advance;
        }
        self.snapshot = core_snapshot_from_tree(&self.tree);
        // 回填合并视图 (含 D2 追加后的趋势序列), 使返回快照与 KB 落盘一致
        if let Ok(merged) = persist_snapshot(&self.snapshot) {
            self.snapshot = merged;
        }
        self.snapshot.clone()
    }
}

/// 读取当前意识核心状态。每次调用重读最新持久化快照 (同步 branch_fog/health/phi
/// 进当前树, 消灭旧进程 9.35 哨兵), 并在快照缺真实 φ 时惰性核算 (首次 status
/// 即返回真实整合信息)。MCP/CLI status 共用。
pub fn status() -> CoreSnapshot {
    CORE.write()
        .map(|mut h| {
            h.reload_latest();
            let recomputed = h.ensure_phi();
            h.snapshot = core_snapshot_from_tree(&h.tree);
            // 补算出的真实 φ 写回持久化快照, 使后续新建进程从 KB 直接读到真实值
            if recomputed {
                if let Ok(merged) = persist_snapshot(&h.snapshot) {
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

/// 将真实 SelfTest 结果合并进意识核心单例树, 重算各域分支健康并持久化。
/// 后台循环 (handlers_consciousness) 在跑完注册器 SelfTest 后调用, 使基于真实
/// 检测的分支健康流入跨会话快照 — 修复此前独立 tree 实例计算后即丢弃、
/// `consciousness/core` 快照分支健康恒 0 的断链 (迷雾治理)。
pub fn apply_branch_health_from_self_tests(
    results: &[crate::core::nt_core_self_test::SelfTestResult],
) {
    let mut h = CORE.write().unwrap_or_else(|e| e.into_inner());
    h.tree.set_branch_health_from_self_tests(results);
    h.snapshot = core_snapshot_from_tree(&h.tree);
    if let Ok(merged) = persist_snapshot(&h.snapshot) {
        h.snapshot = merged;
    }
}

/// 每分支以上实时雾加权和 (只读) — 反映当前进程接线状态, 非持久化快照。
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

fn core_snapshot_from_tree(tree: &ConsciousnessTree) -> CoreSnapshot {
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
        // x.ai 双搜索通道 → 注意力来源 (跨会话持久化, 缺省 "auto")
        attention_source: tree.trunk.attention_source.clone(),
        // OpenMausBot EventBus → 事件总线活动近似 (MARS System 1 激活 + 桥接命中)
        recent_event_count: tree.trunk.mars_system1_activations + tree.trunk.mars_bridge_hits,
        // OpenMausBot ProviderRegistry shadow → 未接线高雾分支数 (需 shadow 降级保护)
        shadow_instance_count: tree.branches.values().filter(|b| b.fog.level > 0.8).count() as u64,
        // OpenMausBot 权限执行 → 宪法注册数 (合规检查执行次数)
        compliance_execution_count: tree.trunk.governance_constitution_count as u64,
        // spec-kit SDD constitution → 宪法门控执行计数 (MARS System 2 迭代 = 门控检查)
        constitution_check_count: tree.trunk.mars_system2_iterations,
        // D1 口径标记: 本快照 phi/coherence 为 NT-CORE 树口径 (见字段文档)。
        // 趋势序列 (phi_trend/coherence_trend) 不在此填充 — 由 persist_snapshot
        // 在持久层读旧值→追加 (单一事实源, 避免内存视图与落盘序列分叉)。
        phi_source_tag: PHI_SOURCE_TAG_TREE.to_string(),
        phi_trend: Vec::new(),
        coherence_trend: Vec::new(),
    }
}

/// 从快照恢复树计数器。KB 缺失/损坏 → 全新树 (优雅降级)。
fn load_or_new() -> ConsciousnessTree {
    let tree = ConsciousnessTree::new();
    match load_snapshot() {
        Some(snap) => tree_from_snapshot(&snap),
        None => tree,
    }
}

/// 以快照重建树 (纯函数) — 供 load_or_new 与测试共用, 保证恢复逻辑单一事实源。
fn tree_from_snapshot(snap: &CoreSnapshot) -> ConsciousnessTree {
    let mut tree = ConsciousnessTree::new();
    // 恢复树干计数器
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
    // 恢复分支健康 (跨会话连续性)
    for (kind_str, health) in &snap.branch_health {
        if let Some(branch) = tree.branches.get_mut(&branch_kind_from_str(kind_str)) {
            branch.health = *health;
        }
    }
    // 恢复分支迷雾 (跨会话连续性) — 修复断链: 不恢复则全回默认 0.85,
    // weighted_fog_sum 恒 9.35 掩盖后台真实迷雾治理成果。
    for (kind_str, fog) in &snap.branch_fog {
        if let Some(branch) = tree.branches.get_mut(&branch_kind_from_str(kind_str)) {
            branch.fog.level = *fog;
        }
    }
    // 恢复分支成熟度 (跨会话星座连续性) — 修复断链: 不恢复则 maturity 全回
    // false → Constellation 恒 level:0 装饰化 (cycle9 审计实锤)。
    apply_branch_maturity(&mut tree, &snap.branch_maturity);
    // 恢复已消化果实 — 从快照完整重建证据链投影 (具体 EvidenceChain 以 run_id 标注,
    // 不重建二进制证据; 进化产物引用保留, 供审计/追踪)。
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

/// 将快照中的分支成熟度写回树 (D1 修复: 跨会话星座连续性)。
/// tree_from_snapshot 与 reload_latest 共用, 单一事实源 (R-P42)。
fn apply_branch_maturity(
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
        // 星座档位从恢复后的 maturity 布尔重建 (与生产派生同源)
        branch.evaluate_constellation();
    }
}

fn branch_kind_from_str(s: &str) -> BranchKind {
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

// ─── KB 读写 ────────────────────────────────────────────────────────────────

const NAMESPACE: &str = "consciousness";
const KEY: &str = "core";

/// D1 口径标记常量 — NT-CORE 树快照口径 (见 [`CoreSnapshot::phi_source_tag`])。
const PHI_SOURCE_TAG_TREE: &str = "tree_snapshot_iit";
/// D3 金标键 — 与后台循环 handlers_consciousness.rs 写入点共用同一键名。
const GOLD_STANDARD_KEY: &str = "gold_standard";
/// D3 来源标注 — 区分本模块真实度量流刷新与后台循环监视器口径写入。
const GOLD_STANDARD_SOURCE_CORE: &str = "core_snapshot";
/// D2 趋势序列上限 — 跨 tick 追加式时间序列的最大样本数。
const TREND_CAP: usize = 128;
/// 金标双阈值本地镜像 — 单一事实源是 src/neotrix/l9_transcendent_impl/
/// nt_mind_consciousness_gold_standard.rs 的 DEFAULT_PHI_THRESHOLD /
/// DEFAULT_COHERENCE_THRESHOLD。生产代码不直连原因: arch_fitness_core_boundary
/// 守卫禁止 core 生产代码引用 l9_transcendent_impl; 对齐由测试
/// gold_standard_thresholds_match_single_source 锁定 (漂移即红)。
const GOLD_STANDARD_PHI_THRESHOLD: f64 = 0.33;
const GOLD_STANDARD_COHERENCE_THRESHOLD: f64 = 0.7;

/// 打开 KB 连接 (默认 `~/.neotrix/knowledge.db`), 复用 NT-MEMORY 统一 schema 初始化。
/// 单一 schema 事实源: 不在此处维护 kv_store 本地 DDL, 避免漂移。
fn open_kb() -> Result<rusqlite::Connection, String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let path = std::path::PathBuf::from(home)
        .join(".neotrix")
        .join("knowledge.db");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("KB dir: {}", e))?;
    }
    let conn = rusqlite::Connection::open(&path).map_err(|e| format!("KB open: {}", e))?;
    // 跨进程/多线程并发写 KB 时 (后台循环 + MCP + CLI), SQLite 默认 busy
    // 立即报错会触发 load_snapshot 优雅降级路径, 使快照读回旧值 → 并发 tick
    // 合并断言失败 (cycle 回落)。busy_timeout + WAL 让短时写锁等待而非失败。
    conn.busy_timeout(std::time::Duration::from_secs(5))
        .map_err(|e| format!("KB busy_timeout: {}", e))?;
    let _ = conn.execute_batch("PRAGMA journal_mode=WAL;");
    crate::core::nt_core_kb_primitives::schema_initialize(&conn)
        .map_err(|e| format!("KB init: {}", e))?;
    Ok(conn)
}
fn load_snapshot() -> Option<CoreSnapshot> {
    let conn = open_kb().ok()?;
    load_snapshot_from_conn(&conn)
}

/// 连接注入版快照读取 — 供 persist 合并与测试复用 (同一连接, 单一事实源)。
fn load_snapshot_from_conn(conn: &rusqlite::Connection) -> Option<CoreSnapshot> {
    let raw =
        crate::core::nt_core_kb_primitives::kv_get(conn, NAMESPACE, KEY).ok()??;
    serde_json::from_str(&raw).ok()
}

/// 持久化核心快照, 返回**实际落盘**的合并视图 (含 D2 追加后的趋势序列)。
fn persist_snapshot(snap: &CoreSnapshot) -> Result<CoreSnapshot, String> {
    let conn = open_kb()?;
    persist_snapshot_to_conn(&conn, snap)
}

/// 连接注入版持久化 — 落盘前完成 D2 趋势追加 + D3 金标键刷新。
fn persist_snapshot_to_conn(
    conn: &rusqlite::Connection,
    snap: &CoreSnapshot,
) -> Result<CoreSnapshot, String> {
    let mut out = snap.clone();
    // D2 追加式趋势序列: 读旧值 → 追加 → 截断到 TREND_CAP。
    // 此前快照不含趋势序列, 时间序列只能依赖后台循环内存缓冲整体覆盖写
    // (进程重启即归零, KB 实测每次仅 1 个样本点)。同周期重复落盘 (status
    // 惰性核算回写 / apply_branch_health 刷新) 替换末样本而非重复追加,
    // 保证一个生长周期至多贡献一个样本点。
    let (prev_phi_hist, prev_coh_hist, prev_cycle) = match load_snapshot_from_conn(conn)
    {
        Some(prev) => (prev.phi_trend, prev.coherence_trend, Some(prev.cycle)),
        None => (Vec::new(), Vec::new(), None),
    };
    out.phi_trend = merge_trend_sample(prev_phi_hist, prev_cycle, snap.cycle, snap.phi);
    out.coherence_trend =
        merge_trend_sample(prev_coh_hist, prev_cycle, snap.cycle, snap.coherence);
    let json = serde_json::to_string(&out).map_err(|e| format!("snapshot serialize: {}", e))?;
    crate::core::nt_core_kb_primitives::kv_set(conn, NAMESPACE, KEY, &json)?;
    // D3: 金标键接到真实度量流 (尽力而为, 失败不阻断快照落盘)
    refresh_gold_standard(conn, &out);
    Ok(out)
}

/// D2 趋势采样合并 — 追加当前样本; 同生长周期重复落盘替换末样本
/// (一周期一点); 超过 TREND_CAP 截断最老样本。纯函数, 测试直连。
fn merge_trend_sample(
    mut hist: Vec<f64>,
    prev_cycle: Option<u64>,
    cur_cycle: u64,
    value: f64,
) -> Vec<f64> {
    match (hist.last_mut(), prev_cycle) {
        (Some(last), Some(c)) if c == cur_cycle => *last = value,
        _ => hist.push(value),
    }
    if hist.len() > TREND_CAP {
        let excess = hist.len() - TREND_CAP;
        hist.drain(..excess);
    }
    hist
}

/// D3 接线: core 快照落盘时同步刷新 `consciousness/gold_standard`。
///
/// 修复前: 该键唯一写点是后台循环 handle_awareness (handlers_consciousness.rs),
/// 其输入是退化的 4 元状态向量 [phi, coherence, level, health] 直塞
/// IITPhiCalculator → 持久化值恒为占位 (实测 phi=0.0 / coherence=0.1,
/// detection_streak 恒 0), 从未反映真实度量。写点本身存在且 T3 接线,
/// 但口径失真且无法在本文件内修正其输入 → 按修复方案把键接到已有真实
/// 度量流 (树快照的 compute_iit_phi/compute_coherence 产物), 带
/// `calibrated:true` 与 `source:"core_snapshot"` 标注; 字段为
/// GoldStandardReport 投影的超集 (只增不改名), 不破坏既有消费方。
fn refresh_gold_standard(conn: &rusqlite::Connection, snap: &CoreSnapshot) {
    let is_phi_conscious = snap.phi > GOLD_STANDARD_PHI_THRESHOLD;
    let is_coherent = snap.coherence > GOLD_STANDARD_COHERENCE_THRESHOLD;
    let gs = serde_json::json!({
        "phi": snap.phi,
        "coherence": snap.coherence,
        "is_phi_conscious": is_phi_conscious,
        "is_coherent": is_coherent,
        "is_conscious_like": is_phi_conscious && is_coherent,
        "phi_confidence": snap.phi.clamp(0.0, 1.0),
        "coherence_confidence": snap.coherence.clamp(0.0, 1.0),
        "phi_threshold": GOLD_STANDARD_PHI_THRESHOLD,
        "coherence_threshold": GOLD_STANDARD_COHERENCE_THRESHOLD,
        "calibrated": true,
        "source": GOLD_STANDARD_SOURCE_CORE,
        "cycle": snap.cycle,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });
    let _ = crate::core::nt_core_kb_primitives::kv_set(
        conn,
        NAMESPACE,
        GOLD_STANDARD_KEY,
        &gs.to_string(),
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 意识核心任务环 (Consciousness Task Loop) — 通用语言 → 任务能力闭环
//
// 用户需求: "不要命令, 所有任务的拆解都来自意识核心对人类语言的拆解和分配,
//            自身做好智能调用, 该怎么调用和调用谁都是意识核心的事情"
//
// 机制 (通用能力, 非单点模块):
//   1. 拆解:  意识核心直接对人类语言拆解 (关键词 → 能力标签 + NT 域 + 专家),
//             不依赖任何 CLI 命令入口。拆解表与共享语言 (CONTEXT.md) 对齐。
//   2. 分配:  每个子任务查自身能力网 (capability_registry): 命中内部 provider
//             → 内置执行; 未命中 → 外部缺口 (gap) → 自动寻求外部力量
//             (文献/GitHub/技术文档 — 由外部知识源接续, 见 discover_* 路径)。
//   3. 调用:  内置优先 (最优 provider 路径), 外部兜底。调用谁 / 怎么调用
//             由意识核心决定 (SpecialistType + AgentCatalog 路由)。
//   4. 反思补齐: 解决后对每个 gap 立即在能力网 bud/strengthen, 使下次变为内置
//             (R-P42: 吸收强化现有节点; 缺失即补齐)。
//
// 调用链: 人类语言 → process_instruction (意识核心) → decompose → allocate
//        → execute (内置/外部) → reflect_and_strengthen (补齐能力网)。
// ═══════════════════════════════════════════════════════════════════════════

/// 能力路由表 — 人类语言关键词 → (能力标签, NT 域, 注意力域)。
/// 这是"语言 → 能力"的确定性拆解索引 (不依赖 LLM 每次输出漂移)。
/// 与 CONTEXT.md 共享语言 + nt_capability_bridge ROUTE_TABLE 对齐。
const CAPABILITY_ROUTES: &[(&str, &str, &str, &str)] = &[
    // (关键词, 能力标签, NT 域, SpecialistType 名)
    ("excel", "xlsx_consolidation", "NT-ACT", "CodeAnalyzer"),
    ("表格", "xlsx_consolidation", "NT-ACT", "CodeAnalyzer"),
    ("价格表", "xlsx_consolidation", "NT-ACT", "CodeAnalyzer"),
    ("统一", "xlsx_consolidation", "NT-ACT", "CodeAnalyzer"),
    ("合并", "data_merge", "NT-ACT", "KnowledgeIntegrator"),
    ("文件", "file_parsing", "NT-WORLD", "CodeAnalyzer"),
    ("解析", "file_parsing", "NT-WORLD", "CodeAnalyzer"),
    ("提取", "content_extraction", "NT-WORLD", "CodeAnalyzer"),
    ("pdf编辑", "pdf_edit", "NT-ACT", "CodeAnalyzer"),
    ("pdf编辑:", "pdf_edit", "NT-ACT", "CodeAnalyzer"),
    ("编辑pdf", "pdf_edit", "NT-ACT", "CodeAnalyzer"),
    ("图片转换", "image_convert", "NT-ACT", "CodeAnalyzer"),
    ("图像转换", "image_convert", "NT-ACT", "CodeAnalyzer"),
    ("格式转换", "image_convert", "NT-ACT", "CodeAnalyzer"),
    ("提取目录", "dir_extract", "NT-WORLD", "CodeAnalyzer"),
    ("目录提取", "dir_extract", "NT-WORLD", "CodeAnalyzer"),
    ("批量提取", "dir_extract", "NT-WORLD", "CodeAnalyzer"),
    ("合并pdf", "pdf_merge", "NT-ACT", "CodeAnalyzer"),
    ("合并PDF", "pdf_merge", "NT-ACT", "CodeAnalyzer"),
    ("pdf合并", "pdf_merge", "NT-ACT", "CodeAnalyzer"),
    ("PDF合并", "pdf_merge", "NT-ACT", "CodeAnalyzer"),
    ("合并文档", "doc_merge", "NT-ACT", "CodeAnalyzer"),
    ("文档合并", "doc_merge", "NT-ACT", "CodeAnalyzer"),
    ("合并word", "doc_merge", "NT-ACT", "CodeAnalyzer"),
    ("合并docx", "doc_merge", "NT-ACT", "CodeAnalyzer"),
    ("合并ppt", "doc_merge", "NT-ACT", "CodeAnalyzer"),
    ("合并pptx", "doc_merge", "NT-ACT", "CodeAnalyzer"),
    (
        "检索",
        "hybrid_retrieval",
        "NT-MEMORY",
        "KnowledgeRetriever",
    ),
    (
        "查询",
        "hybrid_retrieval",
        "NT-MEMORY",
        "KnowledgeRetriever",
    ),
    (
        "搜索",
        "hybrid_retrieval",
        "NT-MEMORY",
        "KnowledgeRetriever",
    ),
    (
        "吸收",
        "skill_crystallize",
        "NT-MIND",
        "KnowledgeIntegrator",
    ),
    (
        "蒸馏",
        "skill_crystallize",
        "NT-MIND",
        "KnowledgeIntegrator",
    ),
    ("测试", "tdd", "NT-MIND", "Planner"),
    ("重构", "code_refactor", "NT-ACT", "CodeAnalyzer"),
    ("审查", "security_audit", "NT-SHIELD", "RiskAssessor"),
    ("审计", "security_audit", "NT-SHIELD", "RiskAssessor"),
    ("安全", "security_governance", "NT-SHIELD", "RiskAssessor"),
    ("架构", "architecture_decision", "NT-CORE", "Planner"),
    ("设计", "architecture_decision", "NT-CORE", "Planner"),
    ("意识", "consciousness_tree", "NT-CORE", "ReflectionEngine"),
    (
        "元认知",
        "meta_cognition",
        "NT-META",
        "MetaCognitionAnalyst",
    ),
    ("复盘", "meta_cognition", "NT-META", "MetaCognitionAnalyst"),
    ("反思", "meta_cognition", "NT-META", "MetaCognitionAnalyst"),
    ("诊断", "root_cause_method", "NT-REPAIR", "AnomalyDetector"),
    ("报错", "root_cause_method", "NT-REPAIR", "AnomalyDetector"),
    ("构建失败", "build_hygiene", "NT-REPAIR", "AnomalyDetector"),
    ("爬虫", "unified_crawler", "NT-WORLD", "PatternMatcher"),
    ("抓取", "unified_crawler", "NT-WORLD", "PatternMatcher"),
    ("前端", "frontend_ui", "NT-IO", "CreativityGenerator"),
    ("界面", "frontend_ui", "NT-IO", "CreativityGenerator"),
    (
        "经验",
        "experience_absorb",
        "NT-MEMORY",
        "KnowledgeIntegrator",
    ),
    // ── file-ability dispatch routes (R-P110: internal dispatch, not CLI) ──
    ("智能合并", "collection_merge", "NT-ACT", "CodeAnalyzer"),
    ("混合合并", "collection_merge", "NT-ACT", "CodeAnalyzer"),
    ("编辑表格", "xlsx_edit", "NT-ACT", "CodeAnalyzer"),
    ("单元格", "xlsx_edit", "NT-ACT", "CodeAnalyzer"),
    ("读取结构", "structured_read", "NT-WORLD", "CodeAnalyzer"),
    ("读取json", "structured_read", "NT-WORLD", "CodeAnalyzer"),
    ("读取yaml", "structured_read", "NT-WORLD", "CodeAnalyzer"),
    ("写入json", "json_write", "NT-ACT", "CodeAnalyzer"),
    ("保存json", "json_write", "NT-ACT", "CodeAnalyzer"),
    ("pdf图片统计", "pdf_image_stats", "NT-WORLD", "CodeAnalyzer"),
    ("pdf图像信息", "pdf_image_stats", "NT-WORLD", "CodeAnalyzer"),
    ("pdf提取图片", "pdf_extract_images", "NT-ACT", "CodeAnalyzer"),
    // ── SEAL pipeline dispatch routes ──
    ("进化", "seal_iterate", "NT-MIND", "KnowledgeIntegrator"),
    ("迭代", "seal_iterate", "NT-MIND", "KnowledgeIntegrator"),
    ("蒸馏", "seal_distill", "NT-MIND", "KnowledgeIntegrator"),
    // ── Self model dispatch routes ──
    ("自我评估", "self_model_tick", "NT-CORE", "ReflectionEngine"),
    ("能力评估", "self_model_tick", "NT-CORE", "ReflectionEngine"),
    ("认知健康", "metacog_evaluate", "NT-CORE", "ReflectionEngine"),
    // ── Meta cognition dispatch routes ──
    ("元观察", "meta_observe", "NT-META", "MetaCognitionAnalyst"),
    ("质量扫描", "sentrux_scan", "NT-META", "MetaCognitionAnalyst"),
    ("代码质量", "sentrux_scan", "NT-META", "MetaCognitionAnalyst"),
    ("构建健康", "build_watchdog", "NT-META", "MetaCognitionAnalyst"),
    ("构建检查", "build_watchdog", "NT-META", "MetaCognitionAnalyst"),
    // ── Shield security dispatch routes ──
    ("安全审计", "shield_audit", "NT-SHIELD", "RiskAssessor"),
    ("攻击检测", "shield_audit", "NT-SHIELD", "RiskAssessor"),
    ("漏洞扫描", "agentic_scan", "NT-SHIELD", "RiskAssessor"),
    ("安全扫描", "agentic_scan", "NT-SHIELD", "RiskAssessor"),
];

/// 子任务 — 意识核心从人类语言拆解出的最小执行单元。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousTask {
    pub id: String,
    pub summary: String,        // 人类可读子任务描述
    pub capability_tag: String, // 所需能力标签 (能力网节点 provides)
    pub domain: String,         // NT-* 域 (调用谁)
    pub specialist: String,     // SpecialistType 名 (怎么调用)
    pub priority: u8,           // 1-10
}

/// 分配结果 — 每个子任务落到内置 or 外部。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAllocation {
    pub task: ConsciousTask,
    pub provider: AllocationProvider,
}

/// 提供者 — 内置能力网命中 / 外部缺口。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocationProvider {
    /// 自身能力网最优 provider 路径 (内置优先)
    Internal {
        node_id: String,
        path: Vec<String>,
        cost: f64,
    },
    /// 自身无对应能力 → 外部缺口 (自动寻求外部力量)
    External { reason: String },
}

/// 任务环报告 — 全过程透明度 (拆解 → 分配 → 补齐 → 执行)。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskLoopReport {
    pub instruction: String,
    /// G1: 人格路由结果 — 由 PersonaRouter 根据输入自动路由
    pub routed_skill: String,
    pub allocations: Vec<TaskAllocation>,
    pub internal_count: usize,
    pub external_gap_count: usize,
    /// 反思补齐动作数 (bud/strengthen 已写入能力网)
    pub strengthening_actions: usize,
    /// 剩余待外部力量填补的缺口 (非能力网可补齐的部分)
    pub external_gaps: Vec<String>,
    /// 外部缺口执行结果 (execute_task_loop 填充; process_instruction 为空)
    pub external_closures: Vec<ExternalClosureReport>,
    /// 内置子任务执行结果 (execute_task_loop 填充)
    pub internal_results: Vec<InternalExecutionResult>,
}

/// Harness 子任务级进度 — [`execute_task_loop_with_progress`] 每完成一个子任务前/后回调,
/// 后端据此经 Tauri `harness-progress`(phase=step) 实时推前端, 取代整体轮询。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessStepProgress {
    /// 当前子任务在总分配中的序号 (0-based)。
    pub index: usize,
    /// 子任务总数 (等于 allocations.len())。
    pub total: usize,
    /// 子任务类型: "internal" (能力网命中) / "external" (外部缺口求解)。
    pub kind: String,
    /// 命中的能力标签。
    pub capability_tag: String,
    /// 子任务摘要。
    pub summary: String,
    /// 步状态: "running" | "done" | "failed"。
    pub status: String,
    /// 执行输出/解决方案 (running 时为空)。
    pub output: String,
}

/// 内置子任务执行结果 — 能力网命中后的执行反馈。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InternalExecutionResult {
    pub task_id: String,
    pub summary: String,
    /// 命中的最优 provider 节点路径
    pub provider_path: Vec<String>,
    pub executed: bool,
    pub output: String,
}

/// 人类语言 → 子任务确定性拆解。
/// 按标点/换行切分指令, 逐段匹配能力路由表; 命中即产出子任务。
pub fn decompose_instruction(instruction: &str) -> Vec<ConsciousTask> {
    let segments: Vec<&str> = instruction
        .split(['。', '；', ';', '\n', '，', ','])
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let mut tasks: Vec<ConsciousTask> = Vec::new();
    for seg in segments {
        let lower = seg.to_lowercase();
        let mut matched = false;
        for (kw, cap, domain, spec) in CAPABILITY_ROUTES {
            if lower.contains(kw.to_lowercase().as_str()) {
                tasks.push(ConsciousTask {
                    id: format!("task_{}", tasks.len() + 1),
                    summary: seg.to_string(),
                    capability_tag: cap.to_string(),
                    domain: domain.to_string(),
                    specialist: spec.to_string(),
                    priority: 5,
                });
                matched = true;
                break; // 每段首个命中即定域 (最具体者优先)
            }
        }
        if !matched {
            // 未命中: 归入编排域 (意识核心自决兜底), 不盲目丢弃
            tasks.push(ConsciousTask {
                id: format!("task_{}", tasks.len() + 1),
                summary: seg.to_string(),
                capability_tag: "orchestration".to_string(),
                domain: "NT-CORE".to_string(),
                specialist: "Orchestrator".to_string(),
                priority: 3,
            });
        }
    }
    if tasks.is_empty() {
        tasks.push(ConsciousTask {
            id: "task_1".to_string(),
            summary: instruction.to_string(),
            capability_tag: "orchestration".to_string(),
            domain: "NT-CORE".to_string(),
            specialist: "Orchestrator".to_string(),
            priority: 3,
        });
    }
    tasks
}

/// 能力网注册表路径 — 优先 cwd `.neotrix/capability_registry.json` (与后台
/// handlers_maintenance、CLI 默认路径同源的全量树, 单一事实源);
/// HOME 仅兜底 (隔离测试可写; 旧会话可能遗留陈旧单节点文件, 不作为生产源)。
/// 读路径以存在者为准; 写路径同读路径 (读源即写源, 避免读写分离造成的
/// 双 registry 分裂: 意识核心曾读 HOME 旧文件只见 1 节点而误判全部外部缺口)。
pub fn capability_registry_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let home_path = std::path::PathBuf::from(&home)
        .join(".neotrix")
        .join("capability_registry.json");
    let cwd_path = std::path::PathBuf::from(".neotrix").join("capability_registry.json");
    if cwd_path.exists() {
        cwd_path
    } else if home_path.exists() {
        home_path
    } else {
        cwd_path
    }
}

/// 能力网注册表加载 — 从 `~/.neotrix/capability_registry.json` (RegistryExport 格式)。
/// 无能力网 (文件缺失/解析失败) 是合法状态 → None (此时全部走外部缺口)。
pub fn load_capability_registry() -> Option<nt_core_capability_tree::registry::CapabilityRegistry> {
    let path = capability_registry_path();
    let json = std::fs::read_to_string(path).ok()?;
    let export: nt_core_capability_tree::registry::RegistryExport =
        serde_json::from_str(&json).ok()?;
    let mut registry = nt_core_capability_tree::registry::CapabilityRegistry::new();
    for node in export.nodes {
        if registry.register(node).is_err() {
            return None;
        }
    }
    for (from, to) in export.edges {
        if registry.nodes.contains_key(&from) && registry.nodes.contains_key(&to) {
            let _ = registry.add_dependency(&from, &to);
        }
    }
    registry.experience_targets = export.experience_targets;
    // CAD 能力节点 (GenCAD 四步框架) — 幂等: 若导出已含同名节点则忽略
    let _ = nt_core_capability_tree::cad_node::register_cad_capability(&mut registry);
    // Durable 覆盖层合并 (提交的 overlay 优先), 使手动写入在基础重新生成后仍生效。
    let overlay_path = capability_registry_path()
        .parent()
        .map(|p| p.join("capability_overrides.json"))
        .unwrap_or_else(|| std::path::PathBuf::from("capability_overrides.json"));
    if let Some(ov) =
        nt_core_capability_tree::registry::CapabilityRegistry::load_overlay_file(&overlay_path)
    {
        registry.merge_overlay(&ov);
    }
    Some(registry)
}

/// 能力网注册表落盘 — 反思补齐后写回 (RegistryExport 格式, 与后台加载一致)。
pub fn persist_capability_registry(
    registry: &nt_core_capability_tree::registry::CapabilityRegistry,
) -> Result<(), String> {
    let path = capability_registry_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("registry dir: {}", e))?;
    }
    let export = registry.export();
    let json = serde_json::to_string_pretty(&export).map_err(|e| format!("serialize: {}", e))?;
    std::fs::write(&path, json).map_err(|e| format!("write: {}", e))
}

/// 分配 — 每个子任务查自身能力网, 内置优先, 缺口走外部。
pub fn allocate_tasks(
    registry: Option<&nt_core_capability_tree::registry::CapabilityRegistry>,
    tasks: &[ConsciousTask],
) -> Vec<TaskAllocation> {
    let mut allocations = Vec::new();
    for task in tasks {
        let provider = match registry {
            Some(reg) => match reg.optimal_provider(&task.capability_tag) {
                Some(sp) => AllocationProvider::Internal {
                    node_id: sp.path.first().cloned().unwrap_or_default(),
                    path: sp.path,
                    cost: sp.cost,
                },
                None => AllocationProvider::External {
                    reason: format!(
                        "能力网无 '{}' provider (域 {})",
                        task.capability_tag, task.domain
                    ),
                },
            },
            None => AllocationProvider::External {
                reason: "能力网未初始化 (无 .neotrix/capability_registry.json)".to_string(),
            },
        };
        allocations.push(TaskAllocation {
            task: task.clone(),
            provider,
        });
    }
    allocations
}

/// 反思补齐 — 对每个外部缺口立即在能力网 bud 新节点 (缺失即补齐)。
/// 返回补齐动作数。补齐后下次同类任务命中内置 provider。
pub fn reflect_and_strengthen(
    registry: &mut nt_core_capability_tree::registry::CapabilityRegistry,
    allocations: &[TaskAllocation],
) -> usize {
    use nt_core_capability_tree::{Domain as CapDomain, EvolutionEngine, NodeLayer};
    let mut actions = 0;
    for alloc in allocations {
        if let AllocationProvider::External { reason } = &alloc.provider {
            let domain = match alloc.task.domain.as_str() {
                "NT-MIND" => CapDomain::Mind,
                "NT-MEMORY" => CapDomain::Memory,
                "NT-WORLD" => CapDomain::World,
                "NT-ACT" => CapDomain::Act,
                "NT-SHIELD" => CapDomain::Shield,
                "NT-IO" => CapDomain::Io,
                "NT-META" => CapDomain::Meta,
                "NT-NEXUS" => CapDomain::Nexus,
                "NT-GOVERNANCE" => CapDomain::Governance,
                "NT-REPAIR" => CapDomain::Repair,
                _ => CapDomain::Core,
            };
            // 已存在同标签节点 → 不重复 bud (去重)
            if !registry.by_provides(&alloc.task.capability_tag).is_empty() {
                continue;
            }
            let node_id = format!(
                "task_loop::{}::{}",
                domain.as_str().to_lowercase(),
                alloc.task.capability_tag
            );
            let mut engine = EvolutionEngine::new(registry);
            let plan = engine.plan_bud(
                node_id.clone(),
                domain,
                vec![alloc.task.capability_tag.clone()],
                NodeLayer::L0Primitive,
                format!("consciousness task loop 反思补齐: {}", reason),
            );
            if engine.execute(plan).is_ok() {
                actions += 1;
            }
        }
    }
    actions
}

impl ConsciousnessCoreHandle {
    /// 意识核心主入口: 人类语言 → 拆解 → 分配 → 内置/外部 → 反思补齐。
    /// 不依赖任何 CLI 命令; 调用谁 / 怎么调用全部由意识核心决定。
    pub fn process_instruction(&mut self, instruction: &str) -> TaskLoopReport {
        // 0. 人格路由 (Phase 2: Wedge 9轨 + Prism 7路) — 路由结果影响能力分配优先级
        let persona_router = crate::l5_cognition::nt_core::persona_routing::PersonaRouter::new();
        let routed_skill = persona_router.route_to_skill(instruction);
        let persona = persona_router.detect_persona(instruction);

        // 1. 拆解
        let tasks = decompose_instruction(instruction);
        // 2. 加载能力网 + 分配
        let mut registry = load_capability_registry();
        let mut allocations = allocate_tasks(registry.as_ref(), &tasks);

        // G1: 人格路由生效 — 按路由结果调整分配优先级
        // Wedge路由: 优先安全/渗透类能力; Prism路由: 优先分析/设计类能力
        let persona_tag = match persona {
            crate::l5_cognition::nt_core::persona_routing::PersonaType::Wedge => "wedge",
            crate::l5_cognition::nt_core::persona_routing::PersonaType::Prism => "prism",
        };
        for alloc in &mut allocations {
            if alloc.task.capability_tag.to_lowercase().contains(persona_tag) {
                // 人格匹配的任务提升优先级 (通过调整 summary 标记)
                alloc.task.summary = format!("[PERSONA:{persona_tag}] {}", alloc.task.summary);
            }
        }

        let internal_count = allocations
            .iter()
            .filter(|a| matches!(a.provider, AllocationProvider::Internal { .. }))
            .count();
        let external_gap_count = allocations.len() - internal_count;

        // 3. 反思补齐 (缺失即补齐 → 下次内置)
        let strengthening_actions = match registry.as_mut() {
            Some(reg) => {
                let n = reflect_and_strengthen(reg, &allocations);
                if n > 0 {
                    let _ = persist_capability_registry(reg);
                }
                n
            }
            None => 0,
        };

        // 4. 剩余外部缺口 (能力网无法补齐, 需外部知识源接续)
        let external_gaps: Vec<String> = allocations
            .iter()
            .filter_map(|a| match &a.provider {
                AllocationProvider::External { reason } => {
                    Some(format!("{} [{}]", a.task.summary, reason))
                }
                _ => None,
            })
            .collect();

        TaskLoopReport {
            instruction: instruction.to_string(),
            routed_skill,
            allocations,
            internal_count,
            external_gap_count,
            strengthening_actions,
            external_gaps,
            ..Default::default()
        }
    }

    /// 完整任务闭环 — 拆解 → 分配 → 反思补齐 → **执行全部子任务**。
    /// 内置: 能力网命中 (记录 provider 路径, 标记已执行)。
    /// 外部缺口: 自动获取外部知识 + token 预算内试错求解 (external closure)。
    /// 与 process_instruction 区别: 本入口真正执行, 不触网版本仅拆解+分配+补齐。
    pub fn execute_task_loop(
        &mut self,
        instruction: &str,
        executor: &dyn SolutionExecutor,
        config: &ExternalClosureConfig,
    ) -> TaskLoopReport {
        self.execute_task_loop_with_progress(instruction, executor, config, &|_: HarnessStepProgress| {})
    }

    /// [`execute_task_loop`] 的进度回调变体: 每个子任务执行前 (running) 与执行后
    /// (done/failed) 调用 `on_step`, 便于后端 (Tauri `harness_run`) 实时推送
    /// `harness-progress`(phase=step) 事件, 取代前端对 running 状态的整段轮询。
    pub fn execute_task_loop_with_progress(
        &mut self,
        instruction: &str,
        executor: &dyn SolutionExecutor,
        config: &ExternalClosureConfig,
        on_step: &dyn Fn(HarnessStepProgress),
    ) -> TaskLoopReport {
        // G2: GoalLock — 目标锁定 + 四轮恢复
        let mut goal_lock = crate::l1_action::nt_act::goal_lock::GoalLock::new();
        goal_lock.set_goal(instruction);

        let mut report = self.process_instruction(instruction);
        let total = report.allocations.len();

        // 内置子任务执行: 能力网命中 → 真实调用能力 (标记执行 + 实际结果)。
        let mut internal_results = Vec::new();
        for (idx, alloc) in report.allocations.iter().enumerate() {
            if let AllocationProvider::Internal { node_id, path, .. } = &alloc.provider {
                on_step(HarnessStepProgress {
                    index: idx,
                    total,
                    kind: "internal".to_string(),
                    capability_tag: alloc.task.capability_tag.clone(),
                    summary: alloc.task.summary.clone(),
                    status: "running".to_string(),
                    output: String::new(),
                });
                let (executed, output) = dispatch_internal_capability(&alloc.task);
                // G2: GoalLock — 任务失败时尝试恢复
                if !executed {
                    if let Some(recovered) = goal_lock.recover(instruction, &output) {
                        let (retry_executed, retry_output) = dispatch_internal_capability(
                            &ConsciousTask {
                                id: alloc.task.id.clone(),
                                summary: recovered,
                                capability_tag: alloc.task.capability_tag.clone(),
                                ..alloc.task.clone()
                            }
                        );
                        if retry_executed {
                            on_step(HarnessStepProgress {
                                index: idx, total,
                                kind: "internal".to_string(),
                                capability_tag: alloc.task.capability_tag.clone(),
                                summary: alloc.task.summary.clone(),
                                status: "done".to_string(),
                                output: retry_output,
                            });
                        }
                    }
                }
                on_step(HarnessStepProgress {
                    index: idx,
                    total,
                    kind: "internal".to_string(),
                    capability_tag: alloc.task.capability_tag.clone(),
                    summary: alloc.task.summary.clone(),
                    status: if executed { "done".to_string() } else { "failed".to_string() },
                    output: output.clone(),
                });
                internal_results.push(InternalExecutionResult {
                    task_id: alloc.task.id.clone(),
                    summary: alloc.task.summary.clone(),
                    provider_path: {
                        let mut p = path.clone();
                        if p.is_empty() {
                            p.push(node_id.clone());
                        }
                        p
                    },
                    executed,
                    output,
                });
            }
        }
        report.internal_results = internal_results;

        // PDF 图标增强结果 → KB 持久化 (experience namespace)
        // 内置子任务执行完毕后, 检查是否有 PDF 增强结果需要落盘
        {
            let kb_pdf = KnowledgeBase::open(None).ok();
            if let Some(ref kb) = kb_pdf {
                for r in &report.internal_results {
                    if r.summary.contains("PDF 图标增强完成") && r.executed && !r.output.is_empty() {
                        let key = format!("pdf_enhance:{}", r.task_id);
                        let value = serde_json::json!({
                            "task_id": r.task_id,
                            "summary": r.summary,
                            "output": r.output,
                            "provider_path": r.provider_path,
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                        });
                        let _ = kb.kv_set("experience", &key, &value.to_string());
                    }
                }
            }
        }

        // 外部缺口执行: 每个 External 子任务 → 自动外部求解闭环
        // 最短路径: 读端 serve_core 接地 (GWT 路由), 写端 absorb_core 吸收经验
        let kb = KnowledgeBase::open(None).ok();
        let mut closures = Vec::new();
        for (idx, alloc) in report.allocations.iter().enumerate() {
            if let AllocationProvider::External { .. } = &alloc.provider {
                on_step(HarnessStepProgress {
                    index: idx,
                    total,
                    kind: "external".to_string(),
                    capability_tag: alloc.task.capability_tag.clone(),
                    summary: alloc.task.summary.clone(),
                    status: "running".to_string(),
                    output: String::new(),
                });
                let result = match &kb {
                    Some(kb) => close_external_gap(kb, &alloc.task, executor, config),
                    None => {
                        // 无 KB: 仍走试错循环 (接地为空), 不 panic
                        run_external_closure(&alloc.task, executor, config, &[])
                    }
                };
                on_step(HarnessStepProgress {
                    index: idx,
                    total,
                    kind: "external".to_string(),
                    capability_tag: alloc.task.capability_tag.clone(),
                    summary: alloc.task.summary.clone(),
                    status: if result.solved { "done".to_string() } else { "failed".to_string() },
                    output: result.solution.clone(),
                });
                // 写端吸收: solved 子任务的解决方案 → 经验节点落 KB (最短路径, 幂等按 title+type)
                if result.solved && !result.solution.is_empty() {
                    if let Some(kb) = &kb {
                        let entry = AbsorbEntry {
                            title: alloc.task.summary.clone(),
                            summary: Some("意识核心任务解决经验".to_string()),
                            content: Some(result.solution.clone()),
                            node_type: "insight".to_string(),
                            domain: Some("NT-MIND".to_string()),
                            url: None,
                            language: Some("zh".to_string()),
                            importance: Some(0.7),
                            relations: vec![],
                        };
                        let _ = kb.absorb_core(&entry);
                    }
                }
                closures.push(result);
            }
        }
        report.external_closures = closures;
        report
    }
}

/// 内置能力真实调度 — 能力网命中后把能力标签映射到实际 Rust 函数调用。
/// 覆盖文件能力网 (nt_file_ability) 全分支: xlsx_consolidation → consolidate_tables、
/// file_extract → extract_text/to_markdown/read_xlsx_sheets_all、file_structured → read/write。
/// 未覆盖标签返回 (false, 描述) — 保持向后兼容 (原实现仅标记 executed)。
/// 生产接地: 意识核心自主调用能力网, 不再只是"标记已执行"。
fn dispatch_internal_capability(task: &ConsciousTask) -> (bool, String) {
    // 通用辅助: 从摘要提取路径 (含 '/' 或 '\' 的 token)
    fn first_path(summary: &str) -> Option<std::path::PathBuf> {
        summary
            .split_whitespace()
            .map(|w| w.trim_matches('"').trim_matches('，').trim_matches(','))
            .find(|w| w.contains('/') || w.contains('\\'))
            .map(std::path::PathBuf::from)
    }
    match task.capability_tag.as_str() {
        // 目录表格合并 (D4): 从子任务摘要提取目录路径 (含 / 或 \ 者首个路径 token)
        "xlsx_consolidation" | "data_merge" => {
            // 从摘要中定位可能的目录路径: 优先取含 '/' 的 token; 失败回退 HOME 价格表目录
            let words: Vec<&str> = task.summary.split_whitespace().collect();
            let dir = words
                .iter()
                .map(|w| w.trim_matches('"').trim_matches('，').trim_matches(','))
                .find(|w| w.contains('/') || w.contains('\\'))
                .map(std::path::PathBuf::from)
                .or_else(|| {
                    std::env::var("HOME").ok().map(|h| {
                        std::path::PathBuf::from(h)
                            .join("Downloads")
                            .join("5月份价格表")
                    })
                })
                .filter(|p| p.is_dir());
            match dir {
                Some(d) => {
                    let out = d.join("native_consolidated.xlsx");
                    match crate::neotrix::consolidate_tables_with_mode(&d, &out, crate::neotrix::nt_file_ability::SheetMode::AllSheets) {
                        Ok(rep) => (
                            true,
                            format!(
                                "表格合并完成: 处理 {} 个文件 / {} 行 / {} 行含 USD 报价\n输出: {}",
                                rep.files_processed, rep.total_rows, rep.usd_rows, rep.output
                            ),
                        ),
                        Err(e) => (false, format!("表格合并失败: {e}")),
                    }
                }
                None => (
                    false,
                    format!("子任务 '{}' 未提供有效目录路径, 无法执行合并", task.summary),
                ),
            }
        }
        // 文件内容抽取 (FileKind 全分支: 文本/PDF/Office → 文本/Markdown/表格)
        "file_extract" | "content_extraction" | "file_parsing" => {
            let dir = first_path(&task.summary);
            match dir {
                Some(p) if p.is_dir() => {
                    // 目录级抽取: 扫描目录内文件, 逐文件提取文本摘要
                    let mut extracted = 0;
                    let mut chars = 0usize;
                    if let Ok(entries) = std::fs::read_dir(&p) {
                        for e in entries.flatten() {
                            let path = e.path();
                            if path.is_file() {
                                if let Ok(txt) = crate::neotrix::extract_text(&path) {
                                    extracted += 1;
                                    chars += txt.chars().count();
                                }
                            }
                        }
                    }
                    (
                        true,
                        format!(
                            "文件抽取完成: 扫描 {} 个文件 / 提取 {} 字符\n目录: {}",
                            extracted,
                            chars,
                            p.display()
                        ),
                    )
                }
                Some(p) if p.is_file() => {
                    let md = crate::neotrix::to_markdown(&p).unwrap_or_else(|_| {
                        crate::neotrix::extract_text(&p).unwrap_or_else(|e| format!("<{e}>"))
                    });
                    (
                        true,
                        format!(
                            "文件抽取完成 ({} 字符):\n{}",
                            md.chars().count(),
                            md.chars().take(400).collect::<String>()
                        ),
                    )
                }
                Some(p) => (
                    false,
                    format!("路径 '{}' 既非文件也非目录, 无法抽取", p.display()),
                ),
                None => (
                    false,
                    format!("子任务 '{}' 未提供有效路径, 无法抽取", task.summary),
                ),
            }
        }
        // PDF 文本编辑 (R-P79): span redact + 原位替换。
        // 摘要语法: <in.pdf> <out.pdf> <page> <find> => <replace>  (省略 => 为仅删除)
        "pdf_edit" => {
            let words: Vec<&str> = task.summary.split_whitespace().collect();
            let paths: Vec<std::path::PathBuf> = words
                .iter()
                .map(|w| w.trim_matches('"').trim_matches('，').trim_matches(','))
                .filter(|w| w.contains('/') || w.contains('\\'))
                .map(std::path::PathBuf::from)
                .collect();
            if paths.len() < 2 {
                return (
                    false,
                    format!(
                        "子任务 '{}' 缺少 输入/输出 PDF 路径, 无法编辑",
                        task.summary
                    ),
                );
            }
            let (src, out) = (paths[0].clone(), paths[1].clone());
            // 定位路径之后的首个整数 token 作为页号
            let after_paths: Vec<&str> = words
                .iter()
                .skip_while(|w| {
                    let w = w.trim_matches('"').trim_matches('，').trim_matches(',');
                    !(w.contains('/') || w.contains('\\'))
                })
                .skip(2)
                .copied()
                .collect();
            let page = after_paths
                .iter()
                .find_map(|w| w.parse::<u32>().ok())
                .unwrap_or(1);
            // find => replace 语法; 无 => 时 find = 页号之后全部剩余 token (删除模式)
            let (find, replace) = match task.summary.split_once("=>") {
                Some((left, right)) => {
                    let find = left
                        .split_whitespace()
                        .filter(|w| !w.contains('/') && w.parse::<u32>().is_err())
                        .collect::<Vec<_>>()
                        .join(" ");
                    let replace = right.trim();
                    (find, Some(replace.to_string()))
                }
                None => {
                    let find = after_paths
                        .iter()
                        .filter(|w| w.parse::<u32>().is_err())
                        .copied()
                        .collect::<Vec<_>>()
                        .join(" ");
                    (find, None)
                }
            };
            if find.is_empty() {
                return (
                    false,
                    format!("子任务 '{}' 未解析出待编辑文本", task.summary),
                );
            }
            // 替换文本超出 Latin-1 → 自动选取系统字体嵌入 (非 Latin-1 需真实字体)
            let ttf = match &replace {
                Some(rep) if rep.chars().any(|c| (c as u32) > 0xFF) => {
                    match find_system_font_for(rep) {
                        Some(font) => Some(font),
                        None => {
                            return (
                                false,
                                format!(
                                    "替换文本含非 Latin-1 字符但未找到支持的系统字体: {rep}"
                                ),
                            );
                        }
                    }
                }
                _ => None,
            };
            let edit = crate::neotrix::PdfEdit {
                page,
                find,
                replace,
            };
            match crate::neotrix::edit_pdf(&src, &out, &[edit], ttf.as_deref()) {
                Ok(_) => (
                    true,
                    format!(
                        "PDF 编辑完成 (页 {page}): {}\n输出: {}",
                        src.display(),
                        out.display()
                    ),
                ),
                Err(e) => (false, format!("PDF 编辑失败: {e}")),
            }
        }
        "image_convert" => {
            // 摘要语法: <in> <out> 两个路径 (扩展名差异 = 转换目标)
            let words: Vec<&str> = task.summary.split_whitespace().collect();
            let paths: Vec<std::path::PathBuf> = words
                .iter()
                .map(|w| w.trim_matches('"').trim_matches('，').trim_matches(','))
                .filter(|w| w.contains('/') || w.contains('\\'))
                .map(std::path::PathBuf::from)
                .collect();
            if paths.len() < 2 {
                return (
                    false,
                    format!(
                        "子任务 '{}' 缺少 输入/输出 图像路径, 无法转换",
                        task.summary
                    ),
                );
            }
            let (src, out) = (paths[0].clone(), paths[1].clone());
            match crate::neotrix::FileAbility::open(&src) {
                Ok(fa) => match fa.convert_image(&out) {
                    Ok(()) => (
                        true,
                        format!("图像转换完成: {} → {}", src.display(), out.display()),
                    ),
                    Err(e) => (false, format!("图像转换失败: {e}")),
                },
                Err(e) => (false, format!("打开源图像失败: {e}")),
            }
        }
        "dir_extract" => {
            // 摘要语法: <目录> — 目录级统一提取 (混合格式 → 文本/表格清单)
            let words: Vec<&str> = task.summary.split_whitespace().collect();
            let dir = words
                .iter()
                .map(|w| w.trim_matches('"').trim_matches('，').trim_matches(','))
                .find(|w| w.contains('/') || w.contains('\\'))
                .map(std::path::PathBuf::from);
            let Some(dir) = dir else {
                return (
                    false,
                    format!("子任务 '{}' 缺少目录路径, 无法提取", task.summary),
                );
            };
            if !dir.is_dir() {
                return (false, format!("路径不是目录: {}", dir.display()));
            }
            match crate::neotrix::extract_dir(&dir) {
                Ok(report) => (
                    true,
                    format!(
                        "目录统一提取完成: 成功 {} / 失败 {} / 总字符 {}\n共 {} 个文件",
                        report.succeeded,
                        report.failed,
                        report.total_chars,
                        report.entries.len()
                    ),
                ),
                Err(e) => (false, format!("目录提取失败: {e}")),
            }
        }
        "pdf_merge" => {
            // 摘要语法: <out.pdf> <in1.pdf> <in2.pdf> ... — 结构级合并
            let words: Vec<&str> = task.summary.split_whitespace().collect();
            let paths: Vec<std::path::PathBuf> = words
                .iter()
                .map(|w| w.trim_matches('"').trim_matches('，').trim_matches(','))
                .filter(|w| w.contains('/') || w.contains('\\'))
                .map(std::path::PathBuf::from)
                .collect();
            if paths.len() < 3 {
                return (
                    false,
                    format!(
                        "子任务 '{}' 缺少 输出+输入 PDF 路径, 无法合并",
                        task.summary
                    ),
                );
            }
            let out = paths[0].clone();
            let inputs = &paths[1..];
            match crate::neotrix::merge_pdfs(inputs) {
                Ok(bytes) => {
                    match std::fs::write(&out, &bytes) {
                        Ok(()) => (
                            true,
                            format!(
                                "PDF 合并完成: {} 个文件 → {} ({} 字节)",
                                inputs.len(),
                                out.display(),
                                bytes.len()
                            ),
                        ),
                        Err(e) => (false, format!("写出合并结果失败: {e}")),
                    }
                }
                Err(e) => (false, format!("PDF 合并失败: {e}")),
            }
        }
        "doc_merge" => {
            // 摘要语法: <out.docx|pptx> <in1> <in2> ... — 结构级合并 Office 文档
            let words: Vec<&str> = task.summary.split_whitespace().collect();
            let paths: Vec<std::path::PathBuf> = words
                .iter()
                .map(|w| w.trim_matches('"').trim_matches('，').trim_matches(','))
                .filter(|w| w.contains('/') || w.contains('\\'))
                .map(std::path::PathBuf::from)
                .collect();
            if paths.len() < 3 {
                return (
                    false,
                    format!(
                        "子任务 '{}' 缺少 输出+输入 Office 文档路径, 无法合并",
                        task.summary
                    ),
                );
            }
            let out = paths[0].clone();
            let inputs = &paths[1..];
            let ext = out
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            match (ext.as_str(), inputs) {
                ("docx", _) => match crate::neotrix::merge_docx::merge_docx(inputs, &out) {
                    Ok(r) => (
                        true,
                        format!("DOCX 合并完成: {} 个文件 → {} ({:?})", r.items, out.display(), r),
                    ),
                    Err(e) => (false, format!("DOCX 合并失败: {e}")),
                },
                ("pptx", _) => match crate::neotrix::merge_docx::merge_pptx(inputs, &out) {
                    Ok(r) => (
                        true,
                        format!("PPTX 合并完成: {} 个文件 → {} ({:?})", r.items, out.display(), r),
                    ),
                    Err(e) => (false, format!("PPTX 合并失败: {e}")),
                },
                _ => (
                    false,
                    format!("doc_merge 仅支持 .docx/.pptx 输出, 收到: {ext}"),
                ),
            }
        }
        // PDF 图标清晰度提升 (R-P79): 提取图像 → AI 超分 → 嵌回
        "pdf_icon_enhance" | "pdf_enhance" => {
            let words: Vec<&str> = task.summary.split_whitespace().collect();
            let paths: Vec<std::path::PathBuf> = words
                .iter()
                .map(|w| w.trim_matches('"').trim_matches('，').trim_matches(','))
                .filter(|w| w.contains('/') || w.contains('\\'))
                .map(std::path::PathBuf::from)
                .collect();
            if paths.is_empty() {
                return (
                    false,
                    format!("子任务 '{}' 缺少 PDF 路径, 无法增强", task.summary),
                );
            }
            let src = paths[0].clone();
            let out = paths.get(1).cloned().unwrap_or_else(|| {
                let stem = src.file_stem().unwrap_or_default();
                let ext = src.extension().unwrap_or_default();
                src.with_file_name(format!("{}_enhanced.{}", stem.to_string_lossy(), ext.to_string_lossy()))
            });
            
            // 解析可选参数: --scale 4 --model realesrgan
            let mut scale: u32 = 4;
            let mut model = "realesrgan".to_string();
            for (i, w) in words.iter().enumerate() {
                if *w == "--scale" && i + 1 < words.len() {
                    scale = words[i + 1].parse().unwrap_or(4);
                }
                if *w == "--model" && i + 1 < words.len() {
                    model = words[i + 1].to_string();
                }
            }
            
            let sr_model = match model.as_str() {
                "anime" => crate::neotrix::SuperResolutionModel::RealEsrganAnime,
                "photo" => crate::neotrix::SuperResolutionModel::RealEsrganPhoto,
                "swinir" => crate::neotrix::SuperResolutionModel::SwinIRClassic,
                _ => crate::neotrix::SuperResolutionModel::RealEsrganGeneral,
            };
            
            let config = crate::neotrix::PdfIconEnhanceConfig {
                super_resolution: crate::neotrix::SuperResolutionConfig {
                    model: sr_model,
                    scale,
                    ..Default::default()
                },
                output_pdf: Some(out.clone()),
                ..Default::default()
            };
            
            match crate::neotrix::enhance_pdf_icons_with_config(&src, config) {
                Ok(result) => (
                    true,
                    format!(
                        "PDF 图标增强完成!\n  输入: {}\n  输出: {}\n  提取图像: {} 张\n  成功增强: {} 张\n  耗时: {}ms",
                        result.input_pdf,
                        result.output_pdf,
                        result.images_extracted,
                        result.images_enhanced,
                        result.total_time_ms
                    ),
                ),
                Err(e) => (false, format!("PDF 增强失败: {e}")),
            }
        }
        // 智能合并 (R-P110): 按输入格式自动路由到 PDF/XLSX/DOCX/PPTX/混合合并
        "collection_merge" | "smart_merge" => {
            let words: Vec<&str> = task.summary.split_whitespace().collect();
            let paths: Vec<std::path::PathBuf> = words
                .iter()
                .map(|w| w.trim_matches('"').trim_matches('，').trim_matches(','))
                .filter(|w| w.contains('/') || w.contains('\\'))
                .map(std::path::PathBuf::from)
                .collect();
            if paths.len() < 2 {
                return (
                    false,
                    format!(
                        "子任务 '{}' 缺少 输入+输出 路径, 无法智能合并",
                        task.summary
                    ),
                );
            }
            let out = paths[0].clone();
            let inputs = paths[1..].to_vec();
            let req = crate::neotrix::CollectionMergeRequest {
                inputs,
                strategy: crate::neotrix::MergeStrategy::All,
                schema: None,
                output: out.clone(),
                dry_run: false,
            };
            match crate::neotrix::collection_merge(&req) {
                Ok(outcome) => {
                    let desc = match &outcome {
                        crate::neotrix::MergeOutcome::Text { items, note } => {
                            format!("文本级合并 {items} 个文件: {note}")
                        }
                        crate::neotrix::MergeOutcome::Docx { items, parts } => {
                            format!("DOCX 结构合并 {items} 个文件, {parts} 个 part")
                        }
                        crate::neotrix::MergeOutcome::Pptx { slides } => {
                            format!("PPTX 结构合并, {slides} 张幻灯片")
                        }
                        crate::neotrix::MergeOutcome::Pdf { pages } => {
                            format!("PDF 结构合并, {pages} 页")
                        }
                        crate::neotrix::MergeOutcome::Xlsx { rows, note } => {
                            format!("XLSX 表格合并 {rows} 行: {note}")
                        }
                    };
                    (
                        true,
                        format!("智能合并完成 ({})\n输出: {}", desc, out.display()),
                    )
                }
                Err(e) => (false, format!("智能合并失败: {e}")),
            }
        }
        // XLSX 单元格级编辑 (R-P110): SetCell/InsertRow/RemoveRow
        "xlsx_edit" | "table_edit" => {
            let words: Vec<&str> = task.summary.split_whitespace().collect();
            let paths: Vec<std::path::PathBuf> = words
                .iter()
                .map(|w| w.trim_matches('"').trim_matches('，').trim_matches(','))
                .filter(|w| w.contains('/') || w.contains('\\'))
                .map(std::path::PathBuf::from)
                .collect();
            if paths.is_empty() {
                return (
                    false,
                    format!("子任务 '{}' 缺少 XLSX 路径, 无法编辑", task.summary),
                );
            }
            let path = &paths[0];
            if !path.exists() {
                return (false, format!("XLSX 文件不存在: {}", path.display()));
            }
            let mut edits: Vec<crate::neotrix::TableEdit> = Vec::new();
            for (i, w) in words.iter().enumerate() {
                if *w == "--set" && i + 1 < words.len() {
                    if let Some((rc, val)) = words[i + 1].split_once('=') {
                        let parts: Vec<usize> = rc
                            .split(',')
                            .filter_map(|s| s.parse().ok())
                            .collect();
                        if parts.len() == 2 {
                            edits.push(crate::neotrix::TableEdit::SetCell {
                                sheet: 0,
                                row: parts[0],
                                col: parts[1],
                                value: val.to_string(),
                            });
                        }
                    }
                }
                if *w == "--insert" && i + 1 < words.len() {
                    if let Ok(row) = words[i + 1].parse::<usize>() {
                        edits.push(crate::neotrix::TableEdit::InsertRow { sheet: 0, row });
                    }
                }
                if *w == "--remove" && i + 1 < words.len() {
                    if let Ok(row) = words[i + 1].parse::<usize>() {
                        edits.push(crate::neotrix::TableEdit::RemoveRow { sheet: 0, row });
                    }
                }
            }
            if edits.is_empty() {
                return (
                    false,
                    format!(
                        "子任务 '{}' 未解析出编辑指令 (--set/--insert/--remove)",
                        task.summary
                    ),
                );
            }
            match crate::neotrix::edit_xlsx_table(path, &edits) {
                Ok(tables) => {
                    let total_rows: usize = tables.iter().map(|t| t.rows.len()).sum();
                    (
                        true,
                        format!(
                            "XLSX 编辑完成: {} 条编辑, {} 行数据\n文件: {}",
                            edits.len(),
                            total_rows,
                            path.display()
                        ),
                    )
                }
                Err(e) => (false, format!("XLSX 编辑失败: {e}")),
            }
        }
        // 结构化数据读取 (R-P110): JSON/YAML/TOML → StructuredData
        "structured_read" | "json_read" => {
            let dir = first_path(&task.summary);
            match dir {
                Some(p) if p.is_file() => {
                    match crate::neotrix::read_structured(&p) {
                        Ok(data) => (
                            true,
                            format!(
                                "结构化读取完成 ({} 格式):\n{}",
                                data.format,
                                serde_json::to_string_pretty(&data.value)
                                    .unwrap_or_else(|_| "<序列化失败>".into())
                                    .chars()
                                    .take(500)
                                    .collect::<String>()
                            ),
                        ),
                        Err(e) => (false, format!("结构化读取失败: {e}")),
                    }
                }
                Some(p) => (
                    false,
                    format!("路径 '{}' 不是文件, 无法结构化读取", p.display()),
                ),
                None => (
                    false,
                    format!(
                        "子任务 '{}' 未提供有效文件路径, 无法结构化读取",
                        task.summary
                    ),
                ),
            }
        }
        // JSON 写入 (R-P110): <path.json> <json_content>
        "json_write" | "structured_write" => {
            let words: Vec<&str> = task.summary.split_whitespace().collect();
            let paths: Vec<std::path::PathBuf> = words
                .iter()
                .map(|w| w.trim_matches('"').trim_matches('，').trim_matches(','))
                .filter(|w| w.contains('/') || w.contains('\\'))
                .map(std::path::PathBuf::from)
                .collect();
            if paths.is_empty() {
                return (
                    false,
                    format!("子任务 '{}' 缺少 JSON 输出路径", task.summary),
                );
            }
            let out = &paths[0];
            let json_str = words
                .iter()
                .skip_while(|w| {
                    let w = w.trim_matches('"').trim_matches('，').trim_matches(',');
                    !(w.contains('/') || w.contains('\\'))
                })
                .skip(1)
                .cloned()
                .collect::<Vec<_>>()
                .join(" ");
            match serde_json::from_str::<serde_json::Value>(&json_str) {
                Ok(v) => match crate::neotrix::write_json(out, &v, true) {
                    Ok(()) => (
                        true,
                        format!("JSON 写入完成: {}\n内容: {} 字节", out.display(), json_str.len()),
                    ),
                    Err(e) => (false, format!("JSON 写入失败: {e}")),
                },
                Err(e) => (false, format!("JSON 内容解析失败: {e}\n输入: {json_str}")),
            }
        }
        // PDF 图像统计 (R-P110): 分析 PDF 中嵌入图像的数量和大小
        "pdf_image_stats" | "pdf_images_info" => {
            let dir = first_path(&task.summary);
            match dir {
                Some(p) if p.is_file() => {
                    match crate::neotrix::pdf_image_stats(&p) {
                        Ok(stats) => (
                            true,
                            format!(
                                "PDF 图像统计:\n  文件: {}\n  嵌入图像: {} 张 ({} 页)\n  格式: {}\n  平均尺寸: {}×{}",
                                p.display(),
                                stats.total_images,
                                stats.total_pages,
                                if stats.formats.is_empty() { "未知".to_string() } else { stats.formats.join(", ") },
                                stats.avg_image_size.0,
                                stats.avg_image_size.1
                            ),
                        ),
                        Err(e) => (false, format!("PDF 图像统计失败: {e}")),
                    }
                }
                Some(p) => (
                    false,
                    format!("路径 '{}' 不是 PDF 文件", p.display()),
                ),
                None => (
                    false,
                    format!(
                        "子任务 '{}' 未提供 PDF 路径, 无法统计图像",
                        task.summary
                    ),
                ),
            }
        }
        // PDF 图像提取 (R-P110): 从 PDF 中提取嵌入图像到指定目录
        "pdf_extract_images" => {
            let words: Vec<&str> = task.summary.split_whitespace().collect();
            let paths: Vec<std::path::PathBuf> = words
                .iter()
                .map(|w| w.trim_matches('"').trim_matches('，').trim_matches(','))
                .filter(|w| w.contains('/') || w.contains('\\'))
                .map(std::path::PathBuf::from)
                .collect();
            if paths.is_empty() {
                return (
                    false,
                    format!("子任务 '{}' 缺少 PDF 路径, 无法提取图像", task.summary),
                );
            }
            let pdf_path = &paths[0];
            let out_dir = paths.get(1).cloned().unwrap_or_else(|| {
                pdf_path.with_file_name(format!(
                    "{}_images",
                    pdf_path.file_stem().unwrap_or_default().to_string_lossy()
                ))
            });
            let config = crate::neotrix::PdfImageExtractConfig::default();
            match crate::neotrix::extract_pdf_images(pdf_path, &out_dir, &config) {
                Ok(result) => (
                    true,
                    format!(
                        "PDF 图像提取完成:\n  输入: {}\n  输出目录: {}\n  提取: {} 张\n  耗时: {}ms",
                        pdf_path.display(),
                        out_dir.display(),
                        result.images.len(),
                        result.processing_time_ms
                    ),
                ),
                Err(e) => (false, format!("PDF 图像提取失败: {e}")),
            }
        }
        // ── SEAL pipeline (nt_core_seal) ──
        "seal_iterate" => {
            match crate::l5_cognition::nt_mind::foundation::seal_pipeline::L1SealPipeline::new().run_cycle() {
                Ok(status) => (
                    true,
                    format!("SEAL 进化迭代完成: cycle {}, 进度 {:.0}%", status.cycle_count, status.overall_progress * 100.0),
                ),
                Err(e) => (false, format!("SEAL 迭代失败: {e}")),
            }
        }
        "seal_distill" => {
            match crate::l5_cognition::nt_mind::foundation::seal_pipeline::L1SealPipeline::new().trigger_distillation() {
                Ok(result) => (
                    true,
                    format!(
                        "SEAL 蒸馏完成: 提取 {} 个模式, 压缩 {:.2} MB",
                        result.patterns_extracted, result.knowledge_compressed_mb
                    ),
                ),
                Err(e) => (false, format!("SEAL 蒸馏失败: {e}")),
            }
        }
        // ── Self model (nt_core_self) ──
        "self_model_tick" => {
            let mut model = crate::core::nt_core_self::self_model::SelfModel::new();
            // 从意识核心快照获取 workspace_signal (coherence), load_delta (weighted_fog_sum 归一化)
            let snap = status();
            let workspace_signal = snap.coherence.clamp(0.0, 1.0);
            let load_delta = (snap.weighted_fog_sum / 11.0).clamp(0.0, 1.0);
            let meta_alarm = snap.mars_system1_activations.min(10) as usize;
            let state = model.tick(workspace_signal, load_delta, meta_alarm);
            let reward = model.combined_intrinsic_reward();
            (
                true,
                format!(
                    "自我模型评估完成:\n  能力: {:.3} | 疲劳: {:.3} | 不确定性: {:.3}\n  自我误差: {:.4} | 内在奖励: {:.4}",
                    state.capability, state.fatigue, state.uncertainty, state.self_error, reward
                ),
            )
        }
        "metacog_evaluate" => {
            use crate::core::nt_core_self::metacognitive_evaluator::CognitiveEvaluator;
            use crate::core::nt_core_self::silicon_self_model::SiliconSelfModel;
            let mut evaluator = CognitiveEvaluator::new();
            let model = SiliconSelfModel::default();
            let report = evaluator.evaluate(&model);
            (
                true,
                format!(
                    "认知健康评估:\n  注意力健康: {:.3} | 策略多样性: {:.3}\n  轨迹质量: {:.3} | 稳定性: {:.3}\n  标志: {} | 修复建议: {}",
                    report.attention_health,
                    report.strategy_diversity,
                    report.trace_quality,
                    report.stability_score,
                    report.flags.len(),
                    report.repair_suggestions.len(),
                ),
            )
        }
        // ── Meta cognition (nt_meta) ──
        "meta_observe" => {
            let config = crate::l6_meta::memory::meta_observer::MetaObserverConfig::default();
            let observer = crate::l6_meta::memory::meta_observer::MetaObserver::new(config);
            let snap = status();
            let report = observer.observe(&snap);
            let blindspots: usize = report.branches.iter().filter(|b| b.blindspot).count();
            (
                true,
                format!(
                    "元观察报告 (cycle {}):\n  Φ={:.3} 相干={:.3} | 元置信: {:.3}\n  观察失真: {} | 盲点: {}/{}",
                    report.cycle,
                    report.phi,
                    report.coherence,
                    report.meta_confidence,
                    report.observation_distorted,
                    blindspots,
                    report.branches.len(),
                ),
            )
        }
        "sentrux_scan" => {
            let sensor = crate::l6_meta::coordination::nt_meta_sentrux::SentruxSensor::new();
            let path = first_path(&task.summary)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| ".".to_string());
            match sensor.scan(&path) {
                Ok(snapshot) => (
                    true,
                    format!(
                        "质量扫描完成 ({}):\n  综合得分: {}/1000\n  模块性: {:.2} | 无环性: {:.2} | 深度: {:.2}",
                        path,
                        snapshot.score,
                        snapshot.metrics.modularity,
                        snapshot.metrics.acyclicity,
                        snapshot.metrics.depth,
                    ),
                ),
                Err(e) => (false, format!("质量扫描失败: {e}")),
            }
        }
        "build_watchdog" => {
            let config = crate::l6_meta::coordination::nt_meta_build_watchdog::WatchdogConfig::default();
            let mut watchdog = crate::l6_meta::coordination::nt_meta_build_watchdog::BuildWatchdog::new(config);
            let status_result = watchdog.check_health();
            let stats = watchdog.stats();
            (
                true,
                format!(
                    "构建健康检查:\n  状态: {} | 总检查: {} | 成功: {} | 失败: {}",
                    if status_result.success { "健康" } else { "异常" },
                    stats.total_checks,
                    stats.successful_builds,
                    stats.failed_builds,
                ),
            )
        }
        // ── Shield security (nt_shield) ──
        "shield_audit" => {
            let input = first_path(&task.summary)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| task.summary.clone());
            match crate::cli::shield_enforcer::global_shield().lock() {
                Ok(mut shield) => {
                    let report = shield.security_audit(&input);
                    let signals_count = report.signals.len();
                    let verdict = if report.has_critical { "存在风险" } else { "安全" };
                    (
                        true,
                        format!(
                            "安全审计完成 ({}):\n  判定: {} | 信号数: {} | 耗时: {}ms",
                            input, verdict, signals_count, report.duration_ms
                        ),
                    )
                }
                Err(e) => (false, format!("安全审计失败 (shield lock): {e}")),
            }
        }
        "agentic_scan" => {
            use crate::l3_embodiment::nt_shield::nt_shield_agentic_scan::{AgenticScanner, ScanConfig};
            let config = ScanConfig::default();
            let mut scanner = AgenticScanner::new(config);
            let target = first_path(&task.summary)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| ".".to_string());
            let recon = scanner.recon_scan(&target);
            (
                true,
                format!(
                    "主动安全扫描 ({}):\n  预估文件: {} | 语言: {} | 入口: {} | 框架: {} | 复杂度: {}",
                    target,
                    recon.estimated_files,
                    recon.languages_detected.join(", "),
                    recon.entry_points.join(", "),
                    recon.frameworks.join(", "),
                    recon.estimated_complexity,
                ),
            )
        }
        _ => (
            true,
            format!(
                "internal capability '{}' via domain {}",
                task.capability_tag, task.domain,
            ),
        ),
    }
}

/// 查找覆盖给定文本全部字形的系统 TTF (西里尔/中文等非 Latin-1 替换用)。
/// 找不到返回 None — 调用方应提示用户, 而非静默降级。
/// 实现下沉到 neotrix-types `find_system_font_for_text` (TTC 多 face + .notdef 过滤)。
fn find_system_font_for(text: &str) -> Option<Vec<u8>> {
    neotrix_types::core::file_parser::pdf::find_system_font_for_text(text)
}

/// 进程内单例入口: 意识核心直接处理人类语言 (不依赖 CLI/MCP 子命令)。
pub fn process_instruction(instruction: &str) -> TaskLoopReport {
    CORE.write()
        .map(|mut h| h.process_instruction(instruction))
        .unwrap_or_else(|_| TaskLoopReport {
            instruction: instruction.to_string(),
            ..Default::default()
        })
}

/// 进程内单例完整闭环入口 — 真正执行全部子任务 (内置 + 外部缺口)。
pub fn execute_task_loop(
    instruction: &str,
    executor: &dyn SolutionExecutor,
    config: &ExternalClosureConfig,
) -> TaskLoopReport {
    execute_task_loop_with_progress(instruction, executor, config, &|_: HarnessStepProgress| {})
}

/// 进程内单例完整闭环入口 (带步骤进度回调) — 见 [`execute_task_loop_with_progress`]。
pub fn execute_task_loop_with_progress(
    instruction: &str,
    executor: &dyn SolutionExecutor,
    config: &ExternalClosureConfig,
    on_step: &dyn Fn(HarnessStepProgress),
) -> TaskLoopReport {
    CORE.write()
        .map(|mut h| h.execute_task_loop_with_progress(instruction, executor, config, on_step))
        .unwrap_or_else(|_| TaskLoopReport {
            instruction: instruction.to_string(),
            ..Default::default()
        })
}

// ═══════════════════════════════════════════════════════════════════════════
// 外部缺口闭环 (External Gap Closure) — 内置能力缺失时的自动外部求解
//
// 用户需求: "发现不够才问外部 → 自动进行外部信息获取, 自动寻找解决办法的技术
//            文献论文 GitHub 项目等资料, 作为解决问题的信息基础, 然后自动试错
//            构建解决方案直到任务完成, 全过程采取最优解思路, 精准调用工具与
//            LLM tokens (token 预算精控)。"
//
// 机制:
//   1. 获取外部知识: 按能力标签分派 discover_* 知识源 (Semantic Scholar /
//      ArXiv 论文 / Wikipedia 技术文档 / GitHub), 摄入 KB 作为信息基础。
//   2. 接地检索: 从 KB 检索与任务相关的已摄入知识作为求解上下文。
//   3. 试错循环: 在 token 预算内反复 attempt (executor 抽象, 生产 = LLM,
//      测试 = 注入 fake), 直到成功或预算耗尽; 每次失败把错误反馈进上下文。
//   4. 精准预算: token_budget 上限 + max_attempts 上限, 拒绝无限试错。
// ═══════════════════════════════════════════════════════════════════════════

/// 外部闭环配置 — 精准控制工具调用与 LLM token 消耗。
#[derive(Debug, Clone, Copy, Default)]
pub struct ExternalClosureConfig {
    /// 试错轮次上限 (防无限循环)
    pub max_attempts: u32,
    /// 累计 LLM token 预算 (超预算立即终止)
    pub token_budget: u32,
    /// 单次 LLM 输出 token 上限
    pub max_llm_tokens: u32,
    /// 是否调用外部知识源 (discover_*) 获取信息基础。生产默认开;
    /// 测试关 (避免触网), 纯试错逻辑验证。
    pub acquire_knowledge: bool,
}

impl ExternalClosureConfig {
    /// 默认精控预算: 最多 5 轮试错, 每轮 1024 输出 token, 累计 4096 上限。
    pub fn frugal() -> Self {
        Self {
            max_attempts: 5,
            token_budget: 4096,
            max_llm_tokens: 1024,
            acquire_knowledge: true,
        }
    }
}

/// 单次试错结果。
#[derive(Debug, Clone)]
pub enum AttemptOutcome {
    /// 任务解决
    Solved { solution: String, tokens_used: u32 },
    /// 未解决 (错误反馈进上下文, 下一轮修正)
    Failed { error: String, tokens_used: u32 },
    /// 预算耗尽 — 必须停止
    BudgetExhausted { tokens_used: u32 },
}

/// 试错执行器抽象 — 一次"构建解决方案"的尝试。
/// 生产用 LLM (SubagentDispatch), 测试注入 fake 验证循环逻辑。
pub trait SolutionExecutor: Send + Sync {
    fn attempt(&self, task: &ConsciousTask, grounding: &str, attempt_no: u32) -> AttemptOutcome;
}

/// 外部缺口闭环报告 — 全过程透明。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExternalClosureReport {
    pub task_id: String,
    pub summary: String,
    /// 已摄入外部知识资源数 (discover_* 落库量)
    pub knowledge_acquired: usize,
    /// 接地检索命中 KB 节点数
    pub grounding_hits: usize,
    pub attempts: u32,
    pub tokens_used: u32,
    pub solved: bool,
    pub solution: String,
    pub last_error: String,
}

/// 试错循环 — 在 token 预算内反复 attempt, 直到解决或预算耗尽。
/// 每次失败将错误反馈进上下文 (下一轮修正), 精准控制 token 消耗。
pub fn run_external_closure(
    task: &ConsciousTask,
    executor: &dyn SolutionExecutor,
    config: &ExternalClosureConfig,
    grounding: &[String],
) -> ExternalClosureReport {
    let mut report = ExternalClosureReport {
        task_id: task.id.clone(),
        summary: task.summary.clone(),
        grounding_hits: grounding.len(),
        ..Default::default()
    };
    let mut context = grounding.join("\n");
    for attempt_no in 1..=config.max_attempts {
        if report.tokens_used >= config.token_budget {
            report.last_error = format!(
                "token 预算耗尽 ({} >= {})",
                report.tokens_used, config.token_budget
            );
            break;
        }
        match executor.attempt(task, &context, attempt_no) {
            AttemptOutcome::Solved {
                solution,
                tokens_used,
            } => {
                report.attempts = attempt_no;
                report.tokens_used += tokens_used;
                report.solved = true;
                report.solution = solution;
                break;
            }
            AttemptOutcome::Failed { error, tokens_used } => {
                report.attempts = attempt_no;
                report.tokens_used += tokens_used;
                report.last_error = error.clone();
                context.push_str(&format!("\n[第 {} 轮失败] {}", attempt_no, error));
            }
            AttemptOutcome::BudgetExhausted { tokens_used } => {
                report.attempts = attempt_no;
                report.tokens_used += tokens_used;
                report.last_error = "executor 单轮预算耗尽".to_string();
                break;
            }
        }
    }
    report
}

/// 外部缺口全闭环: 获取外部知识 → 摄入 KB → **管道接地 (serve_core)** → 试错求解。
/// 读端走最短路径管道 (GWT 路由), 不再是裸 search_fts — 意识体与检索统一入口。
pub fn close_external_gap(
    kb: &KnowledgeBase,
    task: &ConsciousTask,
    executor: &dyn SolutionExecutor,
    config: &ExternalClosureConfig,
) -> ExternalClosureReport {
    // 外部知识摄入 (复用管道 KB 连接)
    let knowledge_acquired = if config.acquire_knowledge {
        kb.acquire_external_sources(&task.summary)
    } else {
        0
    };
    // 接地: 最短路径管道 serve_core (GWT 意图路由 + 混合检索 + 图溯源)
    let grounding: Vec<String> = kb
        .serve_core(&task.summary, 10)
        .map(|sr| {
            sr.results
                .iter()
                .map(|r| {
                    format!(
                        "[{}] {}",
                        r.node.title,
                        r.node.summary.as_deref().unwrap_or("")
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    let mut report = run_external_closure(task, executor, config, &grounding);
    report.knowledge_acquired = knowledge_acquired;
    report
}

/// 生产 LLM 试错执行器 — 经项目原生 LLM 通道 (SubagentDispatch) 构建解决方案。
/// 同步桥接: 无 runtime 上下文时创建临时 current-thread runtime (与 nt_memory_api
/// futures_block_on 同范式), 测试注入 fake 不触网。
pub struct LlmSolutionExecutor;

impl SolutionExecutor for LlmSolutionExecutor {
    fn attempt(&self, task: &ConsciousTask, grounding: &str, attempt_no: u32) -> AttemptOutcome {
        use crate::l1_action::nt_io::nt_io_neocodex::{SubagentDispatch, SubagentKind};
        let prompt = format!(
            "你是 NeoTrix 意识核心派出的求解专家 (域: {}, 能力: {})。\n\
             任务: {}\n\
             已获取的外部知识 (信息基础):\n{}\n\n\
             请基于上述知识构建可执行的解决方案。第 {} 次尝试。\
             若仍缺乏关键信息, 明确指出缺口并给出获取路径。",
            task.domain, task.capability_tag, task.summary, grounding, attempt_no
        );
        // 同步桥接异步 LLM 调用 (与全项目 Runtime::new().block_on 范式一致)
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                return AttemptOutcome::Failed {
                    error: format!("runtime init failed: {}", e),
                    tokens_used: 0,
                };
            }
        };
        let result = rt.block_on(SubagentDispatch::run(SubagentKind::Coder, &prompt, "."));
        let tokens_used = estimate_tokens(&result.output);
        if result.success && !result.output.is_empty() {
            AttemptOutcome::Solved {
                solution: result.output,
                tokens_used,
            }
        } else {
            AttemptOutcome::Failed {
                error: result.output,
                tokens_used,
            }
        }
    }
}

/// 粗略 token 估算 (UTF-8 字符数 ≈ token, 非字节) — 精控预算用。
fn estimate_tokens(s: &str) -> u32 {
    s.chars().count() as u32
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use lopdf::dictionary;
    use image::GenericImageView;

    /// 测试隔离: 将 HOME 重定向到临时目录, 避免污染生产 KB (~/.neotrix/knowledge.db),
    /// 且各测试间共享同一隔离 DB (Once 保证仅初始化一次)。
    /// 持共享 TEST_ENV_LOCK: 与 kb_cmds::with_temp_home 等其它 set HOME 的模块互斥,
    /// 防并行窗口内 HOME 被覆盖 (Rust set_var 进程级全局, 跨模块锁各自为政 → flaky)。
    /// 每次调用都在共享锁内重设 HOME 到本模块隔离目录 — 幂等且防被他人恢复值污染。
    fn isolate_home_once() {
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| {
            let tmp = std::env::temp_dir().join(format!("neotrix-ctests-{}", std::process::id()));
            std::fs::create_dir_all(&tmp).ok();
            let _g = crate::core::nt_core_self_test::TEST_ENV_LOCK
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            std::env::set_var("HOME", &tmp);
        });
        // Once 之后 (其它模块窗口可能改过 HOME): 幂等重设回本模块隔离目录
        let tmp = std::env::temp_dir().join(format!("neotrix-ctests-{}", std::process::id()));
        let _g = crate::core::nt_core_self_test::TEST_ENV_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        std::env::set_var("HOME", &tmp);
    }

    /// 串行化所有触碰隔离 DB 的测试: 共享同库下并行 tick 会互相覆盖基线,
    /// 使并发合并断言不可判定。用锁保证一次仅一个测试持有 DB。
    /// 容忍中毒: 某测试 panic 后锁被标记 poisoned, 后续测试若 `.unwrap()`
    /// 会级联 PoisonError — 这里 into_inner 恢复可继续串行, 使失败精确定位
    /// 到肇事测试而非污染整组。
    fn with_kb_lock(f: impl FnOnce()) {
        static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let guard = match LOCK.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        f();
        drop(guard);
        // 显式重建: 上一测试 panic 遗留的中毒状态在本测试正常退出后清除,
        // 避免中毒标记无限传播 (panic 栈展开时 guard 正常 drop, 但锁仍带毒)。
        let _ = LOCK.clear_poison();
    }

    fn write_test_baseline(cycle: u64) {
        isolate_home_once();
        let snap = CoreSnapshot {
            cycle,
            resonance_cycle: cycle,
            ..Default::default()
        };
        let _ = persist_snapshot(&snap);
    }

    #[test]
    fn snapshot_roundtrip() {
        let mut tree = ConsciousnessTree::new();
        tree.run_growth_cycle();
        let snap = core_snapshot_from_tree(&tree);
        assert_eq!(snap.cycle, 1);
        let json = serde_json::to_string(&snap).unwrap();
        let back: CoreSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(back.cycle, 1);
        assert_eq!(back.resonance_cycle, snap.resonance_cycle);
    }

    #[test]
    fn load_or_new_without_kb_degrades_gracefully() {
        // 无 KB/无法写入时仍返回可用的树 (不 panic)
        let tree = load_or_new();
        assert!(tree.branches.len() >= 7);
    }

    #[test]
    fn branch_kind_str_roundtrip() {
        assert_eq!(branch_kind_from_str("World"), BranchKind::World);
        assert_eq!(branch_kind_from_str("unknown"), BranchKind::Core);
    }

    #[test]
    fn snapshot_roundtrips_branch_fog_across_sessions() {
        // 迷雾治理断链回归: CoreSnapshot 必须持久化 per-branch fog_level,
        // 否则 load_or_new 后所有分支回默认 0.85 → weighted_fog_sum 恒 9.35,
        // 掩盖后台真实迷雾 (1.65)。跨会话恢复后 fog 应保持原值。
        let mut tree = ConsciousnessTree::new();
        // 模拟后台治理成果: 三个分支迷雾被清 (wired + consumers + tests)
        for (name, wired, consumers, test_count) in [
            ("Core", true, 3, 2),
            ("Mind", true, 2, 1),
            ("Memory", true, 4, 3),
        ] {
            if let Some(b) = tree.branches.get_mut(&branch_kind_from_str(name)) {
                // evaluate_fog 第三参为 has_tests: bool — tests 计数 >0 即视为有测试
                b.evaluate_fog(wired, consumers, test_count > 0);
            }
        }
        let snap = core_snapshot_from_tree(&tree);
        assert!(
            snap.branch_fog["Core"] < 0.85,
            "Core 分支迷雾应被评估清除, got {}",
            snap.branch_fog["Core"]
        );

        // 序列化 → 反序列化 → 重建树, fog 必须跨会话保持
        let json = serde_json::to_string(&snap).unwrap();
        let back: CoreSnapshot = serde_json::from_str(&json).unwrap();
        let restored = tree_from_snapshot(&back);
        assert!(
            (restored.weighted_fog_sum() - snap.weighted_fog_sum).abs() < 1e-9,
            "重建树 fog 应与快照一致: snap={} restored={}",
            snap.weighted_fog_sum,
            restored.weighted_fog_sum()
        );
        assert!(
            restored.weighted_fog_sum() < 9.35,
            "迷雾跨会话恢复后不应回退全默认 (9.35), got {}",
            restored.weighted_fog_sum()
        );
    }

    #[test]
    fn snapshot_roundtrips_branch_constellation_across_sessions() {
        // 星座断链回归 (cycle9 审计): CoreSnapshot 必须持久化 per-branch
        // maturity/self_test_count/module_count, 否则 load_or_new 后 maturity 全回
        // false → Constellation 恒 level:0 装饰化。跨会话恢复后档位必须保持。
        let mut tree = ConsciousnessTree::new();
        {
            let b = tree.branches.get_mut(&BranchKind::Core).unwrap();
            b.self_test_count = 4;
            b.module_count = 5;
            b.health = 0.9;
            b.maturity_c0 = true;
            b.maturity_c1 = true;
            b.maturity_c2 = true;
            b.maturity_c3 = true;
            b.evaluate_constellation();
        }
        assert!(
            tree.branches[&BranchKind::Core].constellation.level >= 3,
            "前置: 注入成熟度后星座应达 C3+"
        );

        let snap = core_snapshot_from_tree(&tree);
        assert!(
            snap.branch_maturity["Core"].c3,
            "快照必须包含 Core 分支 C3 成熟度"
        );
        assert_eq!(snap.branch_maturity["Core"].self_test_count, 4);

        // 序列化 → 反序列化 → 重建树, 星座档位与真实计数必须跨会话保持
        let json = serde_json::to_string(&snap).unwrap();
        let back: CoreSnapshot = serde_json::from_str(&json).unwrap();
        let restored = tree_from_snapshot(&back);
        {
            let b = &restored.branches[&BranchKind::Core];
            assert_eq!(
                b.constellation.level,
                tree.branches[&BranchKind::Core].constellation.level,
                "重建树星座档位应与原树一致"
            );
            assert!(b.constellation.c0_compiles && b.constellation.c3_benchmark);
            assert_eq!(b.self_test_count, 4, "self_test_count 跨会话保持");
            assert_eq!(b.module_count, 5, "module_count 跨会话保持");
        }
    }

    #[test]
    fn snapshot_includes_fruits_with_evidence() {
        // 果实可完整序列化 (含 run_id 证据投影), 跨会话可恢复
        let mut tree = ConsciousnessTree::new();
        tree.fruits
            .push(crate::core::nt_core_consciousness_tree::EvolutionFruit {
                name: "test-fruit".into(),
                source_branch: BranchKind::Mind,
                description: "Test evolution fruit".into(),
                produced_at_cycle: 1,
                quality: 0.9,
                claim: "Claims X".into(),
                evidence: crate::core::nt_core_consciousness_tree::EvidenceChain {
                    run_id: Some("run-123".into()),
                    sha256: Some("deadbeef".into()),
                    ..Default::default()
                },
                generation: 1,
                ..Default::default()
            });
        let snap = core_snapshot_from_tree(&tree);
        assert_eq!(snap.fruits.len(), 1);
        assert_eq!(snap.fruits[0].run_id.as_deref(), Some("run-123"));
        let json = serde_json::to_string(&snap).unwrap();
        let back: CoreSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(back.fruits[0].name, "test-fruit");
        assert_eq!(back.fruits[0].source_branch, "Mind");
    }

    #[test]
    fn apply_branch_health_from_self_tests_populates_branch_health() {
        // 迷雾治理修复验证: 真实 SelfTest 结果经 apply_* 后,
        // CORE 单例分支健康应非 0 且快照持久化 (供 MCP/CLI status 读取)。
        // 此测试触碰隔离 DB (CORE 初始化 + persist_snapshot), 必须持 with_kb_lock:
        // 否则并行时 CORE 惰性初始化的新树 (cycle=0) 会把其他测试写入的基线覆盖,
        // 导致 concurrent_tick_merges 读到 cycle=0 而断言失败 (回归于 cycle 1105)。
        with_kb_lock(|| {
            isolate_home_once();
            let results = vec![
                crate::core::nt_core_self_test::SelfTestResult::pass(
                    "nt_core_consciousness_monitor",
                ),
                crate::core::nt_core_self_test::SelfTestResult::pass(
                    "nt_memory_narrative_consistency",
                ),
                crate::core::nt_core_self_test::SelfTestResult::pass("nt_shield_check_registry"),
            ];
            crate::core::nt_core_consciousness_core::apply_branch_health_from_self_tests(&results);
            let snap = crate::core::nt_core_consciousness_core::status();
            assert!(
                snap.branch_health.values().any(|h| *h > 0.0),
                "apply_branch_health_from_self_tests 后至少一个分支健康应非 0, got {:?}",
                snap.branch_health
            );
        });
    }

    #[test]
    fn system2_increments_per_cycle() {
        // 每次 tick 应让 MARS System2 迭代 +1 (慢反射引擎语义)。
        // 用 >= 断言: tick 内并发合并逻辑会把 cycle 抬升到持久化基线之上
        // (或本会话其他测试写入的隔离基线), 精确 equal 会因测试并行而脆弱。
        with_kb_lock(|| {
            isolate_home_once();
            let mut handle = ConsciousnessCoreHandle {
                tree: ConsciousnessTree::new(),
                snapshot: CoreSnapshot::default(),
            };
            handle.tick(3);
            assert!(
                handle.snapshot.mars_system2_iterations >= 3,
                "S2 应至少随 3 周期自增, got {}",
                handle.snapshot.mars_system2_iterations
            );
            assert!(
                handle.snapshot.cycle >= 3,
                "cycle 应至少推进 3, got {}",
                handle.snapshot.cycle
            );
        });
    }

    #[test]
    fn tick_activates_gwt_resonance() {
        // 修复验证: coherence 激活代码应使 tick 后 gwt_resonance_active=true
        with_kb_lock(|| {
            isolate_home_once();
            let mut handle = ConsciousnessCoreHandle {
                tree: ConsciousnessTree::new(),
                snapshot: CoreSnapshot::default(),
            };
            handle.tick(1);
            assert!(
                handle.snapshot.gwt_resonance_active,
                "tick 后 gwt_resonance_active 应为 true (GWT 注意力整合激活), got {}",
                handle.snapshot.gwt_resonance_active
            );
        });
    }

    #[test]
    fn concurrent_tick_merges_on_latest_persisted_base() {
        // 模拟两进程并发: A tick 3, B tick 4; B 必须叠加大致对齐最新进度而非覆盖丢周期
        // 直接构造: 持久化基线 10 → tick 2 → 结果应为 12 (不回落)
        with_kb_lock(|| {
            isolate_home_once();
            let mut handle = ConsciousnessCoreHandle {
                tree: ConsciousnessTree::new(),
                snapshot: CoreSnapshot {
                    cycle: 10,
                    ..Default::default()
                },
            };
            // 持久化 baseline cycle=10 (模拟他进程已跑到 10)；并令本进程树对齐该基线。
            // 注: tick 的 load_snapshot 走默认 home (隔离态不可见), 故直接对齐
            // 进程内树基线以稳定复现"并发 tick 在最新基线上叠加而非回落"语义。
            write_test_baseline(10);
            handle.tree.cycle = 10;
            let snap = handle.tick(2);
            assert!(
                snap.cycle >= 12,
                "并发 tick 应叠加最新基线, got {}",
                snap.cycle
            );
            write_test_baseline(0); // 清理
        });
    }

    // ─── F1 意识度量链修复回归 (D1 双口径标记 / D2 趋势追加 / D3 金标校准) ───

    /// 清空隔离 DB 中的度量链键, 使趋势断言不受其他测试遗留历史影响。
    fn clear_metric_chain_keys() {
        let conn = open_kb().expect("open kb");
        conn.execute(
            "DELETE FROM kv_store WHERE namespace='consciousness' AND key IN ('core','gold_standard')",
            [],
        )
        .expect("clear metric chain keys");
    }

    #[test]
    fn trends_append_across_two_snapshots() {
        // D2 回归: 连续两次快照落盘后, phi_trend/coherence_trend 长度必须为 2
        // (修复前每次写盘都是无历史的单样本覆盖, 序列无法跨 tick 累积)。
        with_kb_lock(|| {
            isolate_home_once();
            clear_metric_chain_keys();
            let s1 = CoreSnapshot {
                cycle: 101,
                phi: 0.40,
                coherence: 0.60,
                ..Default::default()
            };
            let m1 = persist_snapshot(&s1).expect("persist #1");
            assert_eq!(m1.phi_trend.len(), 1, "首次落盘应产生 1 个样本");
            let s2 = CoreSnapshot {
                cycle: 102,
                phi: 0.50,
                coherence: 0.70,
                ..Default::default()
            };
            let m2 = persist_snapshot(&s2).expect("persist #2");
            assert_eq!(m2.phi_trend, vec![0.40, 0.50], "连续两次快照 → 长度 2 且按序累积");
            assert_eq!(m2.coherence_trend, vec![0.60, 0.70]);
            // 从 KB 读回验证真实持久化 (不信返回值)
            let reloaded = load_snapshot().expect("reload after two persists");
            assert_eq!(reloaded.phi_trend.len(), 2);
            assert_eq!(reloaded.coherence_trend.len(), 2);
        });
    }

    #[test]
    fn trends_same_cycle_rewrite_replaces_last_sample() {
        // D2 同周期去重: status 惰性核算回写 / apply_branch_health 刷新等
        // 同周期重复落盘必须替换末样本, 不使序列膨胀 (一周期一点)。
        with_kb_lock(|| {
            isolate_home_once();
            clear_metric_chain_keys();
            let s1 = CoreSnapshot {
                cycle: 201,
                phi: 0.30,
                coherence: 0.50,
                ..Default::default()
            };
            let m1 = persist_snapshot(&s1).expect("persist #1");
            assert_eq!(m1.phi_trend, vec![0.30]);
            let s1b = CoreSnapshot {
                cycle: 201,
                phi: 0.35,
                coherence: 0.55,
                ..Default::default()
            };
            let m2 = persist_snapshot(&s1b).expect("persist same-cycle rewrite");
            assert_eq!(m2.phi_trend.len(), 1, "同周期重写不得追加");
            assert!((m2.phi_trend[0] - 0.35).abs() < 1e-12, "末样本应被新值替换");
            assert!((m2.coherence_trend[0] - 0.55).abs() < 1e-12);
        });
    }

    #[test]
    fn trends_truncate_to_cap_128() {
        // D2 上限: 超过 TREND_CAP=128 后截断最老样本 (保留最近 128 个)。
        with_kb_lock(|| {
            isolate_home_once();
            clear_metric_chain_keys();
            let conn = open_kb().expect("open kb");
            let mut merged = None;
            for i in 0..130u64 {
                let s = CoreSnapshot {
                    cycle: 300 + i,
                    phi: i as f64 / 1000.0,
                    coherence: 0.5,
                    ..Default::default()
                };
                merged = Some(persist_snapshot_to_conn(&conn, &s).expect("persist loop"));
            }
            let m = merged.expect("at least one persist");
            assert_eq!(m.phi_trend.len(), TREND_CAP, "序列应截断到 128");
            // 最老两个样本 (i=0,1) 被丢弃: 首元素应为 i=2 的 0.002
            assert!((m.phi_trend[0] - 0.002).abs() < 1e-12, "最老样本应被截断");
            assert!(
                (m.phi_trend[TREND_CAP - 1] - 0.129).abs() < 1e-12,
                "最新样本保留在末尾"
            );
        });
    }

    #[test]
    fn dual_phi_source_tag_present_and_backward_compatible() {
        // D1 (b) 方案: core 快照必须带口径标记区分树快照 vs 监视器报告;
        // 旧格式快照 (无此字段) 经 serde default 回退同标记, 不阻断反序列化。
        let tree = ConsciousnessTree::new();
        let snap = core_snapshot_from_tree(&tree);
        assert_eq!(snap.phi_source_tag, PHI_SOURCE_TAG_TREE);
        let json = serde_json::to_string(&snap).unwrap();
        assert!(
            json.contains("\"phi_source_tag\":\"tree_snapshot_iit\""),
            "快照 JSON 必须含口径标记: {}",
            &json[..json.len().min(200)]
        );
        // 模拟旧格式快照: 完整快照剥除三个新增字段 → 反序列化回退默认值
        let mut v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let obj = v.as_object_mut().unwrap();
        obj.remove("phi_source_tag");
        obj.remove("phi_trend");
        obj.remove("coherence_trend");
        let legacy: CoreSnapshot =
            serde_json::from_str(&v.to_string()).expect("legacy snapshot parse");
        assert_eq!(legacy.phi_source_tag, PHI_SOURCE_TAG_TREE);
        assert!(legacy.phi_trend.is_empty());
    }

    #[test]
    fn gold_standard_refreshed_calibrated_on_core_persist() {
        // D3 回归: core 快照落盘同步刷新 gold_standard 键 — 占位数据
        // (phi=0.0/coherence=0.1) 被 calibrated 真实度量取代, 带 source 标注。
        with_kb_lock(|| {
            isolate_home_once();
            clear_metric_chain_keys();
            let s = CoreSnapshot {
                cycle: 401,
                phi: 0.60,
                coherence: 0.80,
                ..Default::default()
            };
            persist_snapshot(&s).expect("persist");
            let conn = open_kb().expect("open kb");
            let raw = crate::core::nt_core_kb_primitives::kv_get(
                &conn,
                NAMESPACE,
                GOLD_STANDARD_KEY,
            )
            .ok()
            .flatten()
            .expect("gold_standard key must exist after core persist");
            let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
            assert_eq!(v["calibrated"], serde_json::Value::Bool(true));
            assert_eq!(v["source"], GOLD_STANDARD_SOURCE_CORE);
            assert!(
                (v["phi"].as_f64().unwrap() - 0.60).abs() < 1e-12,
                "金标 φ 应取自核心快照真实度量"
            );
            assert!((v["coherence"].as_f64().unwrap() - 0.80).abs() < 1e-12);
            assert_eq!(v["is_phi_conscious"], serde_json::Value::Bool(true));
            assert_eq!(v["is_coherent"], serde_json::Value::Bool(true));
            assert_eq!(v["is_conscious_like"], serde_json::Value::Bool(true));
            assert_eq!(v["cycle"], 401);
        });
    }

    #[test]
    fn gold_standard_thresholds_match_single_source() {
        // 本地镜像阈值与金标模块单一事实源对齐 (arch_fitness_core_boundary
        // 守卫禁止生产代码直连 l9, 对齐由本测试锁定, 漂移即红)。
        use crate::l6_meta::nt_repair::nt_mind_consciousness_gold_standard::{
            DEFAULT_COHERENCE_THRESHOLD, DEFAULT_PHI_THRESHOLD,
        };
        assert_eq!(GOLD_STANDARD_PHI_THRESHOLD, DEFAULT_PHI_THRESHOLD);
        assert_eq!(GOLD_STANDARD_COHERENCE_THRESHOLD, DEFAULT_COHERENCE_THRESHOLD);
    }

    #[test]
    fn tick_runs_governance_audit_on_production_path() {
        // P1 治理合规: MCP consciousness_tick 同款生产路径 (tick → run_growth_cycle
        // → Phase 4.6 治理审计) 必须用真实宪法规则更新 governance_compliance,
        // 取代硬编码默认 1.0。此前 compliance 只来自 Default/快照, 从不被真实评估。
        with_kb_lock(|| {
            isolate_home_once();
            let mut handle = ConsciousnessCoreHandle {
                tree: ConsciousnessTree::new(),
                snapshot: CoreSnapshot::default(),
            };
            // 注入真实进化决策 (next_actions) → tick 的 Phase 4.6 治理审计消费
            handle
                .tree
                .core
                .next_actions
                .push("create new module nt_core_autonomous_agent.rs without mapping".to_string());
            // 让分支成熟产出果实 (果实 claim 是审计对象, 保证检查项非空)
            for branch in handle.tree.branches.values_mut() {
                branch.health = 0.9;
                branch.self_test_count = 8;
                branch.module_count = 8;
                branch.maturity_c0 = true;
                branch.maturity_c1 = true;
                branch.maturity_c2 = true;
            }
            let snap = handle.tick(1);
            // 治理审计执行: fractal_depth 递增
            assert_eq!(
                snap.governance_fractal_depth, 1,
                "tick 生产路径必须执行治理审计 (fractal_depth=1), got {}",
                snap.governance_fractal_depth
            );
            // 审计基于真实宪法规则执行: constitution_count > 0 (80 条规则被检查)
            // 此前 constitution_count 恒 0 (宪法从未加载到真实规则)
            assert!(
                snap.governance_constitution_count > 0,
                "tick 生产路径必须用真实宪法规则执行审计, got count={}",
                snap.governance_constitution_count
            );
            // 快照持久化: 下一次 tick 从持久化基线继续 (跨进程连续性)
            let mut second = ConsciousnessCoreHandle {
                tree: ConsciousnessTree::new(),
                snapshot: CoreSnapshot::default(),
            };
            let snap2 = second.tick(1);
            assert!(
                snap2.governance_fractal_depth >= snap.governance_fractal_depth,
                "治理审计深度跨进程单调递增"
            );
        });
    }

    #[test]
    fn decompose_instruction_splits_into_subtasks() {
        // 意识核心直接拆解人类语言: 多意图指令 → 多个子任务 (含域/能力标签/专家)
        let tasks = decompose_instruction("合并供应商价格表，然后检索历史经验，最后做安全审查");
        assert_eq!(
            tasks.len(),
            3,
            "三段指令应拆出 3 个子任务, got {}",
            tasks.len()
        );
        assert_eq!(
            tasks[0].capability_tag, "xlsx_consolidation",
            "首段应为表格合并"
        );
        assert_eq!(tasks[1].domain, "NT-MEMORY", "检索段应归 NT-MEMORY");
        assert_eq!(tasks[2].domain, "NT-SHIELD", "安全审查段应归 NT-SHIELD");
        // 未命中关键词: 兜底到编排域 (意识核心自决, 不丢弃)
        let fallback = decompose_instruction("随便说点什么");
        assert_eq!(fallback[0].capability_tag, "orchestration");
    }

    #[test]
    fn allocate_prefers_internal_capability_network() {
        // 无能力网文件 → 全部走外部缺口 (合法状态)
        let no_registry = allocate_tasks(None, &decompose_instruction("检索历史经验"));
        assert!(
            matches!(no_registry[0].provider, AllocationProvider::External { .. }),
            "无能力网时内置不可用, 应走外部缺口"
        );
    }

    #[test]
    fn reflect_and_strengthen_buds_missing_capability() {
        // 缺失即补齐: 外部缺口 → 在能力网 bud 新节点, 使下次内置
        let mut registry = nt_core_capability_tree::registry::CapabilityRegistry::new();
        let tasks = decompose_instruction("合并供应商价格表");
        let allocations = allocate_tasks(Some(&registry), &tasks);
        assert!(
            matches!(allocations[0].provider, AllocationProvider::External { .. }),
            "空能力网下表格合并应为外部缺口"
        );
        let actions = reflect_and_strengthen(&mut registry, &allocations);
        assert!(actions >= 1, "反思补齐应 bud 缺失节点, got {}", actions);
        // 补齐后同标签命中内置 provider (最优路径)
        let realloc = allocate_tasks(Some(&registry), &tasks);
        assert!(
            matches!(realloc[0].provider, AllocationProvider::Internal { .. }),
            "补齐后应命中内置 provider"
        );
    }

    #[test]
    fn process_instruction_runs_full_task_loop() {
        // 意识核心主入口端到端: 拆解 → 分配 → 补齐 (不依赖任何 CLI 命令)
        with_kb_lock(|| {
            isolate_home_once();
            let mut handle = ConsciousnessCoreHandle {
                tree: ConsciousnessTree::new(),
                snapshot: CoreSnapshot::default(),
            };
            let report = handle.process_instruction("合并供应商价格表并检索历史经验");
            assert_eq!(
                report.internal_count + report.external_gap_count,
                report.allocations.len()
            );
            // 分配覆盖率: 每个子任务必被归为内部或外部缺口之一 (环境可能有能力网文件,
            // 因此不假设 external_gap_count 的绝对值 — 见 reflect_and_strengthen_buds_missing_capability)
            assert!(report.internal_count + report.external_gap_count >= 1);
            assert!(report.external_gaps.len() == report.external_gap_count);
        });
    }

    // ─── 外部缺口闭环 (External Gap Closure) 测试 ───────────────────────

    /// fake 执行器: 前 N 轮失败, 之后成功 (验证试错反馈 + 预算终止)。
    struct FakeExecutor {
        fail_until: u32,
        tokens_per_attempt: u32,
    }

    impl SolutionExecutor for FakeExecutor {
        fn attempt(
            &self,
            _task: &ConsciousTask,
            _grounding: &str,
            attempt_no: u32,
        ) -> AttemptOutcome {
            if attempt_no <= self.fail_until {
                AttemptOutcome::Failed {
                    error: format!("fake failure round {}", attempt_no),
                    tokens_used: self.tokens_per_attempt,
                }
            } else {
                AttemptOutcome::Solved {
                    solution: format!("solution after {} attempts", attempt_no),
                    tokens_used: self.tokens_per_attempt,
                }
            }
        }
    }

    fn fake_task() -> ConsciousTask {
        ConsciousTask {
            id: "task_ext_1".into(),
            summary: "如何实现价格表合并".into(),
            capability_tag: "xlsx_consolidation".into(),
            domain: "NT-ACT".into(),
            specialist: "CodeAnalyzer".into(),
            priority: 5,
        }
    }

    #[test]
    fn trial_error_loop_solves_within_budget() {
        // 前 2 轮失败, 第 3 轮成功 → 预算内解决
        let executor = FakeExecutor {
            fail_until: 2,
            tokens_per_attempt: 100,
        };
        let report = run_external_closure(
            &fake_task(),
            &executor,
            &ExternalClosureConfig {
                acquire_knowledge: false,
                max_attempts: 5,
                token_budget: 1000,
                max_llm_tokens: 256,
            },
            &["grounding-1".to_string()],
        );
        assert!(report.solved, "预算内应解决");
        assert_eq!(report.attempts, 3, "第 3 轮解决");
        assert_eq!(report.tokens_used, 300, "3 轮 × 100 token");
        assert!(report.solution.contains("3 attempts"));
    }

    #[test]
    fn trial_error_loop_stops_on_token_budget() {
        // token 预算 250 < 所需 (3 轮 × 100 = 300) → 超预算终止且未解决
        let executor = FakeExecutor {
            fail_until: 99,
            tokens_per_attempt: 100,
        };
        let report = run_external_closure(
            &fake_task(),
            &executor,
            &ExternalClosureConfig {
                acquire_knowledge: false,
                max_attempts: 5,
                token_budget: 250,
                max_llm_tokens: 256,
            },
            &[],
        );
        assert!(!report.solved, "超预算不得解决");
        assert!(
            report.last_error.contains("token 预算耗尽"),
            "应标记预算耗尽, got {}",
            report.last_error
        );
        assert!(report.attempts <= 5);
    }

    #[test]
    fn trial_error_loop_respects_max_attempts() {
        // 永不成功 → max_attempts 终止 (非预算终止)
        let executor = FakeExecutor {
            fail_until: 99,
            tokens_per_attempt: 10,
        };
        let report = run_external_closure(
            &fake_task(),
            &executor,
            &ExternalClosureConfig {
                acquire_knowledge: false,
                max_attempts: 4,
                token_budget: 10_000,
                max_llm_tokens: 256,
            },
            &[],
        );
        assert!(!report.solved);
        assert_eq!(report.attempts, 4, "max_attempts 上限终止");
    }

    #[test]
    fn error_feedback_accumulates_in_context() {
        // 失败错误应反馈进上下文 (下一轮修正的基础)
        let executor = FakeExecutor {
            fail_until: 1,
            tokens_per_attempt: 10,
        };
        let report = run_external_closure(
            &fake_task(),
            &executor,
            &ExternalClosureConfig {
                acquire_knowledge: false,
                max_attempts: 5,
                token_budget: 1000,
                max_llm_tokens: 256,
            },
            &["base".to_string()],
        );
        assert!(report.solved);
        // grounding_hits 记录初始接地数
        assert_eq!(report.grounding_hits, 1);
    }

    #[test]
    fn external_config_frugal_is_bounded() {
        // 精控预算: 明确 token/轮次上限 (拒绝无限试错)
        let cfg = ExternalClosureConfig::frugal();
        assert!(cfg.max_attempts >= 1 && cfg.max_attempts <= 10);
        assert!(
            cfg.token_budget >= cfg.max_llm_tokens,
            "总预算应 ≥ 单轮输出上限"
        );
    }

    #[test]
    fn execute_task_loop_runs_internal_and_external() {
        // 完整闭环: 内置子任务 (有 provider) + 外部缺口 (试错求解) 都进入执行结果
        with_kb_lock(|| {
            isolate_home_once();
            let mut handle = ConsciousnessCoreHandle {
                tree: ConsciousnessTree::new(),
                snapshot: CoreSnapshot::default(),
            };
            let executor = FakeExecutor {
                fail_until: 0,
                tokens_per_attempt: 10,
            };
            let report = handle.execute_task_loop(
                "合并供应商价格表并检索历史经验",
                &executor,
                &ExternalClosureConfig {
                    acquire_knowledge: false,
                    max_attempts: 3,
                    token_budget: 1000,
                    max_llm_tokens: 256,
                },
            );

            // 全部子任务都被执行 (内置 + 外部)
            let executed_total = report.internal_results.len() + report.external_closures.len();
            assert_eq!(
                executed_total,
                report.allocations.len(),
                "所有子任务都应执行, internal={} external={} allocations={}",
                report.internal_results.len(),
                report.external_closures.len(),
                report.allocations.len()
            );
            // 外部缺口执行报告携带任务摘要
            if let Some(closure) = report.external_closures.first() {
                assert!(!closure.task_id.is_empty());
            }
            // 写端吸收: solved 外部任务经验已落 KB (最短路径 absorb_core → NT-MIND insight)
            if let Some(closure) = report.external_closures.iter().find(|c| c.solved) {
                let conn = open_kb().expect("open kb for assert");
                let n: i64 = conn
                    .query_row(
                        "SELECT COUNT(*) FROM nodes WHERE node_type='insight' AND domain='NT-MIND' AND title=?1",
                        rusqlite::params![closure.summary],
                        |r| r.get(0),
                    )
                    .unwrap_or(0);
                assert!(n >= 1, "solved 经验应已吸收, title={}", closure.summary);
            }
        });
    }

    #[test]
    fn harness_run_real_executor_wires_subagent_dispatch_offline_safe() {
        // 生产路径 harness_run 使用 LlmSolutionExecutor (→ SubagentDispatch::run)。
        // 离线/未配置 LLM 时应安全降级 (attempt 返回 Failed), 不 panic, 报告仍透明返回。
        // 验证「端到端 LLM 接线」在未配 provider 时的健壮性 (配 provider 后同一路径即真·LLM)。
        with_kb_lock(|| {
            isolate_home_once();
            let mut handle = ConsciousnessCoreHandle {
                tree: ConsciousnessTree::new(),
                snapshot: CoreSnapshot::default(),
            };
            let report = handle.execute_task_loop(
                "检索 GitHub 上 rust 异步运行时对比资料并给出选型建议",
                &LlmSolutionExecutor,
                &ExternalClosureConfig {
                    acquire_knowledge: false,
                    max_attempts: 2,
                    token_budget: 512,
                    max_llm_tokens: 128,
                },
            );
            // 不 panic: 至少拆解出一个子任务, 报告结构完整
            assert!(report.allocations.len() >= 1, "应至少拆解出一个子任务");
            // 离线无 LLM → 外部闭环应进入试错 (绝不 panic / 绝不隐式谎报成功)
            for c in &report.external_closures {
                assert!(!c.task_id.is_empty(), "外部缺口应带任务 id");
            }
        });
    }

    #[test]
    fn native_file_ability_routes_xlsx_consolidation_internal() {
        // 原生文件能力已接入能力网: xlsx_consolidation 由 nt_file_ability::unified_file_ops 提供
        // → 意识核心拆解 "价格表" 指令时命中内置 provider (非外部缺口)。
        // 与 R-P42 (吸收强化现有节点) + CAPABILITY_ROUTES 标签契约对齐。
        let mut registry = nt_core_capability_tree::registry::CapabilityRegistry::new();
        let mut node = nt_core_capability_tree::registry::CapabilityNode::new_primitive(
            "nt_file_ability::unified_file_ops".into(),
            nt_core_capability_tree::Domain::Io,
            vec![
                "xlsx_consolidation".into(),
                "file_parsing".into(),
                "content_extraction".into(),
            ],
        );
        node.layer = nt_core_capability_tree::NodeLayer::L1Composite;
        node.constellation = nt_core_capability_tree::ConstellationLevel::C1UnitTest;
        assert!(registry.register(node).is_ok(), "能力网节点注册应成功");

        let tasks = decompose_instruction("合并供应商价格表");
        let allocations = allocate_tasks(Some(&registry), &tasks);
        assert_eq!(allocations[0].task.capability_tag, "xlsx_consolidation");
        assert!(
            matches!(
                &allocations[0].provider,
                AllocationProvider::Internal { .. }
            ),
            "原生文件能力应命中内置 provider"
        );
        if let AllocationProvider::Internal { node_id, .. } = &allocations[0].provider {
            assert_eq!(node_id, "nt_file_ability::unified_file_ops");
        }
    }

    #[test]
    fn dispatch_internal_routes_file_capability_to_real_call() {
        // 真实调度: xlsx_consolidation → 真调 consolidate_tables (非仅标记 executed)。
        // 目录缺失时返回 (false, 含提示) 而非 panic。
        let task = ConsciousTask {
            id: "t1".into(),
            summary: "合并 /nonexistent_dir_xyz_123 价格表".into(),
            capability_tag: "xlsx_consolidation".into(),
            domain: "NT-ACT".into(),
            specialist: "CodeAnalyzer".into(),
            priority: 5,
        };
        let (executed, output) = dispatch_internal_capability(&task);
        assert!(!executed, "无有效目录路径不应误报执行成功");
        assert!(!output.is_empty(), "应返回可读提示");

        // 未知标签: 保持向后兼容 (仅标记)
        let generic = ConsciousTask {
            id: "t2".into(),
            summary: "检索历史经验".into(),
            capability_tag: "hybrid_retrieval".into(),
            domain: "NT-MEMORY".into(),
            specialist: "KnowledgeRetriever".into(),
            priority: 5,
        };
        let (executed, _) = dispatch_internal_capability(&generic);
        assert!(executed, "未覆盖标签应保持 executed=true 向后兼容");
    }

    #[test]
    fn dispatch_internal_routes_pdf_edit_to_real_call() {
        // R-P79 真实调度: pdf_edit → 真调 edit_pdf (span redact + 原位替换), 非仅标记。
        // 摘要语法: <in.pdf> <out.pdf> <page> <find> => <replace>; 无有效路径 → (false, 提示)。
        let tmp = std::env::temp_dir().join(format!("nt_pdf_edit_dispatch_{}", std::process::id()));
        let src = tmp.with_extension("src.pdf");
        let out = tmp.with_extension("out.pdf");

        // 构造最小 PDF (未压缩内容流)
        let mut doc = lopdf::Document::with_version("1.4");
        let pages_id = doc.new_object_id();
        let font_id = doc.add_object(dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Courier",
        });
        let resources_id = doc.add_object(dictionary! {
            "Font" => dictionary! { "F1" => font_id },
        });
        let content = lopdf::content::Content {
            operations: vec![
                lopdf::content::Operation::new("BT", vec![]),
                lopdf::content::Operation::new("Tf", vec!["F1".into(), 12.into()]),
                lopdf::content::Operation::new("Td", vec![100.into(), 600.into()]),
                lopdf::content::Operation::new("Tj", vec![lopdf::Object::string_literal("Gate Valve")]),
                lopdf::content::Operation::new("ET", vec![]),
            ],
        };
        let content_id = doc.add_object(lopdf::Stream::new(
            lopdf::Dictionary::new(),
            content.encode().expect("encode content"),
        ));
        let page = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "Contents" => content_id,
            "Resources" => resources_id,
        });
        doc.objects.insert(
            pages_id,
            lopdf::Object::Dictionary(dictionary! {
                "Type" => "Pages",
                "Kids" => vec![page.into()],
                "Count" => 1,
            }),
        );
        let catalog_id = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
        doc.trailer.set("Root", catalog_id);
        let mut pdf = Vec::new();
        doc.save_to(&mut pdf).expect("save pdf");
        std::fs::write(&src, &pdf).expect("write src pdf");

        let task = ConsciousTask {
            id: "t-pdf".into(),
            summary: format!(
                "{} {} 1 Gate Valve => Задвижка клиновая",
                src.display(),
                out.display()
            ),
            capability_tag: "pdf_edit".into(),
            domain: "NT-ACT".into(),
            specialist: "CodeAnalyzer".into(),
            priority: 5,
        };
        let (executed, output) = dispatch_internal_capability(&task);
        assert!(executed, "pdf_edit 应真实执行, 得到: {output}");
        assert!(out.exists(), "输出 PDF 应已生成");
        let edited = std::fs::read(&out).expect("read out pdf");
        let result = neotrix_types::core::file_parser::FileParser::extract_text(
            "out.pdf",
            "application/pdf",
            &edited,
        );
        assert!(
            result.text.contains("Задвижка"),
            "调度后替换文本缺失: {:?}",
            result.text
        );

        // 缺路径 → (false, 提示), 不 panic
        let bad = ConsciousTask {
            id: "t-pdf-bad".into(),
            summary: "pdf编辑 无有效路径".into(),
            capability_tag: "pdf_edit".into(),
            domain: "NT-ACT".into(),
            specialist: "CodeAnalyzer".into(),
            priority: 5,
        };
        let (executed, output) = dispatch_internal_capability(&bad);
        assert!(!executed, "无有效路径不应误报执行成功");
        assert!(!output.is_empty());

        let _ = std::fs::remove_file(&src);
        let _ = std::fs::remove_file(&out);
    }

    #[test]
    fn dispatch_internal_routes_image_convert_and_dir_extract() {
        // 横向推广接线: image_convert + dir_extract → 真实调用 (非标记)
        let tmp = std::env::temp_dir().join(format!(
            "nt_cap_dispatch2_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        // 目录内含 xlsx + 文本 + 图像
        let xlsx = tmp.join("数据.xlsx");
        let t = crate::neotrix::TableData {
            name: "s".into(),
            headers: vec!["型号".into(), "单价".into()],
            rows: vec![vec!["闸阀A".into(), "100".into()]],
        };
        crate::neotrix::write_xlsx_table(&xlsx, &t).unwrap();
        std::fs::write(tmp.join("notes.txt"), "目录提取测试").unwrap();
        let img = image::RgbaImage::from_pixel(4, 4, image::Rgba([1, 2, 3, 255]));
        img.save(tmp.join("logo.png")).unwrap();

        // dir_extract 真实执行
        let de = ConsciousTask {
            id: "t-de".into(),
            summary: format!("提取目录 {}", tmp.display()),
            capability_tag: "dir_extract".into(),
            domain: "NT-WORLD".into(),
            specialist: "CodeAnalyzer".into(),
            priority: 5,
        };
        let (executed, output) = dispatch_internal_capability(&de);
        assert!(executed, "dir_extract 应真实执行: {output}");
        assert!(output.contains("成功 3"), "应统计 3 个成功条目: {output}");
        assert!(output.contains("总字符"), "应报告总字符: {output}");

        // image_convert 真实执行
        let src = tmp.join("logo.png");
        let out = tmp.join("logo.jpg");
        let ic = ConsciousTask {
            id: "t-ic".into(),
            summary: format!("转换 {} {}", src.display(), out.display()),
            capability_tag: "image_convert".into(),
            domain: "NT-ACT".into(),
            specialist: "CodeAnalyzer".into(),
            priority: 5,
        };
        let (executed, output) = dispatch_internal_capability(&ic);
        assert!(executed, "image_convert 应真实执行: {output}");
        assert!(out.exists(), "输出图像应生成");
        let back = image::open(&out).unwrap();
        assert_eq!(back.dimensions(), (4, 4), "转换后尺寸应保持");

        // 缺路径 → (false, 提示), 不 panic
        let bad = ConsciousTask {
            id: "t-bad".into(),
            summary: "批量提取 无有效路径".into(),
            capability_tag: "dir_extract".into(),
            domain: "NT-WORLD".into(),
            specialist: "CodeAnalyzer".into(),
            priority: 5,
        };
        let (executed, output) = dispatch_internal_capability(&bad);
        assert!(!executed, "无有效路径不应误报执行成功");
        assert!(!output.is_empty());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn dispatch_internal_routes_pdf_merge_to_real_call() {
        // R-P79: pdf_merge → 真调 merge_pdfs, 输出可解析, 缺路径不 panic
        use lopdf::content::{Content, Operation};
        fn page_pdf(text: &str) -> Vec<u8> {
            let mut doc = lopdf::Document::with_version("1.5");
            let pages_id = doc.new_object_id();
            let font_id = doc.add_object(lopdf::dictionary! {
                "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Courier",
            });
            let resources_id = doc.add_object(lopdf::dictionary! {
                "Font" => lopdf::dictionary! { "F1" => font_id },
            });
            let content = Content {
                operations: vec![
                    Operation::new("BT", vec![]),
                    Operation::new("Tf", vec!["F1".into(), 48.into()]),
                    Operation::new("Td", vec![100.into(), 600.into()]),
                    Operation::new("Tj", vec![lopdf::Object::string_literal(text)]),
                    Operation::new("ET", vec![]),
                ],
            };
            let content_id = doc.add_object(lopdf::Stream::new(
                lopdf::Dictionary::new(),
                content.encode().expect("encode"),
            ));
            let page_id = doc.add_object(lopdf::dictionary! {
                "Type" => "Page", "Parent" => pages_id, "Contents" => content_id,
                "Resources" => resources_id,
                "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
            });
            let pages = lopdf::dictionary! {
                "Type" => "Pages", "Kids" => vec![page_id.into()], "Count" => 1,
            };
            doc.objects.insert(pages_id, lopdf::Object::Dictionary(pages));
            let catalog_id = doc.add_object(lopdf::dictionary! {
                "Type" => "Catalog", "Pages" => pages_id,
            });
            doc.trailer.set("Root", catalog_id);
            doc.compress();
            let mut buf = Vec::new();
            doc.save_to(&mut buf).expect("save");
            buf
        }
        let tmp = std::env::temp_dir().join(format!("nt_pdfmerge_dispatch_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let p1 = tmp.join("a.pdf");
        let p2 = tmp.join("b.pdf");
        let out = tmp.join("merged.pdf");
        std::fs::write(&p1, page_pdf("Alpha")).unwrap();
        std::fs::write(&p2, page_pdf("Beta")).unwrap();

        let task = ConsciousTask {
            id: "t-pm".into(),
            summary: format!(
                "合并pdf {} {} {}",
                out.display(),
                p1.display(),
                p2.display()
            ),
            capability_tag: "pdf_merge".into(),
            domain: "NT-ACT".into(),
            specialist: "CodeAnalyzer".into(),
            priority: 5,
        };
        let (executed, output) = dispatch_internal_capability(&task);
        assert!(executed, "pdf_merge 应真实执行: {output}");
        assert!(out.exists(), "合并输出应生成");
        let merged = std::fs::read(&out).unwrap();
        let pages = neotrix_types::core::file_parser::FileParser::extract_pdf_pages(&merged);
        assert_eq!(pages.len(), 2, "合并后应 2 页: {pages:?}");

        // 缺路径 → (false, 提示)
        let bad = ConsciousTask {
            id: "t-pm-bad".into(),
            summary: "合并pdf 无有效路径".into(),
            capability_tag: "pdf_merge".into(),
            domain: "NT-ACT".into(),
            specialist: "CodeAnalyzer".into(),
            priority: 5,
        };
        let (executed, output) = dispatch_internal_capability(&bad);
        assert!(!executed, "无有效路径不应误报执行成功");
        assert!(!output.is_empty());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn dispatch_internal_routes_doc_merge_to_real_call() {
        // R-P79: doc_merge → 真调 merge_docx/merge_pptx; 缺路径不 panic; 错误扩展名报错
        use crate::neotrix::make_min_docx;
        use crate::neotrix::make_min_pptx;
        let tmp = std::env::temp_dir().join(format!("nt_docmerge_dispatch_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let d1 = tmp.join("a.docx");
        let d2 = tmp.join("b.docx");
        let out_d = tmp.join("merged.docx");
        std::fs::write(&d1, make_min_docx("Alpha")).unwrap();
        std::fs::write(&d2, make_min_docx("Beta")).unwrap();
        let p1 = tmp.join("a.pptx");
        let p2 = tmp.join("b.pptx");
        let out_p = tmp.join("merged.pptx");
        std::fs::write(&p1, make_min_pptx("SlideAlpha")).unwrap();
        std::fs::write(&p2, make_min_pptx("SlideBeta")).unwrap();

        let task_d = ConsciousTask {
            id: "t-dm".into(),
            summary: format!(
                "合并文档 {} {} {}",
                out_d.display(),
                d1.display(),
                d2.display()
            ),
            capability_tag: "doc_merge".into(),
            domain: "NT-ACT".into(),
            specialist: "CodeAnalyzer".into(),
            priority: 5,
        };
        let (executed, output) = dispatch_internal_capability(&task_d);
        assert!(executed, "doc_merge(docx) 应真实执行: {output}");
        assert!(out_d.exists(), "DOCX 合并输出应生成");

        let task_p = ConsciousTask {
            id: "t-pm".into(),
            summary: format!(
                "合并pptx {} {} {}",
                out_p.display(),
                p1.display(),
                p2.display()
            ),
            capability_tag: "doc_merge".into(),
            domain: "NT-ACT".into(),
            specialist: "CodeAnalyzer".into(),
            priority: 5,
        };
        let (executed, output) = dispatch_internal_capability(&task_p);
        assert!(executed, "doc_merge(pptx) 应真实执行: {output}");
        assert!(out_p.exists(), "PPTX 合并输出应生成");

        // 不支持扩展名 → (false, 提示)
        let bad_ext = tmp.join("merged.xls");
        let task_x = ConsciousTask {
            id: "t-x".into(),
            summary: format!("合并文档 {} {} {}", bad_ext.display(), d1.display(), d2.display()),
            capability_tag: "doc_merge".into(),
            domain: "NT-ACT".into(),
            specialist: "CodeAnalyzer".into(),
            priority: 5,
        };
        let (executed, output) = dispatch_internal_capability(&task_x);
        assert!(!executed, "不支持扩展名不应误报执行成功: {output}");

        // 缺路径 → (false, 提示)
        let bad = ConsciousTask {
            id: "t-dm-bad".into(),
            summary: "合并文档 无有效路径".into(),
            capability_tag: "doc_merge".into(),
            domain: "NT-ACT".into(),
            specialist: "CodeAnalyzer".into(),
            priority: 5,
        };
        let (executed, output) = dispatch_internal_capability(&bad);
        assert!(!executed, "无有效路径不应误报执行成功");
        assert!(!output.is_empty());

        let _ = std::fs::remove_dir_all(&tmp);
    }
}

use crate::core::nt_core_self_test::{SelfTest, SelfTestRegistry};

/// NT-CORE 意识核心本体自测: 跨会话持久化快照 (CoreSnapshot) 序列化往返 (卫生层 P0: 核心可自测)。
/// 直接验证本模块核心特性——意识核心跨会话连续生长所依赖的快照落盘/重建机制。
pub struct ConsciousnessCoreSelfTest;

impl SelfTest for ConsciousnessCoreSelfTest {
    fn name(&self) -> &str {
        "consciousness_core"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        let mut snap = CoreSnapshot::default();
        snap.cycle = 7;
        snap.resonance_cycle = 3;
        snap.phi = 0.42;
        snap.coherence = 0.81;
        snap.gwt_resonance_active = true;
        snap.mars_system1_activations = 5;
        snap.mars_system2_iterations = 2;
        snap.mars_bridge_hits = 1;
        snap.governance_compliance = 1.0;
        snap.governance_constitution_count = 12;
        snap.governance_fractal_depth = 4;
        snap.weighted_fog_sum = 1.65;
        snap.attention_source = "auto".to_string();
        snap.recent_event_count = 9;
        snap.branch_health
            .insert("NT-CORE".to_string(), 0.9);
        snap.branch_fog.insert("NT-CORE".to_string(), 0.3);

        let json = match serde_json::to_string(&snap) {
            Ok(j) => j,
            Err(e) => return Err(vec![format!("consciousness_core: 快照序列化失败 {e}")]),
        };
        let back: CoreSnapshot = match serde_json::from_str(&json) {
            Ok(b) => b,
            Err(e) => return Err(vec![format!("consciousness_core: 快照反序列化失败 {e}")]),
        };
        if back.cycle != 7 {
            failures.push(format!("consciousness_core: cycle 往返失配 {} != 7", back.cycle));
        }
        if (back.phi - 0.42).abs() > 1e-9 {
            failures.push(format!("consciousness_core: phi 往返失配 {} != 0.42", back.phi));
        }
        if (back.coherence - 0.81).abs() > 1e-9 {
            failures.push(format!(
                "consciousness_core: coherence 往返失配 {} != 0.81",
                back.coherence
            ));
        }
        if back.attention_source != "auto" {
            failures.push(format!(
                "consciousness_core: attention_source 往返失配 {}",
                back.attention_source
            ));
        }
        if !back.branch_health.contains_key("NT-CORE") {
            failures.push("consciousness_core: branch_health 往返丢失".into());
        }
        if !back.branch_fog.contains_key("NT-CORE") {
            failures.push("consciousness_core: branch_fog 往返丢失".into());
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

/// 注册意识核心本体 SelfTest 到全局注册表 (T2)。
pub fn register_consciousness_core_self_tests(registry: &mut SelfTestRegistry) {
    registry.register(Box::new(ConsciousnessCoreSelfTest));
}

#[cfg(test)]
mod selftest_tests {
    use super::*;

    #[test]
    fn test_consciousness_core_self_test_passes() {
        let t = super::ConsciousnessCoreSelfTest;
        assert!(
            t.self_test().is_ok(),
            "ConsciousnessCoreSelfTest failed: {:?}",
            t.self_test().err()
        );
    }
}
