//! Catalog — 分类清理目标表 (吸收 PureMac CLI Catalog.swift)
//!
//! 域: NT-ACT · 层: L1 Action
//! 类别 id: dev | junk | ai | trash (对齐 puremac clean CLI)

use std::path::PathBuf;

/// 单个清理目标 (工具名 + 候选路径 + 是否默认选中 + 是否展开 contents)
#[derive(Debug, Clone)]
pub struct Target {
    pub tool: &'static str,
    pub paths: Vec<PathBuf>,
    pub selected_by_default: bool,
    pub contents: bool,
}

impl Target {
    fn tool(tool: &'static str, paths: &[&str], home: &PathBuf) -> Self {
        Self {
            tool,
            paths: paths
                .iter()
                .map(|p| home.join(p.trim_start_matches("~/")))
                .collect(),
            selected_by_default: true,
            contents: false,
        }
    }
    fn optional(tool: &'static str, paths: &[&str], home: &PathBuf) -> Self {
        let mut t = Self::tool(tool, paths, home);
        t.selected_by_default = false;
        t
    }
    fn contents(tool: &'static str, path: &str, home: &PathBuf) -> Self {
        let mut t = Self::tool(tool, &[path], home);
        t.contents = true;
        t
    }
}

/// 类别 id → 展示名
pub const CATEGORY_TITLES: &[(&str, &str)] = &[
    ("dev", "Dev Tools"),
    ("junk", "System Junk"),
    ("ai", "AI Junk"),
    ("trash", "Trash"),
];

/// 全部合法类别 id
pub const VALID_IDS: &[&str] = &["dev", "junk", "ai", "trash"];

/// 按类别返回目标表 (home 解析一次)
pub fn targets_for(id: &str, home: &PathBuf) -> Vec<Target> {
    match id {
        "dev" => dev_targets(home),
        "junk" => junk_targets(home),
        "ai" => ai_targets(home),
        _ => Vec::new(),
    }
}

fn dev_targets(h: &PathBuf) -> Vec<Target> {
    vec![
        Target::tool("Homebrew", &["Library/Caches/Homebrew"], h),
        Target::tool("npm", &[".npm/_cacache"], h),
        Target::tool(
            "Yarn",
            &["Library/Caches/Yarn", ".cache/yarn", ".yarn/berry/cache"],
            h,
        ),
        Target::tool(
            "pnpm",
            &["Library/pnpm/store", ".pnpm-store", ".cache/pnpm"],
            h,
        ),
        Target::tool("pip", &["Library/Caches/pip", ".cache/pip"], h),
        Target::tool(
            "Cargo (Rust)",
            &[
                ".cargo/registry/cache",
                ".cargo/registry/index",
                ".cargo/git/db",
                ".cargo/git/checkouts",
            ],
            h,
        ),
        Target::tool("Go", &["Library/Caches/go-build", "go/pkg/mod/cache"], h),
        Target::tool("CocoaPods", &["Library/Caches/CocoaPods"], h),
        Target::tool(
            "VS Code",
            &[
                "Library/Application Support/Code/Cache",
                "Library/Application Support/Code/CachedData",
                "Library/Application Support/Code/CachedExtensionVSIXs",
                "Library/Application Support/Code/logs",
            ],
            h,
        ),
        Target::tool(
            "JetBrains",
            &["Library/Caches/JetBrains", "Library/Logs/JetBrains"],
            h,
        ),
        Target::optional(
            "Maven (~/.m2 — may hold local installs)",
            &[".m2/repository"],
            h,
        ),
        Target::tool(
            "Gradle",
            &[".gradle/caches", ".gradle/daemon", ".gradle/wrapper/dists"],
            h,
        ),
        Target::tool("Poetry", &["Library/Caches/pypoetry", ".cache/pypoetry"], h),
        Target::tool("uv", &[".cache/uv", "Library/Caches/uv"], h),
        Target::tool("Bun", &[".bun/install/cache"], h),
        Target::tool("Deno", &["Library/Caches/deno", ".cache/deno"], h),
        Target::tool("mise", &[".cache/mise"], h),
        Target::tool("Flutter / Dart pub", &[".pub-cache"], h),
        Target::optional(
            "NuGet / .NET (~/.nuget — may hold local packages)",
            &[".nuget/packages", ".local/share/NuGet/http-cache"],
            h,
        ),
        Target::tool(
            "Swift Package Manager",
            &["Library/Caches/org.swift.swiftpm"],
            h,
        ),
        Target::tool(
            "Docker Desktop",
            &[
                "Library/Containers/com.docker.docker/Data/cache",
                "Library/Containers/com.docker.docker/Data/log",
                "Library/Containers/com.docker.docker/Data/tmp",
                "Library/Group Containers/group.com.docker/Caches",
                ".docker/cli-plugins/.cache",
                ".docker/buildx/cache",
            ],
            h,
        ),
        Target::tool(
            "OrbStack",
            &[
                ".orbstack/log",
                "Library/Caches/dev.kdrag0n.MacVirt",
                "Library/Logs/OrbStack",
            ],
            h,
        ),
    ]
}

fn junk_targets(h: &PathBuf) -> Vec<Target> {
    vec![
        Target::contents("User logs", "Library/Logs", h),
        Target::optional(
            "Saved application state",
            &["Library/Saved Application State"],
            h,
        ),
        Target::tool(
            "Xcode DerivedData",
            &["Library/Developer/Xcode/DerivedData"],
            h,
        ),
        Target::optional("Xcode Archives", &["Library/Developer/Xcode/Archives"], h),
        Target::tool(
            "iOS DeviceSupport",
            &["Library/Developer/Xcode/iOS DeviceSupport"],
            h,
        ),
        Target::tool(
            "watchOS DeviceSupport",
            &["Library/Developer/Xcode/watchOS DeviceSupport"],
            h,
        ),
        Target::tool(
            "tvOS DeviceSupport",
            &["Library/Developer/Xcode/tvOS DeviceSupport"],
            h,
        ),
        Target::tool(
            "CoreSimulator caches",
            &["Library/Developer/CoreSimulator/Caches"],
            h,
        ),
        Target::tool("Xcode app cache", &["Library/Caches/com.apple.dt.Xcode"], h),
        Target::tool("XCTestDevices", &["Library/Developer/XCTestDevices"], h),
        Target::tool(
            "SwiftUI Previews",
            &["Library/Developer/Xcode/UserData/Previews"],
            h,
        ),
    ]
}

fn ai_targets(h: &PathBuf) -> Vec<Target> {
    vec![
        Target::tool(
            "Ollama (logs/cache)",
            &[
                ".ollama/logs",
                "Library/Caches/ollama",
                "Library/Caches/com.electron.ollama",
            ],
            h,
        ),
        Target::tool("LM Studio (logs)", &[".lmstudio/server-logs"], h),
        Target::tool(
            "Cursor (cache)",
            &[
                "Library/Application Support/Cursor/Cache",
                "Library/Application Support/Cursor/CachedData",
                "Library/Application Support/Cursor/logs",
            ],
            h,
        ),
    ]
}

/// trash 根: ~/.Trash + /Volumes/*/.Trashes/<uid>
pub fn trash_roots(home: &PathBuf) -> Vec<PathBuf> {
    let mut roots = vec![home.join(".Trash")];
    if let Ok(vols) = std::fs::read_dir("/Volumes") {
        for vol in vols.flatten() {
            roots.push(vol.path().join(".Trashes").join(current_uid_string()));
        }
    }
    roots
}

fn current_uid_string() -> String {
    // 无 unsafe：用 `id -u`（POSIX）
    std::process::Command::new("id")
        .arg("-u")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "501".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_valid_ids() {
        assert_eq!(VALID_IDS, &["dev", "junk", "ai", "trash"]);
        assert!(CATEGORY_TITLES.iter().any(|(id, _)| *id == "dev"));
    }

    #[test]
    fn test_dev_targets_home_joined() {
        let home = PathBuf::from("/Users/neo");
        let t = targets_for("dev", &home);
        assert!(!t.is_empty());
        let npm = t.iter().find(|x| x.tool == "npm").expect("npm target");
        assert_eq!(npm.paths[0], PathBuf::from("/Users/neo/.npm/_cacache"));
        let maven = t
            .iter()
            .find(|x| x.tool.starts_with("Maven"))
            .expect("maven");
        assert!(!maven.selected_by_default);
    }

    #[test]
    fn test_unknown_category_empty() {
        assert!(targets_for("nope", &PathBuf::from("/h")).is_empty());
    }

    #[test]
    fn test_trash_roots_contains_home_trash() {
        let home = Path::new("/Users/neo");
        let roots = trash_roots(&home.to_path_buf());
        assert!(roots[0].ends_with(".Trash"));
    }
}
