//! Safety — 删除前安全判定 (吸收 PureMac CLI Safety + CleaningEngine 校验)
//!
//! 域: NT-ACT · 层: L1 Action
//! 顺序: 根路径 → 符号链接分量 → 关键根 → 系统拒绝树 → 凭据 → cloud → 排除 → allow-list

use std::path::{Path, PathBuf};

use super::exclusions::excludes;
use super::locations::{
    credential_allow_exceptions, denied_credential_roots, is_provider_owned, is_safe_to_delete,
    normalize,
};

/// 判定结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SafetyVerdict {
    Ok,
    Reject(&'static str),
}

impl SafetyVerdict {
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok)
    }
    pub fn reason(&self) -> Option<&'static str> {
        match self {
            Self::Ok => None,
            Self::Reject(r) => Some(r),
        }
    }
}

/// 纯词法路径清洗（不碰文件系统、不解析符号链接）：压掉 `.`/`..`/多余 `/`。
/// 与 `locations::normalize`（= canonicalize，会解 symlink）互补：
/// 安全判定里凡是"走原始路径看有没有 symlink 分量"必须用它，否则解析后永远看不见。
pub fn lexical_clean(path: &Path) -> PathBuf {
    use std::path::Component;
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() {
                    out.push("..");
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    if out.as_os_str().is_empty() {
        out.push("/");
    }
    out
}

/// 路径上是否存在符号链接分量 (排除 /tmp 与 /var 的历史别名)
///
/// 注意：走词法路径逐分量 `symlink_metadata`，不先 canonicalize
/// （解析后 symlink 分量已消失，会漏检真攻击）。
pub fn has_symlink_component(path: &Path) -> bool {
    let mut cur = lexical_clean(path);
    if !path.is_absolute() {
        return false;
    }
    while cur != Path::new("/") {
        if let Ok(meta) = std::fs::symlink_metadata(&cur) {
            if meta.file_type().is_symlink() && cur != Path::new("/tmp") && cur != Path::new("/var")
            {
                return true;
            }
        }
        match cur.parent() {
            Some(p) if p != cur => cur = p.to_path_buf(),
            _ => break,
        }
    }
    false
}

fn starts_with_component(path: &Path, root: &str) -> bool {
    path == Path::new(root) || path.starts_with(root)
}

/// 关键根 (不可整树删)
pub fn critical_roots(home: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = [
        "/",
        "/System",
        "/Library",
        "/Applications",
        "/Users",
        "/usr",
        "/bin",
        "/sbin",
        "/etc",
        "/var",
        "/private",
        "/private/etc",
        "/cores",
    ]
    .iter()
    .map(PathBuf::from)
    .collect();
    v.push(home.to_path_buf());
    v.push(home.join("Library"));
    v.push(home.join("Documents"));
    v.push(home.join("Desktop"));
    v.push(home.join("Downloads"));
    v.push(home.join(".config"));
    v.push(home.join(".cache"));
    v
}

fn is_credential_root(path: &Path, home: &Path) -> bool {
    for allow in credential_allow_exceptions(home) {
        if path == allow || path.starts_with(&allow) {
            return false;
        }
    }
    denied_credential_roots(home)
        .iter()
        .any(|root| path == root || path.starts_with(root))
}

/// 主判定：允许删除则 Ok，否则 Reject(reason)
///
/// `use_allowlist=true` 时还要求路径落在 CleaningEngine.allow-list 内
/// (默认 CLI 场景为 true；测试可关)。
pub fn can_remove(
    path: &Path,
    home: &Path,
    excluded: &[PathBuf],
    use_allowlist: bool,
) -> SafetyVerdict {
    if !path.is_absolute() {
        return SafetyVerdict::Reject("not absolute");
    }
    let std = normalize(path);
    if std == Path::new("/") {
        return SafetyVerdict::Reject("root path");
    }
    // home 也解析一次：TempDir / CI 等场景 home 自带 symlink（/var→/private/var），
    // resolved/canonical 侧比较必须用同一坐标系，否则 allow-list 恒拒、protected-root 恒漏。
    // 真实 $HOME 无 symlink 时 home_std == home，行为零变化。
    let home_std = normalize(home);
    if has_symlink_component(path) || has_symlink_component(&std) {
        return SafetyVerdict::Reject("symlink");
    }
    let resolved = path.canonicalize().unwrap_or_else(|_| std.clone());
    if resolved == Path::new("/") {
        return SafetyVerdict::Reject("root path");
    }

    for c in critical_roots(&home_std) {
        if std == c || resolved == c {
            return SafetyVerdict::Reject("protected root");
        }
    }

    let denied_trees = [
        "/System",
        "/Library",
        "/Applications",
        "/usr",
        "/bin",
        "/sbin",
        "/etc",
        "/private/etc",
        "/cores",
    ];
    for t in denied_trees {
        // /Library/Caches /Library/Logs 与 /private/tmp 等在 allow-list 中仍可过，
        // 但整树 /System… 等一律拒。
        if starts_with_component(&resolved, t) {
            // 允许的系统缓存子树 (PureMac allowedRoots 交集)
            let allowed_sys = [
                "/Library/Caches",
                "/Library/Logs",
                "/opt/homebrew/Library/Caches",
                "/usr/local/Homebrew/Library/Caches",
                "/private/var/log",
                "/private/var/tmp",
                "/private/tmp",
                "/var/log",
                "/var/tmp",
                "/tmp",
            ];
            if !allowed_sys
                .iter()
                .any(|a| resolved == Path::new(a) || resolved.starts_with(a))
            {
                return SafetyVerdict::Reject("protected system path");
            }
        }
    }

    // 不删其他用户文件
    if let Ok(users) = std::fs::canonicalize("/Users") {
        if resolved.starts_with(&users) && !resolved.starts_with(&home_std) {
            return SafetyVerdict::Reject("another user's files");
        }
    }

    if is_credential_root(&std, &home_std) || is_credential_root(&resolved, &home_std) {
        return SafetyVerdict::Reject("config/credentials dir");
    }
    if is_provider_owned(&std, &home_std) || is_provider_owned(&resolved, &home_std) {
        return SafetyVerdict::Reject("cloud/provider state");
    }
    if excludes(path, excluded) || excludes(&std, excluded) {
        return SafetyVerdict::Reject("excluded");
    }
    if !path.exists() && path.symlink_metadata().is_err() {
        return SafetyVerdict::Reject("missing");
    }
    if use_allowlist && !is_safe_to_delete(&resolved, &home_std) {
        return SafetyVerdict::Reject("outside allow-list");
    }
    SafetyVerdict::Ok
}

/// TOCTOU 复查：解析结果与首次一致且仍无意外符号链接
pub fn revalidate(path: &Path, first_resolved: &Path) -> SafetyVerdict {
    if has_symlink_component(path) {
        return SafetyVerdict::Reject("became a symlink");
    }
    let again = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    if again != first_resolved {
        return SafetyVerdict::Reject("resolution changed");
    }
    SafetyVerdict::Ok
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_reject_root_and_relative() {
        let home = Path::new("/Users/neo");
        assert!(!can_remove(Path::new("/"), home, &[], true).is_ok());
        assert!(!can_remove(Path::new("relative"), home, &[], true).is_ok());
    }

    #[test]
    fn test_reject_credentials() {
        let home = Path::new("/Users/neo");
        let v = can_remove(Path::new("/Users/neo/.ssh/id_rsa"), home, &[], false);
        assert!(!v.is_ok());
        // allow-list 外的 Documents 也拒
        let v2 = can_remove(Path::new("/Users/neo/Documents/notes.md"), home, &[], true);
        assert!(!v2.is_ok());
    }

    #[test]
    fn test_allow_caches_when_allowlist() {
        let dir = TempDir::new().unwrap();
        let home = dir.path().to_path_buf();
        let cache = home.join("Library/Caches/app");
        std::fs::create_dir_all(&cache).unwrap();
        let v = can_remove(&cache, &home, &[], true);
        assert!(v.is_ok(), "verdict={:?}", v);
    }

    #[test]
    fn test_symlink_component_detected() {
        let dir = TempDir::new().unwrap();
        let target = dir.path().join("real");
        std::fs::create_dir(&target).unwrap();
        let link = dir.path().join("link");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        assert!(has_symlink_component(&link));
        assert!(has_symlink_component(&link.join("x")));
    }

    #[test]
    fn test_exclusion_rejects() {
        let dir = TempDir::new().unwrap();
        let home = dir.path().to_path_buf();
        let keep = home.join("Library/Caches/Keep");
        std::fs::create_dir_all(&keep).unwrap();
        let v = can_remove(&keep, &home, &[keep.clone()], true);
        assert!(!v.is_ok());
        assert_eq!(v.reason(), Some("excluded"));
    }
}
