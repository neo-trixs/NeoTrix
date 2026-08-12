//! NT-ACT 磁盘沙盒 — 每任务磁盘 allowlist 越界检查 (缺陷网 D18 修复)。
//!
//! 参照: Opptrix agent-workspace (文件/Shell/HTTP/浏览器/Python/密钥保险箱) +
//!       codex-build (disk allowlist 越界检查 traveled/staged/deleted 全检)。
//! 机制: 每个任务分配工作目录 allowlist, 任何写/读/删除若越出 allowlist → 拦截。

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// 越界检查结果。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DiskVerdict {
    /// 在 allowlist 内, 允许
    Allowed,
    /// 越出 allowlist, 拦截
    Blocked(String),
}

/// P1-19 三級风险分类 (吸收 macos-disk-cleanup SAFE/CONFIRM/DANGER 模式):
/// 破坏性操作按目标类型分级 — 而非一刀切放行/拦截。
/// macos-disk-cleanup 原文: "three-level safety classification
/// (SAFE / CONFIRM / DANGER) for destructive operations"。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// 安全: 直接放行 (临时文件/缓存/日志)
    Safe,
    /// 需确认: 用户确认后放行 (用户数据/配置文件)
    Confirm,
    /// 危险: 拦截 (系统目录/根路径/源码树)
    Danger,
}

impl RiskLevel {
    /// 按目标路径特征判定风险级 (确定性启发式)。
    pub fn classify(target: &Path) -> RiskLevel {
        let s = target.to_string_lossy().to_lowercase();
        // Danger: 系统/根/源码树关键路径
        if s == "/" || s.starts_with("/etc") || s.starts_with("/usr") || s.starts_with("/bin")
            || s.starts_with("/sbin") || s.starts_with("/System") || s.starts_with("/Library")
            || s.starts_with("/Applications") || s.contains("/.git/") || s.contains("/node_modules/")
        {
            return RiskLevel::Danger;
        }
        // Safe: 明确临时/缓存/日志
        if s.contains("/tmp/") || s.contains("/cache/") || s.contains("/logs/")
            || s.contains(".log") || s.contains(".tmp") || s.contains(".cache")
        {
            return RiskLevel::Safe;
        }
        // 其余: 用户数据 → 需确认
        RiskLevel::Confirm
    }
}

/// 磁盘守卫 — 维护任务的工作目录 allowlist。
#[derive(Debug, Clone, Default)]
pub struct DiskGuard {
    allowlist: Vec<PathBuf>,
    /// 已放行操作数 (telemetry)
    pub allowed_count: u64,
    /// 已拦截越界数
    pub blocked_count: u64,
}

impl DiskGuard {
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册一个允许访问的根目录 (任务工作区)。
    pub fn allow(&mut self, dir: &Path) {
        self.allowlist.push(dir.to_path_buf());
    }

    /// 目标路径是否在任一 allowlist 根目录之下。
    pub fn is_within(&self, target: &Path) -> bool {
        self.allowlist.iter().any(|root| target.starts_with(root))
    }

    /// 检查一次写/读/删操作。operation ∈ {"write","read","delete"}。
    pub fn check(&mut self, operation: &str, target: &Path) -> DiskVerdict {
        if self.is_within(target) {
            self.allowed_count += 1;
            DiskVerdict::Allowed
        } else {
            self.blocked_count += 1;
            DiskVerdict::Blocked(format!(
                "{} 越界: {} 不在任务 allowlist",
                operation,
                target.display()
            ))
        }
    }

    /// 批量检查 (traveled/staged/deleted 全检, codex-build 参照)。
    /// 返回被拦截的路径列表 (空 = 全放行)。
    pub fn check_many(&mut self, operation: &str, targets: &[PathBuf]) -> Vec<PathBuf> {
        targets
            .iter()
            .filter(|t| !matches!(self.check(operation, t), DiskVerdict::Allowed))
            .cloned()
            .collect()
    }

    /// P1-19 三級风险检查 (macos-disk-cleanup 模式):
    /// 越界 → Blocked; 界内按 RiskLevel 分级:
    /// - Safe → Allowed (直接放行)
    /// - Confirm → 需确认 (返回 Blocked 带 confirm 标记, 调用方决定是否二次确认)
    /// - Danger → Blocked (拦截)
    /// 返回 (verdict, risk_level)。
    pub fn check_risk(&mut self, operation: &str, target: &Path) -> (DiskVerdict, RiskLevel) {
        if !self.is_within(target) {
            self.blocked_count += 1;
            return (
                DiskVerdict::Blocked(format!("{} 越界: {} 不在任务 allowlist", operation, target.display())),
                RiskLevel::Danger,
            );
        }
        let risk = RiskLevel::classify(target);
        match risk {
            RiskLevel::Safe => {
                self.allowed_count += 1;
                (DiskVerdict::Allowed, risk)
            }
            RiskLevel::Confirm => {
                // 界内但需确认: 不计数为 blocked (留给调用方二次确认), 返回 confirm 语义
                (DiskVerdict::Blocked(format!("{} 需确认: {}", operation, target.display())), risk)
            }
            RiskLevel::Danger => {
                self.blocked_count += 1;
                (DiskVerdict::Blocked(format!("{} 危险操作拦截: {}", operation, target.display())), risk)
            }
        }
    }

    pub fn allowlist(&self) -> &[PathBuf] {
        &self.allowlist
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn guard_ws() -> DiskGuard {
        let mut g = DiskGuard::new();
        g.allow(Path::new("/tmp/ws"));
        g
    }

    #[test]
    fn within_allowlist_allowed() {
        let mut g = guard_ws();
        assert_eq!(g.check("write", Path::new("/tmp/ws/file.txt")), DiskVerdict::Allowed);
        assert_eq!(g.check("read", Path::new("/tmp/ws/sub/note.md")), DiskVerdict::Allowed);
        assert_eq!(g.allowed_count, 2);
    }

    #[test]
    fn outside_allowlist_blocked() {
        let mut g = guard_ws();
        assert!(matches!(
            g.check("write", Path::new("/etc/passwd")),
            DiskVerdict::Blocked(_)
        ));
        assert!(matches!(
            g.check("delete", Path::new("/home/user/secret")),
            DiskVerdict::Blocked(_)
        ));
        assert_eq!(g.blocked_count, 2);
    }

    #[test]
    fn check_many_returns_blocked_paths() {
        let mut g = guard_ws();
        let blocked = g.check_many(
            "write",
            &[PathBuf::from("/tmp/ws/ok.rs"), PathBuf::from("/etc/hosts"), PathBuf::from("/tmp/ws/lib.rs")],
        );
        assert_eq!(blocked.len(), 1);
        assert_eq!(blocked[0], PathBuf::from("/etc/hosts"));
    }

    #[test]
    fn siblings_not_allowed() {
        let mut g = guard_ws();
        // /tmp/ws2 是 /tmp/ws 的兄弟, 不应放行
        assert!(matches!(
            g.check("write", Path::new("/tmp/ws2/pwn")),
            DiskVerdict::Blocked(_)
        ));
    }
}
