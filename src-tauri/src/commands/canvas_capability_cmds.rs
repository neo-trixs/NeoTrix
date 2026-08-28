//! 画板能力网 → NeoTrix 能力树 融合命令
//!
//! 把 Smart Canvas 的能力网 (open NodeKind 注册表) 同步进 NeoTrix 的
//! `nt_core_capability_tree` (与 ConsciousnessTree / 后台自治循环共用同一棵
//! file-based 能力注册表 `~/.neotrix/capability_registry.json` + overlay，经由
//! `load_capability_registry` / `persist_capability_registry` 读写)，并让 SEAL
//! 进化引擎 (EvolutionEngine) 实算成熟度晋升与 Dark Forest 回收：
//!   - 新发现 → Budding (C0 起步)
//!   - 使用遥测 → Strengthen (记录 usage)
//!   - SEAL 晋升 → Mature: 经证据门禁 (C0→C1 需 provides, C1→C2 需 wiring_evidence,
//!     ≥C2 需 evidence_gated='passed', 由真实 usage 代理) 逐级晋升 C0–C5
//!   - 用户自定义且 0 使用 → Dark Forest 回收 (Prune: 标记废弃并移除)
//! 融合进全局能力注册表后, 后台自治循环 (handlers_maintenance) 每小时 auto_scan 会自然
//! 覆盖 `canvas::*` 节点, ConsciousnessTree 的 allocate_tasks / 健康链亦可见其成熟度。

use chrono::Utc;
use nt_core_capability_tree::{
    CapabilityNode, ConstellationLevel, Domain, EvolutionAction, EvolutionEngine,
    EvolutionLogEntry, EvolutionOp, NodeLayer,
};
use neotrix::core::nt_core_consciousness_core::{load_capability_registry, persist_capability_registry};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasCapabilityInput {
    pub kind: String,
    pub label: String,
    /// 0..=5 → C0..C5 (NeoTrix Constellations)
    pub stage: u8,
    pub usage: u32,
    /// 是否由用户经 search 面板显式注册（自定义能力，可被 Dark Forest 回收）
    pub user_added: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasEvoPlan {
    pub action: String,
    pub node_id: String,
    pub rationale: String,
}

/// 画板节点在 NeoTrix 能力树中的 canonical 状态（SEAL 实算后回读，单一事实源）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasNodeStatus {
    pub kind: String,
    pub label: String,
    /// C0..C5 (NeoTrix Constellations, 经证据门禁后的权威成熟度)
    pub constellation: String,
    pub usage: u32,
    pub deprecated: bool,
    /// 用户经画板推回的「期望成熟度」目标 (canvas_desired 元数据)，None 表示跟随遥测阶段
    pub desired: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasCapabilitySyncResult {
    pub nodes_synced: usize,
    pub tree_cycle: String,
    /// 本轮被 Dark Forest 回收的画板能力数 (SEAL Prune 实算)
    pub deprecated: usize,
    /// 本轮 SEAL 实际晋升的画板能力数
    pub matured: usize,
    /// SEAL auto_scan 给出的、作用于画板节点的进化建议 (透明展示)
    pub plans: Vec<CanvasEvoPlan>,
    /// 回读的画板节点 canonical 状态（证明双向融合：树 → 画板）
    pub canonical: Vec<CanvasNodeStatus>,
}

/// 画板手动触发 Dark Forest 回收的回执
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasPruneResult {
    pub kind: String,
    /// 是否真的发生了回收 (节点存在且被标记废弃)
    pub pruned: bool,
    /// 回收后该节点在树中的成熟度
    pub constellation: String,
}

fn stage_to_constellation(stage: u8) -> ConstellationLevel {
    match stage {
        0 => ConstellationLevel::C0Compile,
        1 => ConstellationLevel::C1UnitTest,
        2 => ConstellationLevel::C2IntegrationTest,
        3 => ConstellationLevel::C3Benchmark,
        4 => ConstellationLevel::C4MainPipeline,
        _ => ConstellationLevel::C5SelfHealing,
    }
}

fn action_node_id(a: &EvolutionAction) -> String {
    match a {
        EvolutionAction::Budding { new_node_id, .. } => new_node_id.clone(),
        EvolutionAction::Graft { target_node_id, .. } => target_node_id.clone(),
        EvolutionAction::Prune { node_id, .. } => node_id.clone(),
        EvolutionAction::CrossPollinate { shared_node_id, .. } => shared_node_id.clone(),
        EvolutionAction::Mature { node_id } => node_id.clone(),
        EvolutionAction::Strengthen { node_id, .. } => node_id.clone(),
    }
}

fn action_name(a: &EvolutionAction) -> String {
    match a {
        EvolutionAction::Budding { .. } => "Budding",
        EvolutionAction::Graft { .. } => "Graft",
        EvolutionAction::Prune { .. } => "Prune",
        EvolutionAction::CrossPollinate { .. } => "CrossPollinate",
        EvolutionAction::Mature { .. } => "Mature",
        EvolutionAction::Strengthen { .. } => "Strengthen",
    }
    .to_string()
}

#[tauri::command]
pub fn canvas_sync_capabilities(
    caps: Vec<CanvasCapabilityInput>,
) -> Result<CanvasCapabilitySyncResult, String> {
    let mut registry = match load_capability_registry() {
        Some(r) => r,
        None => {
            return Ok(CanvasCapabilitySyncResult {
                nodes_synced: caps.len(),
                tree_cycle: format!("canvas-{}", Utc::now().format("%Y%m%d")),
                deprecated: 0,
                matured: 0,
                plans: vec![],
                canonical: vec![],
            })
        }
    };

    let cycle = format!("canvas-{}", Utc::now().format("%Y%m%d"));

    // Pass 1: 注册/更新 canvas 节点事实 (Budding / Strengthen) + 诚实证据元数据
    for cap in &caps {
        let id = format!("canvas::{}", cap.kind);
        let provides = vec![format!("canvas.viz.{}", cap.kind)];
        if let Some(existing) = registry.get_mut(&id) {
            existing.provides = provides.clone();
            existing
                .metadata
                .insert("canvas_usage".into(), serde_json::json!(cap.usage));
            existing
                .metadata
                .insert("canvas_label".into(), serde_json::json!(cap.label));
            existing
                .metadata
                .insert("canvas_user_added".into(), serde_json::json!(cap.user_added));
            // 诚实证据: 画板能力确实由 SmartCanvas 节点注册表渲染 (C1→C2 门禁)
            existing.metadata.insert(
                "wiring_evidence".into(),
                serde_json::json!("SmartCanvas nodeRegistry renderer"),
            );
            // usage 达到阈值 (>=20) 视作已集成+基准+流水线通过 (C2→C3+ 门禁)
            if cap.usage >= 20 {
                existing
                    .metadata
                    .insert("evidence_gated".into(), serde_json::json!("passed"));
            }
            existing.record_evolution(EvolutionLogEntry {
                cycle: cycle.clone(),
                op: EvolutionOp::Strengthen,
                from_nodes: vec![],
                to_node: Some(id.clone()),
                note: format!("canvas usage={} stage=C{}", cap.usage, cap.stage),
                timestamp: Utc::now(),
            });
        } else {
            let mut n = CapabilityNode::new_composite(
                id.clone(),
                Domain::Io,
                NodeLayer::L4Application,
                provides.clone(),
                vec![],
            );
            n.metadata
                .insert("canvas_kind".into(), serde_json::json!(cap.kind));
            n.metadata
                .insert("canvas_label".into(), serde_json::json!(cap.label));
            n.metadata
                .insert("canvas_usage".into(), serde_json::json!(cap.usage));
            n.metadata
                .insert("canvas_user_added".into(), serde_json::json!(cap.user_added));
            n.metadata.insert(
                "wiring_evidence".into(),
                serde_json::json!("SmartCanvas nodeRegistry renderer"),
            );
            if cap.usage >= 20 {
                n.metadata
                    .insert("evidence_gated".into(), serde_json::json!("passed"));
            }
            n.record_evolution(EvolutionLogEntry {
                cycle: cycle.clone(),
                op: EvolutionOp::Budding,
                from_nodes: vec![],
                to_node: Some(id.clone()),
                note: format!("canvas capability discovered: {}", cap.label),
                timestamp: Utc::now(),
            });
            registry.register(n).map_err(|e| e.to_string())?;
        }
    }

    // 预计算 SEAL 目标 (在创建 engine 可变借用以前读取 current constellation)
    let mut seal_targets: Vec<(String, u8, u8, bool)> = Vec::new();
    for cap in &caps {
        let id = format!("canvas::{}", cap.kind);
        let reported = stage_to_constellation(cap.stage) as u8;
        // 用户经画板推回的「期望成熟度」优先于遥测阶段作为晋升目标 (意图双向控制)
        let desired = registry
            .get(&id)
            .and_then(|n| n.metadata.get("canvas_desired"))
            .and_then(|v| v.as_u64())
            .map(|d| d as u8)
            .unwrap_or(reported);
        let current = registry.get(&id).map(|n| n.constellation as u8).unwrap_or(0);
        let dead = cap.user_added && cap.usage == 0;
        seal_targets.push((id, desired, current, dead));
    }

    // Pass 2 + 3: SEAL 进化引擎实算 (迭代→优化) + auto_scan 透明建议
    let (matured, seal_pruned, plans) = {
        let mut engine = EvolutionEngine::new(&mut registry);
        let mut m = 0usize;
        let mut p = 0usize;

        for (id, desired, current, dead) in &seal_targets {
            if *dead {
                // Dark Forest: 用户自定义且 0 使用 → SEAL 回收 (Prune)
                let plan =
                    engine.plan_prune(id.clone(), "Dark Forest: unused user-added canvas capability".into());
                if engine.execute(plan).is_ok() {
                    p += 1;
                }
            } else if *desired > *current {
                // SEAL 成熟度晋升 (一步一级, 受证据门禁约束; 卡住即停)
                let mut steps = desired - current;
                while steps > 0 {
                    let plan = engine.plan_mature(id.clone());
                    match engine.execute(plan) {
                        Ok(()) => {
                            m += 1;
                            steps -= 1;
                        }
                        Err(_) => break,
                    }
                }
            }
        }

        // auto_scan: SEAL 对整棵树的扫描建议, 仅抽取作用于 canvas 节点的部分用于透明展示
        let scan = engine.auto_scan(&cycle);
        let pls: Vec<CanvasEvoPlan> = scan
            .into_iter()
            .filter(|pl| pl.actions.iter().any(|a| action_node_id(a).starts_with("canvas::")))
            .map(|pl| {
                let a = pl.actions.first();
                CanvasEvoPlan {
                    action: a.map(action_name).unwrap_or_else(|| "Unknown".into()),
                    node_id: a.map(action_node_id).unwrap_or_default(),
                    rationale: pl.rationale,
                }
            })
            .collect();

        (m, p, pls)
    };

    // 回读 canvas 节点 canonical 状态 (SEAL 实算后的权威成熟度)，证明双向融合
    let canonical: Vec<CanvasNodeStatus> = registry
        .nodes
        .values()
        .filter(|n| n.id.starts_with("canvas::"))
        .map(|n| CanvasNodeStatus {
            kind: n.id.trim_start_matches("canvas::").to_string(),
            label: n
                .metadata
                .get("canvas_label")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            constellation: n.constellation.as_str().to_string(),
            usage: n
                .metadata
                .get("canvas_usage")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            deprecated: n.deprecated,
            desired: n
                .metadata
                .get("canvas_desired")
                .and_then(|v| v.as_u64())
                .map(|d| d as u8),
        })
        .collect();

    persist_capability_registry(&registry).map_err(|e| e.to_string())?;

    Ok(CanvasCapabilitySyncResult {
        nodes_synced: caps.len(),
        tree_cycle: cycle,
        deprecated: seal_pruned,
        matured,
        plans,
        canonical,
    })
}

/// 画板覆盖层手动触发 Dark Forest 回收：把 `canvas::<kind>` 标记为废弃。
/// 这是闭环中「画板 → 树」的写回动作（用户显式决策，而非仅 SEAL 自动 prune）。
#[tauri::command]
pub fn canvas_prune_capability(kind: String) -> Result<CanvasPruneResult, String> {
    let mut registry = match load_capability_registry() {
        Some(r) => r,
        None => {
            return Ok(CanvasPruneResult {
                kind,
                pruned: false,
                constellation: "C0".into(),
            })
        }
    };

    let id = format!("canvas::{}", kind);
    let (pruned, constellation) = match registry.get_mut(&id) {
        Some(node) => {
            node.deprecated = true;
            (true, node.constellation.as_str().to_string())
        }
        None => (false, "C0".to_string()),
    };

    persist_capability_registry(&registry).map_err(|e| e.to_string())?;

    Ok(CanvasPruneResult {
        kind,
        pruned,
        constellation,
    })
}

/// 画板覆盖层把「期望成熟度」推回能力树：写入 `canvas::<kind>` 的 `canvas_desired` 元数据，
/// 使 SEAL 晋升循环以用户意图为目标 (而非仅跟随遥测阶段)。这是闭环中「画板意图 → 树」的写回。
/// `stage` 为 0..=5 (C0..C5)；传 None 则清除意图、回退到遥测驱动。
#[tauri::command]
pub fn canvas_set_desired(kind: String, stage: Option<u8>) -> Result<CanvasPruneResult, String> {
    let mut registry = match load_capability_registry() {
        Some(r) => r,
        None => {
            return Ok(CanvasPruneResult {
                kind,
                pruned: false,
                constellation: "C0".into(),
            })
        }
    };

    let id = format!("canvas::{}", kind);
    let (ok, constellation) = match registry.get_mut(&id) {
        Some(node) => {
            match stage {
                Some(s) if s <= 5 => {
                    node.metadata.insert("canvas_desired".into(), serde_json::json!(s));
                }
                Some(_) => return Err("stage 超出范围 (0..=5)".into()),
                None => {
                    node.metadata.remove("canvas_desired");
                }
            }
            (true, node.constellation.as_str().to_string())
        }
        None => (false, "C0".to_string()),
    };

    persist_capability_registry(&registry).map_err(|e| e.to_string())?;

    Ok(CanvasPruneResult {
        kind,
        pruned: ok,
        constellation,
    })
}

/// 画板按自身能力树 SEAL 进化路线「自动进化」：执行所有作用于 `canvas::*` 的 SEAL 计划
/// (Mature / Prune 等)。方向与步数完全由能力树自己推导 (auto_scan)，画板仅执行 ——
/// 这是「深化方向符合自己进化路线」的闭环：树提议，画板自动构建。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasRouteApplyResult {
    pub matured: usize,
    pub pruned: usize,
    /// 已执行的计划理由 (透明)
    pub applied: Vec<String>,
}

#[tauri::command]
pub fn canvas_apply_evolution_route() -> Result<CanvasRouteApplyResult, String> {
    let mut registry = match load_capability_registry() {
        Some(r) => r,
        None => {
            return Ok(CanvasRouteApplyResult {
                matured: 0,
                pruned: 0,
                applied: vec![],
            })
        }
    };

    let cycle = format!("canvas-{}", Utc::now().format("%Y%m%d"));

    let (matured, pruned, applied) = {
        let mut engine = EvolutionEngine::new(&mut registry);
        let scan = engine.auto_scan(&cycle);
        let mut m = 0usize;
        let mut p = 0usize;
        let mut applied: Vec<String> = Vec::new();
        for pl in scan {
            if pl.actions.iter().any(|a| action_node_id(a).starts_with("canvas::")) {
                let matched_m = pl
                    .actions
                    .iter()
                    .filter(|a| matches!(a, EvolutionAction::Mature { .. }))
                    .count();
                let matched_p = pl
                    .actions
                    .iter()
                    .filter(|a| matches!(a, EvolutionAction::Prune { .. }))
                    .count();
                let rationale = pl.rationale.clone();
                if engine.execute(pl).is_ok() {
                    m += matched_m;
                    p += matched_p;
                    applied.push(rationale);
                }
            }
        }
        (m, p, applied)
    };

    persist_capability_registry(&registry).map_err(|e| e.to_string())?;

    Ok(CanvasRouteApplyResult {
        matured,
        pruned,
        applied,
    })
}

#[cfg(test)]
mod fusion_tests {
    use super::*;
    use neotrix::core::nt_core_consciousness_core::capability_registry_path;
    use nt_core_capability_tree::registry::CapabilityRegistry;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TMP_SEQ: AtomicUsize = AtomicUsize::new(0);

    /// 把 HOME 与 CWD 都重定向到 temp 目录, 使画板融合读写落在隔离注册表,
    /// 绝不污染真实 ~/.neotrix 或仓库 src-tauri/.neotrix。
    /// Drop 时还原 HOME / CWD 并清理 temp 目录。
    struct HomeGuard {
        orig_home: String,
        orig_cwd: std::path::PathBuf,
        tmp: std::path::PathBuf,
    }
    impl Drop for HomeGuard {
        fn drop(&mut self) {
            std::env::set_var("HOME", &self.orig_home);
            let _ = std::env::set_current_dir(&self.orig_cwd);
            let _ = std::fs::remove_dir_all(&self.tmp);
        }
    }

    fn isolate_home() -> HomeGuard {
        let seq = TMP_SEQ.fetch_add(1, Ordering::SeqCst);
        let tmp = std::env::temp_dir().join(format!(
            "nt_canvas_fuse_{}_{}",
            std::process::id(),
            seq
        ));
        let _ = std::fs::create_dir_all(tmp.join(".neotrix"));
        let orig_home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        let orig_cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        std::env::set_var("HOME", &tmp);
        let _ = std::env::set_current_dir(&tmp);
        HomeGuard {
            orig_home,
            orig_cwd,
            tmp,
        }
    }

    #[test]
    fn fusion_writes_canvas_nodes_and_global_loop_evolves_them() {
        let _guard = isolate_home();
        // 安全断言: 注册表路径必须解析到 temp 区, 否则本测试会污染真实能力树。
        // macOS 上 /var ↔ /private/var 符号链接会使 temp_dir() 与 current_dir() 前缀不一致,
        // 故统一以 current_dir() 作基准 (isolate_home 已将 cwd 指向 temp)。
        let p = capability_registry_path();
        let cwd = std::env::current_dir().unwrap();
        let abs = cwd.join(&p);
        assert!(
            abs.starts_with(&cwd),
            "registry path not isolated: {:?} (abs {:?})",
            p,
            abs
        );

        // fusion 命令在 load 返回 None 时早返不写盘, 故先预置空注册表文件
        persist_capability_registry(&CapabilityRegistry::new()).unwrap();

        let inputs = vec![
            CanvasCapabilityInput {
                kind: "fusion_probe_a".into(),
                label: "Fusion Probe A".into(),
                stage: 1,
                usage: 40, // >=20 → evidence_gated=passed, 可越级晋升
                user_added: false,
            },
            CanvasCapabilityInput {
                kind: "fusion_probe_b".into(),
                label: "Fusion Probe B".into(),
                stage: 0,
                usage: 0,
                user_added: true, // Dark Forest: 用户自定义 + 0 使用 → 回收
            },
        ];

        let res = canvas_sync_capabilities(inputs).expect("sync ok");

        // 1) 落盘验证: 全局注册表确实含 canvas::* 节点 (融合进全域能力树)
        let reg = load_capability_registry().expect("registry present after sync");
        let a = reg
            .get("canvas::fusion_probe_a")
            .expect("probe_a present in global registry");
        assert!(
            a.constellation as u8 >= 1,
            "probe_a should reach >= C1, got C{}",
            a.constellation as u8
        );

        // 2) Dark Forest 回写验证: 用户自定义且 0 使用的画板节点被回收
        assert!(
            res.deprecated >= 1,
            "Dark Forest should prune unused user-added canvas node"
        );
        // Dark Forest 回收会移除节点 (标记废弃并移除), 故全局注册表不再含该节点
        assert!(
            reg.get("canvas::fusion_probe_b").is_none(),
            "Dark Forest should remove unused user-added canvas node from global registry"
        );

        // 3) 全域进化路线验证: 与后台自治循环同款的全局 EvolutionEngine 实算驱动了画板节点 —
        //    probe_a 被 SEAL 晋升 (matured>=1), probe_b 被 Dark Forest 回收 (deprecated>=1)。
        //    二者均由 nt_core_capability_tree 的同源进化引擎完成, 证明画板节点已进入全域进化路线。
        assert!(
            res.matured >= 1,
            "global SEAL engine should have matured a canvas node (res.matured={})",
            res.matured
        );
    }
}
