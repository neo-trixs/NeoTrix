//! ScanEngine — 分类扫描 (吸收 PureMac CLI CategoryScanner + GUI ScanEngine)
//!
//! 域: NT-ACT · 层: L1 Action
//! 输出 CleanableItem 列表；每项带 path/size/selected_by_default。

use std::path::{Path, PathBuf};

use super::catalog::{targets_for, trash_roots, Target};
use super::exclusions::excludes;
use super::locations::normalize;
use super::safety::{can_remove, has_symlink_component};
use super::shared::{calculate_age_days, calculate_directory_size};

/// 可清理项 (对齐 CleanableItem / ScanItem)
#[derive(Debug, Clone)]
pub struct CleanableItem {
    pub name: String,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub category: String,
    pub selected: bool,
    pub age_days: u32,
    pub tool: String,
}

/// 单类别扫描结果
#[derive(Debug, Clone)]
pub struct CategoryResult {
    pub id: String,
    pub title: String,
    pub items: Vec<CleanableItem>,
}

impl CategoryResult {
    pub fn total_bytes(&self) -> u64 {
        self.items.iter().map(|i| i.size_bytes).sum()
    }
    pub fn selected_bytes(&self) -> u64 {
        self.items
            .iter()
            .filter(|i| i.selected)
            .map(|i| i.size_bytes)
            .sum()
    }
}

/// 最小可报告大小 (1 KiB)
pub const MIN_SIZE: u64 = 1024;

/// 扫描一类 (trash 特殊路径)
pub fn scan_category(id: &str, title: &str, home: &Path, excluded: &[PathBuf]) -> CategoryResult {
    if id == "trash" {
        return scan_trash(title, home, excluded);
    }
    let home_buf = home.to_path_buf();
    let targets = targets_for(id, &home_buf);
    let mut items = Vec::new();
    for t in &targets {
        for path in collect_candidate_paths(t) {
            if let Some(item) = measure_item(&path, id, title, t, home, excluded) {
                items.push(item);
            }
        }
    }
    // 去重: 更短路径优先 (父目录先于子项)
    items = dedupe(items);
    CategoryResult {
        id: id.to_string(),
        title: title.to_string(),
        items,
    }
}

fn collect_candidate_paths(t: &Target) -> Vec<PathBuf> {
    if !t.contents {
        return t.paths.clone();
    }
    let mut out = Vec::new();
    for root in &t.paths {
        if has_symlink_component(root) {
            continue;
        }
        if let Ok(rd) = std::fs::read_dir(root) {
            for e in rd.flatten() {
                out.push(e.path());
            }
        }
    }
    out
}

fn measure_item(
    path: &Path,
    id: &str,
    title: &str,
    t: &Target,
    home: &Path,
    excluded: &[PathBuf],
) -> Option<CleanableItem> {
    if has_symlink_component(path) {
        return None;
    }
    let meta = std::fs::symlink_metadata(path).ok()?;
    let age_days = calculate_age_days(&meta);
    let size = if meta.is_dir() {
        calculate_directory_size(path)
    } else {
        meta.len()
    };
    if size < MIN_SIZE {
        return None;
    }
    // 预过滤: 排除表 + 凭据 + provider (不强制 allow-list — 清理阶段再过)
    if excludes(path, excluded) {
        return None;
    }
    if let Some(reason) = can_remove(path, home, excluded, false).reason() {
        if matches!(
            reason,
            "symlink"
                | "config/credentials dir"
                | "cloud/provider state"
                | "excluded"
                | "protected root"
                | "protected system path"
                | "another user's files"
        ) {
            return None;
        }
    }
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned());
    Some(CleanableItem {
        name,
        path: path.to_path_buf(),
        size_bytes: size,
        category: id.to_string(),
        selected: t.selected_by_default,
        age_days,
        tool: title.to_string(),
    })
}

fn scan_trash(title: &str, home: &Path, excluded: &[PathBuf]) -> CategoryResult {
    let home_buf = home.to_path_buf();
    let mut items = Vec::new();
    for root in trash_roots(&home_buf) {
        if has_symlink_component(&root) {
            continue;
        }
        let Ok(rd) = std::fs::read_dir(&root) else {
            continue;
        };
        let mut children: Vec<PathBuf> = rd.flatten().map(|e| e.path()).collect();
        children.sort();
        for path in children {
            if has_symlink_component(&path) || excludes(&path, excluded) {
                continue;
            }
            let Ok(meta) = std::fs::symlink_metadata(&path) else {
                continue;
            };
            let size = if meta.is_dir() {
                calculate_directory_size(&path)
            } else {
                meta.len()
            };
            if size < MIN_SIZE {
                continue;
            }
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            items.push(CleanableItem {
                name,
                path,
                size_bytes: size,
                category: "trash".into(),
                selected: true,
                age_days: calculate_age_days(&meta),
                tool: title.to_string(),
            });
        }
    }
    items = dedupe(items);
    CategoryResult {
        id: "trash".into(),
        title: title.to_string(),
        items,
    }
}

/// 路径去重: 规范化后，已接受父路径则跳过子路径
fn dedupe(items: Vec<CleanableItem>) -> Vec<CleanableItem> {
    let mut ordered: Vec<CleanableItem> = items;
    ordered.sort_by_key(|i| i.path.as_os_str().len());
    let mut accepted: Vec<PathBuf> = Vec::new();
    let mut seen: Vec<PathBuf> = Vec::new();
    let mut out = Vec::new();
    for item in ordered {
        let canon = normalize(&item.path);
        if seen.contains(&canon) {
            continue;
        }
        if accepted.iter().any(|a| canon.starts_with(a) || *a == canon) {
            // 子路径已被父覆盖
            continue;
        }
        seen.push(canon.clone());
        accepted.push(canon);
        out.push(item);
    }
    out
}

/// 扫描全部有效类别
pub fn scan_all(home: &Path, excluded: &[PathBuf]) -> Vec<CategoryResult> {
    super::catalog::VALID_IDS
        .iter()
        .filter_map(|id| {
            let title = super::catalog::CATEGORY_TITLES
                .iter()
                .find(|(i, _)| i == id)
                .map(|(_, t)| *t)?;
            let r = scan_category(id, title, home, excluded);
            if r.items.is_empty() {
                None
            } else {
                Some(r)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_scan_dev_finds_npm_cache() {
        let dir = TempDir::new().unwrap();
        let home = dir.path().to_path_buf();
        let npm = home.join(".npm/_cacache");
        std::fs::create_dir_all(&npm).unwrap();
        std::fs::write(npm.join("blob"), vec![0u8; 4096]).unwrap();
        let r = scan_category("dev", "Dev Tools", &home, &[]);
        assert!(r.items.iter().any(|i| i.path.ends_with("_cacache")));
        assert!(r.total_bytes() >= 4096);
    }

    #[test]
    fn test_dedupe_parent_wins() {
        let items = vec![
            CleanableItem {
                name: "a".into(),
                path: PathBuf::from("/tmp/x"),
                size_bytes: 100,
                category: "dev".into(),
                selected: true,
                age_days: 0,
                tool: "t".into(),
            },
            CleanableItem {
                name: "b".into(),
                path: PathBuf::from("/tmp/x/y"),
                size_bytes: 50,
                category: "dev".into(),
                selected: true,
                age_days: 0,
                tool: "t".into(),
            },
        ];
        let d = dedupe(items);
        assert_eq!(d.len(), 1);
        assert!(d[0].path.ends_with("x"));
    }

    #[test]
    fn test_skips_small_files() {
        let dir = TempDir::new().unwrap();
        let home = dir.path().to_path_buf();
        let npm = home.join(".npm/_cacache");
        std::fs::create_dir_all(&npm).unwrap();
        std::fs::write(npm.join("tiny"), b"x").unwrap(); // < 1KiB
        let r = scan_category("dev", "Dev Tools", &home, &[]);
        assert!(r.items.is_empty());
    }
}
