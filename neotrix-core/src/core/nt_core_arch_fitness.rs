//! # NT-CORE-ARCH-FITNESS: 架构适应度函数 (Fitness Functions)
//!
//! 演进式架构 (Evolutionary Architecture): 架构约束编码为可执行守卫,
//! 违反即报警 (SelfTest Err), 驱动渐进收敛而非一次性大重构。
//!
//! P0 六个守卫:
//!   1. LayerBoundaryFitness    — 层边界: L1 不得直接依赖 L8+ (现有 3 文件违规)
//!   2. NoCycleFitness          — 能力网 DAG 无环
//!   3. CapabilityConsistencyFitness — 能力网幂等: registry 重复边 = 0
//!   4. TreeSingletonFitness    — ConsciousnessTree 生产单例 (实例化点 ≤ 1)
//!   5. DeadCodeFitness         — dead_code warning = 0
//!   6. PanicDensityFitness     — panic 债务 (unwrap/expect) 密度告警 (ADR-0002)
//!
//! 设计原则:
//!   - 纯只读扫描 (不修改代码), 违规返回 Err 附明细
//!   - 仓库根由 CARGO_MANIFEST_DIR 定位, 不依赖 cwd
//!   - 可注册到 SelfTestRegistry (register_absorbed_modules) 与
//!     SelfTestStage::process (生产接线, T3)
//!   - 守卫是"机制", 不追求一次通过 — 报警即暴露, 驱动修复

use crate::core::nt_core_self_test::SelfTest;
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

/// L1 目录: neotrix-core/src/neotrix/l1_body_impl
fn l1_root() -> PathBuf {
    src_root().join("neotrix").join("l1_body_impl")
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

// ─────────────────────────────────────────────────────────────
// 1. LayerBoundaryFitness — 层边界守卫
// ─────────────────────────────────────────────────────────────

/// 层边界: L1 (body) 文件不得直接引用 L8+ (l8_/l9_/l10_) 模块。
/// 当前已知违规 3 文件 (nt_io_agent_loop / nt_io_web::server /
/// nt_act_autonomy::nt_mind_automation), 守卫报警驱动修复到 0。
pub struct LayerBoundaryFitness;

impl SelfTest for LayerBoundaryFitness {
    fn name(&self) -> &str {
        "arch_fitness_layer_boundary"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let l1 = l1_root();
        let re = Regex::new(r"l(?:8|9|10)_").expect("valid regex");
        let mut violations = Vec::new();
        for file in rs_files(&l1) {
            let Ok(content) = std::fs::read_to_string(&file) else {
                continue;
            };
            for (i, line) in content.lines().enumerate() {
                if re.is_match(line) && line.contains("crate::neotrix") {
                    violations.push(format!(
                        "L1→L8+ 越层: {}:{} | {}",
                        file.strip_prefix(repo_root()).unwrap_or(&file).display(),
                        i + 1,
                        line.trim()
                    ));
                }
            }
        }
        if violations.is_empty() {
            Ok(())
        } else {
            let mut msg = vec![format!(
                "层边界违规 {} 处 (L1 不得依赖 L8+)",
                violations.len()
            )];
            msg.extend(violations.iter().take(20).cloned());
            Err(msg)
        }
    }
}

// ─────────────────────────────────────────────────────────────
// 2. NoCycleFitness — 能力网 DAG 无环守卫
// ─────────────────────────────────────────────────────────────

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

/// ConsciousnessTree 生产单例: 生产代码中 `ConsciousnessTree::new` 实例化点
/// 不得超过 1 (background_loop 为唯一持有者)。测试代码不计。
pub struct TreeSingletonFitness;

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
            // 单例工厂宿主豁免: nt_core_consciousness_core.rs 的 load_or_new()
            // 是 CORE 进程单例的唯一工厂 (LazyLock)。工厂内实例化是单例链起点,
            // 守卫检测的是工厂之外的散落实例化。
            if file.ends_with("nt_core_consciousness_core.rs") {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(&file) else {
                continue;
            };
            let lines: Vec<&str> = content.lines().collect();
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
                    if in_test_context(&content, i) {
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
                "ConsciousnessTree 生产实例化点 {} 处 (应单例 ≤1)",
                sites.len()
            )];
            msg.extend(sites);
            Err(msg)
        }
    }
}

/// 粗略判断行号是否在测试上下文内 (#[cfg(test)] / #[test] / mod tests)
fn in_test_context(content: &str, line_idx: usize) -> bool {
    let lines: Vec<&str> = content.lines().collect();
    let mut in_test_mod = false;
    let mut depth = 0i32;
    for (i, l) in lines.iter().enumerate() {
        if i > line_idx {
            break;
        }
        let trimmed = l.trim();
        if trimmed.starts_with("#[cfg(test)]") || trimmed.starts_with("#[test]") {
            in_test_mod = true;
        }
        if in_test_mod && i == line_idx {
            return true;
        }
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
    }
    in_test_mod
}

// ─────────────────────────────────────────────────────────────
// 5. DeadCodeFitness — dead_code 守卫
// ─────────────────────────────────────────────────────────────

/// dead_code: 运行 cargo check, 抓取 dead_code / never used / never constructed
/// warning, 数量必须为 0。crate 级 `#![allow(dead_code)]` 抑制也视为违规
/// (掩盖死代码而非消除)。
pub struct DeadCodeFitness;

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
// 批量注册
// ─────────────────────────────────────────────────────────────

/// 全部架构适应度函数 (供 register_absorbed_modules 与 SelfTestStage 复用)
pub fn arch_fitness_tests() -> Vec<Box<dyn SelfTest>> {
    vec![
        Box::new(LayerBoundaryFitness),
        Box::new(NoCycleFitness),
        Box::new(CapabilityConsistencyFitness),
        Box::new(TreeSingletonFitness),
        Box::new(DeadCodeFitness),
        Box::new(PanicDensityFitness::default()),
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
}
