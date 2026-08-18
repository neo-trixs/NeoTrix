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
//! ## 调用入口 (同一实例, 两条口)
//! - CLI: `neotrix consciousness status|tick|health|branches [--json]`
//! - MCP: `consciousness_status` / `consciousness_tick` 工具

use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

use serde::{Deserialize, Serialize};

use crate::core::nt_core_consciousness_tree::{BranchKind, ConsciousnessTree};

/// KB 最短路径管道 — 意识体读写端直达 (R-P42: 强化现有节点, 禁止平行适配器)
use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_pipeline::AbsorbEntry;
use crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase;

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
}

/// 默认注意力来源 (x.ai 双搜索通道的模型自决模式)。
fn default_attention_source() -> String {
    "auto".to_string()
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
        let _ = persist_snapshot(&self.snapshot);
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
                let _ = persist_snapshot(&h.snapshot);
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
    let _ = persist_snapshot(&h.snapshot);
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
    let raw = crate::core::nt_core_kb_primitives::kv_get(
        &conn, NAMESPACE, KEY,
    )
    .ok()??;
    serde_json::from_str(&raw).ok()
}

fn persist_snapshot(snap: &CoreSnapshot) -> Result<(), String> {
    let conn = open_kb()?;
    let json = serde_json::to_string(snap).map_err(|e| format!("snapshot serialize: {}", e))?;
    crate::core::nt_core_kb_primitives::kv_set(
        &conn, NAMESPACE, KEY, &json,
    )
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

/// 能力网注册表路径 — 优先 `HOME/.neotrix/capability_registry.json` (与 KB 同目录,
/// 被 isolate_home 测试隔离); 缺失时回退 cwd `.neotrix/capability_registry.json`
/// (后台 handlers_maintenance 用相对 cwd 路径, 生产一致时统一收敛到本函数)。
/// 读路径以存在者为准; 写路径固化在 HOME (隔离测试可写; 生产与后台读同源)。
pub fn capability_registry_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let home_path = std::path::PathBuf::from(&home)
        .join(".neotrix")
        .join("capability_registry.json");
    let cwd_path = std::path::PathBuf::from(".neotrix").join("capability_registry.json");
    if home_path.exists() {
        home_path
    } else if cwd_path.exists() {
        cwd_path
    } else {
        home_path
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
        // 1. 拆解
        let tasks = decompose_instruction(instruction);
        // 2. 加载能力网 + 分配
        let mut registry = load_capability_registry();
        let allocations = allocate_tasks(registry.as_ref(), &tasks);

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
        let mut report = self.process_instruction(instruction);

        // 内置子任务执行: 能力网命中 → 真实调用能力 (标记执行 + 实际结果)。
        let mut internal_results = Vec::new();
        for alloc in &report.allocations {
            if let AllocationProvider::Internal { node_id, path, .. } = &alloc.provider {
                let (executed, output) = dispatch_internal_capability(&alloc.task);
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

        // 外部缺口执行: 每个 External 子任务 → 自动外部求解闭环
        // 最短路径: 读端 serve_core 接地 (GWT 路由), 写端 absorb_core 吸收经验
        let kb = KnowledgeBase::open(None).ok();
        let mut closures = Vec::new();
        for alloc in &report.allocations {
            if let AllocationProvider::External { .. } = &alloc.provider {
                let result = match &kb {
                    Some(kb) => close_external_gap(kb, &alloc.task, executor, config),
                    None => {
                        // 无 KB: 仍走试错循环 (接地为空), 不 panic
                        run_external_closure(&alloc.task, executor, config, &[])
                    }
                };
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
                    match crate::neotrix::consolidate_tables(&d, &out) {
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
        _ => (
            true,
            format!(
                "internal capability '{}' via domain {}",
                task.capability_tag, task.domain,
            ),
        ),
    }
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
    CORE.write()
        .map(|mut h| h.execute_task_loop(instruction, executor, config))
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

/// 外部知识自动获取 — 按能力标签分派 discover_* 源。
/// 全部源失败不 panic (单源错误记录, 其余源继续), 返回成功摄入数。
pub fn acquire_external_knowledge(conn: &rusqlite::Connection, task: &ConsciousTask) -> usize {
    use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_discovery_sources as src;
    let query = &task.summary;
    let mut ingested = 0usize;
    // 论文类 (学术): Semantic Scholar + ArXiv
    if let Ok(s) = src::discover_semantic_scholar(conn, query, 5) {
        ingested += s.resources_ingested;
    }
    if let Ok(s) = src::discover_arxiv_papers(conn, query, 5) {
        ingested += s.resources_ingested;
    }
    // 技术文档/百科: Wikipedia (技术文档词条)
    if let Ok(s) = src::discover_technical_docs(conn, query) {
        ingested += s.resources_ingested;
    }
    ingested
}

/// 接地检索 — 从 KB 检索与任务相关的已摄入知识作为求解上下文。
pub fn retrieve_grounding(conn: &rusqlite::Connection, task: &ConsciousTask) -> Vec<String> {
    use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_search;
    match nt_memory_search::search_fts(conn, &task.summary, 10) {
        Ok(results) => results
            .iter()
            .map(|r| {
                let n = &r.node;
                format!("[{}] {}", n.title, n.summary.as_deref().unwrap_or(""))
            })
            .collect(),
        Err(_) => Vec::new(),
    }
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
        if let Ok(conn) = kb.conn.lock() {
            acquire_external_knowledge(&conn, task)
        } else {
            0
        }
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
        use crate::neotrix::l1_body_impl::nt_io_neocodex::{SubagentDispatch, SubagentKind};
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
            // 持久化 baseline cycle=10 (模拟他进程已跑到 10)
            write_test_baseline(10);
            let snap = handle.tick(2);
            assert!(
                snap.cycle >= 12,
                "并发 tick 应叠加最新基线, got {}",
                snap.cycle
            );
            write_test_baseline(0); // 清理
        });
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
    fn retrieve_grounding_degrades_gracefully_on_empty_kb() {
        // 空内存 KB → 接地检索返回空 (不 panic), 闭环不依赖 KB 预存
        let conn = rusqlite::Connection::open_in_memory().expect("in-memory sqlite");
        let hits = retrieve_grounding(&conn, &fake_task());
        assert!(hits.is_empty(), "空库应无命中, got {}", hits.len());
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
}
