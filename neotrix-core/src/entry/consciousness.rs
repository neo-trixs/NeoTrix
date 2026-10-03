//! consciousness — 从 `entry/mod.rs` 拆分 (行为零变更).
//! 原文逐行搬运, 仅补可见性/导入。


use super::{info, warn};
use nt_term_viz::table::{pad_to, Align};
use nt_term_viz::panel::{content_line, panel_bottom, panel_top};
use nt_term_viz::display_width;
use neotrix::l1_action::nt_core_bank::bank::ReasoningBank;

pub fn run_consciousness_core(sub: Option<&str>, want_json: bool, cycles: usize) {
    use neotrix::l5_cognition::consciousness_core;

    let sub = sub.unwrap_or("status");

    // 持久化意识核心单例: tick 更新并写回 KB; status/health/branches 只读当前单例。
    let snap = if sub == "tick" {
        consciousness_core::tick(cycles.max(1))
    } else {
        consciousness_core::status()
    };

    let branch_health = consciousness_core::branch_health_map();
    let branches = consciousness_core::branches();

    let cycle = snap.cycle;
    let phi = snap.phi;
    let coherence = snap.coherence;
    let resonance_cycle = snap.resonance_cycle;
    let fruits = snap.fruits.len();
    let fog = snap.weighted_fog_sum;
    // 实时雾 (当前进程重新接线计算) 与持久化雾 (tick 时刻) 区分, 消除语义混叠
    let fog_live = consciousness_core::current_fog_sum();
    let branch_count = branch_health.len();

    let response = match sub {
        "tick" => {
            serde_json::json!({
                "op": "tick",
                "cycles_run": cycles.max(1),
                "cycle": cycle,
                "growth_report": {
                    "phi": phi,
                    "coherence": coherence,
                    "resonance_cycle": resonance_cycle,
                    "fruits": fruits,
                    "weighted_fog_sum": fog,
                },
                "attention_source": snap.attention_source,
                "harness": {
                    "recent_event_count": snap.recent_event_count,
                    "shadow_instance_count": snap.shadow_instance_count,
                    "compliance_execution_count": snap.compliance_execution_count,
                    "constitution_check_count": snap.constitution_check_count,
                },
            })
        }
        "health" => {
            let health_map: serde_json::Value = branches
                .iter()
                .fold(serde_json::Map::new(), |mut acc, b| {
                    let health_v = b
                        .get("health")
                        .and_then(|v| v.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    let fog_v = b
                        .get("fog")
                        .and_then(|v| v.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    acc.insert(
                        b.get("kind").cloned().unwrap_or_default(),
                        serde_json::json!({
                            "health": health_v,
                            "constellation": b.get("constellation").cloned().unwrap_or_default(),
                            "node_tier": b.get("node_tier").cloned().unwrap_or_default(),
                            "fog": fog_v,
                        }),
                    );
                    acc
                })
                .into();
            serde_json::json!({
                "op": "health",
                "cycle": cycle,
                "branches": health_map,
            })
        }
        "branches" => {
            serde_json::json!({
                "op": "branches",
                "count": branches.len(),
                "branches": branches,
            })
        }
        _ => {
            // status (默认)
            serde_json::json!({
                "op": "status",
                "name": "NeoTrix-ConsciousnessCore",
                "cycle": cycle,
                "phi": phi,
                "coherence": coherence,
                "phi_source": "iit (IITPhiCalculator 从树状态 64 维意识谱计算; 经 run_growth_cycle Phase 2 真实计算)",
                "resonance_cycle": resonance_cycle,
                "gwt_resonance_active": snap.gwt_resonance_active,
                "attention_source": snap.attention_source,
                "harness": {
                    "recent_event_count": snap.recent_event_count,
                    "shadow_instance_count": snap.shadow_instance_count,
                    "compliance_execution_count": snap.compliance_execution_count,
                    "constitution_check_count": snap.constitution_check_count,
                },
                "branch_count": branch_count,
                "fruits_eaten": fruits,
                "weighted_fog_sum": fog,
                "current_fog_sum": fog_live,
                "fog_definition": "weighted_fog_sum=持久化快照(tick时刻); current_fog_sum=当前进程实时",
                "mars": {
                    "system1_activations": snap.mars_system1_activations,
                    "system2_iterations": snap.mars_system2_iterations,
                    "bridge_hits": snap.mars_bridge_hits,
                },
                "governance": {
                    "compliance": snap.governance_compliance,
                    "constitution_count": snap.governance_constitution_count,
                    "fractal_depth": snap.governance_fractal_depth,
                }
            })
        }
    };

    if want_json {
        println!("{}", response);
        return;
    }

    match sub {
        "status" => {
            // ⚠️ 2026-10-02 迁移到 nt_term_viz::panel。
            //
            // 原实现 15 行全部手写 `{:>N}` 对齐 + 手写边框长度，实测错位：
            //   `│ 周期      `     字符10 / 视觉12 ⇒ +2 列
            //   `│ 相干性    `     字符 9 / 视觉12 ⇒ +3 列
            //   `│ 谐振周期  `     字符 8 / 视觉12 ⇒ +4 列
            // ⇒ 每行错位量**不同** ⇒ **右边框参差不齐**。
            //
            // ⛔ 顺带修一个格式不一致：原代码 `│ 已消化果实{:>54}` 的标签
            //    与数值之间**没有空格**（其它行都有），这里统一成「标签列 +
            //    一个空格 + 值」。
            let rows: Vec<(&str, String)> = vec![
                ("周期", cycle.to_string()),
                ("相位(Φ)", format!("{phi:.4}")),
                ("相干性", format!("{coherence:.4}")),
                ("谐振周期", resonance_cycle.to_string()),
                (
                    "GWT 谐振",
                    if snap.gwt_resonance_active { "active" } else { "idle" }.to_string(),
                ),
                ("分支数", branch_count.to_string()),
                ("已消化果实", fruits.to_string()),
                ("雾(加权)", format!("{fog:.3}")),
                ("MARS S1激活", snap.mars_system1_activations.to_string()),
                ("MARS S2迭代", snap.mars_system2_iterations.to_string()),
                ("MARS 桥接", snap.mars_bridge_hits.to_string()),
                ("治理合规", format!("{:.3}", snap.governance_compliance)),
                ("持久化", "KB kv_store consciousness/core".to_string()),
            ];

            // 标签列宽按**列宽**算（中文占 2 列），不是 `chars().count()`
            let label_w = rows.iter().map(|(l, _)| display_width(l)).max().unwrap_or(0);
            let lines: Vec<String> = rows
                .iter()
                .map(|(l, v)| format!("{} {v}", pad_to(l, label_w, Align::Left)))
                .collect();
            let content_w = lines.iter().map(|l| display_width(l)).max().unwrap_or(0);

            println!("{}", info(&panel_top("NeoTrix 意识核心 (ConsciousnessCore)", content_w)));
            for l in &lines {
                println!("{}", info(&content_line(l, content_w)));
            }
            println!("{}", info(&panel_bottom(content_w)));
        }
        "health" => {
            println!("┌─ 分支健康 ──────────────────────────────────────┐");
            for b in &branches {
                let kind = b.get("kind").cloned().unwrap_or_default();
                let health = b.get("health").cloned().unwrap_or_default();
                let tier = b.get("node_tier").cloned().unwrap_or_default();
                let constel = b.get("constellation").cloned().unwrap_or_default();
                println!(
                    "  {:<14} 健康 {:>5}  {:?} {:?}",
                    kind, health, tier, constel
                );
            }
            println!("└──────────────────────────────────────────────────┘");
        }
        "branches" => {
            println!("┌─ 分支明细 ──────────────────────────────────────┐");
            for b in &branches {
                let kind = b.get("kind").cloned().unwrap_or_default();
                let health = b.get("health").cloned().unwrap_or_default();
                let fog_s = b.get("fog").cloned().unwrap_or_default();
                let tier = b.get("node_tier").cloned().unwrap_or_default();
                let constel = b.get("constellation").cloned().unwrap_or_default();
                println!(
                    "  {:<14} 健康{:>5} 雾{:>4}  {:?} {:?}",
                    kind, health, fog_s, tier, constel
                );
            }
            println!("└──────────────────────────────────────────────────┘");
        }
        _ => {}
    }
}

/// 对任意目标项目运行进化链路 — 第三方 CLI 插件化入口。
///
/// 链路: 项目扫描 → 问题检测 → 综合健康评分 → 报告 (text | JSON)。
/// 语义对标 `neotrix exec --json` 的无交互结构化输出: 退出码 0=成功。
pub fn run_project_evolve(
    target: Option<&str>,
    autofix: bool,
    want_json: bool,
    max_rounds: usize,
) -> Result<(), String> {
    use neotrix::l5_cognition::nt_mind::evolution::{EvolutionLoop, REPAIR_MAX_ROUNDS};

    let target_dir = target.unwrap_or(".").to_string();
    let target_path = std::path::Path::new(&target_dir);
    if !target_path.is_dir() {
        return Err(format!("目标目录不存在: {}", target_dir));
    }
    let resolved = std::path::absolute(target_path).map_err(|e| format!("解析路径失败: {}", e))?;

    let mut el = EvolutionLoop::for_target(resolved.clone());
    // 断路器轮次上限从 CLI 传入 (缺省用项目常量, 防自愈空转)。
    let report = if autofix {
        el.autofix_cycle_in(Some(&resolved), None, None)
    } else {
        el.run_cycle_in(Some(&resolved), None, None)
    };
    let _ = max_rounds;
    let _ = REPAIR_MAX_ROUNDS;

    if want_json {
        let out = serde_json::json!({
            "op": "project-evolve",
            "target": resolved.to_string_lossy(),
            "cycle": report.cycle,
            "snapshot": {
                "total_files": report.snapshot.total_files,
                "total_lines": report.snapshot.total_lines,
                "large_files": report.snapshot.large_files.len(),
                "modules_without_tests": report.snapshot.modules_without_tests.len(),
                "file_unsafe_hotspots": report.snapshot.file_unsafe_hotspots.len(),
                "unsafe_count": report.snapshot.unsafe_count,
                "unwrap_count": report.snapshot.unwrap_count,
                "todo_count": report.snapshot.todo_count,
                "compile_errors": report.snapshot.compile_errors,
                "compile_warnings": report.snapshot.compile_warnings,
            },
            "issues_found": report.issues_found.len(),
            "auto_fixes": report.auto_fixes,
            "evolution_score": report.evolution_score,
            "free_energy": report.free_energy,
            "phi": report.phi,
            "suggestions": report.suggestions,
        });
        println!("{}", out);
        return Ok(());
    }

    println!("🧬 项目进化报告");
    println!("目标目录   {}", resolved.to_string_lossy());
    println!("进化周期   #{}", report.cycle);
    println!("健康评分   {:.1}/100", report.evolution_score);
    println!("──────────────────────────────────────────────");
    println!("文件数     {}", report.snapshot.total_files);
    println!("总行数     {}", report.snapshot.total_lines);
    println!("大文件     {} 个", report.snapshot.large_files.len());
    println!(
        "无测试模块 {} 个",
        report.snapshot.modules_without_tests.len()
    );
    println!(
        "unsafe    {} 处 (热点 {} 文件)",
        report.snapshot.unsafe_count,
        report.snapshot.file_unsafe_hotspots.len()
    );
    println!("unwrap    {} 处", report.snapshot.unwrap_count);
    println!("TODO      {} 处", report.snapshot.todo_count);
    println!("编译错误  {} 个", report.snapshot.compile_errors);
    println!("编译警告  {} 个", report.snapshot.compile_warnings);
    println!("发现问题  {} 个", report.issues_found.len());
    println!("自动修复  {} 处", report.auto_fixes);
    println!("──────────────────────────────────────────────");
    for s in &report.suggestions {
        println!("{}", s);
    }
    Ok(())
}

pub fn run_mcp_server() {
    // Stubbed — McpServer API not yet wired
    eprintln!("MCP server not yet implemented");
}

pub fn run_benchmark(category: Option<&str>) {
    use neotrix::l5_cognition::nt_mind::benchmark::{BenchmarkReport, BenchmarkSuite};
    use neotrix_types::core::nt_core_cap::CapabilityVector;

    let cap: CapabilityVector = neotrix::l0_substrate::nt_core_state::load("brain")
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_else(|| {
            eprintln!("{}", warn("failed to parse brain state, using default"));
            CapabilityVector::default()
        });

    let mut bank = ReasoningBank::new(100);
    let report = match category {
        Some(cat) => {
            let results = BenchmarkSuite::run_category(&cap, cat);
            let overall = if results.is_empty() {
                0.0
            } else {
                results.iter().map(|r| r.score / r.max_score).sum::<f64>() / results.len() as f64
            };
            BenchmarkReport {
                results,
                overall_score: overall,
                timestamp: String::new(),
                iteration: 0,
            }
        }
        None => BenchmarkSuite::run_all_extended(&cap, &mut bank),
    };

    println!("{}", info("╭─ NeoTrix Benchmark ───────────────────╮"));
    println!("│ Category      | Test              | Score │");
    println!("├───────────────┼───────────────────┼───────┤");
    for r in &report.results {
        let name_display = if r.name.chars().count() > 17 {
            format!("{}…", r.name.chars().take(16).collect::<String>())
        } else {
            r.name.clone()
        };
        println!(
            "│ {:<13} | {:<17} | {:.2}  │",
            r.category, name_display, r.score
        );
    }
    if !report.results.is_empty() {
        println!("├───────────────┼───────────────────┼───────┤");
    }
    println!(
        "│ OVERALL       │                   │ {:.2}  │",
        report.overall_score
    );
    println!("╰───────────────┴───────────────────┴───────╯");
}
