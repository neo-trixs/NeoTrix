//! 自进化循环 — 评估阶段 (`nt_loop_evaluate`).
//!
//! 项目扫描 + 精确度量 (unsafe/unwrap/TODO) + 自由能/Φ 派生 + 综合健康评分。
//! 从 `evolution_loop.rs` 纯搬移, 行为零变更。

use super::nt_loop_types::{
    EvolutionLoop, Issue, EXCESS_UNSAFE_THRESHOLD, EXCESS_UNWRAP_THRESHOLD,
    LARGE_FILE_THRESHOLD, MISSING_TESTS_THRESHOLD, TODO_LEFTOVERS_THRESHOLD,
};
use crate::l5_cognition::l1_facade::ProjectSnapshot;
use crate::l5_cognition::nt_core::nt_iit_phi::IITPhiCalculator;
use crate::l5_cognition::nt_mind::evolution::autofixer::AutoFixer;
use crate::l2_perception::nt_world::nt_world_infer::ActiveInferenceEngine;

// ============================================================
// 精确计数函数
// ============================================================

pub(crate) fn count_actual_unsafe(content: &str) -> usize {
    let mut count = 0usize;
    for line in content.lines() {
        let t = line.trim();
        if t.starts_with("//") || t.starts_with("//!") || t.starts_with("/*") || t.starts_with("*") {
            continue;
        }
        if t.contains("#![forbid(unsafe_code)]")
            || t.contains("#![deny(unsafe_code)]")
            || t.contains("#![allow(unsafe")
        {
            continue;
        }
        if line.contains("matches(\"unsafe\"") || line.contains("contains(\"unsafe\"") {
            continue;
        }
        if t.contains("unsafe {") || t.contains("unsafe fn") || t.contains("unsafe trait") || t.contains("unsafe impl") {
            count += 1;
        }
    }
    count
}

impl EvolutionLoop {
    /// 项目扫描 (基于文件系统, 当前 Cargo 项目)
    pub fn scan_project(&self) -> ProjectSnapshot {
        self._scan_project_in(None)
    }

    /// 项目扫描 — 对指定目标目录扫描（target=None 回落自身；target 为非 Rust 项目时仍扫描 .rs 文件并做 cargo check）
    pub(crate) fn _scan_project_in(&self, target: Option<&std::path::Path>) -> ProjectSnapshot {
        // 目标目录解析: 显式 target > self.target_dir > 自身 Cargo 项目根
        let root = match target {
            Some(t) => t.to_path_buf(),
            None => match &self.target_dir {
                Some(t) => t.clone(),
                None => std::path::Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf(),
            },
        };
        let src_dir = if root.join("src").is_dir() {
            root.join("src")
        } else {
            root.clone()
        };
        let mut total_files = 0usize;
        let mut total_lines = 0usize;
        let mut large_files = Vec::new();
        let mut modules_without_tests = Vec::new();
        let mut unsafe_count = 0usize;
        let mut unwrap_count = 0usize;
        let mut todo_count = 0usize;
        let mut file_unsafe_hotspots: Vec<String> = Vec::new();

        if let Ok(entries) = Self::walk_rust_files(&src_dir) {
            for path in &entries {
                total_files += 1;
                if let Ok(content) = std::fs::read_to_string(path) {
                    let line_count = content.lines().count();
                    total_lines += line_count;

                    if line_count > LARGE_FILE_THRESHOLD {
                        large_files.push(path.to_string_lossy().to_string());
                    }

                    let is_test_file = path.to_string_lossy().contains("tests")
                        || content.contains("#[cfg(test)]")
                        || content.contains("#[test]");

                    // 精确 unsafe 计数: 只计 unsafe { } / unsafe fn / unsafe trait / unsafe impl 块
                    let file_unsafe = count_actual_unsafe(&content);
                    unsafe_count += file_unsafe;
                    if file_unsafe > EXCESS_UNSAFE_THRESHOLD {
                        file_unsafe_hotspots.push(path.to_string_lossy().to_string());
                    }

                    // unwrap 计数 (排除测试文件和注释行)
                    if !is_test_file {
                        for line in content.lines() {
                            if line.contains(".unwrap(") && !line.trim_start().starts_with("//") {
                                unwrap_count += 1;
                            }
                        }
                    }

                    // 精确 TODO 计数: 只计 // TODO / // FIXME / // HACK 行
                    let file_todo = content.lines()
                        .filter(|l| {
                            let t = l.trim();
                            t.starts_with("// TODO")
                                || t.starts_with("//TODO")
                                || t.starts_with("// FIXME")
                                || t.starts_with("//FIXME")
                                || t.starts_with("// HACK")
                                || t.starts_with("//HACK")
                        })
                        .count();
                    todo_count += file_todo;

                    // Check for missing tests
                    if line_count > MISSING_TESTS_THRESHOLD
                        && !content.contains("#[cfg(test)]")
                        && !content.contains("#[test]")
                    {
                        modules_without_tests.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }

        // 测试时跳过 cargo check（避免 build lock 死锁）
        let (compile_errs, compile_warns) = if cfg!(test) {
            (0, 0)
        } else {
            match AutoFixer::cargo_check_in(Some(&root)) {
                Ok((e, w)) => (e, w),
                Err(_) => (0, 0),
            }
        };
        ProjectSnapshot {
            total_files,
            total_lines,
            large_files,
            modules_without_tests,
            file_unsafe_hotspots,
            unsafe_count,
            unwrap_count,
            todo_count,
            compile_errors: compile_errs,
            compile_warnings: compile_warns,
            test_count: 0,
            test_failures: 0,
        }
    }

    /// 递归搜索 Rust 源文件
    fn walk_rust_files(dir: &std::path::Path) -> std::io::Result<Vec<std::path::PathBuf>> {
        const SKIP_DIRS: [&str; 5] = ["target", ".git", ".backup", "node_modules", "_archive"];
        let mut files = Vec::new();
        if dir.is_dir() {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    // 跳过 target/.git/.backup/node_modules 等非源码目录
                    if path
                        .file_name()
                        .map(|n| !SKIP_DIRS.contains(&n.to_str().unwrap_or("")))
                        .unwrap_or(true)
                    {
                        files.extend(Self::walk_rust_files(&path)?);
                    }
                } else if path.extension().map(|e| e == "rs").unwrap_or(false) {
                    files.push(path);
                }
            }
        }
        Ok(files)
    }

    /// 从项目快照派生 free_energy / phi (供无世界模型上下文的调用方使用)。
    ///
    /// 语义:
    /// - `free_energy`: ActiveInference 预测误差 — 项目风险维度 (issue 密度/unsafe/unwrap/
    ///   TODO/大文件/编译警告) 越高, 自由能越高。
    /// - `phi`: IIT 集成信息 — 将健康维度构成状态向量, 度量各健康维度间的
    ///   共振整合度; 纯噪声/均衡态 → 低 Φ, 结构化分化 → 高 Φ。
    pub(crate) fn derive_free_energy_phi(snap: &ProjectSnapshot) -> (f64, f64) {
        let file_n = snap.total_files.max(1) as f64;

        // 项目风险密度 (每文件归一化)
        let issue_density = (snap.large_files.len() as f64 + snap.modules_without_tests.len() as f64) / file_n;
        let unsafe_density = snap.unsafe_count as f64 / file_n;
        let unwrap_density = snap.unwrap_count as f64 / file_n;
        let todo_density = snap.todo_count as f64 / file_n;
        let warning_density = snap.compile_warnings as f64 / file_n;

        // ActiveInference 变分自由能: F = β·E_JEPA - H(E8)/T + γ·|∇E8|
        //   jepa_energy(预测能量)   = 风险密度 (项目越脏, 世界模型预测误差越大)
        //   e8_entropy(E8 状态熵)   = 结构复杂度 (文件量级信息)
        //   e8_energy_gradient      = 0 (单次扫描无时间演化)
        let jepa_energy = issue_density + unsafe_density + unwrap_density + todo_density + warning_density;
        let e8_entropy = (1.0 + snap.total_lines as f64).ln();
        let free_energy = ActiveInferenceEngine::new()
            .compute_free_energy(jepa_energy, e8_entropy, 0.0)
            .variational_fe;

        // IIT Φ: 健康维度状态向量 → 共振集成度
        //   state = [health_ratio, large_file_ratio, no_test_ratio, unsafe_ratio,
        //            unwrap_ratio, todo_ratio, warning_ratio] (0..1 归一)
        let (large_ratio, test_ratio, unsafe_ratio) = (
            snap.large_files.len() as f64 / file_n,
            snap.modules_without_tests.len() as f64 / file_n,
            snap.unsafe_count as f64 / file_n,
        );
        let state = vec![
            1.0 - (jepa_energy / 4.0).min(1.0), // 健康度 (逆风险密度)
            large_ratio,
            test_ratio,
            unsafe_ratio,
            unwrap_density / 4.0,
            todo_density / 2.0,
            warning_density,
        ];
        let phi = IITPhiCalculator::new().compute_phi(&state).phi;

        (free_energy, phi)
    }

    /// 综合健康评分 (0-100)
    pub(crate) fn compute_evolution_score(&self, snap: &ProjectSnapshot, _issues: &[Issue]) -> f64 {
        let mut score = 100.0;

        // 大文件惩罚
        score -= snap.large_files.len() as f64 * 5.0;

        // 无测试惩罚
        score -= snap.modules_without_tests.len() as f64 * 3.0;

        // unsafe 惩罚
        if snap.unsafe_count > EXCESS_UNSAFE_THRESHOLD {
            score -= (snap.unsafe_count - EXCESS_UNSAFE_THRESHOLD) as f64 * 2.0;
        }

        // unwrap 惩罚
        if snap.unwrap_count > EXCESS_UNWRAP_THRESHOLD {
            score -= (snap.unwrap_count - EXCESS_UNWRAP_THRESHOLD) as f64 * 1.0;
        }

        // 未完成 TODO 惩罚
        if snap.todo_count > TODO_LEFTOVERS_THRESHOLD {
            score -= (snap.todo_count - TODO_LEFTOVERS_THRESHOLD) as f64 * 2.0;
        }

        // 编译错误: 致命
        if snap.compile_errors > 0 {
            score -= 30.0;
        }

        // 编译警告
        score -= snap.compile_warnings.min(20) as f64 * 1.0;

        score.clamp(0.0, 100.0)
    }
}
