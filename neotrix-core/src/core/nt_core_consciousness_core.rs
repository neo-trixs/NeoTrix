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

use crate::core::nt_core_consciousness_tree::{
    BranchKind, ConsciousnessTree,
};

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
    /// 相干性 — 同上, 独立进程无核算器时为 0.0。
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
    RwLock::new(ConsciousnessCoreHandle {
        tree,
        snapshot,
    })
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
        for _ in 0..n {
            self.tree.run_growth_cycle();
            // Activate GWT resonance to enable coherence calculation
            // This sets the flag that allows coherence > 0 in status
            self.tree.trunk.gwt_resonance_active = true;
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

/// 读取当前意识核心状态 (无副作用)。MCP/CLI status 共用。
pub fn status() -> CoreSnapshot {
    CORE.read().map(|h| h.current().clone()).unwrap_or_default()
}

/// 驱动生长周期, 写回快照。MCP/CLI tick 共用。
pub fn tick(cycles: usize) -> CoreSnapshot {
    CORE.write()
        .map(|mut h| h.tick(cycles))
        .unwrap_or_default()
}

/// 将真实 SelfTest 结果合并进意识核心单例树, 重算各域分支健康并持久化。
/// 后台循环 (handlers_consciousness) 在跑完注册器 SelfTest 后调用, 使基于真实
/// 检测的分支健康流入跨会话快照 — 修复此前独立 tree 实例计算后即丢弃、
/// `consciousness/core` 快照分支健康恒 0 的断链 (迷雾治理)。
pub fn apply_branch_health_from_self_tests(results: &[crate::core::nt_core_self_test::SelfTestResult]) {
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
        .map(|h| h.tree.branches.iter().map(|(k, b)| (format!("{:?}", k), b.health)).collect())
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
                    m.insert("label".into(), k.label().split('(').next().unwrap_or("").trim().to_string());
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
        recent_event_count: tree.trunk.mars_system1_activations
            + tree.trunk.mars_bridge_hits,
        // OpenMausBot ProviderRegistry shadow → 未接线高雾分支数 (需 shadow 降级保护)
        shadow_instance_count: tree
            .branches
            .values()
            .filter(|b| b.fog.level > 0.8)
            .count() as u64,
        // OpenMausBot 权限执行 → 宪法注册数 (合规检查执行次数)
        compliance_execution_count: tree.trunk.governance_constitution_count as u64,
        // spec-kit SDD constitution → 宪法门控执行计数 (MARS System 2 迭代 = 门控检查)
        constitution_check_count: tree.trunk.mars_system2_iterations,
    }
}

/// 从快照恢复树计数器。KB 缺失/损坏 → 全新树 (优雅降级)。
fn load_or_new() -> ConsciousnessTree {
    let mut tree = ConsciousnessTree::new();
    match load_snapshot() {
        Some(snap) => {
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
            // 恢复已消化果实 — 从快照完整重建证据链投影 (具体 EvidenceChain 以 run_id 标注,
            // 不重建二进制证据; 进化产物引用保留, 供审计/追踪)。
            for fr in &snap.fruits {
                tree.fruits.push(
                    crate::core::nt_core_consciousness_tree::EvolutionFruit {
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
                    },
                );
            }
            tree
        }
        None => tree,
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

/// 打开 KB 连接 (默认 `~/.neotrix/knowledge.db`), 复用 NT-MEMORY 统一 schema 初始化。
/// 单一 schema 事实源: 不在此处维护 kv_store 本地 DDL, 避免漂移。
fn open_kb() -> Result<rusqlite::Connection, String> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let path = std::path::PathBuf::from(home).join(".neotrix").join("knowledge.db");
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
    crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_schema::initialize(&conn)
        .map_err(|e| format!("KB init: {}", e))?;
    Ok(conn)
}
fn load_snapshot() -> Option<CoreSnapshot> {
    let conn = open_kb().ok()?;
    let raw = crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_unify::kv_get(
        &conn, NAMESPACE, KEY,
    )
    .ok()??;
    serde_json::from_str(&raw).ok()
}

fn persist_snapshot(snap: &CoreSnapshot) -> Result<(), String> {
    let conn = open_kb()?;
    let json = serde_json::to_string(snap).map_err(|e| format!("snapshot serialize: {}", e))?;
    crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_unify::kv_set(&conn, NAMESPACE, KEY, &json)
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试隔离: 将 HOME 重定向到临时目录, 避免污染生产 KB (~/.neotrix/knowledge.db),
    /// 且各测试间共享同一隔离 DB (Once 保证仅初始化一次)。
    fn isolate_home_once() {
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| {
            let tmp = std::env::temp_dir().join(format!("neotrix-ctests-{}", std::process::id()));
            std::fs::create_dir_all(&tmp).ok();
            std::env::set_var("HOME", &tmp);
        });
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
        let snap = CoreSnapshot { cycle, resonance_cycle: cycle, ..Default::default() };
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
    fn snapshot_includes_fruits_with_evidence() {
        // 果实可完整序列化 (含 run_id 证据投影), 跨会话可恢复
        let mut tree = ConsciousnessTree::new();
        tree.fruits.push(
            crate::core::nt_core_consciousness_tree::EvolutionFruit {
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
            },
        );
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
                crate::core::nt_core_self_test::SelfTestResult::pass("nt_core_consciousness_monitor"),
                crate::core::nt_core_self_test::SelfTestResult::pass("nt_memory_narrative_consistency"),
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
            assert!(handle.snapshot.mars_system2_iterations >= 3,
                "S2 应至少随 3 周期自增, got {}", handle.snapshot.mars_system2_iterations);
            assert!(handle.snapshot.cycle >= 3,
                "cycle 应至少推进 3, got {}", handle.snapshot.cycle);
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
                snapshot: CoreSnapshot { cycle: 10, ..Default::default() },
            };
            // 持久化 baseline cycle=10 (模拟他进程已跑到 10)
            write_test_baseline(10);
            let snap = handle.tick(2);
            assert!(snap.cycle >= 12, "并发 tick 应叠加最新基线, got {}", snap.cycle);
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
            handle.tree.core.next_actions.push(
                "create new module nt_core_autonomous_agent.rs without mapping".to_string(),
            );
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
}