//! Locations — 允许删除的路径根 (吸收 PureMac CleaningEngine.isSafeToDelete)
//!
//! 域: NT-ACT · 层: L1 Action
//! 只有落在 `allowed_roots` 下的路径才允许 unlink；其余一律拒绝。

use std::path::{Path, PathBuf};

/// 云 File Provider 硬拒绝根 (PureMac ProviderPaths.deniedRoots)
/// 这些目录是 bird/cloudd/fileproviderd 的活数据库，删了会破坏同步状态。
pub fn denied_provider_roots(home: &Path) -> Vec<PathBuf> {
    [
        "Library/Mobile Documents",
        "Library/CloudStorage",
        "Library/Application Support/FileProvider",
        "Library/Application Support/CloudDocs",
        "Library/Daemon Containers",
        "Library/Caches/CloudKit",
        "Library/Caches/com.apple.bird",
        "Library/Caches/com.apple.cloudkit",
        "Library/Caches/com.apple.cloudd",
        "Library/Caches/com.apple.FileProvider",
    ]
    .iter()
    .map(|rel| home.join(rel))
    .collect()
}

/// 凭据/配置目录 (CLI Safety.credential roots) — 除白名单窄根外永不删
pub fn denied_credential_roots(home: &Path) -> Vec<PathBuf> {
    [
        ".ssh",
        ".aws",
        ".gnupg",
        ".gpg",
        ".kube",
        ".docker",
        ".claude",
        ".config",
        ".cargo",
        ".rustup",
        ".gem",
        ".nvm",
        ".pyenv",
        ".rbenv",
        ".ollama",
        ".lmstudio",
    ]
    .iter()
    .map(|rel| home.join(rel))
    .collect()
}

/// 凭据根内的窄白名单 (允许被归入对应清理类别的缓存子树)
pub fn credential_allow_exceptions(home: &Path) -> Vec<PathBuf> {
    [
        ".docker/cli-plugins/.cache",
        ".docker/buildx/cache",
        ".cargo/registry/cache",
        ".cargo/registry/index",
        ".cargo/git/db",
        ".cargo/git/checkouts",
        ".ollama/logs",
        ".lmstudio/server-logs",
    ]
    .iter()
    .map(|rel| home.join(rel))
    .collect()
}

/// 允许整树删除的根 (PureMac CleaningEngine.allowedRoots)
pub fn allowed_roots(home: &Path) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = [
        "Library/Caches",
        "Library/Logs",
        "Library/Saved Application State",
        "Library/HTTPStorages",
        "Library/WebKit",
        "Library/Containers",
        "Library/Group Containers",
        "Library/Application Support",
        "Library/Preferences",
        "Library/LaunchAgents",
        "Library/Mail Downloads",
        "Library/Developer/Xcode/DerivedData",
        "Library/Developer/Xcode/Archives",
        "Library/Developer/CoreSimulator/Caches",
        "Library/Developer/Xcode/iOS DeviceSupport",
        "Library/Developer/Xcode/watchOS DeviceSupport",
        "Library/Developer/Xcode/tvOS DeviceSupport",
        "Library/Developer/XCTestDevices",
        "Library/Developer/Xcode/UserData/Previews",
        "Library/org.swift.swiftpm",
        "Library/pnpm/store",
        "Library/Containers/com.docker.docker",
        ".Trash",
        ".npm",
        ".pnpm-store",
        ".cache",
        ".local/share/pnpm/store",
        ".docker/cli-plugins/.cache",
        ".docker/buildx/cache",
        ".orbstack/log",
    ]
    .iter()
    .map(|rel| home.join(rel))
    .collect();

    // 系统侧只读白名单 (与 PureMac 对齐；不含 /System 等 SIP 树)
    roots.extend(
        [
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
        ]
        .iter()
        .map(PathBuf::from),
    );
    roots
}

/// 路径标准化 + 符号链接解析 (等价 NSString.standardizingPath + resolvingSymlinksInPath)
pub fn normalize(path: &Path) -> PathBuf {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    // strip \\?\\ prefix on Windows is not needed (macOS target)
    canonical
}

/// path 是否等于 root 或严格位于 root 之下 (防 /tmpfoo 穿过 /tmp)
pub fn is_inside_or_eq(path: &Path, root: &Path) -> bool {
    if path == root {
        return true;
    }
    path.starts_with(root)
}

/// 是否落在任一 allowed_roots 下 (整树 wipe 或子树)
pub fn is_safe_to_delete(resolved: &Path, home: &Path) -> bool {
    if is_provider_owned(resolved, home) {
        return false;
    }
    allowed_roots(home)
        .iter()
        .any(|root| is_inside_or_eq(resolved, root))
}

/// 云 provider 状态检测 (字面路径 + 解析后路径)
pub fn is_provider_owned(path: &Path, home: &Path) -> bool {
    let candidates = [path.to_path_buf(), normalize(path)];
    for c in &candidates {
        let s = c.to_string_lossy();
        if s.contains("com~apple~") {
            return true;
        }
        for root in denied_provider_roots(home) {
            if is_inside_or_eq(c, &root) {
                return true;
            }
        }
    }
    false
}

/// 单文件显式删除 (Downloads/Documents/Desktop 由 scanner 逐文件发出，不整树)
pub fn is_explicit_single_file_deletable(resolved: &Path, home: &Path) -> bool {
    is_inside_or_eq(resolved, &home.join("Downloads"))
        || is_inside_or_eq(resolved, &home.join("Documents"))
        || is_inside_or_eq(resolved, &home.join("Desktop"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_inside_or_eq() {
        assert!(is_inside_or_eq(Path::new("/tmp/a"), Path::new("/tmp")));
        assert!(is_inside_or_eq(Path::new("/tmp"), Path::new("/tmp")));
        assert!(!is_inside_or_eq(Path::new("/tmpfoo"), Path::new("/tmp")));
    }

    #[test]
    fn test_safe_to_delete_caches() {
        let home = Path::new("/Users/neo");
        assert!(is_safe_to_delete(
            Path::new("/Users/neo/Library/Caches/foo"),
            home
        ));
        assert!(!is_safe_to_delete(
            Path::new("/Users/neo/Documents/secret"),
            home
        ));
        assert!(!is_safe_to_delete(Path::new("/System/Library"), home));
        assert!(is_safe_to_delete(Path::new("/tmp/x"), home));
    }

    #[test]
    fn test_provider_denied() {
        let home = Path::new("/Users/neo");
        assert!(is_provider_owned(
            Path::new("/Users/neo/Library/Mobile Documents/com~apple~CloudDocs"),
            home
        ));
        assert!(is_provider_owned(
            Path::new("/Users/neo/Library/Caches/com.apple.bird/x"),
            home
        ));
        assert!(!is_provider_owned(
            Path::new("/Users/neo/Library/Caches/Other"),
            home
        ));
    }
}
