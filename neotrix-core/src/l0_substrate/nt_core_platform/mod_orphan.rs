#![forbid(unsafe_code)]

//! mod-tree 孤儿检测原语 —— 「文件在磁盘上但不在编译树里」
//!
//! # ⭐ 为什么要有它
//!
//! 2026-10-03 审计发现 `l5_cognition/nt_mind/seal/` 有**两个 `.rs` 从未
//! 进入编译树**：`source_adapter.rs`(464 行) 与 `domain_mapper.rs`(661 行)，
//! 两者互相依赖 ⇒ 1125 行互链逻辑整体未被编译。
//!
//! ⛔ **这类缺陷对现有所有门都是隐形的**：
//!
//! | 门 | 为什么看不见 |
//! |---|---|
//! | `cargo check` / `cargo test` | **不存在的代码不可能失败** |
//! | `nt_lock_audit` | 没编译就没有锁 |
//! | `check-feature-gates` | 同上 |
//! | `dead_code` lint | `pub` 在 `pub mod` 内不触发；**未编译的文件更不触发** |
//! | `git diff` | ⚠️ 改一个不被编译的文件，`git diff` **照样显示改动** —— 最具欺骗性 |
//!
//! ⇒ 我据此更正过一次汇报：以为「修好了 5 处」，实际只有 4 处生效。
//!
//! # 判据
//!
//! 一个 `foo.rs` 进入编译树的方式只有两种：
//! 1. 同目录 `mod.rs` 里有 `pub mod foo;` / `mod foo;`
//! 2. 被同目录另一个已进入树的模块用 `#[path = "foo.rs"] mod foo;` 引入
//!
//! 两者都不满足 ⇒ **孤儿**。
//!
//! ⚠️ 本模块**只做词法判定**，不构建完整 module graph —— 它检查
//! 「本目录 mod.rs 的直接声明」与「`#[path]` 引入」两种形态。
//! 跨目录的 `mod x;`（Rust 2018 路径语义）由调用方按需补充；
//! 遇到无法判定时**倾向报告为孤儿**（宁可多报，不漏报），
//! 因为漏报正是本缺陷隐蔽的原因。

use std::collections::HashSet;
use std::path::Path;

/// 从 `mod.rs` 内容中提取已声明的模块名。
///
/// 同时识别三种写法：
/// - `pub mod foo;`
/// - `mod foo;`（私有）
/// - `pub(crate) mod foo;`
///
/// ⛔ **不**匹配 `mod.rs` 里的 `mod tests {`（内联模块，不是文件）。
pub fn declared_mods(mod_rs: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    for line in mod_rs.lines() {
        let t = line.trim();
        // 跳过注释
        if t.starts_with("//") || t.starts_with("/*") || t.starts_with('*') {
            continue;
        }
        // `#[path = "…"]` 修饰的 mod 语句
        let is_path_attr = t.starts_with("#[path");
        if is_path_attr {
            continue;
        }
        // 逐个扫描本行（`mod.rs` 可能一行多个声明，靠 find_map）
        let mut rest = t;
        while let Some(pos) = rest.find("mod ") {
            let after = &rest[pos + 4..];
            rest = after;
            let after = after.trim_start();
            // 内联模块 `mod foo {` —— 不是文件声明。
            // ⛔ 判据不能只看 `starts_with('{')`：`mod foo {` 里 `{` 在**名字之后**，
            //    自测抓到 `pub mod delta { }` 被误当文件声明。
            let name: String = after
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if name.is_empty() {
                continue;
            }
            // 名字之后紧跟 `{` ⇒ 内联模块
            let after_name = after[name.len()..].trim_start();
            if after_name.starts_with('{') {
                continue;
            }
            out.insert(name);
        }
    }
    out
}

/// 从任意源码文件内容中提取 `#[path = "…"] mod foo;` 引入的文件名。
///
/// ⛔ **不**处理跨目录 `mod foo;`（Rust 2018 语义按调用方路径解析），
///    本函数只认显式 `#[path]`。
pub fn path_attr_files(content: &str) -> HashSet<String> {
    let mut out = HashSet::new();
    for line in content.lines() {
        let t = line.trim();
        let Some(rest) = t.strip_prefix("#[path") else {
            continue;
        };
        let Some(q1) = rest.find('"') else { continue };
        let after = &rest[q1 + 1..];
        let Some(q2) = after.find('"') else { continue };
        let path = &after[..q2];
        // 取末段（可能是 `sub/foo.rs`）
        let stem = path
            .rsplit('/')
            .next()
            .unwrap_or(path)
            .trim_end_matches(".rs")
            .to_string();
        if !stem.is_empty() {
            out.insert(stem);
        }
    }
    out
}

/// 一个孤儿文件的判定结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrphanFile {
    /// 文件名（不含 `.rs`）。
    pub stem: String,
    /// 行数（供评估「值不值得修」）。
    pub lines: usize,
    /// 该目录的 `mod.rs` 路径（相对）。
    pub mod_rs: String,
}

/// 列出某目录下所有**未被声明**的 `.rs` 文件。
///
/// ⚠️ **前提**：传入的 `mod_rs` 必须是**当前目录真实的 `mod.rs`**。
/// 若目录没有 `mod.rs`，则该目录下所有 `.rs` 都被视为孤儿
/// （Rust 要求子模块必须由某个 `mod.rs` 或父模块声明）。
///
/// `path_attr_sources` 是调用方从**已确认在编译树里的文件**收集的
/// `#[path]` 引入清单 —— 这样跨目录的显式引入也能被识别。
pub fn orphans_in_dir(
    dir: &Path,
    mod_rs_content: Option<&str>,
    path_attr_sources: &[&str],
) -> Vec<OrphanFile> {
    let declared = mod_rs_content.map(declared_mods).unwrap_or_default();

    // 收集所有已知的额外引入（来自 #[path]）
    let mut extra: HashSet<String> = HashSet::new();
    for src in path_attr_sources {
        extra.extend(path_attr_files(src));
    }

    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for entry in entries.flatten() {
        let p = entry.path();
        if !p.is_file() {
            continue;
        }
        if p.extension().map(|e| e != "rs").unwrap_or(true) {
            continue;
        }
        let Some(stem) = p.file_stem().map(|s| s.to_string_lossy().to_string()) else {
            continue;
        };
        if stem == "mod" || stem == "lib" || stem == "main" {
            continue;
        }
        if declared.contains(&stem) || extra.contains(&stem) {
            continue;
        }
        let lines = std::fs::read_to_string(&p)
            .map(|c| c.lines().count())
            .unwrap_or(0);
        let mod_rs = dir
            .join("mod.rs")
            .to_string_lossy()
            .to_string();
        out.push(OrphanFile { stem, lines, mod_rs });
    }
    out.sort_by(|a, b| b.lines.cmp(&a.lines).then(a.stem.cmp(&b.stem)));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn tmpdir(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("nt_orphan_test_{}_{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).expect("create temp dir");
        d
    }

    // ── declared_mods ─────────────────────────────────────────

    #[test]
    fn extracts_all_declaration_forms() {
        let src = r#"
pub mod alpha;
mod beta;
pub(crate) mod gamma;
pub mod delta { }        // 内联模块，不是文件
// mod commented_out;
use foo::bar;            // 不含 "mod " 关键字形态
pub use alpha::x;
"#;
        let got = declared_mods(src);
        assert!(got.contains("alpha"), "{:?}", got);
        assert!(got.contains("beta"), "私有 mod 也是声明: {:?}", got);
        assert!(got.contains("gamma"), "pub(crate) 也是: {:?}", got);
        assert!(!got.contains("delta"), "内联模块不算文件: {:?}", got);
        assert!(!got.contains("commented_out"), "注释不算: {:?}", got);
    }

    #[test]
    fn ignores_commented_declarations() {
        let got = declared_mods("// pub mod ghost;\npub mod real;");
        assert!(got.contains("real"));
        assert!(!got.contains("ghost"));
    }

    // ── path_attr_files ───────────────────────────────────────

    #[test]
    fn extracts_path_attributes() {
        let src = r#"
#[path = "other.rs"]
mod other;
#[path = "nested/deep.rs"]
mod deep;
mod plain;
"#;
        let got = path_attr_files(src);
        assert!(got.contains("other"), "{:?}", got);
        assert!(got.contains("deep"), "跨目录也取末段: {:?}", got);
        assert!(!got.contains("plain"), "无 #[path] 不算: {:?}", got);
    }

    // ── orphans_in_dir（真实文件系统）──────────────────────────

    #[test]
    fn finds_files_missing_from_mod_rs() {
        let d = tmpdir("basic");
        std::fs::write(d.join("mod.rs"), "pub mod alpha;\npub mod beta;\n").expect("write");
        std::fs::write(d.join("alpha.rs"), "pub fn a() {}\n").expect("write");
        std::fs::write(d.join("beta.rs"), "pub fn b() {}\n").expect("write");
        std::fs::write(d.join("gamma.rs"), "pub fn g() {}\n").expect("write");
        std::fs::write(d.join("delta.rs"), &"x\n".repeat(100)).expect("write");

        let mod_rs = std::fs::read_to_string(d.join("mod.rs")).expect("read");
        let orphans = orphans_in_dir(&d, Some(&mod_rs), &[]);

        let stems: Vec<&str> = orphans.iter().map(|o| o.stem.as_str()).collect();
        assert!(stems.contains(&"gamma"), "未声明的必须报出: {:?}", stems);
        assert!(stems.contains(&"delta"), "未声明的必须报出: {:?}", stems);
        assert!(!stems.contains(&"alpha"), "已声明的不报: {:?}", stems);
        assert!(!stems.contains(&"beta"), "已声明的不报: {:?}", stems);
        assert!(!stems.contains(&"mod"), "mod.rs 自身不报");

        // ⭐ 按行数降序 —— 便于先看值不值得修的大块
        assert_eq!(orphans[0].stem, "delta", "应按行数降序");
        assert_eq!(orphans[0].lines, 100);

        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn path_attribute_import_is_recognized() {
        let d = tmpdir("pathattr");
        std::fs::write(d.join("mod.rs"), "pub mod alpha;\n").expect("write");
        std::fs::write(d.join("alpha.rs"), "#[path = \"gamma.rs\"]\nmod g;\n").expect("write");
        std::fs::write(d.join("gamma.rs"), "pub fn g() {}\n").expect("write");

        let mod_rs = std::fs::read_to_string(d.join("mod.rs")).expect("read");
        let alpha = std::fs::read_to_string(d.join("alpha.rs")).expect("read");
        // 把 alpha.rs 的内容作为「已在编译树里的源」传入
        let orphans = orphans_in_dir(&d, Some(&mod_rs), &[&alpha]);
        let stems: Vec<&str> = orphans.iter().map(|o| o.stem.as_str()).collect();
        assert!(
            !stems.contains(&"gamma"),
            "经 #[path] 引入的不算孤儿: {:?}",
            stems
        );
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn dir_without_mod_rs_treats_everything_as_orphan() {
        let d = tmpdir("nomodrs");
        std::fs::write(d.join("solo.rs"), "pub fn s() {}\n").expect("write");
        let orphans = orphans_in_dir(&d, None, &[]);
        assert_eq!(orphans.len(), 1);
        assert_eq!(orphans[0].stem, "solo");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn nonexistent_dir_yields_empty_not_panic() {
        let orphans = orphans_in_dir(Path::new("/nonexistent/xyz/abc"), None, &[]);
        assert!(orphans.is_empty());
    }

    #[test]
    fn non_rust_files_are_ignored() {
        let d = tmpdir("nonrust");
        std::fs::write(d.join("mod.rs"), "").expect("write");
        std::fs::write(d.join("notes.md"), "hi").expect("write");
        std::fs::write(d.join("data.json"), "{}").expect("write");
        let orphans = orphans_in_dir(&d, Some(""), &[]);
        assert!(orphans.is_empty(), "非 .rs 不该报: {:?}", orphans);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn subdirectories_are_not_reported_as_files() {
        let d = tmpdir("subdir");
        std::fs::write(d.join("mod.rs"), "pub mod child;").expect("write");
        std::fs::create_dir_all(d.join("child")).expect("mkdir");
        let orphans = orphans_in_dir(&d, Some("pub mod child;"), &[]);
        assert!(orphans.is_empty(), "子目录不是文件: {:?}", orphans);
        let _ = std::fs::remove_dir_all(&d);
    }
}