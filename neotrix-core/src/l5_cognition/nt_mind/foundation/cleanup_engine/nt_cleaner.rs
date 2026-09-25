//! nt_cleaner — 清理引擎 (CleanupEngine/_CleanupResult/SelfTest) + 命令式清理 (_CommandCleaner)，行为零变更纯搬移.

use chrono::{Local, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use super::nt_archiver::_Archiver;
use super::nt_types::{CleanupKind, CleanupPattern, CleanupRiskLevel, Platform, _CleanupLog, _CleanupLogEntry};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _CleanupResult {
    pub kind: CleanupKind,
    pub scanned_count: usize,
    pub deletable_count: usize,
    pub estimated_bytes: u64,
    pub pattern_matches: Vec<String>,
    pub dry_run: bool,
    pub errors: Vec<String>,
    pub timestamp: i64,
}

impl _CleanupResult {
    pub fn new(kind: CleanupKind) -> Self {
        Self {
            kind,
            scanned_count: 0,
            deletable_count: 0,
            estimated_bytes: 0,
            pattern_matches: Vec::new(),
            dry_run: true,
            errors: Vec::new(),
            timestamp: Utc::now().timestamp(),
        }
    }

    pub fn summary(&self) -> String {
        let mode = if self.dry_run { "预览" } else { "已清理" };
        format!(
            "[{}] {:?}: 扫描 {} 项, 可删除 {} 项 (约 {:.1} MB), {} 个错误",
            mode,
            self.kind,
            self.scanned_count,
            self.deletable_count,
            self.estimated_bytes as f64 / 1_048_576.0,
            self.errors.len()
        )
    }
}

pub struct CleanupEngine {
    pub patterns: Vec<CleanupPattern>,
    pub whitelist: Vec<PathBuf>,
    pub history: Vec<_CleanupResult>,
    pub dry_run_default: bool,
    pub archive_on_clean: bool, // true = 归档而非删除
    pub project_root: PathBuf,
    pub risk_gate: CleanupRiskLevel, // 默认仅执行 <= 该风险级规则
    pub command_cleaner: Option<_CommandCleaner>, // 命令式清理 (SystemServices)
    max_history: usize,
}

impl Default for CleanupEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl CleanupEngine {
    pub fn new() -> Self {
        Self {
            patterns: CleanupPattern::all_patterns()
                .into_iter()
                .filter(|p| p.platform.matches(Platform::current()))
                .collect(),
            whitelist: vec![
                PathBuf::from("~/.config"),
                PathBuf::from("~/.ssh"),
                PathBuf::from("~/.gnupg"),
            ],
            history: Vec::new(),
            dry_run_default: true,
            archive_on_clean: true,
            project_root: PathBuf::from("."),
            risk_gate: CleanupRiskLevel::Medium,
            command_cleaner: Some(_CommandCleaner::new()),
            max_history: 50,
        }
    }

    pub fn _with_risk_gate(mut self, gate: CleanupRiskLevel) -> Self {
        self.risk_gate = gate;
        self
    }

    pub fn with_project_root(mut self, root: PathBuf) -> Self {
        self.project_root = root;
        self
    }

    pub fn add_whitelist(&mut self, path: PathBuf) {
        self.whitelist.push(path);
    }

    pub(crate) fn is_whitelisted(&self, path: &Path) -> bool {
        // whitelist 条目存字面 `~/.config`/`%VAR%`, 与 expand 后的绝对扫描路径
        // starts_with 永不匹配 — 先 expand 再比较 (修复: 白名单永久失效缺陷)
        self.whitelist.iter().any(|w| {
            let expanded = CleanupPattern::expand(&w.to_string_lossy());
            let expanded_path = PathBuf::from(expanded);
            !expanded_path.as_os_str().is_empty() && path.starts_with(expanded_path)
        })
    }

    pub fn scan(&self, kind: CleanupKind, dry_run: bool) -> _CleanupResult {
        let mut result = _CleanupResult::new(kind);
        result.dry_run = dry_run;

        let relevant: Vec<&CleanupPattern> = self
            .patterns
            .iter()
            .filter(|p| kind == CleanupKind::All || p.kind == kind)
            .filter(|p| p.risk <= self.risk_gate)
            .collect();

        for pattern in &relevant {
            for glob_pat in &pattern.patterns {
                let pat_str = CleanupPattern::expand(glob_pat);
                if let Ok(entries) = glob::glob(&pat_str) {
                    for entry in entries.flatten() {
                        if self.is_whitelisted(&entry) {
                            continue;
                        }
                        // 路径安全护栏: 拒绝系统根目录
                        if CleanupPattern::_is_system_root_dir(&entry) {
                            continue;
                        }
                        let is_old = if let Some(max_days) = pattern.max_age_days {
                            match std::fs::metadata(&entry) {
                                Ok(meta) => {
                                    if let Ok(modified) = meta.modified() {
                                        // age = now - modified；未来 mtime (时钟偏移/解包) 视为 0，绝不删除
                                        let age = modified
                                            .elapsed()
                                            .map(|d| d.as_secs() as i64)
                                            .unwrap_or(0);
                                        age > max_days * 86400
                                    } else {
                                        false
                                    }
                                }
                                Err(_) => false,
                            }
                        } else {
                            true
                        };

                        if is_old {
                            result.deletable_count += 1;
                            let size = CleanupPattern::_entry_size(&entry);
                            result.estimated_bytes += size;
                            if result.pattern_matches.len() < 20 {
                                result
                                    .pattern_matches
                                    .push(entry.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
        }
        result
    }

    /// 执行清理: 如果 archive_on_clean 则归档, 否则直接删除
    pub fn clean(&mut self, kind: CleanupKind) -> _CleanupResult {
        // SystemServices 类别走命令式清理通道
        if kind == CleanupKind::SystemServices {
            return self.clean_services(kind);
        }
        let mut result = self.scan(kind, false);

        if !result.dry_run && !result.pattern_matches.is_empty() {
            if self.archive_on_clean {
                // 归档模式: 移动而非删除
                let mut archiver = _Archiver::new(&self.project_root);
                match archiver._archive_paths(&result.pattern_matches, &format!("{:?}", kind)) {
                    Ok(manifest) => {
                        result.estimated_bytes = manifest.total_bytes;
                        log::info!(
                            "[cleanup] 已归档 {} 项到 .cleanup/archive/{}",
                            manifest.total_items,
                            manifest.batch_id
                        );
                    }
                    Err(e) => {
                        log::warn!("[cleanup] 归档失败, 回退到直接删除: {}", e);
                        self.delete_paths(&mut result);
                    }
                }
            } else {
                self.delete_paths(&mut result);
            }
        }

        // 记录日志
        let log_dir = self.project_root.join(".cleanup").join("log");
        _CleanupLog::log(
            &log_dir,
            &_CleanupLogEntry {
                action: if self.archive_on_clean {
                    "archive"
                } else {
                    "clean"
                }
                .into(),
                kind: format!("{:?}", kind),
                items: result.deletable_count,
                bytes: result.estimated_bytes,
                batch_id: Local::now().format("%Y-%m-%d_%H%M%S").to_string(),
                success: result.errors.is_empty(),
                error: if result.errors.is_empty() {
                    None
                } else {
                    Some(result.errors.join("; "))
                },
            },
        );

        self.history.push(result.clone());
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
        result
    }

    /// 命令式系统服务清理 (SystemServices): 遍历 command_cleaner 项执行
    fn clean_services(&mut self, kind: CleanupKind) -> _CleanupResult {
        let mut result = _CleanupResult::new(kind);
        result.dry_run = self.dry_run_default;
        let cleaner = match self.command_cleaner.as_ref() {
            Some(c) => c,
            None => {
                result.errors.push("无 command_cleaner".into());
                return result;
            }
        };
        let confirm = !result.dry_run; // dry-run 无需确认; 实删需确认 (由调用方传 --confirm)
        for (item, _) in cleaner.scan() {
            let r = cleaner.execute(item, confirm);
            result.scanned_count += 1;
            match r.status.as_str() {
                "executed" => {
                    result.deletable_count += 1;
                    if result.pattern_matches.len() < 20 {
                        result
                            .pattern_matches
                            .push(format!("{}: {}", item.name, r.output.trim()));
                    }
                }
                "needs_confirm" => {
                    if result.pattern_matches.len() < 20 {
                        result
                            .pattern_matches
                            .push(format!("{}: 需 --confirm", item.name));
                    }
                }
                "failed" => result.errors.push(format!("{}: {}", item.name, r.output)),
                // dry_run / skipped: 计入可用项 (供后台 dry-run 报告), 不计数 deletable
                _ => {
                    if result.pattern_matches.len() < 20 && r.status == "dry_run" {
                        let head = r.output.lines().take(1).next().unwrap_or("");
                        result
                            .pattern_matches
                            .push(format!("{} (dry-run): {}", item.name, head));
                    }
                }
            }
        }
        result
    }

    fn delete_paths(&self, result: &mut _CleanupResult) {
        for path_str in &result.pattern_matches {
            let path = Path::new(path_str);
            if self.is_whitelisted(path) {
                continue;
            }
            if CleanupPattern::_is_system_root_dir(path) {
                continue;
            }
            if path.is_dir() {
                if let Err(e) = std::fs::remove_dir_all(path) {
                    result
                        .errors
                        .push(format!("删除目录失败 {}: {}", path_str, e));
                }
            } else if let Err(e) = std::fs::remove_file(path) {
                result
                    .errors
                    .push(format!("删除文件失败 {}: {}", path_str, e));
            }
        }
    }

    pub fn prune_brain_snapshots(max_keep: usize) -> usize {
        let home = dirs::home_dir().unwrap_or_default();
        let snap_dir = home.join(".neotrix").join("snapshots");
        if !snap_dir.exists() {
            return 0;
        }
        let mut entries: Vec<_> = std::fs::read_dir(&snap_dir)
            .map(|d| d.filter_map(|e| e.ok()).collect::<Vec<_>>())
            .unwrap_or_default();
        entries.sort_by_key(|e| e.path());
        let mut removed = 0;
        if entries.len() > max_keep {
            for entry in entries.iter().take(entries.len() - max_keep) {
                if std::fs::remove_file(entry.path()).is_ok() {
                    removed += 1;
                }
            }
        }
        removed
    }

    /// 项目蜕皮: 将 project_root 顶层旧躯壳目录 (legacy/old/*_v0/*_backup* 命名)
    /// 整体归档至 .cleanup/archive/, 活动树只留最新态。
    ///
    /// 安全护栏 (三道闸):
    ///   ① 白名单路径 (is_whitelisted) 绝不蜕皮;
    ///   ② 系统根目录 (_is_system_root_dir) 绝不蜕皮;
    ///   ③ project_root 自身绝不蜕皮 (仅扫描其下一级, 不递归).
    pub fn molt_project(&mut self) -> _CleanupResult {
        let mut result = _CleanupResult::new(CleanupKind::ProjectMolting);
        result.dry_run = self.dry_run_default;

        let root = self.project_root.clone();
        let mut shells: Vec<PathBuf> = Vec::new();
        if let Ok(rd) = fs::read_dir(&root) {
            for entry in rd.flatten() {
                let p = entry.path();
                if !p.is_dir() {
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_lowercase();
                let is_shell = name == "legacy"
                    || name.contains("_legacy")
                    || name.starts_with("legacy_")
                    || name.starts_with("old_")
                    || name.ends_with("_old")
                    || name.contains("_v0")
                    || name.contains("_v1")
                    || name.contains("_backup");
                if is_shell {
                    shells.push(p);
                }
            }
        }
        // 安全护栏: 过滤白名单/系统根/root 自身
        shells.retain(|s| {
            !self.is_whitelisted(s) && !CleanupPattern::_is_system_root_dir(s) && s != &root
        });
        result.scanned_count = shells.len();
        result.deletable_count = shells.len();
        for s in &shells {
            result.estimated_bytes = result
                .estimated_bytes
                .saturating_add(CleanupPattern::_entry_size(s));
            if result.pattern_matches.len() < 20 {
                result.pattern_matches.push(s.to_string_lossy().to_string());
            }
        }

        // 非 dry-run + 归档模式: 执行蜕皮归档
        if !result.dry_run && !shells.is_empty() && self.archive_on_clean {
            let path_strs: Vec<String> = shells
                .iter()
                .map(|s| s.to_string_lossy().to_string())
                .collect();
            let mut archiver = _Archiver::new(&root);
            match archiver._archive_paths(&path_strs, "ProjectMolting") {
                Ok(m) => {
                    result.estimated_bytes = m.total_bytes;
                    log::info!(
                        "[molting] 归档 {} 个旧躯壳到 .cleanup/archive/{}",
                        m.total_items,
                        m.batch_id
                    );
                }
                Err(e) => result.errors.push(format!("蜕皮归档失败: {}", e)),
            }
        }

        // 记录日志
        let log_dir = root.join(".cleanup").join("log");
        _CleanupLog::log(
            &log_dir,
            &_CleanupLogEntry {
                action: "molt".into(),
                kind: "ProjectMolting".into(),
                items: shells.len(),
                bytes: result.estimated_bytes,
                batch_id: Local::now().format("%Y-%m-%d_%H%M%S").to_string(),
                success: result.errors.is_empty(),
                error: if result.errors.is_empty() {
                    None
                } else {
                    Some(result.errors.join("; "))
                },
            },
        );

        self.history.push(result.clone());
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
        result
    }
}

/// SelfTest — 清理/蜕皮引擎检测能力自检 (T1)。
///
/// 检测以下退化:
///   ① 规则库空 (CleanupPattern 未加载 → 清理静默失效);
///   ② 蜕皮模式缺失 (ProjectMolting 未注册 → 蜕皮能力被删/退化);
///   ③ 白名单空 (is_whitelisted 永假 → 危险: 白名单失效会误删受保护路径);
///   ④ 安全护栏失效 (系统根目录可被蜕皮 → 数据风险)。
#[derive(Default)]
pub struct CleanupEngineSelfTest;

impl crate::l0_substrate::nt_core_self_test::SelfTest for CleanupEngineSelfTest {
    fn name(&self) -> &str {
        "nt_mind_cleanup_engine"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();

        let all = CleanupPattern::all_patterns();
        if all.is_empty() {
            failures.push("CleanupPattern::all_patterns() 为空: 清理规则库退化".into());
        }
        if !all.iter().any(|p| p.kind == CleanupKind::ProjectMolting) {
            failures.push("ProjectMolting 蜕皮规则未注册: 蜕皮能力缺失".into());
        }
        if !all.iter().any(|p| p.kind == CleanupKind::ProjectArtifacts) {
            failures.push("ProjectArtifacts 构建产物规则未注册".into());
        }

        let engine = CleanupEngine::new();
        if engine.whitelist.is_empty() {
            failures.push("白名单为空: is_whitelisted 永假, 受保护路径可被误删".into());
        }
        if !engine.is_whitelisted(
            &dirs::home_dir()
                .unwrap_or_default()
                .join(".config")
                .join("app"),
        ) {
            failures.push("白名单 ~/.config 未匹配绝对路径: expand 失效".into());
        }

        // 安全护栏: 系统根目录必须被蜕皮拒绝
        if !CleanupPattern::_is_system_root_dir(std::path::Path::new("/")) {
            failures.push("系统根目录安全护栏失效 (/)".into());
        }
        if !CleanupPattern::_is_system_root_dir(&dirs::home_dir().unwrap_or_default()) {
            failures.push("系统根目录安全护栏失效 ($HOME)".into());
        }

        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

/// 运行外部命令, 校验 exit status。成功返回 stdout, 失败返回 Err (含 stderr)。
/// (修复: 此前不校验退出码, brew 输出含 "Error" 字样会误判 failed, 命令真失败却误判成功)
fn run_output(cmd: &str, args: &[&str]) -> Result<String, String> {
    match std::process::Command::new(cmd).args(args).output() {
        Ok(o) => {
            if o.status.success() {
                Ok(String::from_utf8_lossy(&o.stdout).to_string())
            } else {
                let stderr = String::from_utf8_lossy(&o.stderr);
                let code = o
                    .status
                    .code()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "?".into());
                Err(format!("{} 退出码 {}: {}", cmd, code, stderr.trim()))
            }
        }
        Err(e) => Err(format!("{} 启动失败: {}", cmd, e)),
    }
}

// ============================================================
// 命令式清理 (_CommandCleanup) — 系统服务级清理执行器
//
// 吸收来源 (GitHub 项目特性):
//   - mac-janitor / GuacSweep: Time Machine 本地快照 (tmutil listlocalsnapshots)
//   - PureMac: Docker prune (docker system prune)
//   - mac-janitor: Homebrew 缓存 (brew cleanup)
// 设计: 每项命令自带 dry-run 前缀 + 风险级 + 平台门控, 复用 CleanupEngine.risk_gate。
// 安全: 默认 dry_run=true; 高风险项 (TM 快照删除) 强制要求确认标志。
// ============================================================

/// 命令式清理项 — 一条可执行的外部清理命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _CommandCleanup {
    pub name: &'static str,
    pub kind: CleanupKind,
    /// 实际执行命令 (argv)
    pub cmd: &'static str,
    pub args: Vec<&'static str>,
    /// dry-run 变体命令 (不产生实际删除)
    pub dry_run_cmd: &'static str,
    pub dry_run_args: Vec<&'static str>,
    /// 是否强制需用户确认 (拒绝无确认执行)
    pub requires_confirm: bool,
    pub platform: Platform,
    pub risk: CleanupRiskLevel,
    pub description: &'static str,
}

impl _CommandCleanup {
    pub fn all() -> Vec<Self> {
        vec![
            // Homebrew 缓存清理 (mac-janitor: brew cleanup) — 低危可自动
            Self {
                name: "Homebrew cleanup",
                kind: CleanupKind::SystemServices,
                cmd: "brew",
                args: vec!["cleanup", "--prune=all"],
                dry_run_cmd: "brew",
                dry_run_args: vec!["cleanup", "--dry-run"],
                requires_confirm: false,
                platform: Platform::MacOS,
                risk: CleanupRiskLevel::Low,
                description: "brew cleanup --prune=all: 清理旧版本与下载缓存",
            },
            // Docker 未使用资源 (PureMac: docker prune) — 中危, 需确认
            Self {
                name: "Docker system prune",
                kind: CleanupKind::SystemServices,
                cmd: "docker",
                args: vec!["system", "prune", "-f"],
                dry_run_cmd: "docker",
                dry_run_args: vec!["system", "df"],
                requires_confirm: true,
                platform: Platform::All,
                risk: CleanupRiskLevel::Medium,
                description: "docker system prune -f: 移除停止容器/悬空镜像/未用网络与构建缓存",
            },
            // Time Machine 本地快照 (mac-janitor/GuacSweep) — 高危, 需确认
            Self {
                name: "Time Machine local snapshots",
                kind: CleanupKind::SystemServices,
                cmd: "tmutil",
                args: vec!["deletelocalsnapshots"],
                dry_run_cmd: "tmutil",
                dry_run_args: vec!["listlocalsnapshots", "/"],
                requires_confirm: true,
                platform: Platform::MacOS,
                risk: CleanupRiskLevel::High,
                description: "删除 Time Machine 本地快照 (需显式快照名, 执行前先列示)",
            },
        ]
    }

    /// 当前平台 + 风险阀过滤后的可用项
    pub(crate) fn _active_on_current(&self, gate: CleanupRiskLevel) -> bool {
        self.platform.matches(Platform::current()) && self.risk <= gate
    }
}

/// 命令式清理执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _CommandResult {
    pub name: String,
    pub status: String, // "skipped" | "dry_run" | "executed" | "failed" | "needs_confirm"
    pub output: String,
    pub dry_run: bool,
    pub error: Option<String>,
}

/// 命令式清理执行器 — 挂载于 CleanupEngine
pub struct _CommandCleaner {
    pub items: Vec<_CommandCleanup>,
    pub dry_run: bool,
    pub risk_gate: CleanupRiskLevel,
}

impl Default for _CommandCleaner {
    fn default() -> Self {
        Self::new()
    }
}

impl _CommandCleaner {
    pub fn new() -> Self {
        Self {
            items: _CommandCleanup::all(),
            dry_run: true,
            risk_gate: CleanupRiskLevel::Medium,
        }
    }

    pub(crate) fn _with_dry_run(mut self, dry: bool) -> Self {
        self.dry_run = dry;
        self
    }
    pub(crate) fn _with_risk_gate(mut self, gate: CleanupRiskLevel) -> Self {
        self.risk_gate = gate;
        self
    }

    /// 扫描可执行项 (平台 + 风险过滤), 返回 (名称, 是否可用)
    pub fn scan(&self) -> Vec<(&_CommandCleanup, bool)> {
        self.items
            .iter()
            .filter(|i| i._active_on_current(self.risk_gate))
            .map(|i| (i, true))
            .collect()
    }

    /// 执行单个清理项。requires_confirm 项在非 dry-run 且无 confirm 时拒绝执行。
    pub fn execute(&self, item: &_CommandCleanup, confirm: bool) -> _CommandResult {
        if !item._active_on_current(self.risk_gate) {
            return _CommandResult {
                name: item.name.into(),
                status: "skipped".into(),
                output: "平台/风险不适用".into(),
                dry_run: self.dry_run,
                error: None,
            };
        }
        if self.dry_run {
            // dry-run 变体: brew cleanup --dry-run / docker system df / tmutil listlocalsnapshots
            let out = run_output(item.dry_run_cmd, &item.dry_run_args).unwrap_or_else(|e| e);
            return _CommandResult {
                name: item.name.into(),
                status: "dry_run".into(),
                output: out,
                dry_run: true,
                error: None,
            };
        }
        if item.requires_confirm && !confirm {
            return _CommandResult {
                name: item.name.into(),
                status: "needs_confirm".into(),
                output: format!("需要显式确认 (--confirm) 才执行: {}", item.description),
                dry_run: false,
                error: None,
            };
        }
        // TM 快照: 先列示再逐个删除
        if item.cmd == "tmutil" {
            let snapshots = self._list_tm_snapshots();
            if snapshots.is_empty() {
                return _CommandResult {
                    name: item.name.into(),
                    status: "executed".into(),
                    output: "无本地快照可删".into(),
                    dry_run: false,
                    error: None,
                };
            }
            let mut log = Vec::new();
            for snap in &snapshots {
                let out = run_output("tmutil", &["deletelocalsnapshots", snap])
                    .unwrap_or_else(|e| format!("ERR: {}", e));
                log.push(format!("{}: {}", snap, out.trim()));
            }
            return _CommandResult {
                name: item.name.into(),
                status: "executed".into(),
                output: log.join("\n"),
                dry_run: false,
                error: None,
            };
        }
        let out = run_output(item.cmd, &item.args).unwrap_or_else(|e| e);
        _CommandResult {
            name: item.name.into(),
            status: if out.contains("ERR") || out.contains("error") {
                "failed"
            } else {
                "executed"
            }
            .into(),
            output: out,
            dry_run: false,
            error: None,
        }
    }

    /// 列示 Time Machine 本地快照
    pub(crate) fn _list_tm_snapshots(&self) -> Vec<String> {
        run_output("tmutil", &["listlocalsnapshots", "/"])
            .ok()
            .map(|o| {
                o.lines()
                    .filter_map(|l| l.trim().strip_prefix("com.apple.TimeMachine."))
                    .map(|s| s.trim().to_string())
                    .collect()
            })
            .unwrap_or_default()
    }
}
