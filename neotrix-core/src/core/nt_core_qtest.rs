//! # 量子态测试选择引擎 (Quantum Collapse Test Selection)
//!
//! 目标: **避免全量测试 (千级) → 只跑与变更"纠缠"的测试子集**。
//!
//! 量子隐喻 (计算隐喻 + VSA 权重, 非物理量子计算):
//! - **叠加态**: 所有测试潜在可运行 (superposition)。
//! - **纠缠**: 文件 ↔ 能力节点 ↔ 测试 之间的加权关联
//!   (引用/覆盖率/依赖边)。
//! - **坍缩 (collapse)**: 给定变更文件集 Δ, 沿模块引用图求闭包 Δ*,
//!   只让与 Δ* 纠缠的测试坍缩为确定运行集。
//! - **观察者**: 意识核心权重 (phi/coherence/GWT) 调制坍缩优先级。
//!
//! 安全门 (防欠测):
//! 1. Δ* 触及公共基础设施 (KB/schema/event_bus/hcube/self_test) → 回退全量 `--lib`。
//! 2. 无测试索引时回退全量 (fail-safe)。
//! 3. 低层 (l0-l2) 变更自动放大选择范围。
//!
//! 设计蓝本: `docs/1-DESIGN/2026-08-14-quantum-state-link-capability-network-fusion.md`

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

/// 测试条目 — 文件级测试锚点。
#[derive(Debug, Clone, PartialEq)]
pub struct TestEntry {
    pub file: PathBuf,
    pub name: String,
    pub crate_name: String,
}

/// 文件级依赖边。
#[derive(Debug, Clone, PartialEq)]
pub struct DepEdge {
    pub src: PathBuf,
    pub dst: PathBuf,
}

/// 公共基础设施前缀 — 触及即回退全量。
const PUBLIC_INFRA_PREFIXES: &[&str] = &[
    "src/kb",
    "src/core/nt_core_knowledge",
    "src/core/nt_core_vector_store",
    "src/core/nt_core_hcube",
    "src/core/nt_core_self_test",
    "src/core/nt_core_schema_watchdog",
    "src/core/nt_core_event",
    "src/neotrix/nt_memory_kb",
];

/// 低层域前缀 — 变更自动放大选择范围。
const LOW_LAYER_PREFIXES: &[&str] = &[
    "src/core/l0_substrate",
    "src/core/l1_body",
    "src/core/l2_perception",
    "src/neotrix/l1_body_impl",
    "src/neotrix/l2_world_impl",
];

/// 测试选择结果。
#[derive(Debug, Clone)]
pub struct SelectionReport {
    pub fallback_full: bool,
    pub selected_files: Vec<PathBuf>,
    pub selected_tests: Vec<String>,
    pub cargo_filters: Vec<String>,
    pub closure_size: usize,
    pub changed_count: usize,
    pub reason: String,
    pub index_total_files: usize,
}

/// 测试索引 — 扫描源树, 建立 文件 → 测试 的纠缠映射。
#[derive(Debug, Clone, Default)]
pub struct QTestIndex {
    pub tests_by_file: HashMap<PathBuf, Vec<String>>,
    pub deps: HashMap<PathBuf, Vec<PathBuf>>,
    pub roots: Vec<PathBuf>,
}

/// 从源码行提取 `#[test]`/`#[tokio::test]` 后的 fn 名。
/// 支持两种布局: `#[test]\nfn name` 与 `#[test] fn name` (同行)。
fn extract_test_fn(line: &str) -> Option<String> {
    let t = line.trim_start();
    if t.starts_with("fn ") {
        let rest = &t[3..];
        let name: String = rest
            .trim()
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() {
            return Some(name);
        }
    }
    // 同行布局: `#[test] fn name(...)`
    if let Some(fn_pos) = t.find(" fn ") {
        let after = &t[fn_pos + 4..];
        let name: String = after
            .trim_start()
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() {
            return Some(name);
        }
    }
    None
}

/// 提取 `use crate::...` 模块路径 (最后 1-2 段, 用于文件映射)。
fn extract_use_paths(line: &str) -> Vec<String> {
    let t = line.trim();
    if !(t.starts_with("use ") || t.contains("crate::")) {
        return Vec::new();
    }
    let mut out = Vec::new();
    if let Some(stripped) = t.strip_prefix("use ") {
        let path: String = stripped
            .trim_start()
            .chars()
            .take_while(|c| *c != ';' && *c != '{' && *c != ',')
            .collect();
        let path = path.trim().trim_end_matches('{').trim();
        if path.starts_with("crate::") {
            out.push(path["crate::".len()..].to_string());
        } else if path.starts_with("super::") {
            out.push(path.to_string());
        }
    }
    out
}

impl QTestIndex {
    pub fn new(roots: Vec<PathBuf>) -> Self {
        Self {
            tests_by_file: HashMap::new(),
            deps: HashMap::new(),
            roots,
        }
    }

    /// 递归收集目录下所有 .rs 文件。
    fn collect_rs_files(&self, dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else { return };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                if name == "target" || name == ".git" || name == "node_modules" {
                    continue;
                }
                self.collect_rs_files(&path, out);
            } else if path.extension().map(|e| e == "rs").unwrap_or(false) {
                out.push(path);
            }
        }
    }

    /// 扫描 crate 根, 建立测试索引 + 文件依赖图。
    pub fn build(&mut self) -> usize {
        let mut files = Vec::new();
        for root in &self.roots {
            self.collect_rs_files(root, &mut files);
        }
        for file in &files {
            let Ok(content) = std::fs::read_to_string(file) else { continue };
            let lines: Vec<&str> = content.lines().collect();
            let mut i = 0;
            let mut tests: Vec<String> = Vec::new();
            while i < lines.len() {
                let l = lines[i].trim();
                let is_test_attr = l.starts_with("#[test]")
                    || l.starts_with("#[tokio::test]")
                    || l.contains("#[test]")
                    || l.contains("#[tokio::test]");
                if is_test_attr {
                    // 同行布局: `mod t { #[test] fn name(...)` → 直接在当前行找 fn
                    if let Some(name) = extract_test_fn(l) {
                        tests.push(name);
                        i += 1;
                        continue;
                    }
                    // 分离布局: `#[test]` 独立行 → 下一非空行应为 fn
                    let mut j = i + 1;
                    while j < lines.len() && lines[j].trim().is_empty() {
                        j += 1;
                    }
                    if j < lines.len() {
                        if let Some(name) = extract_test_fn(lines[j]) {
                            tests.push(name);
                        }
                    }
                    i = j;
                } else {
                    i += 1;
                }
            }
            if !tests.is_empty() {
                self.tests_by_file.insert(file.clone(), tests);
            }
            let mut deps: Vec<PathBuf> = Vec::new();
            for line in lines {
                for p in extract_use_paths(line) {
                    if let Some(resolved) = self.resolve_module_path(&p, file) {
                        if resolved != *file && !deps.contains(&resolved) {
                            deps.push(resolved);
                        }
                    }
                }
            }
            if !deps.is_empty() {
                self.deps.insert(file.clone(), deps);
            }
        }
        self.tests_by_file.len()
    }

    /// 把模块路径 (crate::a::b / super::x) 解析为文件路径。
    /// `use crate::alpha::lib::ping` 末段是符号, 逐段截短重试解析。
    fn resolve_module_path(&self, mod_path: &str, from_file: &Path) -> Option<PathBuf> {
        let segs: Vec<&str> = mod_path.split("::").collect();
        if segs.is_empty() {
            return None;
        }
        // 从完整路径逐段截短 (符号在末段时, 截掉后仍可命中模块文件)
        for cut in (1..=segs.len()).rev() {
            let try_segs = &segs[..cut];
            for root in &self.roots {
                if try_segs[0] == "super" {
                    let parent = from_file.parent()?;
                    let mut p = parent.to_path_buf();
                    for s in &try_segs[1..] {
                        p.push(s);
                    }
                    for cand in [p.with_extension("rs"), p.join("mod.rs")] {
                        if cand.exists() {
                            return Some(cand);
                        }
                    }
                } else {
                    let mut p = root.to_path_buf();
                    for s in try_segs {
                        p.push(s);
                    }
                    for cand in [p.with_extension("rs"), p.join("mod.rs")] {
                        if cand.exists() {
                            return Some(cand);
                        }
                    }
                }
            }
        }
        None
    }

    /// 沿依赖边求变更闭包 Δ*。
    fn transitive_closure(&self, changed: &[PathBuf]) -> HashSet<PathBuf> {
        let mut closed: HashSet<PathBuf> = HashSet::new();
        let mut queue: Vec<PathBuf> = changed.to_vec();
        while let Some(f) = queue.pop() {
            if !closed.insert(f.clone()) {
                continue;
            }
            if let Some(deps) = self.deps.get(&f) {
                for d in deps {
                    if !closed.contains(d) {
                        queue.push(d.clone());
                    }
                }
            }
        }
        closed
    }

    /// 该文件是否属于公共基础设施。
    fn is_public_infra(path: &Path) -> bool {
        let s = path.to_string_lossy();
        PUBLIC_INFRA_PREFIXES.iter().any(|p| s.contains(p))
    }

    /// 该文件是否属于低层域。
    fn is_low_layer(path: &Path) -> bool {
        let s = path.to_string_lossy();
        LOW_LAYER_PREFIXES.iter().any(|p| s.contains(p))
    }

    /// 量子态坍缩: 给定变更文件集, 返回最小充分测试集。
    pub fn collapse(
        &self,
        changed: &[PathBuf],
        _consciousness_weights: Option<&HashMap<String, f64>>,
    ) -> SelectionReport {
        let index_total = self.tests_by_file.len();
        let changed_count = changed.len();
        if changed.is_empty() {
            return SelectionReport {
                fallback_full: false,
                selected_files: Vec::new(),
                selected_tests: Vec::new(),
                cargo_filters: Vec::new(),
                closure_size: 0,
                changed_count: 0,
                reason: "无变更文件, 不执行测试".into(),
                index_total_files: index_total,
            };
        }

        let closure = self.transitive_closure(changed);
        if closure.iter().any(|f| Self::is_public_infra(f)) {
            return SelectionReport {
                fallback_full: true,
                selected_files: Vec::new(),
                selected_tests: Vec::new(),
                cargo_filters: vec!["--lib".into()],
                closure_size: closure.len(),
                changed_count,
                reason: "变更触及公共基础设施 → 全量回退".into(),
                index_total_files: index_total,
            };
        }

        let low_layer_triggered = closure.iter().any(|f| Self::is_low_layer(f));

        let mut entangled_files: BTreeSet<PathBuf> = BTreeSet::new();
        for tfile in self.tests_by_file.keys() {
            if closure.contains(tfile) {
                entangled_files.insert(tfile.clone());
                continue;
            }
            if let Some(deps) = self.deps.get(tfile) {
                if deps.iter().any(|d| closure.contains(d)) {
                    entangled_files.insert(tfile.clone());
                }
            }
        }

        if low_layer_triggered {
            for tf in self.tests_by_file.keys() {
                entangled_files.insert(tf.clone());
            }
        }

        let mut cargo_filters: Vec<String> = Vec::new();
        let mut selected_tests: Vec<String> = Vec::new();
        if entangled_files.is_empty() {
            return SelectionReport {
                fallback_full: true,
                selected_files: Vec::new(),
                selected_tests: Vec::new(),
                cargo_filters: vec!["--lib".into()],
                closure_size: closure.len(),
                changed_count,
                reason: "变更与测试索引零纠缠 → 全量回退".into(),
                index_total_files: index_total,
            };
        }
        for tf in &entangled_files {
            if let Some(tests) = self.tests_by_file.get(tf) {
                for t in tests {
                    selected_tests.push(format!("{}::{}", tf.display(), t));
                    cargo_filters.push(t.clone());
                }
            }
            cargo_filters.push(format!("file:{}", tf.display()));
        }
        let reason = if low_layer_triggered {
            "低层变更 → 放大为全测试集 (保守)".into()
        } else {
            format!("纠缠测试坍缩: Δ*={} 文件, 命中 {} 测试文件", closure.len(), entangled_files.len())
        };

        SelectionReport {
            fallback_full: false,
            selected_files: entangled_files.into_iter().collect(),
            selected_tests,
            cargo_filters,
            closure_size: closure.len(),
            changed_count,
            reason,
            index_total_files: index_total,
        }
    }

    /// 全量测试文件清单 (索引规模报告)。
    pub fn test_file_count(&self) -> usize {
        self.tests_by_file.len()
    }

    /// 全量测试函数计数。
    pub fn test_fn_count(&self) -> usize {
        self.tests_by_file.values().map(|v| v.len()).sum()
    }
}

/// QTestEngine SelfTest — 索引健康自检 (T1: 实现 SelfTest, T2: 注册)。
///
/// 自检语义: 索引必须能建立且非空; 空索引意味着扫描失效 (依赖树变化/路径错误),
/// 此时量子坍缩会静默退化到全量 — 需显式报告。
pub struct QTestEngineSelfTest;

impl crate::core::nt_core_self_test::SelfTest for QTestEngineSelfTest {
    fn name(&self) -> &str {
        "nt_core_qtest_engine"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
        let mut idx = QTestIndex::new(vec![root]);
        let files = idx.build();
        if files == 0 {
            return Err(vec!["qtest 索引为空: 扫描失效, 量子坍缩将静默回退全量".into()]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 在临时目录搭建迷你 crate 树, 用于 collapse 语义验证。
    fn scaffold(tmp: &Path) -> (PathBuf, Vec<PathBuf>) {
        let src = tmp.join("src");
        std::fs::create_dir_all(src.join("alpha")).unwrap();
        std::fs::create_dir_all(src.join("beta")).unwrap();

        let alpha_lib = src.join("alpha/lib.rs");
        std::fs::write(
            &alpha_lib,
            "pub fn ping() -> u8 { 1 }\n#[cfg(test)]\nmod tests { #[test] fn ping_works() { assert_eq!(super::ping(), 1); } }\n",
        )
        .unwrap();
        let beta_lib = src.join("beta/lib.rs");
        std::fs::write(
            &beta_lib,
            "pub fn pong() -> u8 { 2 }\n#[cfg(test)]\nmod tests { #[test] fn pong_works() { assert_eq!(super::pong(), 2); } }\n",
        )
        .unwrap();
        let integration = src.join("integration.rs");
        std::fs::write(
            &integration,
            "use crate::alpha::lib::ping;\n#[test]\nfn integration_ping() { assert_eq!(ping(), 1); }\n",
        )
        .unwrap();

        let changed = vec![alpha_lib.clone(), beta_lib.clone()];
        (src, changed)
    }

    fn temp_dir(tag: &str) -> PathBuf {
        std::env::temp_dir().join(format!("qtest_{}_{}", tag, std::process::id()))
    }

    #[test]
    fn test_index_build_scans_tests() {
        let tmp = temp_dir("scaffold");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let (src, _) = scaffold(&tmp);
        let mut idx = QTestIndex::new(vec![src]);
        let file_count = idx.build();
        assert_eq!(file_count, 3, "3 个文件含测试");
        assert_eq!(idx.test_fn_count(), 3);
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_collapse_selects_entangled_only() {
        let tmp = temp_dir("collapse");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let (src, changed) = scaffold(&tmp);
        let mut idx = QTestIndex::new(vec![src.clone()]);
        idx.build();
        let report = idx.collapse(&changed, None);
        assert!(!report.fallback_full, "非公共设施不应回退: {}", report.reason);
        assert!(report.selected_files.len() >= 2);
        assert_eq!(report.cargo_filters.len(), 6, "3 函数 + 3 文件级, 实际 {}", report.cargo_filters.len());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_collapse_empty_changes_noop() {
        let tmp = temp_dir("empty");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let (src, _) = scaffold(&tmp);
        let mut idx = QTestIndex::new(vec![src]);
        idx.build();
        let report = idx.collapse(&[], None);
        assert!(!report.fallback_full);
        assert!(report.selected_files.is_empty());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_public_infra_falls_back_full() {
        let tmp = temp_dir("infra");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let (src, _) = scaffold(&tmp);
        let infra = src.join("kb/schema.rs");
        std::fs::create_dir_all(src.join("kb")).unwrap();
        std::fs::write(&infra, "pub fn migrate() {}\n#[cfg(test)]\nmod t { #[test] fn m() {} }\n").unwrap();
        let mut idx = QTestIndex::new(vec![src]);
        idx.build();
        let report = idx.collapse(&[infra], None);
        assert!(report.fallback_full, "公共基础设施必须回退全量");
        assert_eq!(report.cargo_filters, vec!["--lib"]);
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
