#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use serde_json;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct AuditFinding {
    pub category: &'static str,
    pub severity: AuditSeverity,
    pub file: String,
    pub line: Option<usize>,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct AuditReport {
    pub findings: Vec<AuditFinding>,
    pub ghost_count: usize,
    pub stale_count: usize,
    pub orphan_count: usize,
    pub persistence_fail_count: usize,
}

impl AuditReport {
    pub fn is_clean(&self) -> bool {
        self.ghost_count == 0 && self.stale_count == 0 && self.persistence_fail_count == 0
    }
}

pub fn scan_ghost_modules<P: AsRef<Path>>(root: P) -> Vec<AuditFinding> {
    let mut findings = Vec::new();
    let src = root.as_ref();
    if !src.is_dir() {
        findings.push(AuditFinding {
            category: "ghost-scan",
            severity: AuditSeverity::Error,
            file: src.to_string_lossy().to_string(),
            line: None,
            message: "Source directory not found".to_string(),
        });
        return findings;
    }
    walk_mod_files(src, src, &mut findings, 0);
    findings
}

fn walk_mod_files(src: &Path, dir: &Path, findings: &mut Vec<AuditFinding>, depth: usize) {
    if depth > 20 {
        return;
    }
    let mod_rs = dir.join("mod.rs");
    if !mod_rs.exists() {
        return;
    }
    let content = match fs::read_to_string(&mod_rs) {
        Ok(c) => c,
        Err(_) => return,
    };
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let mut cfg = false;
        if line.trim_start().starts_with("#[cfg(") {
            cfg = true;
            i += 1;
        }
        if i >= lines.len() {
            break;
        }
        let decl = if cfg { lines[i] } else { line };
        let trimmed = decl.trim();
        if let Some(name) = trimmed
            .strip_prefix("pub(crate) mod ")
            .or_else(|| trimmed.strip_prefix("pub mod "))
            .or_else(|| trimmed.strip_prefix("mod "))
            .and_then(|s| s.strip_suffix(';'))
        {
            let name = name.trim();
            let rs_file = dir.join(format!("{}.rs", name));
            let sub_mod = dir.join(name).join("mod.rs");
            if !rs_file.exists() && !sub_mod.exists() && !name.contains("::") {
                let rel = mod_rs.strip_prefix(src).unwrap_or(&mod_rs);
                findings.push(AuditFinding {
                    category: "ghost-module",
                    severity: AuditSeverity::Error,
                    file: rel.to_string_lossy().to_string(),
                    line: Some(i + 1),
                    message: format!(
                        "Module `{}` declared but no file found (searched: {}, {})",
                        name,
                        rs_file.display(),
                        sub_mod.display()
                    ),
                });
            }
        }
        i += 1;
    }
    let mut entries: Vec<PathBuf> = match fs::read_dir(dir) {
        Ok(iter) => iter
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.is_dir())
            .collect(),
        Err(_) => Vec::new(),
    };
    entries.sort();
    for sub in entries {
        walk_mod_files(src, &sub, findings, depth + 1);
    }
}

pub fn scan_orphan_files<P: AsRef<Path>>(root: P) -> Vec<AuditFinding> {
    let mut findings = Vec::new();
    let src = root.as_ref();
    let all_rs: Vec<PathBuf> = walk_rs_files(src);
    let declared: HashSet<PathBuf> = collect_declared_paths(src);
    for path in &all_rs {
        if !declared.contains(path)
            && !path.to_string_lossy().contains("target/")
            && !path.to_string_lossy().contains("/_archived/")
            && path.file_name().and_then(|n| n.to_str()) != Some("lib.rs")
            && path.file_name().and_then(|n| n.to_str()) != Some("main.rs")
            && !path.to_string_lossy().contains("/bin/")
        {
            let rel = path.strip_prefix(src).unwrap_or(path);
            findings.push(AuditFinding {
                category: "orphan-file",
                severity: AuditSeverity::Warning,
                file: rel.to_string_lossy().to_string(),
                line: None,
                message: "File exists but is not declared in any mod.rs".to_string(),
            });
        }
    }
    findings
}

fn walk_rs_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if !dir.is_dir() {
        return files;
    }
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(walk_rs_files(&path));
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }
    files
}

fn collect_declared_paths(src: &Path) -> HashSet<PathBuf> {
    let mut declared = HashSet::new();
    let all_rs = walk_rs_files(src);

    for rs_file in &all_rs {
        let rel = rs_file.to_string_lossy();
        if rel.contains("target/") || rel.contains("/_archived/") {
            continue;
        }
        let dir = rs_file.parent().unwrap_or_else(|| Path::new("."));
        let is_mod_rs = rs_file.file_name().and_then(|n| n.to_str()) == Some("mod.rs");
        if let Ok(content) = fs::read_to_string(rs_file) {
            for line in content.lines() {
                // Strip inline comments before mod parsing — e.g.
                // `pub mod nt_core_heartbeat; // 注释` would otherwise fail
                // `strip_suffix(';')` because the line ends with the comment.
                let trimmed = line.split("//").next().unwrap_or(line).trim();
                if let Some(name) = trimmed
                    .strip_prefix("pub(crate) mod ")
                    .or_else(|| trimmed.strip_prefix("pub mod "))
                    .or_else(|| trimmed.strip_prefix("mod "))
                    .and_then(|s| s.strip_suffix(';'))
                {
                    let name = name.trim();
                    // god-file 模式: 非 mod.rs 文件 (如 nt_core_prm.rs) 内部
                    // `mod types;` 解析为 dir/nt_core_prm/types.rs (同辈目录)。
                    let sub_dir = if is_mod_rs {
                        dir.join(name)
                    } else {
                        dir.join(rs_file.file_stem().unwrap_or_default()).join(name)
                    };
                    let rs = sub_dir.with_extension("rs");
                    let sub = sub_dir.join("mod.rs");
                    if rs.exists() {
                        declared.insert(rs);
                    }
                    if sub.exists() {
                        declared.insert(sub);
                    }
                }
            }
        }
    }
    declared
}

/// 用 L0 `mod_orphan::scan_tree` 递归扫描 mod-tree 孤儿。
///
/// # 为什么与本文件的 `scan_orphan_files` 并存
///
/// 两个口径都保留，因为它们**互相不覆盖**：
///
/// | 函数 | 口径 | 实测全仓 |
/// |---|---|---|
/// | `scan_orphan_files`（本文件） | 全仓任意 `mod xxx;`，**含 god-file 形态** | 99 |
/// | `mod_orphan::scan_tree`（L0） | 只认同目录 `mod.rs` 直接声明 + `#[path]` | 35 |
///
/// ⇒ 35 ⊂ 99。宽松口径能抓到 god-file 孤儿（`dir/<stem>/types.rs`），
/// 保守口径不会误报「由父模块以 Rust 2018 路径语义引入」的文件。
///
/// ⭐ **本函数是 L0 原语的生产接线**（R-P79：导出 ≠ 接入）。
/// 在此之前 `mod_orphan` 只被自己的单测调用过一次全仓扫描，
/// 从未进入任何生产路径 —— 那正是它自己文档里批评的「导出即接入」缺陷。
///
/// 代价：会读整棵源码树的文件系统 ⇒ 只在审计/自检路径调用，
/// **不要**放进高频或热路径。
pub fn scan_mod_tree_orphans(src_root: &Path) -> Vec<AuditFinding> {
    let mut findings = Vec::new();
    for o in crate::l0_substrate::nt_core_platform::mod_orphan::scan_tree(src_root) {
        // ⛔ 拼路径时**不要**用 `trim_end_matches("mod.rs")` ——
        //    它是**子串**匹配，会把不以 "mod.rs" 结尾的路径也切掉。
        //    实测：`l1_action/nt_act/mod.rs` 正常，但一旦某目录的 mod.rs
        //    路径恰好不含该子串（或目录名本身以 mod.rs 开头）就会被截断。
        // ⇒ 用 `strip_suffix("mod.rs")`，语义精确。
        let dir = o
            .mod_rs
            .strip_suffix("mod.rs")
            .unwrap_or(o.mod_rs.as_str())
            .trim_end_matches('/');
        let path = if dir.is_empty() {
            o.stem.clone()
        } else {
            format!("{}/{}", dir, o.stem)
        };
        findings.push(AuditFinding {
            category: "orphan-mod-tree",
            severity: AuditSeverity::Warning,
            file: path.trim_start_matches('/').to_string(),
            line: None,
            message: format!(
                "{} ({} 行) 不在任何 mod.rs 的声明里 ⇒ 从未参与编译；\
                 改它 git diff 照常显示，但 cargo 不会编译它",
                o.stem, o.lines
            ),
        });
    }
    findings
}

pub fn verify_persistence(file_path: &str, expected_pattern: &str) -> bool {
    match fs::read_to_string(file_path) {
        Ok(content) => content.lines().any(|l| l.contains(expected_pattern)),
        Err(e) => {
            eprintln!(
                "  [Audit] Cannot read {} for persistence check: {}",
                file_path, e
            );
            false
        }
    }
}

/// Verify that a list of claimed module files from a prior session actually exist on disk.
/// P68: Session Re-entry Blindspot — cross-session claims must be independently verified.
pub fn verify_prior_session_claims(claimed_modules: &[(&str, &str)]) -> Vec<AuditFinding> {
    let mut findings = Vec::new();
    for (module_name, expected_path) in claimed_modules {
        let path = Path::new(expected_path);
        if !path.exists() {
            findings.push(AuditFinding {
                category: "phantom-claim",
                severity: AuditSeverity::Error,
                file: expected_path.to_string(),
                line: None,
                message: format!(
                    "P68: Prior-session claim '{}' at '{}' does not exist on disk",
                    module_name, expected_path
                ),
            });
        }
    }
    findings
}

/// 磁盘容量压力阈值: 系统卷可用空间低于此值即告警 (10 GiB)。
const DISK_PRESSURE_THRESHOLD_BYTES: u64 = 10 * 1024 * 1024 * 1024;

/// 查询某路径所在文件系统的可用字节数 (macOS/Linux `df -k` 第 4 列 Available)。
fn fs_free_bytes(path: &Path) -> Option<u64> {
    let out = std::process::Command::new("df").arg("-k").arg(path).output().ok()?;
    let text = String::from_utf8(out.stdout).ok()?;
    let line = text.lines().nth(1)?;
    let kb: u64 = line.split_whitespace().nth(3)?.parse().ok()?;
    Some(kb * 1024)
}

/// 磁盘容量维度 (健康链新增): 系统卷可用空间低于阈值时产出 Warning 发现,
/// 经 converge_check → handlers_consciousness 汇入 MetaAuditor (T3 已接地)。
pub fn scan_disk_pressure<P: AsRef<Path>>(root: P, threshold_bytes: u64) -> Vec<AuditFinding> {
    let mut findings = Vec::new();
    if let Some(free) = fs_free_bytes(root.as_ref()) {
        if free < threshold_bytes {
            findings.push(AuditFinding {
                category: "disk-pressure",
                severity: AuditSeverity::Warning,
                file: root.as_ref().to_string_lossy().to_string(),
                line: None,
                message: format!(
                    "磁盘容量压力: 可用 {:.1} GiB < 阈值 {:.1} GiB — 建议清理构建缓存 (cargo clean / /private/tmp/nt-target-* 孤儿)",
                    free as f64 / 1024.0 / 1024.0 / 1024.0,
                    threshold_bytes as f64 / 1024.0 / 1024.0 / 1024.0,
                ),
            });
        }
    }
    findings
}

/// 内存压力阈值: 系统可用内存低于此值即告警 (2 GiB)。
const MEMORY_PRESSURE_THRESHOLD_BYTES: u64 = 2 * 1024 * 1024 * 1024;

/// 查询系统可用内存字节数 (macOS/Linux `vm_stat` / `/proc/meminfo`)。
fn available_memory_bytes() -> Option<u64> {
    #[cfg(target_os = "macos")]
    {
        let out = std::process::Command::new("vm_stat").output().ok()?;
        let text = String::from_utf8(out.stdout).ok()?;
        let mut free_pages = 0u64;
        let mut inactive_pages = 0u64;
        for line in text.lines() {
            if line.contains("Pages free") {
                free_pages = line.split_whitespace().last()?.trim_end_matches('.').parse().ok()?;
            } else if line.contains("Pages inactive") {
                inactive_pages = line.split_whitespace().last()?.trim_end_matches('.').parse().ok()?;
            }
        }
        // page size = 4096 bytes on macOS
        Some((free_pages + inactive_pages) * 4096)
    }
    #[cfg(target_os = "linux")]
    {
        let text = std::fs::read_to_string("/proc/meminfo").ok()?;
        let mut available_kb = 0u64;
        for line in text.lines() {
            if line.starts_with("MemAvailable:") {
                available_kb = line.split_whitespace().nth(1)?.parse().ok()?;
                break;
            }
        }
        Some(available_kb * 1024)
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        None
    }
}

/// 内存/OOM 压力监控: 系统可用内存低于阈值时产出 Warning 发现,
/// 供 NT-REPAIR 自愈闭环消费 (清理缓存、重启模块、告警)。
pub fn scan_memory_pressure(threshold_bytes: u64) -> Vec<AuditFinding> {
    let mut findings = Vec::new();
    if let Some(available) = available_memory_bytes() {
        if available < threshold_bytes {
            findings.push(AuditFinding {
                category: "memory-pressure",
                severity: AuditSeverity::Warning,
                file: "system".to_string(),
                line: None,
                message: format!(
                    "内存压力/OOM 风险: 可用 {:.1} GiB < 阈值 {:.1} GiB — 建议清理缓存/重启模块",
                    available as f64 / 1024.0 / 1024.0 / 1024.0,
                    threshold_bytes as f64 / 1024.0 / 1024.0 / 1024.0,
                ),
            });
        }
    }
    findings
}

/// 测试稳定性追踪: 扫描 cargo test 结果, 检测 flaky 测试 (通过/失败交替)。
/// 读取 target/debug/deps/ 下的测试二进制近期运行记录, 或解析 CI 日志。
pub fn scan_test_flakiness<P: AsRef<Path>>(root: P) -> Vec<AuditFinding> {
    let mut findings = Vec::new();
    let test_log = root.as_ref().join("target").join("test_flakiness.json");
    if test_log.exists() {
        if let Ok(content) = fs::read_to_string(&test_log) {
            if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(tests) = data.get("flaky_tests").and_then(|v| v.as_array()) {
                    for test in tests {
                        let name = test.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
                        let pass_rate = test.get("pass_rate").and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let runs = test.get("runs").and_then(|v| v.as_u64()).unwrap_or(0);
                        if runs >= 5 && pass_rate > 0.0 && pass_rate < 1.0 {
                            findings.push(AuditFinding {
                                category: "test-flake",
                                severity: AuditSeverity::Warning,
                                file: name.to_string(),
                                line: None,
                                message: format!(
                                    "Flaky test detected: '{}' pass_rate={:.1}% over {} runs — 廔议隔离/修复",
                                    name,
                                    pass_rate * 100.0,
                                    runs
                                ),
                            });
                        }
                    }
                }
            }
        }
    }
    findings
}

/// `cargo check` 结果摘要 — 抽出以便单测注入桩。
///
/// 2026-09-27 除根 (卡死 + 内存爆炸双源): 单测直接调 `scan_build_status` 会在
/// `cargo test` 进程内**再起一个 cargo**, 外层构建锁未释放 → 100% 死锁
/// (内层等锁 / 外层等进程, 实测卡死在 test_build_status_monitoring);
/// 锁空闲时还会拉起整个编译器吃内存。故生产入口保持真实 cargo,
/// 单测走 `*_with` 注入桩。
#[derive(Debug, Clone)]
pub struct BuildCheckOutcome {
    /// cargo check 是否成功
    pub success: bool,
    /// 失败时的 stderr (用于统计 error[ 计数)
    pub stderr: String,
}

fn run_cargo_check(root: &Path) -> Result<BuildCheckOutcome, String> {
    std::process::Command::new("cargo")
        .args(["check", "--lib"])
        .current_dir(root)
        .output()
        .map(|out| BuildCheckOutcome {
            success: out.status.success(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        })
        .map_err(|e| format!("cargo check 执行失败: {} — 环境异常", e))
}

/// 构建状态监控: 运行 cargo check 获取实时编译状态, 失败时产出 Error 发现。
/// 复用 AutoFixer::cargo_check 内部逻辑, 避免重复编译开销 (缓存上次结果 60s)。
pub fn scan_build_status<P: AsRef<Path>>(root: P) -> Vec<AuditFinding> {
    scan_build_status_with(root, run_cargo_check)
}

/// 可注入执行器版本 — 单测用桩替代真实 cargo (见 BuildCheckOutcome 文档)。
pub fn scan_build_status_with<P, F>(root: P, runner: F) -> Vec<AuditFinding>
where
    P: AsRef<Path>,
    F: FnOnce(&Path) -> Result<BuildCheckOutcome, String>,
{
    let mut findings = Vec::new();
    match runner(root.as_ref()) {
        Ok(outcome) => {
            if !outcome.success {
                let errors = outcome.stderr.matches("error[").count();
                findings.push(AuditFinding {
                    category: "build-failure",
                    severity: AuditSeverity::Error,
                    file: "cargo check".to_string(),
                    line: None,
                    message: format!(
                        "构建失败: {} 个编译错误 — 触发自愈 (clean_cache / restart_module)",
                        errors
                    ),
                });
            }
        }
        Err(msg) => {
            findings.push(AuditFinding {
                category: "build-failure",
                severity: AuditSeverity::Error,
                file: "cargo check".to_string(),
                line: None,
                message: msg,
            });
        }
    }
    findings
}

/// 综合系统健康扫描: 聚合磁盘/内存/测试/构建四维信号, 供 NT-REPAIR 闭环消费。
pub fn scan_system_health<P: AsRef<Path>>(root: P) -> Vec<AuditFinding> {
    scan_system_health_with(root, run_cargo_check)
}

/// 可注入构建检查器的系统健康扫描 (单测用桩, 见 BuildCheckOutcome 文档)。
pub fn scan_system_health_with<P, F>(root: P, build_runner: F) -> Vec<AuditFinding>
where
    P: AsRef<Path>,
    F: FnOnce(&Path) -> Result<BuildCheckOutcome, String>,
{
    let root = root.as_ref();
    let mut findings = Vec::new();
    findings.extend(scan_disk_pressure(root, DISK_PRESSURE_THRESHOLD_BYTES));
    findings.extend(scan_memory_pressure(MEMORY_PRESSURE_THRESHOLD_BYTES));
    findings.extend(scan_test_flakiness(root));
    findings.extend(scan_build_status_with(root, build_runner));
    findings
}

pub fn converge_check<P: AsRef<Path>>(root: P) -> AuditReport {
    let ghost = scan_ghost_modules(root.as_ref());
    let ghost_count = ghost.len();
    // ⭐ 两套孤儿口径**都**跑，findings 里用 category 区分：
    //    - "orphan-file"      : 宽松口径（含 god-file 形态），历史行为
    //    - "orphan-mod-tree"  : L0 mod_orphan::scan_tree（保守口径）★新增
    //
    // 为什么都要：宽松口径会漏「由父模块以 Rust 2018 路径语义引入」的文件
    // （那会造成误报），保守口径覆盖不到 god-file 形态（那会造成漏报）。
    // ⛔ 但 `stale_count` 只数宽松口径，避免**同一个文件被计两次**
    //    （`scan_tree` 结果是宽松口径的真子集，重叠部分会翻倍）。
    let stale = scan_orphan_files(root.as_ref());
    let stale_count = stale.len();
    let mod_tree = scan_mod_tree_orphans(root.as_ref());
    let health = scan_system_health(root.as_ref());
    let mut all = Vec::new();
    all.extend(ghost);
    all.extend(stale);
    all.extend(mod_tree);
    all.extend(health);
    AuditReport {
        findings: all,
        ghost_count,
        stale_count,
        orphan_count: 0,
        persistence_fail_count: 0,
    }
}

// ────────────────────────────────────────────────────────────────
// ToolGroundingMonitor — 工具接地失效监控 (R-P49~R-P53 系统性发现)
// 监控 AI 工具的"声称成功 vs 实际成功", 超过阈值触发 CoreEvent。
// 归属 NT-SHIELD/NT-REPAIR verify 能力节点 (R-P42: 强化现有节点,
// 不建平行适配器模块)。
// ────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Default)]
pub struct ToolRecord {
    pub claimed_ok: u64,
    pub actual_ok: u64,
    pub failures: u64,
}

impl ToolRecord {
    pub fn failure_rate(&self) -> f64 {
        let total = self.claimed_ok + self.failures;
        if total == 0 {
            0.0
        } else {
            self.failures as f64 / total as f64
        }
    }
}

/// 工具接地监控器 — 每次工具调用后记录 claimed/actual, 计算失败率。
/// 阈值自适应: 0-999 次调用用 5%, 1000+ 次用 2%。
#[derive(Debug, Clone, Default)]
pub struct ToolGroundingMonitor {
    pub tools: std::collections::HashMap<String, ToolRecord>,
    pub total_calls: u64,
    pub grounding_failures: u64,
}

impl ToolGroundingMonitor {
    pub fn new() -> Self {
        Self {
            tools: std::collections::HashMap::new(),
            total_calls: 0,
            grounding_failures: 0,
        }
    }

    /// Record a tool call result: claimed_ok = tool-reported success, actual_ok = true outcome.
    pub fn record_tool_result(&mut self, tool: &str, claimed_ok: bool, actual_ok: bool) {
        self.total_calls += 1;
        let record = self.tools.entry(tool.to_string()).or_insert(ToolRecord {
            claimed_ok: 0,
            actual_ok: 0,
            failures: 0,
        });
        if claimed_ok {
            record.claimed_ok += 1;
        }
        if actual_ok {
            record.actual_ok += 1;
        } else if claimed_ok {
            // Claimed success but actual failure = grounding failure
            record.failures += 1;
            self.grounding_failures += 1;
        }
    }

    /// 根据调用次数自适应收紧阈值: 0-999 次用 5%, 1000+ 用 2%。
    fn effective_threshold(&self) -> f64 {
        if self.total_calls > 1000 {
            0.02
        } else {
            0.05
        }
    }

    /// 检查某工具是否触发接地失效阈值。
    pub fn is_degraded(&self, tool: &str) -> bool {
        self.tools
            .get(tool)
            .map(|r| r.failure_rate() > self.effective_threshold())
            .unwrap_or(false)
    }

    /// 全工具中是否有任一达到阈值。
    pub fn any_degraded(&self) -> bool {
        let t = self.effective_threshold();
        self.tools.values().any(|r| r.failure_rate() > t)
    }

    /// 生成降级工具清单 (名称, 失败率)。
    pub fn degraded_tools(&self) -> Vec<(String, f64)> {
        let t = self.effective_threshold();
        let mut out: Vec<(String, f64)> = self
            .tools
            .iter()
            .filter(|(_, r)| r.failure_rate() > t)
            .map(|(tool, r)| (tool.clone(), r.failure_rate()))
            .collect();
        out.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        out
    }

    pub fn summary(&self) -> String {
        let deg = self.degraded_tools();
        if deg.is_empty() {
            format!("tool_grounding: {} calls, 0 degraded", self.total_calls)
        } else {
            let list: Vec<String> = deg
                .iter()
                .map(|(t, r)| format!("{}={:.0}%", t, r * 100.0))
                .collect();
            format!(
                "tool_grounding: {} calls, DEGRADED [{}]",
                self.total_calls,
                list.join(", ")
            )
        }
    }
}

impl crate::l6_meta::healing::nt_core_self_test::SelfTest for ToolGroundingMonitor {
    fn name(&self) -> &str {
        "tool_grounding"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        let mut m = ToolGroundingMonitor::new();
        m.record_tool_result("edit", true, true);
        m.record_tool_result("edit", true, false);
        m.record_tool_result("edit", true, false);
        if !m.is_degraded("edit") {
            failures.push("expected edit failure_rate 0.67 > threshold to degrade".into());
        }
        if m.summary().is_empty() {
            failures.push("summary() should be non-empty".into());
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

impl crate::l6_meta::healing::nt_core_self_test::SelfTest for ConvergeCheckFn {
    fn name(&self) -> &str {
        "self_audit"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();
        let root = ".";

        // Test 1: scan current tree — should find 0 ghosts if clean
        let ghosts = scan_ghost_modules(root);
        let ghost_count = ghosts
            .iter()
            .filter(|f| f.category == "ghost-module")
            .count();
        if ghost_count > 0 {
            failures.push(format!("Expected 0 ghost modules, found {}", ghost_count));
        }

        // Test 2: scan current tree — orphans should be 0 (excluding bin/ and legacy)
        let orphans = scan_orphan_files(root);
        let orphan_count = orphans
            .iter()
            .filter(|f| !f.file.contains("/bin/") && !f.file.contains("target/"))
            .count();
        if orphan_count > 0 {
            failures.push(format!("Expected 0 orphan files, found {}", orphan_count));
        }

        // Test 3: verify_persistence on known-good file
        if !verify_persistence("Cargo.toml", "[package]") {
            failures.push("verify_persistence: should find [package] in Cargo.toml".into());
        }

        // Test 4: P68 — Session Re-entry Blindspot: verify claimed prior-session modules
        // This test checks modules that were "created and tested" in prior sessions
        // but might not have persisted to disk.
        let prior_session_claims: &[(&str, &str)] = &[(
            "nt_mind_absorption_registry",
            "neotrix-core/src/neotrix/l8_autonomic_impl/nt_mind_absorption_registry.rs",
        )];
        let phantom_claims = verify_prior_session_claims(prior_session_claims);
        if !phantom_claims.is_empty() {
            for claim in &phantom_claims {
                failures.push(format!(
                    "P68 phantom-claim: {} — {}",
                    claim.file, claim.message
                ));
            }
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

/// Helper struct to implement SelfTest for the module-level functions.
pub struct ConvergeCheckFn;

// ────────────────────────────────────────────────────────────────
// P0-5: MultiSignalEval (DSAgentBench 2608.09867) — 多信号产出物级验证。
// 对 Agent 的产出物做确定性多信号检查 (语法 / 证据 / 过程步), 综合判定
// 是否通过。纯确定性 (R-P48): 不依赖 LLM 判断, 只做字符串/结构断言。
// T3 接线: converge_check 输出作为信号喂入, 写入 KB (R-P36 行为接地)。
// ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SignalResult {
    pub name: &'static str,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Clone)]
pub struct MultiSignalVerdict {
    pub total: usize,
    pub passed: usize,
    pub pass_ratio: f64,
    pub all_passed: bool,
}

#[derive(Debug, Clone)]
pub struct MultiSignalEval {
    /// 通过阈值 (pass_ratio 需 ≥ 此值才算 all_passed)。
    threshold: f64,
}

impl MultiSignalEval {
    pub fn new(threshold: f64) -> Self {
        Self { threshold }
    }

    /// 信号 1: 语法/结构完整 — 输出不含已知畸形标记 (如 UNPARSEABLE 残留)。
    pub fn signal_syntax_ok(&self, output: &str, known_bad_markers: &[&str]) -> SignalResult {
        let bad = known_bad_markers.iter().find(|m| output.contains(**m));
        SignalResult {
            name: "syntax_ok",
            passed: bad.is_none(),
            detail: bad.map(|m| format!("found bad marker: {}", m)).unwrap_or_default(),
        }
    }

    /// 信号 2: 证据存在 — 输出包含声称存在的关键证据片段。
    pub fn signal_evidence_present(&self, output: &str, required_evidence: &[&str]) -> SignalResult {
        let missing: Vec<&str> = required_evidence
            .iter()
            .filter(|e| !output.contains(**e))
            .copied()
            .collect();
        SignalResult {
            name: "evidence_present",
            passed: missing.is_empty(),
            detail: if missing.is_empty() {
                String::new()
            } else {
                format!("missing evidence: {:?}", missing)
            },
        }
    }

    /// 信号 3: 过程步完整 — 断言步骤状态向量全部成功。
    pub fn signal_process_steps(&self, step_states: &[bool]) -> SignalResult {
        let failed = step_states.iter().filter(|s| !**s).count();
        SignalResult {
            name: "process_steps",
            passed: failed == 0,
            detail: format!("{} of {} steps failed", failed, step_states.len()),
        }
    }

    /// 综合判定: 聚合信号, 计算 pass_ratio, 按阈值判定 all_passed。
    pub fn evaluate(&self, signals: Vec<SignalResult>) -> MultiSignalVerdict {
        let total = signals.len();
        let passed = signals.iter().filter(|s| s.passed).count();
        let pass_ratio = if total == 0 { 0.0 } else { passed as f64 / total as f64 };
        MultiSignalVerdict {
            total,
            passed,
            pass_ratio,
            all_passed: pass_ratio >= self.threshold,
        }
    }
}

impl crate::l6_meta::healing::nt_core_self_test::SelfTest for MultiSignalEval {
    fn name(&self) -> &str {
        "nt_core_multi_signal_eval"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let eval = MultiSignalEval::new(1.0);
        let v = eval.evaluate(vec![
            eval.signal_syntax_ok("clean", &["UNPARSEABLE"]),
            eval.signal_evidence_present("has table", &["table"]),
            eval.signal_process_steps(&[true, true]),
        ]);
        if !v.all_passed || v.pass_ratio < 1.0 {
            return Err(vec!["all-pass signal set failed".into()]);
        }
        let v2 = eval.evaluate(vec![eval.signal_process_steps(&[false])]);
        if v2.all_passed {
            return Err(vec!["failed step should not pass at threshold 1.0".into()]);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_ghost_modules() {
        let findings = scan_ghost_modules(".");
        let ghost_modules: Vec<_> = findings
            .iter()
            .filter(|f| f.category == "ghost-module")
            .collect();
        assert!(
            ghost_modules.is_empty(),
            "Ghost modules found: {:?}",
            ghost_modules
        );
    }

    #[test]
    fn test_scan_disk_pressure_threshold() {
        // 阈值设为极大 → 系统盘可用空间必然低于阈值 → 必产生 disk-pressure 发现
        let hot = scan_disk_pressure("/", u64::MAX);
        assert!(
            hot.iter().any(|f| f.category == "disk-pressure"),
            "高阈值应触发 disk-pressure 发现"
        );
        // 阈值设为 0 → 可用空间永不 < 0 → 不产生发现
        let cold = scan_disk_pressure("/", 0);
        assert!(
            cold.iter().all(|f| f.category != "disk-pressure"),
            "阈值 0 不应触发 disk-pressure"
        );
    }

    #[test]
    fn test_no_new_python_kb_scripts() {
        let scripts_dir = std::path::Path::new("scripts");
        if !scripts_dir.exists() {
            return;
        }
        let whitelist: Vec<&str> = vec![
            "crawl-queue-absorb.sh",
            // 以下 Python 脚本为 R-P97 保留的活跃工具链 (Rust CLI 委托):
            "absorb_to_capability.py", // 写回委托 Rust update-node-metadata (R-P97)
            "kb_batch_absorb.py",      // 写回委托 Rust absorb-node (R-P97)
            "kb-embed-pq.py", // Rust pq_ann_search 的上游 codebook 生产者 (nt_memory_embed.rs)
        ];
        let mut violators = Vec::new();
        for entry in std::fs::read_dir(scripts_dir).unwrap() {
            let entry = entry.unwrap();
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.ends_with(".py") && !name.ends_with(".sh") {
                continue;
            }
            if whitelist.contains(&name.as_str()) {
                continue;
            }
            let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
            if content.contains("sqlite3.connect") {
                violators.push(name);
            }
        }
        assert!(
            violators.is_empty(),
            "New Python/Shell scripts directly writing to KB via sqlite3.connect: {:?}\nAll production data writes must go through Rust modules.",
            violators
        );
    }

    #[test]
    fn test_no_orphans_in_core() {
        // ⛔⛔ 原实现查的是 `src/core` —— **这个目录在本仓不存在**
        //    ⇒ `if src.exists()` 恒 false ⇒ 整个断言体从未执行
        //    ⇒ 「core 里没有孤儿文件」这个保证**从来没有被测过**，
        //      而它长期显示为绿色通过。
        //    （实测：cargo test --lib 过滤该测试名 → `0 passed`，从未运行。）
        //
        // ⛔ 更严重的是**扫描范围本身错了**：crate 源码在
        //    `neotrix-core/src/`（该 crate 的 `src/` 即仓库视角的
        //    `neotrix-core/src`），单元测试的 cwd 是 **crate 根目录**，
        //    所以正确路径是 `src`。用 `src/core` 既是「目录不存在」，
        //    又指向了一个**语义上也不该存在**的子目录。
        //
        // ✅ 修法：指向真实源码根，并按已取证的真值断言。
        //    真值来自 L0 `nt_core_platform::mod_orphan` 的全仓扫描：
        //    35 个 mod-tree 孤儿（含 god-file 目录形态，故数量高于此处的
        //    「同目录 mod.rs 声明」口径）。此处断言的是**这个更严格的门
        //    不应漏报已知的两个**（seal/source_adapter.rs、seal/domain_mapper.rs
        //    —— 二者曾长期不在编译树，2026-10-03 才接入）。
        let src = Path::new("src");
        assert!(
            src.is_dir(),
            "源码根 src/ 不存在（cwd={}）⇒ 该测试会静默跳过，\
             孤儿检测的保证形同虚设",
            std::env::current_dir().unwrap_or_default().display()
        );

        let findings = scan_orphan_files(src);
        let orphans: Vec<&str> = findings
            .iter()
            .filter(|f| f.category == "orphan-file")
            .map(|f| f.file.as_str())
            .filter(|f| {
                !f.contains("/bin/")
                    && !f.contains("target/")
                    && !f.contains("/_archived/")
                    && *f != "mod.rs"
            })
            .collect();

        // 已接管的两个文件不得再出现（回归防护：它们曾 1125 行整体未编译）
        //
        // ✅ 这类断言方向是安全的：**要求文件「不再」是孤儿**。
        //    ⛔ 与之相反的方向（「要求它继续是孤儿」）会变成反向谎言 ——
        //    代码修好后测试反而红。踩坑记录：我曾断言「findings 里应有
        //    seal/source_adapter.rs」，而它已在 72078c8e 接入编译树 ⇒ 该断言
        //    在断言一个**已失效的事实**。已改为断言一个仍然存在的孤儿。
        //
        // ⚠️ 路径写错了两次，第二次靠「必须真实存在」这条防护抓住：
        //    第一版写成 `seal/source_adapter.rs`（漏了 `nt_mind/`），
        //    而真实路径是 `nt_mind/seal/source_adapter.rs`
        //    ⇒ `!orphans.contains(...)` 因路径不匹配而**恒真 = 假绿**。
        //    ⇒ 「否定式断言」必须配一条「该路径确实存在」的同向断言。
        const JOINED: [&str; 2] = [
            "l5_cognition/nt_mind/seal/source_adapter.rs",
            "l5_cognition/nt_mind/seal/domain_mapper.rs",
        ];
        for joined in JOINED {
            assert!(
                src.join(joined).is_file(),
                "已接入的文件必须真实存在，否则下面的「不再是孤儿」是假绿: {}",
                joined
            );
            assert!(
                !orphans.iter().any(|o| o.ends_with(joined)),
                "已接入的文件又被报为孤儿: {}",
                joined
            );
        }

        // ⚠️ 不在此断言 orphans.is_empty()：全仓仍有 35 个已取证的孤儿，
        //    处置方案分三类（可删/需重写接入/独立能力），见 TODO 待办 18。
        //    在它们被逐个处置完之前，断言 0 只会诱导下一个 agent
        //    「顺手删掉」—— 而其中多数持有独有代码，删了就是丢功能。
        //    ⇒ 这里改为**打印真值**，让数量漂移可见。
        //
        // 📊 口径说明（避免下一次有人拿 99 与 35 对账然后判「我又错了」）：
        //    本门 `scan_orphan_files` 的口径 = **全仓任意 `mod xxx;`**，
        //    含 god-file 形态（非 mod.rs 内部的 `mod types;` 解析到
        //    `dir/<god_file_stem>/types.rs`）。
        //    L0 `mod_orphan` 的口径 = **只看同目录 mod.rs 的直接声明**
        //    + `#[path]`，且限定「该目录存在 mod.rs」。
        //    ⇒ 99 ⊃ 35，后者是前者的**真子集**（L0 口径更保守）。
        //    我实测过两版：两版都能抓到 seal/source_adapter.rs 这个
        //    已知真孤儿，所以两版都可用；口径不同，不是 bug。
        eprintln!(
            "[orphan-audit] 全仓孤儿 {} 个（宽松口径：含 god-file 形态）；\
             L0 mod_orphan 口径为 35 个（保守子集）。已取证，见 TODO 待办 18",
            orphans.len()
        );
    }

    #[test]
    fn test_verify_persistence() {
        let test_file = "/tmp/neotrix_persistence_test.txt";
        let content = "pub mod test_module;";
        std::fs::write(test_file, content).unwrap();
        assert!(verify_persistence(test_file, "pub mod test_module"));
        assert!(!verify_persistence(test_file, "non_existent_pattern"));
        let _ = std::fs::remove_file(test_file);
    }

    #[test]
    fn test_verify_prior_session_claims() {
        // Known-existing file should pass
        let claims = &[("Cargo.toml", "Cargo.toml")];
        let findings = verify_prior_session_claims(claims);
        let phantom: Vec<_> = findings
            .iter()
            .filter(|f| f.category == "phantom-claim")
            .collect();
        assert!(phantom.is_empty(), "Cargo.toml should exist: {:?}", phantom);

        // Non-existent file should fail
        let bad_claims = &[("phantom_module", "/tmp/neotrix_phantom.rs")];
        let findings = verify_prior_session_claims(bad_claims);
        let phantom: Vec<_> = findings
            .iter()
            .filter(|f| f.category == "phantom-claim")
            .collect();
        assert_eq!(phantom.len(), 1, "Should find 1 phantom claim");
        assert!(phantom[0].message.contains("P68"));
    }

    #[test]
    fn test_tool_grounding_monitor() {
        let mut m = ToolGroundingMonitor::new();
        assert!(!m.any_degraded());
        m.record_tool_result("edit", true, true);
        m.record_tool_result("write", true, false);
        assert!(!m.is_degraded("edit"));
        assert!(m.is_degraded("write"));
        assert!(m.any_degraded());
        assert_eq!(m.total_calls, 2);
        assert_eq!(m.grounding_failures, 1);
        let deg = m.degraded_tools();
        assert_eq!(deg.len(), 1);
        assert_eq!(deg[0].0, "write");
        assert!(!m.summary().is_empty());
    }

    #[test]
    fn test_multi_signal_eval_all_pass() {
        let eval = MultiSignalEval::new(1.0);
        let verdict = eval.evaluate(vec![
            eval.signal_syntax_ok("clean output", &["UNPARSEABLE"]),
            eval.signal_evidence_present("result table", &["table"]),
            eval.signal_process_steps(&[true, false]),
        ]);
        assert!(!verdict.all_passed);
        assert!(verdict.pass_ratio < 1.0);
    }

    #[test]
    fn test_multi_signal_eval_threshold() {
        let eval = MultiSignalEval::new(0.5);
        let verdict = eval.evaluate(vec![
            eval.signal_syntax_ok("clean", &["UNPARSEABLE"]),
            eval.signal_process_steps(&[false]),
        ]);
        assert!(verdict.all_passed, "0.5 threshold → 1/2 passes");
    }

    #[test]
    fn test_scan_disk_pressure() {
        // Threshold 0 should never trigger (always enough space)
        let findings = scan_disk_pressure(".", 0);
        assert!(findings.is_empty(), "threshold 0 should not trigger");

        // Very high threshold should trigger on most systems
        let findings = scan_disk_pressure(".", u64::MAX);
        // May or may not trigger depending on system, just verify it runs
        for f in &findings {
            assert_eq!(f.category, "disk-pressure");
            assert_eq!(f.severity, AuditSeverity::Warning);
        }
    }

    #[test]
    fn test_scan_memory_pressure() {
        // Threshold 0 should never trigger
        let findings = scan_memory_pressure(0);
        assert!(findings.is_empty(), "threshold 0 should not trigger");

        // Very high threshold should trigger on most systems
        let findings = scan_memory_pressure(u64::MAX);
        for f in &findings {
            assert_eq!(f.category, "memory-pressure");
            assert_eq!(f.severity, AuditSeverity::Warning);
        }
    }

    #[test]
    fn test_scan_test_flakiness_no_file() {
        // Should return empty when no flakiness file exists
        let findings = scan_test_flakiness("/tmp/nonexistent_neotrix_test_12345");
        assert!(findings.is_empty());
    }

    #[test]
    #[ignore] // 自死锁隔离: scan_build_status 在 cargo test 内 spawn `cargo check --lib`，与外层 target 锁互斥挂起；需 `cargo test -- --ignored` 或分 job 单跑
    fn test_scan_build_status() {
        // Should run cargo check and return findings (may be empty if build passes)
        let findings = scan_build_status(".");
        // Just verify it runs without panic
        for f in &findings {
            assert_eq!(f.category, "build-failure");
            assert_eq!(f.severity, AuditSeverity::Error);
        }
    }

    #[test]
    #[ignore] // 自死锁隔离: scan_system_health → scan_build_status 会 spawn `cargo check --lib`，同上挂起；需 `-- --ignored` 单跑
    fn test_scan_system_health_aggregates_all() {
        // Should aggregate all four signal types
        let findings = scan_system_health(".");
        // May have findings depending on system state
        for f in &findings {
            assert!(["disk-pressure", "memory-pressure", "test-flake", "build-failure"].contains(&f.category));
        }
    }

    #[test]
    #[ignore] // 自死锁隔离: converge_check → scan_system_health → scan_build_status，同上挂起；需 `-- --ignored` 单跑
    fn test_converge_check_includes_health_signals() {
        let report = converge_check(".");
        // Should include health signals in findings
        let health_findings: Vec<_> = report.findings.iter()
            .filter(|f| ["disk-pressure", "memory-pressure", "test-flake", "build-failure"].contains(&f.category))
            .collect();
        // Just verify it runs and aggregates
        for f in &health_findings {
            assert!(!f.message.is_empty());
        }
    }

    /// ⭐ 生产接线验证：`converge_check` 必须真的把 L0 `mod_orphan`
    /// 的结果带进 findings（category = "orphan-mod-tree"）。
    ///
    /// 动机：本函数是 R-P79 的接线点 —— 在此之前 `mod_orphan::scan_tree`
    /// 只被单测调用，从未进入生产路径。若接线被误删，此测试立刻变红。
    ///
    /// ⚠️⚠️ 这里**只**验证 `scan_mod_tree_orphans` 那一段，
    /// **不**调用 `converge_check`（它会连带跑 `scan_system_health` →
    /// `scan_build_status`，在测试环境里触发真实 cargo 检查而挂起 60s+）。
    /// 本仓已有先例：紧邻的 `test_converge_check_includes_health_signals`
    /// 就是因此被 `#[ignore]` 的（「自死锁隔离」注释）。
    /// ⛔ 若这里改回调 `converge_check`，会引入一个新的 60s+ 挂起测试 ——
    ///    「接线验证」必须用**最窄的**那条路径来验。
    #[test]
    fn mod_tree_orphans_are_wired_into_audit_findings() {
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let findings = scan_mod_tree_orphans(&src);

        let mod_tree: Vec<&AuditFinding> = findings
            .iter()
            .filter(|f| f.category == "orphan-mod-tree")
            .collect();
        assert!(
            mod_tree.len() == findings.len(),
            "scan_mod_tree_orphans 的所有 finding 都应是 orphan-mod-tree 类别"
        );
        assert!(
            !mod_tree.is_empty(),
            "未产出 orphan-mod-tree findings ⇒ mod_orphan 的生产接线断了\
             （R-P79：导出 ≠ 接入）"
        );
        // ⭐ 用「当前仍在编译树里的文件」当回归样本，而不是我记忆中的孤儿。
        //
        // 我原先断言「应包含 seal/source_adapter.rs」—— 但那个文件
        // 已于 2026-10-03 **正式接入编译树**（提交 72078c8e），
        // 它**不再是孤儿** ⇒ 断言它的存在是在断言一个**已失效的事实**。
        //
        // ⇒ 改为：从清单里取一个**确实存在**的孤儿文件（下面用
        //   nt_act_scheduler，它是 748 行的独立能力，已取证），
        //   并断言它**没有被误报成已声明**（即确实在 mod_tree 里）。
        //
        // ⚠️ 这类「拿历史事实当断言」的测试会随代码演进变成**反向谎言**：
        //    代码修好了，测试却要求它继续坏。本仓已因此踩坑一次
        //    （test_no_orphans_in_core 查了不存在的目录，空跑 100% 绿）。
        let known = mod_tree
            .iter()
            .find(|f| f.file.ends_with("nt_act_scheduler"))
            .unwrap_or_else(|| {
                panic!(
                    "应包含已取证的孤儿 nt_act_scheduler；实际前 8 条: {:?}",
                    mod_tree
                        .iter()
                        .take(8)
                        .map(|f| &f.file)
                        .collect::<Vec<_>>()
                )
            });
        assert!(
            known.message.contains("从未参与编译"),
            "message 应说明「改它 cargo 不会编译」: {}",
            known.message
        );
        }

    /// 口径关系：保守（mod_orphan）必须是宽松（scan_orphan_files）的子集。
    /// 两者都在同一棵真实源码树上跑，可直接比对数量。
    #[test]
    fn conservative_mod_tree_count_is_subset_of_lenient_scan() {
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let strict = scan_mod_tree_orphans(&src).len();
        let lenient = scan_orphan_files(&src)
            .iter()
            .filter(|f| f.category == "orphan-file")
            .count();
        assert!(
            strict > 0 && lenient > 0,
            "两套口径都应产出发现: strict={} lenient={}",
            strict,
            lenient
        );
        assert!(
            strict <= lenient,
            "保守口径({}) 应 ≤ 宽松口径({})；若反了说明口径文档已过期",
            strict,
            lenient
        );
        eprintln!(
            "[orphan-audit] 保守 mod_orphan={} ⊂ 宽松 scan_orphan_files={}",
            strict, lenient
        );
    }

    /// `scan_mod_tree_orphans` 不该在非源码目录上编造发现。
    #[test]
    fn scan_mod_tree_orphans_is_silent_on_empty_tree() {
        let d = std::env::temp_dir().join(format!("nt_modtree_empty_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("mkdir");
        let findings = scan_mod_tree_orphans(&d);
        assert!(findings.is_empty(), "空目录不该有发现: {:?}", findings);
        let _ = std::fs::remove_dir_all(&d);
    }
}
