//! nt_mind_guard — 周天星系大阵的守卫层 (NT-SHIELD)
//!
//! 由 shell 脚本 Rust 化而来 (cycle 207 事故教训):
//!   - kb-guard.sh      → KbGuard        (KB 备份/校验/自动恢复)
//!   - workspace-guard.sh → WorkspaceGuard (git 工作区异常检测)
//!   - file-edit-safety.sh → FileEditSafety (编辑前备份 + 完整性校验)
//!
//! 接入点: BackgroundLoop 的 spawn_handler! (每 10min guard / 每 6h backup)。
//! 设计原则:
//!   - 备份存独立目录 (~/Library/Application Support/NeoTrix/backups),
//!     绝不放 ~/.neotrix 内 (该目录可能被并发 session 整体删除)
//!   - 用 rusqlite .backup API 做 WAL 安全一致快照, 不用 cp 主库
//!   - guard 检测 db 缺失/损坏时自动从最近备份恢复
//!   - 文件操作全部原子 (tmp + rename), 无破坏性中间态

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use rusqlite::Connection;

/// 备份根目录 (独立于 ~/.neotrix, 防整目录被删)
fn backup_root() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home)
        .join("Library/Application Support/NeoTrix/backups")
}

/// KB 主库路径
fn kb_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".neotrix/knowledge.db")
}

/// 判定 sqlite 库是否健康: 文件存在 + integrity_check == ok + 关键表存在
/// 快速健康检查: 文件存在 + 非空 + 可打开 + 关键表存在 (跳过 PRAGMA integrity_check)
/// 用于常规守卫检查 (每 10min), 避免 2GB 库跑完整性校验太慢
pub fn db_healthy_fast(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    // 空文件 (<1KB) 不是有效库
    if fs::metadata(path)
        .map(|m| m.len() < 1024)
        .unwrap_or(true)
    {
        return false;
    }
    match Connection::open(path) {
        Ok(conn) => {
            // 校验关键表存在 (空库 integrity_check 也返回 ok, 需要 schema 校验兜底)
            conn.query_row(
                "SELECT name FROM sqlite_master WHERE type='table' AND name='kv_store'",
                [],
                |_| Ok(()),
            )
            .is_ok()
        }
        Err(_) => false,
    }
}

/// 完整健康检查: 快速检查 + PRAGMA integrity_check
/// 用于备份验证、恢复前确认等关键路径
pub fn db_healthy(path: &Path) -> bool {
    if !db_healthy_fast(path) {
        return false;
    }
    // 完整性校验 (慢, 仅用于关键路径)
    match Connection::open(path) {
        Ok(conn) => conn
            .query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
            .map(|s| s == "ok")
            .unwrap_or(false),
        Err(_) => false,
    }
}

/// 用 rusqlite .backup API 做 WAL 安全一致快照
/// 失败时清理残留的 dst (防止 0 字节/部分文件被当成合法备份)
fn snapshot_to(src: &Path, dst: &Path) -> Result<(), String> {
    use rusqlite::{Connection, backup::Backup};
    let cleanup = |e: String| {
        let _ = fs::remove_file(dst);
        let _ = fs::remove_file(PathBuf::from(format!("{}-shm", dst.display())));
        let _ = fs::remove_file(PathBuf::from(format!("{}-wal", dst.display())));
        e
    };
    let conn = Connection::open(src)
        .map_err(|e| cleanup(format!("open src: {e}")))?;
    let mut dst_conn = Connection::open(dst)
        .map_err(|e| cleanup(format!("open dst: {e}")))?;
    let backup = Backup::new(&conn, &mut dst_conn)
        .map_err(|e| cleanup(format!("backup init: {e}")))?;
    // 大页批 + 微让步: run_to_completion(5, 100ms) 对 28 万边库每步只拷 5 页且 sleep 100ms,
    // 实测 150s+ 未完成 (sqlite3 .backup 同库仅 ~14s)。改 500 页/步 + 1ms 让步接近线性。
    backup
        .run_to_completion(500, std::time::Duration::from_millis(1), None)
        .map_err(|e| cleanup(format!("backup run: {e}")))
}

/// 查找最近一份有效备份 (跳过损坏/空文件)
fn find_latest_backup(dir: &Path) -> Option<PathBuf> {
    let mut backups: Vec<PathBuf> = fs::read_dir(dir)
        .ok()?
        .flatten()
        .filter(|e| {
            let s = e.file_name().to_string_lossy().into_owned();
            s.starts_with("knowledge-") && s.ends_with(".db")
        })
        .map(|e| e.path())
        .filter(|p| {
            // 只接受健康备份 — 坏文件 (0字节/无 kv_store 表) 不能用于恢复
            db_healthy_fast(p)
        })
        .collect();
    backups.sort_by_key(|p| {
        fs::metadata(p)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });
    backups.pop()
}

// ═══════════════════════════════════════════════════════════════════
// KbGuard — KB 备份 / 校验 / 自动恢复
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct KbGuardConfig {
    pub keep_n: usize,
}

impl Default for KbGuardConfig {
    fn default() -> Self {
        Self { keep_n: 10 }
    }
}

#[derive(Debug, Default)]
pub struct KbGuardReport {
    pub backed_up: bool,
    pub restored: bool,
    pub backup_path: Option<PathBuf>,
    pub healthy: bool,
}

/// KB 守卫: 备份/校验/恢复
#[derive(Debug, Default)]
pub struct KbGuard {
    config: KbGuardConfig,
}

impl KbGuard {
    pub fn new(config: KbGuardConfig) -> Self {
        Self { config }
    }

    /// 执行 WAL 安全一致快照备份 + 轮转
    pub fn backup(&self) -> Result<PathBuf, String> {
        let dir = backup_root();
        fs::create_dir_all(&dir).map_err(|e| format!("create backup dir: {e}"))?;

        let src = kb_path();
        // fast 健康检查: 完整 integrity_check 对 28 万边库耗时 ~34s (实测), 不适合备份热路径;
        // sqlite .backup API 本身对损坏源会失败, fast schema 校验足够准入。
        if !db_healthy_fast(&src) {
            return Err("source KB not healthy, refusing to backup".into());
        }

        let stamp = chrono_timestamp_stamp();
        let dst = dir.join(format!("knowledge-{stamp}.db"));
        snapshot_to(&src, &dst)?;

        // 校验快照自洽 (fast: 刚由 sqlite .backup 生成的结构完整即可;
        // 完整 integrity_check 会二次全表扫描 28 万边库, 每次数秒拖慢备份)
        if !db_healthy_fast(&dst) {
            let _ = fs::remove_file(&dst);
            return Err("backup snapshot integrity failed, discarded".into());
        }
        // 清理 .backup 校验时生成的 shm/wal 残留
        let _ = fs::remove_file(dst.with_extension("db-shm"));
        let _ = fs::remove_file(dst.with_extension("db-wal"));

        self.rotate(&dir);
        Ok(dst)
    }

    /// 轮转: 只保留最近 keep_n 份 (坏文件直接清理, 不计入额度)
    fn rotate(&self, dir: &Path) {
        let mut backups: Vec<PathBuf> = fs::read_dir(dir)
            .ok()
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                name.starts_with("knowledge-") && name.ends_with(".db")
            })
            .map(|e| e.path())
            .collect();
        // 清理损坏的备份文件 (0字节/无 kv_store 表) — 防它们混入轮转计数与恢复候选
        backups.retain(|p| {
            if db_healthy(p) {
                true
            } else {
                let _ = fs::remove_file(p);
                false
            }
        });
        backups.sort_by_key(|p| {
            fs::metadata(p)
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        });
        while backups.len() > self.config.keep_n {
            if let Some(oldest) = backups.first() {
                let stem = oldest.with_extension("db-shm");
                let _ = fs::remove_file(stem);
                let wal = oldest.with_extension("db-wal");
                let _ = fs::remove_file(wal);
                let _ = fs::remove_file(oldest);
                backups.remove(0);
            }
        }
    }

    /// 从最近备份恢复主库
    pub fn restore_latest(&self) -> Result<(), String> {
        let dir = backup_root();
        let latest = find_latest_backup(&dir)
            .ok_or_else(|| format!("no backup found in {}", dir.display()))?;
        let src = kb_path();
        if let Some(parent) = src.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("create kb dir: {e}"))?;
        }
        fs::copy(&latest, &src).map_err(|e| format!("restore copy: {e}"))?;
        let _ = fs::remove_file(PathBuf::from(format!("{}-wal", src.display())));
        let _ = fs::remove_file(PathBuf::from(format!("{}-shm", src.display())));
        Ok(())
    }

    /// 守卫主流程: 健康则无事, 缺失/损坏则自动恢复
    pub fn guard(&self) -> KbGuardReport {
        let src = kb_path();
        let mut report = KbGuardReport::default();
        if db_healthy_fast(&src) {
            report.healthy = true;
            return report;
        }
        match self.restore_latest() {
            Ok(()) => {
                report.restored = true;
                report.healthy = db_healthy(&src);
            }
            Err(e) => log::warn!("[kb-guard] auto-restore failed: {e}"),
        }
        report
    }
}

fn chrono_timestamp_stamp() -> String {
    // 无 chrono 依赖, 用时间戳 (可读性由文件系统 mtime 保证)
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".into())
}

// ═══════════════════════════════════════════════════════════════════
// MapeGate — 多指标验证门 + burn-in 回滚 (SEA absorb)
// ═══════════════════════════════════════════════════════════════════
// 任何候选变更 (经验/技能/检索调参) 需经多指标验证门 + burn-in (默认 20 次
// 评估) 才能晋升 stable; burn-in 期表现未达阈值 → 回滚 last-good。
// 状态存独立目录 (与备份同域, 绝不放 ~/.neotrix 内), 不依赖 KB。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct MapeGateConfig {
    /// burn-in 评估次数 (SEA: 20)
    pub burn_in_cycles: u32,
    /// 晋升所需最低通过指标数
    pub min_metrics_pass: u32,
    /// 判定失败的最小指标通过数 (低于则提前回滚)
    pub min_metrics_to_continue: u32,
}

impl Default for MapeGateConfig {
    fn default() -> Self {
        Self {
            burn_in_cycles: 20,
            min_metrics_pass: 2,
            min_metrics_to_continue: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricEval {
    pub name: String,
    pub score: f64,
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapeVerdict {
    pub candidate: String,
    pub evaluations: u32,
    pub accepted: bool,
    pub promoted: bool,
    pub rollback: bool,
    pub metrics: Vec<MetricEval>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct BurnInState {
    evaluations: u32,
    pass_count: u32,
    last_metrics: Vec<MetricEval>,
    committed: bool,
}

#[derive(Debug)]
pub struct MapeGate {
    config: MapeGateConfig,
    state_dir: PathBuf,
}

fn mape_state_dir() -> PathBuf {
    backup_root().join("mape")
}

impl MapeGate {
    pub fn new(config: MapeGateConfig) -> Self {
        Self {
            config,
            state_dir: mape_state_dir(),
        }
    }

    /// 测试/可注入状态目录 (隔离真实备份域)
    pub fn with_state_dir(config: MapeGateConfig, state_dir: PathBuf) -> Self {
        Self { config, state_dir }
    }

    fn state_path(&self, candidate: &str) -> PathBuf {
        let safe: String = candidate
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect();
        self.state_dir.join(format!("{safe}.json"))
    }

    fn load(&self, candidate: &str) -> BurnInState {
        let path = self.state_path(candidate);
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    fn save(&self, candidate: &str, state: &BurnInState) {
        let dir = &self.state_dir;
        if let Err(e) = std::fs::create_dir_all(dir) {
            log::warn!("[mape-gate] mkdir {}: {e}", dir.display());
            return;
        }
        if let Ok(json) = serde_json::to_string(state) {
            let tmp = self.state_path(candidate).with_extension("tmp");
            if std::fs::write(&tmp, json).is_ok() {
                let _ = std::fs::rename(tmp, self.state_path(candidate));
            }
        }
    }

    /// 记录一次评估, 累计 burn-in; 达到阈值晋升, 未达回滚。幂等: 同一 candidate
    /// 晋升后不再重复晋升 (committed 标记)。
    pub fn evaluate(&mut self, candidate: &str, metrics: Vec<MetricEval>) -> MapeVerdict {
        let mut state = self.load(candidate);
        if state.committed {
            return MapeVerdict {
                candidate: candidate.to_string(),
                evaluations: state.evaluations,
                accepted: true,
                promoted: false,
                rollback: false,
                metrics: state.last_metrics.clone(),
                note: "already committed".to_string(),
            };
        }
        state.evaluations += 1;
        let pass = metrics.iter().filter(|m| m.passed).count() as u32;
        state.pass_count += pass;
        state.last_metrics = metrics.clone();

        // burn-in 结束: 按累计平均通过指标数裁决 (promote / rollback)
        if state.evaluations >= self.config.burn_in_cycles {
            let avg_pass = state.pass_count as f64 / state.evaluations as f64;
            let last_pass = state.last_metrics.iter().filter(|m| m.passed).count() as u32;
            if avg_pass >= self.config.min_metrics_pass as f64 && last_pass >= self.config.min_metrics_pass {
                state.committed = true;
                self.save(candidate, &state);
                return MapeVerdict {
                    candidate: candidate.to_string(),
                    evaluations: state.evaluations,
                    accepted: true,
                    promoted: true,
                    rollback: false,
                    metrics,
                    note: format!("burn-in complete, avg_pass={:.2} → promoted", avg_pass),
                };
            }
            let verdict = MapeVerdict {
                candidate: candidate.to_string(),
                evaluations: state.evaluations,
                accepted: false,
                promoted: false,
                rollback: true,
                metrics,
                note: format!("burn-in complete, avg_pass={:.2} below threshold → rollback", avg_pass),
            };
            let _ = std::fs::remove_file(self.state_path(candidate));
            return verdict;
        }

        // burn-in 早期提前回滚: 连续从未通过任何指标且已采样 ≥3 次 → 不值得继续
        if state.pass_count == 0 && state.evaluations >= 3 {
            let verdict = MapeVerdict {
                candidate: candidate.to_string(),
                evaluations: state.evaluations,
                accepted: false,
                promoted: false,
                rollback: true,
                metrics,
                note: "burn-in early failure (never passed any metric)".to_string(),
            };
            let _ = std::fs::remove_file(self.state_path(candidate));
            return verdict;
        }

        self.save(candidate, &state);
        MapeVerdict {
            candidate: candidate.to_string(),
            evaluations: state.evaluations,
            accepted: true,
            promoted: false,
            rollback: false,
            metrics,
            note: format!("burn-in {}/{}", state.evaluations, self.config.burn_in_cycles),
        }
    }

    /// 显式回滚: 外部 (如目标失效) 主动回滚候选并清除状态
    pub fn rollback(&self, candidate: &str) -> Result<(), String> {
        std::fs::remove_file(self.state_path(candidate))
            .map_err(|e| format!("rollback state: {e}"))
    }
}

// ═══════════════════════════════════════════════════════════════════
// WorkspaceGuard — git 工作区异常检测
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct WorkspaceGuardConfig {
    pub repo_root: PathBuf,
    pub snapshot_dir: PathBuf,
    pub max_snapshots: usize,
}

#[derive(Debug, Default)]
pub struct WorkspaceGuardReport {
    pub staged_lost: bool,
    pub modified_reverted: bool,
    pub prev_staged: usize,
    pub curr_staged: usize,
    pub prev_modified: usize,
    pub curr_modified: usize,
}

/// 工作区守卫: 检测 git status 未预期清空 (R-P53 并发 reset 盲区)
#[derive(Debug)]
pub struct WorkspaceGuard {
    config: WorkspaceGuardConfig,
}

impl WorkspaceGuard {
    pub fn new(config: WorkspaceGuardConfig) -> Self {
        Self { config }
    }

    pub fn default_for(repo_root: PathBuf) -> Self {
        Self {
            config: WorkspaceGuardConfig {
                repo_root,
                snapshot_dir: PathBuf::from("/tmp/neotrix-ws-guard"),
                max_snapshots: 10,
            },
        }
    }

    fn git_status_short(&self) -> Result<String, String> {
        let out = Command::new("git")
            .args(["-C", self.config.repo_root.to_str().unwrap_or("."), "status", "--short"])
            .output()
            .map_err(|e| format!("git status failed: {e}"))?;
        if !out.status.success() {
            return Err(String::from_utf8_lossy(&out.stderr).to_string());
        }
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    }

    fn count_staged(s: &str) -> usize {
        s.lines().filter(|l| {
            let b = l.as_bytes();
            b.len() >= 2 && b[0].is_ascii_uppercase()
        }).count()
    }

    fn count_modified(s: &str) -> usize {
        s.lines().filter(|l| l.starts_with(" M ")).count()
    }

    /// 对比前后快照, 检测 staged 文件消失 / modified 被还原
    pub fn check(&mut self) -> WorkspaceGuardReport {
        let mut report = WorkspaceGuardReport::default();
        let curr = match self.git_status_short() {
            Ok(s) => s,
            Err(_) => return report,
        };
        fs::create_dir_all(&self.config.snapshot_dir).ok();

        let mut snapshots: Vec<PathBuf> = fs::read_dir(&self.config.snapshot_dir)
            .ok()
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .collect();
        snapshots.sort();
        let prev = snapshots.last().cloned();

        if let Some(prev_path) = prev {
            if let Ok(prev) = fs::read_to_string(&prev_path) {
                let prev_staged = Self::count_staged(&prev);
                let curr_staged = Self::count_staged(&curr);
                let prev_modified = Self::count_modified(&prev);
                let curr_modified = Self::count_modified(&curr);
                report.prev_staged = prev_staged;
                report.curr_staged = curr_staged;
                report.prev_modified = prev_modified;
                report.curr_modified = curr_modified;
                if prev_staged > 0 && curr_staged == 0 {
                    report.staged_lost = true;
                }
                if prev_modified > 0 && curr_modified == 0 {
                    report.modified_reverted = true;
                }
            }
        }

        // 写本次快照
        let stamp = chrono_timestamp_stamp();
        let snap = self.config.snapshot_dir.join(format!("snap-{stamp}.txt"));
        fs::write(&snap, curr).ok();

        // 只保留最近 max_snapshots 份
        let mut all: Vec<PathBuf> = fs::read_dir(&self.config.snapshot_dir)
            .ok()
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .collect();
        all.sort();
        while all.len() > self.config.max_snapshots {
            if let Some(oldest) = all.first() {
                let _ = fs::remove_file(oldest);
                all.remove(0);
            }
        }
        report
    }
}

// ═══════════════════════════════════════════════════════════════════
// FileEditSafety — 编辑前备份 + 完整性校验
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug, Default)]
pub struct FileEditSafetyReport {
    pub backed_up: bool,
    pub backup_path: Option<PathBuf>,
    pub verified: bool,
}

/// 文件编辑安全: 编辑前备份原文件, 编辑后校验行数
#[derive(Debug, Default)]
pub struct FileEditSafety;

impl FileEditSafety {
    /// 编辑前备份 (原子写)
    pub fn protect_file(&self, file: &Path) -> Result<FileEditSafetyReport, String> {
        if !file.is_file() {
            return Ok(FileEditSafetyReport::default());
        }
        let dir = PathBuf::from(std::env::var("NEOTRIX_SAFETY_BACKUP_DIR")
            .unwrap_or_else(|_| "/tmp/neotrix-edit-backups".into()));
        fs::create_dir_all(&dir).map_err(|e| format!("create backup dir: {e}"))?;
        let safe_name = file.to_string_lossy().replace('/', "_");
        let dst = dir.join(format!("{safe_name}.bak"));
        fs::copy(file, &dst).map_err(|e| format!("backup copy: {e}"))?;
        Ok(FileEditSafetyReport {
            backed_up: true,
            backup_path: Some(dst),
            verified: true,
        })
    }

    /// 编辑后校验: 文件存在且行数 >= 期望
    pub fn verify_file(&self, file: &Path, min_lines: usize) -> Result<FileEditSafetyReport, String> {
        if !file.is_file() {
            return Err(format!("file {} no longer exists", file.display()));
        }
        let content = fs::read_to_string(file).map_err(|e| format!("read {file:?}: {e}"))?;
        let lines = content.lines().count();
        if lines < min_lines {
            return Err(format!(
                "{} truncated: expected >= {min_lines} lines, got {lines}",
                file.display()
            ));
        }
        Ok(FileEditSafetyReport {
            verified: true,
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // 这些测试都通过全局 HOME env 定位备份目录, 并行运行会互相踩踏。
    // 用一个模块级互斥锁串行化 (零依赖, 无需 serial_test crate)。
    static HOME_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_db_healthy_detects_missing() {
        assert!(!db_healthy(Path::new("/nonexistent/kb.db")));
    }

    #[test]
    fn test_kb_guard_restore_with_no_backup() {
        let _guard = HOME_TEST_LOCK.lock().unwrap();
        // 用唯一临时 HOME 隔离 (防与其他测试共享目录产生残留备份)
        let test_home = std::env::temp_dir().join(format!(
            "neotrix-kbguard-nobackup-{}",
            std::process::id()
        ));
        std::env::set_var("HOME", &test_home);
        let guard = KbGuard::new(KbGuardConfig { keep_n: 3 });
        let report = guard.guard();
        // 无备份可恢复, 但不应 panic
        assert!(!report.healthy);
        let _ = fs::remove_dir_all(&test_home);
    }

    #[test]
    fn test_kb_guard_backup_restore_full_cycle() {
        // 真实 sqlite 库 → backup → 删库 → guard 自动恢复 全链路
        let _guard = HOME_TEST_LOCK.lock().unwrap();
        let test_home = std::env::temp_dir().join(format!("neotrix-kbguard-{}", std::process::id()));
        let kb = test_home.join(".neotrix/knowledge.db");
        fs::create_dir_all(kb.parent().unwrap()).unwrap();

        // 建一个真实 sqlite 库并写入数据
        {
            use rusqlite::Connection;
            let conn = Connection::open(&kb).unwrap();
            conn.execute_batch(
                "CREATE TABLE kv_store (namespace TEXT, key TEXT, value TEXT, PRIMARY KEY(namespace,key));
                 INSERT INTO kv_store VALUES ('experience','hub','{\"cycles\":{\"207\":{\"count\":6}}}');",
            )
            .unwrap();
        }
        assert!(db_healthy(&kb));

        std::env::set_var("HOME", &test_home);
        let guard = KbGuard::new(KbGuardConfig { keep_n: 3 });

        // 备份
        let backup_path = guard.backup().expect("backup should succeed");
        assert!(backup_path.exists());
        assert!(db_healthy(&backup_path));
        // 快照里应含写入的数据
        {
            use rusqlite::Connection;
            let conn = Connection::open(&backup_path).unwrap();
            let n: i64 = conn
                .query_row("SELECT COUNT(*) FROM kv_store WHERE key='hub'", [], |r| r.get(0))
                .unwrap();
            assert_eq!(n, 1, "backup snapshot must contain data");
        }

        // 模拟库被删
        fs::remove_file(&kb).unwrap();
        assert!(!db_healthy(&kb));

        // guard 自动恢复
        let report = guard.guard();
        assert!(report.restored, "guard should restore");
        assert!(report.healthy, "restored db should be healthy");
        assert!(db_healthy(&kb));

        // 清理
        let _ = fs::remove_dir_all(&test_home);
    }

    #[test]
    fn test_corrupt_backup_rejected_and_cleaned() {
        // 坏备份 (0字节/无 kv_store 表) 不得被选中恢复, 且 rotate 会清理
        let _guard = HOME_TEST_LOCK.lock().unwrap();
        let test_home = std::env::temp_dir().join(format!("neotrix-kbguard-bad-{}", std::process::id()));
        let bdir = test_home.join("Library/Application Support/NeoTrix/backups");
        fs::create_dir_all(&bdir).unwrap();

        // 造一个 0 字节坏文件 + 一个无表坏文件
        let bad_empty = bdir.join("knowledge-bad-empty.db");
        fs::write(&bad_empty, b"").unwrap();
        let bad_noschema = bdir.join("knowledge-bad-noschema.db");
        {
            use rusqlite::Connection;
            let conn = Connection::open(&bad_noschema).unwrap();
            conn.execute_batch("CREATE TABLE foo (x);").unwrap();
        }
        // 造一个真健康备份
        let good = bdir.join("knowledge-good.db");
        {
            use rusqlite::Connection;
            let conn = Connection::open(&good).unwrap();
            conn.execute_batch(
                "CREATE TABLE kv_store (namespace TEXT, key TEXT, value TEXT, PRIMARY KEY(namespace,key));
                 INSERT INTO kv_store VALUES ('experience','hub','{}');",
            )
            .unwrap();
        }
        // 确保 good 是 mtime 最新 (find_latest_backup 按 mtime 取最新)
        let now = std::time::SystemTime::now();
        let _ = filetime::set_file_mtime(&bad_empty, now.into());
        let _ = filetime::set_file_mtime(&bad_noschema, now.into());
        let _ = filetime::set_file_mtime(&good, now.into());

        assert!(!db_healthy(&bad_empty), "0-byte file must be unhealthy");
        assert!(!db_healthy(&bad_noschema), "no kv_store table must be unhealthy");
        assert!(db_healthy(&good));

        let latest = find_latest_backup(&bdir).expect("should find good backup");
        assert_eq!(latest, good, "corrupt files must never be selected for restore");

        // rotate 应清理坏文件
        std::env::set_var("HOME", &test_home);
        let guard = KbGuard::new(KbGuardConfig { keep_n: 10 });
        guard.rotate(&bdir);
        assert!(!bad_empty.exists(), "0-byte corrupt backup must be cleaned");
        assert!(!bad_noschema.exists(), "noschema corrupt backup must be cleaned");
        assert!(good.exists(), "good backup must survive rotation");

        let _ = fs::remove_dir_all(&test_home);
    }

    #[test]
    fn test_workspace_guard_reports_clean() {
        let mut guard = WorkspaceGuard::default_for(PathBuf::from("/nonexistent-repo"));
        let report = guard.check();
        // git 失败时不误报
        assert!(!report.staged_lost);
        assert!(!report.modified_reverted);
    }

    #[test]
    fn test_file_edit_safety_verify_truncated() {
        let tmp = std::env::temp_dir().join("neotrix-edit-safety-test.txt");
        fs::write(&tmp, "line1\nline2\n").unwrap();
        let safety = FileEditSafety;
        let r = safety.verify_file(&tmp, 5);
        assert!(r.is_err());
        let r2 = safety.verify_file(&tmp, 1);
        assert!(r2.is_ok());
        let _ = fs::remove_file(&tmp);
    }

    #[test]
    fn test_mape_gate_burn_in_promotes_good_candidate() {
        let mut gate = MapeGate::with_state_dir(
            MapeGateConfig {
                burn_in_cycles: 3,
                min_metrics_pass: 2,
                min_metrics_to_continue: 1,
            },
            temp_mape_dir(),
        );
        for _ in 0..2 {
            let v = gate.evaluate("exp-alpha", vec![
                MetricEval { name: "confidence".into(), score: 0.8, passed: true },
                MetricEval { name: "feedback".into(), score: 0.9, passed: true },
            ]);
            assert!(!v.rollback, "burn-in 期不应回滚");
        }
        let final_v = gate.evaluate("exp-alpha", vec![
            MetricEval { name: "confidence".into(), score: 0.8, passed: true },
            MetricEval { name: "feedback".into(), score: 0.9, passed: true },
        ]);
        assert!(final_v.promoted, "burn-in 满且指标通过应晋升");
        let again = gate.evaluate("exp-alpha", vec![]);
        assert!(!again.promoted);
        assert!(again.accepted);
    }

    #[test]
    fn test_mape_gate_early_rollback_on_persistent_failure() {
        let mut gate = MapeGate::with_state_dir(
            MapeGateConfig {
                burn_in_cycles: 10,
                min_metrics_pass: 2,
                min_metrics_to_continue: 1,
            },
            temp_mape_dir(),
        );
        let v1 = gate.evaluate("exp-bad", vec![
            MetricEval { name: "confidence".into(), score: 0.1, passed: false },
        ]);
        assert!(!v1.rollback, "前两次采样不应回滚");
        let v2 = gate.evaluate("exp-bad", vec![
            MetricEval { name: "confidence".into(), score: 0.1, passed: false },
        ]);
        assert!(!v2.rollback);
        let v3 = gate.evaluate("exp-bad", vec![
            MetricEval { name: "confidence".into(), score: 0.1, passed: false },
        ]);
        assert!(v3.rollback, "连续 3 次全失败应提前回滚");
    }

    #[test]
    fn test_mape_gate_rollback_on_burn_in_underperformance() {
        let mut gate = MapeGate::with_state_dir(
            MapeGateConfig {
                burn_in_cycles: 3,
                min_metrics_pass: 2,
                min_metrics_to_continue: 1,
            },
            temp_mape_dir(),
        );
        for _ in 0..2 {
            let _ = gate.evaluate("exp-mid", vec![
                MetricEval { name: "confidence".into(), score: 0.6, passed: true },
                MetricEval { name: "feedback".into(), score: 0.2, passed: false },
            ]);
        }
        let final_v = gate.evaluate("exp-mid", vec![
            MetricEval { name: "confidence".into(), score: 0.6, passed: true },
            MetricEval { name: "feedback".into(), score: 0.2, passed: false },
        ]);
        assert!(final_v.rollback, "avg_pass=1 < min_metrics_pass=2 应回滚");
    }

    #[test]
    fn test_mape_gate_state_isolated_per_candidate() {
        let mut gate = MapeGate::with_state_dir(
            MapeGateConfig {
                burn_in_cycles: 2,
                min_metrics_pass: 1,
                min_metrics_to_continue: 1,
            },
            temp_mape_dir(),
        );
        for _ in 0..2 {
            let _ = gate.evaluate("exp-a", vec![MetricEval { name: "c".into(), score: 1.0, passed: true }]);
        }
        let vb = gate.evaluate("exp-b", vec![MetricEval { name: "c".into(), score: 0.0, passed: false }]);
        assert!(!vb.promoted);
    }
}

#[allow(dead_code)]
fn temp_mape_dir() -> PathBuf {
    std::env::temp_dir().join(format!(
        "neotrix-mape-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("now is after UNIX_EPOCH")
            .as_nanos()
    ))
}
