//! # NT-CORE-ARCH-FITNESS: 架构适应度函数 (Fitness Functions)
//!
//! 演进式架构 (Evolutionary Architecture): 架构约束编码为可执行守卫,
//! 违反即报警 (SelfTest Err), 驱动渐进收敛而非一次性大重构。
//!
//! P0 六个有效守卫 (LayerBoundaryFitness/CoreBoundaryFitness 已退役，ADR-0003；
//!   退役原因见 SIM-28/34：所扫目录不存在，功能由 ConfidenceLabelFitness＋脚本承接):
//!   1. NoCycleFitness               — 能力网 DAG 无环
//!   2. CapabilityConsistencyFitness — 能力网幂等: registry 重复边 = 0
//!   3. TreeSingletonFitness         — ConsciousnessTree 生产单例 (持有者 ≤ 1, 按 owner 豁免)
//!   4. DeadCodeFitness              — dead_code warning = 0
//!   5. PanicDensityFitness          — panic 债务 (unwrap/expect) 密度告警 (ADR-0002)
//!   6. ConfidenceLabelFitness       — 跨层引用置信标注 + 干净对回归 (P1-04/B2, SIM-27/28)
//!
//! 设计原则:
//!   - 纯只读扫描 (不修改代码), 违规返回 Err 附明细
//!   - 仓库根由 CARGO_MANIFEST_DIR 定位, 不依赖 cwd
//!   - 可注册到 SelfTestRegistry (register_absorbed_modules) 与
//!     SelfTestStage::process (生产接线, T3)
//!   - 守卫是"机制", 不追求一次通过 — 报警即暴露, 驱动修复

use crate::l0_substrate::nt_core_self_test::SelfTest;
use regex::Regex;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// 仓库根: neotrix-core/Cargo.toml 的父目录
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

/// neotrix-core/src 目录
fn src_root() -> PathBuf {
    repo_root().join("neotrix-core").join("src")
}

/// 扫描目录下所有 .rs 文件
fn rs_files(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
        .filter(|p| p.extension().map(|e| e == "rs").unwrap_or(false))
        .collect()
}

/// 能力网无环: .neotrix/capability_registry.json 的 edges 必须构成 DAG。
/// 环 = 能力路由 (optimal_provider) 死循环风险。
pub struct NoCycleFitness;

fn load_registry_edges() -> Option<Vec<(String, String)>> {
    let path = repo_root()
        .join(".neotrix")
        .join("capability_registry.json");
    let content = std::fs::read_to_string(path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;
    let edges = v.get("edges")?.as_array()?;
    let mut out = Vec::new();
    for e in edges {
        let a = e.get(0)?.as_str()?.to_string();
        let b = e.get(1)?.as_str()?.to_string();
        out.push((a, b));
    }
    Some(out)
}

/// 检测有向图是否有环 (朴素 DFS 三色标记)
fn has_cycle(edges: &[(String, String)]) -> bool {
    use std::collections::HashMap;
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for (a, b) in edges {
        adj.entry(a.as_str()).or_default().push(b.as_str());
    }
    #[derive(Clone, Copy, PartialEq)]
    enum Color {
        Gray,
        Black,
    }
    fn dfs<'a>(
        node: &'a str,
        adj: &HashMap<&'a str, Vec<&'a str>>,
        color: &mut HashMap<&'a str, Color>,
    ) -> bool {
        match color.get(node) {
            Some(Color::Gray) => return true,
            Some(Color::Black) => return false,
            _ => {}
        }
        color.insert(node, Color::Gray);
        if let Some(neighbors) = adj.get(node) {
            for nb in neighbors {
                if dfs(nb, adj, color) {
                    return true;
                }
            }
        }
        color.insert(node, Color::Black);
        false
    }
    let mut color: HashMap<&str, Color> = HashMap::new();
    let nodes: Vec<&str> = adj.keys().copied().collect();
    for n in nodes {
        if dfs(n, &adj, &mut color) {
            return true;
        }
    }
    false
}

impl SelfTest for NoCycleFitness {
    fn name(&self) -> &str {
        "arch_fitness_capability_acyclic"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let Some(edges) = load_registry_edges() else {
            return Err(vec![
                "能力注册表不可读或为空 (预期 .neotrix/capability_registry.json)".into(),
            ]);
        };
        if edges.is_empty() {
            return Ok(());
        }
        if has_cycle(&edges) {
            Err(vec![format!("能力网 DAG 存在环 ({} 条边)", edges.len())])
        } else {
            Ok(())
        }
    }
}

// ─────────────────────────────────────────────────────────────
// 3. CapabilityConsistencyFitness — 能力网幂等守卫
// ─────────────────────────────────────────────────────────────

/// 能力网幂等: registry edges 必须无重复 (petgraph multigraph 平行边
/// 会污染最优解路由与统计)。修复: add_dependency 幂等 + export 去重。
pub struct CapabilityConsistencyFitness;

impl SelfTest for CapabilityConsistencyFitness {
    fn name(&self) -> &str {
        "arch_fitness_capability_idempotent"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let Some(edges) = load_registry_edges() else {
            return Err(vec![
                "能力注册表不可读或为空 (预期 .neotrix/capability_registry.json)".into(),
            ]);
        };
        let mut seen: HashSet<(String, String)> = HashSet::new();
        let mut dup = Vec::new();
        for e in &edges {
            if !seen.insert(e.clone()) {
                dup.push(format!("{} → {}", e.0, e.1));
            }
        }
        if dup.is_empty() {
            Ok(())
        } else {
            let mut msg = vec![format!("能力网重复边 {} 条 (应幂等合并为 0)", dup.len())];
            let mut uniq: Vec<String> = dup.clone();
            uniq.sort();
            uniq.dedup();
            msg.extend(uniq.iter().take(10).cloned());
            Err(msg)
        }
    }
}

// ─────────────────────────────────────────────────────────────
// 4. TreeSingletonFitness — ConsciousnessTree 生产单例守卫
// ─────────────────────────────────────────────────────────────

/// ConsciousnessTree 生产单例: 生产代码中「单例持有者」级的
/// `ConsciousnessTree::new` 实例化点不得超过 1 (background_loop 为唯一持有者)。
/// 工厂/自测/快照重建等非持有者实例化点按 owner 维度豁免 (见
/// `TREE_SINGLETON_EXEMPTIONS`, 每条携带豁免原因)。测试代码不计。
pub struct TreeSingletonFitness;

/// owner 维度豁免表: (文件后缀, 所属函数名, 豁免原因)。
/// 判决依据: 每个生产模块就是其自家树实例的唯一 owner —— 独立生命周期、
/// 互不共享状态、不并发持有同一逻辑单例 ⇒ 不构成「重建/双持有」。
/// 本表只豁免「非持有者」实例化点; 真正的持有者 (background_loop) 必须保留 1 个。
const TREE_SINGLETON_EXEMPTIONS: &[(&str, &str, &str)] = &[
    (
        "consciousness_core/kb_persistence.rs",
        "load_or_new",
        "CORE 进程唯一 tree 工厂 (LazyLock 语义), 单例链起点, 实例随后归 consciousness_core 独占持有",
    ),
    (
        "consciousness_core/core.rs",
        "tree_from_snapshot",
        "快照重建工厂, 仅被 load_or_new 调用, 不独立持有",
    ),
    (
        "nt_core_consciousness_tree.rs",
        "self_test",
        "T3 SelfTest 无状态自测实例, 用后即弃, 非生产持有者",
    ),
];

/// 自行向上定位某行所在的所属函数名 (最近一个 `fn ` 开头的行)。
fn enclosing_fn_name<'a>(lines: &'a [&'a str], idx: usize) -> Option<&'a str> {
    let mut i = idx as isize - 1;
    while i >= 0 {
        let t = lines[i as usize].trim_start();
        if t.starts_with("fn ") || t.contains(" fn ") {
            let rest = t.splitn(2, "fn ").nth(1)?;
            return rest.split(|c: char| !(c.is_alphanumeric() || c == '_')).next();
        }
        i -= 1;
    }
    None
}

impl SelfTest for TreeSingletonFitness {
    fn name(&self) -> &str {
        "arch_fitness_tree_singleton"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let src = src_root();
        // 兼容符号重命名: 传统 `ConsciousnessTree::new` 与并发会话的 `n()` 别名
        let re = Regex::new(r"ConsciousnessTree::new|let (?:mut )?tree = n\(\)|tree:\s*n\(\)")
            .expect("valid regex");
        let mut sites = Vec::new();
        for file in rs_files(&src) {
            // 排除守卫自身: 本文件含正则模式字符串 (如 `ConsciousnessTree::new`),
            // 会自我误报。守卫检测的是生产代码, 自身定义不在其内。
            if file.ends_with("nt_core_arch_fitness.rs") {
                continue;
            }
            // 单例工厂宿主按 owner 维度豁免 (见 TREE_SINGLETON_EXEMPTIONS);
            // 不在此处整文件豁免, 以便同文件其他生产持有者仍被检出。
            let Ok(content) = std::fs::read_to_string(&file) else {
                continue;
            };
            let lines: Vec<&str> = content.lines().collect();
            let test_ctx = test_context_flags(&content);
            for (i, line) in lines.iter().enumerate() {
                let trimmed = line.trim_start();
                // 跳过注释与 doc 注释 (守卫自身文档含示例字符串)
                if trimmed.starts_with("//") {
                    continue;
                }
                if re.is_match(line) {
                    // SelfTest 注册豁免: `register(Box::new(...ConsciousnessTree::new()))`
                    // 是 T3 SelfTest 基建的无状态自测实例, 非生产单例持有。
                    // 多行参数时 `ConsciousnessTree::new()` 在后续行, 需检查上一行。
                    let in_register = trimmed.contains("register(Box::new(")
                        || (i > 0 && lines[i - 1].trim_start().contains("register(Box::new("));
                    if in_register {
                        continue;
                    }
                    let rel = file.strip_prefix(repo_root()).unwrap_or(&file).display();
                    let site = format!("{}:{}", rel, i + 1);
                    // 测试代码豁免: 测试函数/模块内的实例化不计入生产单例
                    if test_ctx.get(i).copied().unwrap_or(false) {
                        continue;
                    }
                    // owner 维度豁免: 命中 TREE_SINGLETON_EXEMPTIONS 的
                    // (文件后缀, 所属函数) 对即为「非持有者」实例化点, 不计入。
                    let exempt = TREE_SINGLETON_EXEMPTIONS.iter().any(|(suf, fn_name, _reason)| {
                        file.ends_with(suf)
                            && enclosing_fn_name(&lines, i) == Some(*fn_name)
                    });
                    if exempt {
                        continue;
                    }
                    sites.push(site);
                }
            }
        }
        if sites.len() <= 1 {
            Ok(())
        } else {
            let mut msg = vec![format!(
                "ConsciousnessTree 生产单例持有者 {} 处 (应唯一 = 1, 工厂/自测点已按 owner 豁免)",
                sites.len()
            )];
            msg.extend(sites);
            Err(msg)
        }
    }
}

/// 逐行标注"是否处于测试上下文" — 一次扫描产出全文件结果。
///
/// 2026-09-27 性能除根: 旧 `in_test_context(content, i)` 每次调用都重新
/// `content.lines()` 全量切分, 而两个架构守卫是"遍历全仓 .rs × 逐行调用" → 每文件
/// O(命中数 × 行数) 二次方膨胀, 表现为 self-test 长时间空转 (采样实证
/// `in_test_context` 占 300-800/804 帧) 并连带内存churn。改为按文件预计算一次。
///
/// 语义与旧函数逐行严格等价: flags[i] = (处理完 i 行 cfg/test 标记后的状态) ||
/// (处理完 i 行深度复位后的状态) —— 即旧函数在 line_idx=i 处的返回值。
fn test_context_flags(content: &str) -> Vec<bool> {
    let lines: Vec<&str> = content.lines().collect();
    let mut flags = vec![false; lines.len()];
    let mut in_test_mod = false;
    let mut depth = 0i32;
    for (i, l) in lines.iter().enumerate() {
        let trimmed = l.trim();
        if trimmed.starts_with("#[cfg(test)]") || trimmed.starts_with("#[test]") {
            in_test_mod = true;
        }
        let after_marker = in_test_mod;
        if trimmed.starts_with("mod tests") || trimmed.starts_with("mod test") {
            in_test_mod = true;
        }
        if trimmed.starts_with("fn ") && !in_test_mod {
            in_test_mod = false;
        }
        // 简单括号深度跟踪, 探测测试模块闭合
        depth += trimmed.matches('{').count() as i32 - trimmed.matches('}').count() as i32;
        if in_test_mod && depth <= 0 && trimmed.starts_with('}') {
            in_test_mod = false;
        }
        flags[i] = after_marker || in_test_mod;
    }
    flags
}

/// 粗略判断行号是否在测试上下文内 (#[cfg(test)] / #[test] / mod tests)
fn in_test_context(content: &str, line_idx: usize) -> bool {
    test_context_flags(content)
        .get(line_idx)
        .copied()
        .unwrap_or(false)
}

// ─────────────────────────────────────────────────────────────
// 5. DeadCodeFitness — dead_code 守卫
// ─────────────────────────────────────────────────────────────

/// dead_code: 运行 cargo check, 抓取 dead_code / never used / never constructed
/// warning, 数量必须为 0。crate 级 `#![allow(dead_code)]` 抑制也视为违规
/// (掩盖死代码而非消除)。
pub struct DeadCodeFitness;

/// cargo check 层是否跳过 — 测试构建内禁止起子进程 (见 self_test 内注释)。
fn skip_cargo_check_tier() -> bool {
    cfg!(test) || std::env::var_os("NT_SKIP_CARGO_CHECK").is_some()
}

impl SelfTest for DeadCodeFitness {
    fn name(&self) -> &str {
        "arch_fitness_dead_code"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();

        // 1. crate 级 allow(dead_code) 抑制
        let src = src_root();
        let re = Regex::new(r"#!\[allow\(dead_code\)\]").expect("valid regex");
        for file in rs_files(&src) {
            let Ok(content) = std::fs::read_to_string(&file) else {
                continue;
            };
            if re.is_match(&content) {
                // 跳过注释/doc 里的示例文本
                let code_only: Vec<&str> = content
                    .lines()
                    .filter(|l| !l.trim_start().starts_with("//"))
                    .collect();
                if re.is_match(&code_only.join("\n")) {
                    failures.push(format!(
                        "crate 级 allow(dead_code) 抑制: {}",
                        file.strip_prefix(repo_root()).unwrap_or(&file).display()
                    ));
                }
            }
        }

        // 2. cargo check 抓 dead_code warning (仅 lib, 对齐 CI)
        //
        // 2026-09-27 除根 (第 8 条卡死): 本检测件经 arch_fitness_tests() 注册进
        // 全量 SelfTestRegistry, 单测 `test_all_have_names` 会 run_all 到这里。
        // 在 `cargo test` 进程内再起 `cargo check` → 与外层 cargo 抢 target 构建锁,
        // 外层测试阶段持锁不放 → 内层永久等待 (栈实证 read_output→poll),
        // 全量套件卡死在此; 锁空闲时还会拉起整个编译器吃内存。
        // 测试构建内跳过该层 (静态 allow(dead_code) 扫描仍跑, 语义不丢);
        // 生产/CI 仍真跑。也可显式 NT_SKIP_CARGO_CHECK=1 跳过。
        if skip_cargo_check_tier() {
            log::debug!("[arch_fitness] dead_code cargo-check tier skipped");
            return if failures.is_empty() {
                Ok(())
            } else {
                Err(failures)
            };
        }
        let output = std::process::Command::new("cargo")
            .args(["check", "--lib", "-p", "neotrix"])
            .current_dir(repo_root())
            .output();
        match output {
            Ok(o) => {
                let stderr = String::from_utf8_lossy(&o.stderr);
                let dead_warnings: Vec<String> = stderr
                    .lines()
                    .filter(|l| {
                        l.contains("dead_code")
                            || l.contains("never used")
                            || l.contains("never constructed")
                    })
                    .map(|l| l.to_string())
                    .collect();
                if !dead_warnings.is_empty() {
                    failures.push(format!(
                        "cargo check dead_code warning {} 处: {}",
                        dead_warnings.len(),
                        dead_warnings
                            .iter()
                            .take(5)
                            .cloned()
                            .collect::<Vec<_>>()
                            .join("; ")
                    ));
                }
            }
            Err(e) => {
                failures.push(format!("cargo check 运行失败: {}", e));
            }
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

// ─────────────────────────────────────────────────────────────
// 6. PanicDensityFitness — panic 债务密度守卫
// ─────────────────────────────────────────────────────────────

/// panic 债务 (unwrap/expect) 密度: 扫描 neotrix-core/src 统计 unwrap/expect
/// 调用次数与每千行密度, 输出结构化告警 (SQALE 可维护性维度)。
///
/// 阈值策略:
///   - 绝对数超阈值 (当前 2,213 unwrap + 1,167 expect 基线) → 告警
///   - 密度 (每千行) 超阈值 → 告警
///   - 趋势由 SEAL 周期聚合 (单调递减目标, ADR-0002)
pub struct PanicDensityFitness {
    pub max_abs: usize,
    pub max_per_kloc: f64,
}

impl Default for PanicDensityFitness {
    fn default() -> Self {
        Self {
            max_abs: 3000,
            max_per_kloc: 12.0,
        }
    }
}

fn count_panics_in(file: &Path) -> (usize, usize) {
    let Ok(content) = std::fs::read_to_string(file) else {
        return (0, 0);
    };
    let mut unwraps = 0usize;
    let mut expects = 0usize;
    for line in content.lines() {
        let trimmed = line.trim_start();
        // 跳过注释与字符串字面量 (粗略过滤, 足够统计趋势)
        if trimmed.starts_with("//") || trimmed.starts_with("///") || trimmed.starts_with("#[") {
            continue;
        }
        if trimmed.contains("/*") {
            continue;
        }
        unwraps += count_occurrences(line, ".unwrap(");
        expects += count_occurrences(line, ".expect(");
    }
    (unwraps, expects)
}

/// 统计字符串在行中非重叠出现次数
fn count_occurrences(haystack: &str, needle: &str) -> usize {
    let mut count = 0usize;
    let mut start = 0usize;
    while let Some(rel) = haystack[start..].find(needle) {
        count += 1;
        start += rel + needle.len();
    }
    count
}

/// 扫描 src 目录, 返回 (总 unwrap, 总 expect, 总行数)
fn scan_panic_debt() -> (usize, usize, usize) {
    let src = src_root();
    let mut unwraps = 0usize;
    let mut expects = 0usize;
    let mut lines = 0usize;
    for file in rs_files(&src) {
        let (u, e) = count_panics_in(&file);
        unwraps += u;
        expects += e;
        lines += std::fs::read_to_string(&file)
            .map(|c| c.lines().count())
            .unwrap_or(0);
    }
    (unwraps, expects, lines)
}

impl SelfTest for PanicDensityFitness {
    fn name(&self) -> &str {
        "arch_fitness_panic_density"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let (unwraps, expects, lines) = scan_panic_debt();
        let per_kloc = (unwraps + expects) as f64 / (lines as f64 / 1000.0);
        let mut failures = Vec::new();

        let total = unwraps + expects;
        if total > self.max_abs {
            failures.push(format!(
                "panic 调用绝对数超阈值: {} (unwrap={} expect={}, 阈值={})",
                total, unwraps, expects, self.max_abs
            ));
        }
        if per_kloc > self.max_per_kloc {
            failures.push(format!(
                "panic 密度超阈值: {:.1}/千行 (阈值={:.1}/千行, 样本 {} 行)",
                per_kloc, self.max_per_kloc, lines
            ));
        }

        if failures.is_empty() {
            log::info!(
                "[arch-fitness] panic_density OK: {} (unwrap={} expect={}, {:.1}/千行)",
                total,
                unwraps,
                expects,
                per_kloc
            );
            Ok(())
        } else {
            log::info!(
                "[arch-fitness] panic_density ALARM: unwrap={} expect={} ({:.1}/千行)",
                unwraps,
                expects,
                per_kloc
            );
            Err(failures)
        }
    }
}

// ─────────────────────────────────────────────────────────────
// 8. ConfidenceLabelFitness — 跨层引用置信标注 + 干净对回归 (P1-04/B2, SIM-27/28)
// ─────────────────────────────────────────────────────────────

/// L0–L6 真实目录上的跨层引用守卫 (LayerBoundaryFitness 扫的是已不存在的
/// neotrix/ 旧路径 —— 本守卫扫 `src/l<N>_*` 真实层目录)。
/// 每条直接引用按定义标注 EXTRACTED；2026-09-21 机测干净的三对做回归断言；
/// 其余方向只计数上报 (info)，消减归 P1-02，本守卫不抢活。
/// 已知局限: 注释行已排除；字符串字面量可能计入 (保守多报不少报)。
pub struct ConfidenceLabelFitness;

/// 机测干净对 (2026-09-21 bash 复核): 必须保持零引用，否则回归失败。
/// (L2→L6, L3→L4, L4→L6)。
const CLEAN_PAIRS: &[(&str, &str)] = &[("l2", "l6"), ("l3", "l4"), ("l4", "l6")];

/// 真实层子目录 (L0 无上游可违，L6 无下游可违，故只扫 L1–L5)。
const LAYER_DIRS: &[&str] = &[
    "l1_action",
    "l2_perception",
    "l3_embodiment",
    "l4_emotion",
    "l5_cognition",
];

/// 层序号: "l1_action" → 1。未知返回 99 (保守: 未知目标视为上层，宁可多报)。
fn layer_rank(name: &str) -> u32 {
    name.strip_prefix('l')
        .and_then(|s| s.chars().next())
        .and_then(|c| c.to_digit(10))
        .unwrap_or(99)
}

/// 上层引用 (from_rank < to_rank) 即越层。调用方只喂真实层目录/模块名。
fn is_forbidden(from_dir: &str, to_mod: &str) -> bool {
    layer_rank(from_dir) < layer_rank(to_mod)
}

/// 整行注释判定 (镜像 scripts/check-layer-deps.sh 过滤)。
fn is_comment_line(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("//") || t.starts_with("/*")
}

/// 从一行提取 `crate::lN_...` 目标模块段 (如 "l2_perception")。
fn extract_layer_targets(line: &str, re: &Regex) -> Vec<String> {
    re.captures_iter(line)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect()
}

impl SelfTest for ConfidenceLabelFitness {
    fn name(&self) -> &str {
        "arch_fitness_confidence_labels"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let re = Regex::new(r"crate::(l[0-6]_[a-z0-9_]+)").expect("valid regex");
        let mut clean_violations = Vec::new();
        let mut extracted_total = 0usize;
        for dir in LAYER_DIRS {
            let root = src_root().join(dir);
            for file in rs_files(&root) {
                let rel = file
                    .strip_prefix(&root)
                    .unwrap_or(&file)
                    .to_string_lossy()
                    .to_string();
                // Facade 桥接与 traits 定义是 sanctioned 通道；测试文件天然引用实现层。
                if rel.contains("facade") || rel.ends_with("traits.rs") {
                    continue;
                }
                if rel.ends_with("tests.rs") || rel.ends_with("_tests.rs") {
                    continue;
                }
                let Ok(content) = std::fs::read_to_string(&file) else {
                    continue;
                };
                let test_ctx = test_context_flags(&content);
                for (i, line) in content.lines().enumerate() {
                    if is_comment_line(line) || test_ctx.get(i).copied().unwrap_or(false) {
                        continue;
                    }
                    for target in extract_layer_targets(line, &re) {
                        if !is_forbidden(dir, &target) {
                            continue;
                        }
                        // 直接引用即 EXTRACTED (定义使然)；计数上报，消减归 P1-02。
                        extracted_total += 1;
                        let from_tag = &dir[..2];
                        let to_tag = &target[..2];
                        if CLEAN_PAIRS.contains(&(from_tag, to_tag)) {
                            clean_violations.push(format!(
                                "干净对回归: {}→{} 在 {}:{} | {}",
                                from_tag,
                                to_tag,
                                file.strip_prefix(repo_root()).unwrap_or(&file).display(),
                                i + 1,
                                line.trim()
                            ));
                        }
                    }
                }
            }
        }
        log::info!(
            "[arch-fitness] confidence_labels OK: {} extracted refs (info only; P1-02 owns reduction)",
            extracted_total
        );
        if clean_violations.is_empty() {
            Ok(())
        } else {
            let mut msg = vec![format!(
                "干净对出现新的跨层引用 {} 处 (L2→L6/L3→L4/L4→L6 必须保持零引用)",
                clean_violations.len()
            )];
            msg.extend(clean_violations.iter().take(20).cloned());
            Err(msg)
        }
    }
}

// ─────────────────────────────────────────────────────────────
// 批量注册
// ─────────────────────────────────────────────────────────────

/// 全部架构适应度函数 (供 register_absorbed_modules 与 SelfTestStage 复用)
pub fn arch_fitness_tests() -> Vec<Box<dyn SelfTest>> {
    vec![
        Box::new(NoCycleFitness),
        Box::new(CapabilityConsistencyFitness),
        Box::new(TreeSingletonFitness),
        Box::new(DeadCodeFitness),
        Box::new(PanicDensityFitness::default()),
        Box::new(ConfidenceLabelFitness),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_cycle_detects_cycle() {
        let edges = vec![
            ("a".to_string(), "b".to_string()),
            ("b".to_string(), "c".to_string()),
            ("c".to_string(), "a".to_string()),
        ];
        assert!(has_cycle(&edges));
    }

    #[test]
    fn test_has_cycle_clean_dag() {
        let edges = vec![
            ("a".to_string(), "b".to_string()),
            ("b".to_string(), "c".to_string()),
        ];
        assert!(!has_cycle(&edges));
    }

    #[test]
    fn test_has_cycle_self_loop() {
        let edges = vec![("a".to_string(), "a".to_string())];
        assert!(has_cycle(&edges));
    }

    #[test]
    fn test_in_test_context_detects_test_fn() {
        let content = "#[test]\nfn foo() {\n    ConsciousnessTree::new();\n}\n";
        // 行索引 2 (0-based) 是 new 调用行
        assert!(in_test_context(content, 2));
    }

    #[test]
    fn test_in_test_context_prod_fn() {
        let content = "fn foo() {\n    ConsciousnessTree::new();\n}\n";
        assert!(!in_test_context(content, 1));
    }

    #[test]
    fn test_count_panics_skips_comments() {
        let tmp = std::env::temp_dir().join("panic_count_test.rs");
        std::fs::write(
            &tmp,
            "let a = x.unwrap();\n// let b = y.unwrap();\nlet c = z.expect(\"m\");\n",
        )
        .unwrap();
        let (u, e) = count_panics_in(&tmp);
        std::fs::remove_file(&tmp).ok();
        assert_eq!(u, 1);
        assert_eq!(e, 1);
    }

    #[test]
    fn test_panic_density_runs() {
        let guard = PanicDensityFitness::default();
        // 守卫可运行且返回 Ok 或 Err 都算通过 (仅验证不 panic)
        let _ = guard.self_test();
    }

    #[test]
    fn test_layer_rank_orders_l0_to_l6() {
        assert_eq!(super::layer_rank("l0_substrate"), 0);
        assert_eq!(super::layer_rank("l1_action"), 1);
        assert_eq!(super::layer_rank("l5_cognition"), 5);
        assert_eq!(super::layer_rank("l6_meta"), 6);
        assert_eq!(super::layer_rank("neotrix"), 99);
        assert_eq!(super::layer_rank("traits"), 99);
    }

    #[test]
    fn test_is_forbidden_direction() {
        // 方向即层级: 低层 (小编号) 引用高层 (大编号) 才算越层。
        assert!(super::is_forbidden("l1_action", "l2_perception"));
        assert!(super::is_forbidden("l2_perception", "l6_meta"));
        assert!(!super::is_forbidden("l2_perception", "l1_action"));
        assert!(!super::is_forbidden("l3_embodiment", "l3_embodiment"));
        // 未知目标保守判违规 (宁可多报)。
        assert!(super::is_forbidden("l1_action", "neotrix"));
    }

    #[test]
    fn test_is_comment_line() {
        assert!(super::is_comment_line("// migrated from x"));
        assert!(super::is_comment_line("   /// doc"));
        assert!(super::is_comment_line("//! module"));
        assert!(super::is_comment_line("/* block */"));
        assert!(!super::is_comment_line("use crate::l2_perception::x;"));
        assert!(!super::is_comment_line(""));
    }

    #[test]
    fn test_extract_layer_targets() {
        let re = regex::Regex::new(r"crate::(l[0-6]_[a-z0-9_]+)").expect("valid regex");
        let hits = super::extract_layer_targets(
            "use crate::l2_perception::nt_world::{A, B}; let x = crate::l5_cognition::y;",
            &re,
        );
        assert_eq!(
            hits,
            vec!["l2_perception".to_string(), "l5_cognition".to_string()]
        );
        let commented = super::extract_layer_targets("// use crate::l2_perception::x;", &re);
        // 注意: 本函数不过滤注释 (仍能提取)，注释过滤由调用方 is_comment_line 负责。
        assert_eq!(commented, vec!["l2_perception".to_string()]);
        let none = super::extract_layer_targets("let x = 1;", &re);
        assert!(none.is_empty());
    }

    #[test]
    fn test_clean_pairs_are_the_measured_three() {
        // 校准来源: 2026-09-21 bash 复核 (SIM-28)。改这里必须同步改 CLEAN_PAIRS 并重测。
        assert_eq!(super::CLEAN_PAIRS.len(), 3);
        assert!(super::CLEAN_PAIRS.contains(&("l2", "l6")));
        assert!(super::CLEAN_PAIRS.contains(&("l3", "l4")));
        assert!(super::CLEAN_PAIRS.contains(&("l4", "l6")));
    }

    #[test]
    fn test_confidence_guard_passes_on_real_repo() {
        let guard = super::ConfidenceLabelFitness;
        // 针对真实仓库运行: 干净对必须零引用，否则本次接线自身即引入违规。
        // (全仓实跑断言范式：守卫跑真实仓库，现状必须通过。)
        let result = guard.self_test();
        assert!(result.is_ok(), "干净对守卫必须在现状下通过: {:?}", result);
    }
}

/// 读取死代码检测的豁免基线 —— `.neotrix/arch-fitness-exempt.txt`，每行一条子串。
///
/// 语义：某条 failure 含任一豁免子串 ⇒ 跳过。空文件 ⇒ 无豁免（最严）。
/// ⛔ 豁免必须**逐条**、带理由（写在文件内 `#` 注释里）；本函数只取子串。
pub fn arch_fitness_exemptions() -> Vec<String> {
    let p = repo_root().join(".neotrix/arch-fitness-exempt.txt");
    let Ok(s) = std::fs::read_to_string(&p) else {
        return Vec::new();
    };
    s.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod dead_code_gate {
    use super::*;
    use crate::l0_substrate::nt_core_self_test::SelfTest;

    /// 裁定 1A（2026-10-07）：把 dead_code 检测结果接进门。
    ///
    /// **背景裁定**：`lib.rs:23` 的 `#![allow(dead_code)]` 是**有意的** R-P1 约定
    /// （旁证：`recovery.rs:37-39` 特意加模块级 `#![deny(dead_code)]` 对抗它）。
    /// ⇒ 保留约定，用 `DeadCodeFitness` 补位被压掉的编译器信号。
    ///   （裁定 B「删 allow、交还编译器」被否：2483 文件/784,660 行从未被编译器审过。）
    ///
    /// **本测试补的是判据②**：结果**被消费为门**。
    /// 现状（已修）：`test_all_have_names` 只 `assert!(!r.name.is_empty())`，
    /// 把 `SelfTestResult.passed` 丢弃 ⇒ 该检测件检出了也没人看。
    /// 判据③（抑制器不遮视野）的补法是**显式豁免基线**，不是删 allow。
    ///
    /// ⛔ 本测试只覆盖 `self_test()` 的**静态 allow 扫描**层；
    /// `cargo check` 层按 `skip_cargo_check_tier()` 在 `cargo test` 内跳过
    /// （见本文件内 2026-09-27 除根注释：测试进程内起 cargo 会与外层抢 target 锁）。
    #[test]
    fn arch_fitness_dead_code_result_is_consumed() {
        // 直接调 self_test() 取 failures —— 断言的正是「results 曾被丢弃」这件事。
        let failures: Vec<String> = match DeadCodeFitness.self_test() {
            Ok(()) => Vec::new(),
            Err(f) => f,
        };
        let exempt = arch_fitness_exemptions();
        let unexpected: Vec<&String> = failures
            .iter()
            .filter(|f| !exempt.iter().any(|e: &String| f.contains(e.as_str())))
            .collect();

        assert!(
            unexpected.is_empty(),
            "arch_fitness_dead_code 检出 {} 项未豁免的死代码信号：\n{}\n\n             ⚠️ 若要豁免（如 `lib.rs` 的 crate 级 allow(dead_code) 是有意约定），\n\
             请把该路径登记进 `.neotrix/arch-fitness-exempt.txt`。\n\
             本门替代被 `lib.rs:23` 压掉的编译器信号（裁定 1A）。",
            unexpected.len(),
            unexpected
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}
