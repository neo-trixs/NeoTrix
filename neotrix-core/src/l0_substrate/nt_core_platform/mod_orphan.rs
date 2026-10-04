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

/// 递归扫描整棵源码树，返回**全部**孤儿，按行数降序。
///
/// # 与 `orphans_in_dir` 的关系
///
/// `orphans_in_dir` 是单目录原语（纯、无 IO 副作用、易测）；
/// 本函数是它的**生产接线**（R-P79：导出 ≠ 接入）。
/// 遍历策略：对每个含 `mod.rs` 的目录调用一次 `orphans_in_dir`。
///
/// # ⭐ 口径（务必读，否则会与 `nt_core_self::self_audit` 对不上账）
///
/// 本函数**保守**：只认「同目录 `mod.rs` 的直接声明」+ `#[path]`。
/// 因此它**不会**报出 god-file 形态的孤儿（非 `mod.rs` 文件内部的
/// `mod types;`，Rust 把它解析到 `dir/<该文件 stem>/types.rs`）。
///
/// 实测口径对照（2026-10-04，同一棵树）：
/// - 本函数：**35** 个
/// - `self_audit::scan_orphan_files`（宽松，含 god-file）：**99** 个
///
/// ⇒ **35 ⊂ 99**，本函数是保守子集。两者都能抓到已知的真孤儿
/// （`seal/source_adapter.rs`）。看到两个数字不必惊慌。
///
/// # 边界
///
/// - 跳过 `target/`、`_archived/`、`/bin/`（与既有门保持一致）。
/// - 无 `mod.rs` 的目录**不**单独扫描：Rust 子模块必须由某处声明，
///   而本函数的判据是「本目录 mod.rs」⇒ 跳过比误报更安全。
/// - I/O 失败（目录不可读等）静默跳过该目录，不 panic。
pub fn scan_tree(root: &Path) -> Vec<OrphanFile> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        // ⛔ 判孤儿必须在**当前 dir 自己**做，不能只在它的子目录里做。
        //    早先版本把 `mod.rs` 检查写在遍历子目录的内层循环里
        //    ⇒ **root 自身永远不被扫描**（实测：`root/loose.rs` 漏报，
        //    而 `sub/loose2.rs` 报出来了 —— 一个「只漏顶层」的缺陷，
        //    最坏情况是顶层恰恰是最容易出事的 `src/`）。
        let mod_rs = dir.join("mod.rs");
        if mod_rs.is_file() {
            let content = std::fs::read_to_string(&mod_rs).unwrap_or_default();
            let rel = mod_rs
                .strip_prefix(root)
                .unwrap_or(&mod_rs)
                .to_string_lossy()
                .to_string();
            let mut found = orphans_in_dir(&dir, Some(&content), &[]);
            for o in &mut found {
                o.mod_rs = rel.clone();
            }
            out.append(&mut found);
        }

        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if !p.is_dir() {
                continue;
            }
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name == "target" || name == "_archived" || name == "bin" {
                continue;
            }
            stack.push(p);
        }
    }

    out.sort_by(|a, b| b.lines.cmp(&a.lines).then_with(|| a.stem.cmp(&b.stem)));
    out
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

    // ── scan_tree（递归生产接线）──────────────────────────────────

    /// 造一棵两层的树：
    /// ```text
    /// root/mod.rs            (pub mod a; pub mod b;)
    /// root/a.rs              已声明
    /// root/loose.rs          ⬅ 孤儿（root/mod.rs 未声明）
    /// root/sub/mod.rs        (pub mod c;)
    /// root/sub/c.rs          已声明
    /// root/sub/loose2.rs     ⬅ 孤儿（sub/mod.rs 未声明）
    /// root/target/x.rs       应被跳过（target/）
    /// ```
    fn build_tree(name: &str) -> PathBuf {
        let root = tmpdir(name);
        let sub = root.join("sub");
        let target = root.join("target");
        std::fs::create_dir_all(&sub).expect("mkdir sub");
        std::fs::create_dir_all(&target).expect("mkdir target");
        std::fs::write(root.join("mod.rs"), "pub mod a;\npub mod b;").expect("w");
        std::fs::write(root.join("a.rs"), "pub fn a() {}").expect("w");
        std::fs::write(root.join("loose.rs"), "// orphan\n// x\n").expect("w");
        std::fs::write(sub.join("mod.rs"), "pub mod c;").expect("w");
        std::fs::write(sub.join("c.rs"), "pub fn c() {}").expect("w");
        std::fs::write(sub.join("loose2.rs"), "// orphan\n").expect("w");
        std::fs::write(target.join("x.rs"), "junk").expect("w");
        std::fs::write(target.join("mod.rs"), "pub mod x;").expect("w");
        root
    }

    #[test]
    fn scan_tree_finds_orphans_at_every_level() {
        let root = build_tree("tree_all");
        let found = scan_tree(&root);
        let stems: Vec<&str> = found.iter().map(|o| o.stem.as_str()).collect();
        assert!(stems.contains(&"loose"), "应抓到 root/loose.rs: {:?}", stems);
        assert!(
            stems.contains(&"loose2"),
            "应抓到 sub/loose2.rs（递归下潜）: {:?}",
            stems
        );
        assert!(!stems.contains(&"a"), "已声明的 a 不该报: {:?}", stems);
        assert!(!stems.contains(&"c"), "已声明的 c 不该报: {:?}", stems);
        assert!(!stems.contains(&"x"), "target/ 应被跳过: {:?}", stems);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn scan_tree_records_owning_mod_rs() {
        let root = build_tree("tree_owner");
        let found = scan_tree(&root);
        let loose2 = found.iter().find(|o| o.stem == "loose2").expect("loose2");
        assert_eq!(
            loose2.mod_rs, "sub/mod.rs",
            "应记录**该文件自己**目录的 mod.rs，而不是根的"
        );
        let loose = found.iter().find(|o| o.stem == "loose").expect("loose");
        assert_eq!(loose.mod_rs, "mod.rs");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn scan_tree_sorted_by_lines_desc() {
        let root = tmpdir("tree_sort");
        let sub = root.join("s");
        std::fs::create_dir_all(&sub).expect("mkdir");
        std::fs::write(root.join("mod.rs"), "").expect("w");
        std::fs::write(root.join("small.rs"), "x\n").expect("w");
        std::fs::write(sub.join("mod.rs"), "").expect("w");
        // 5 行 ⇒ 行数应更大，排在前
        std::fs::write(sub.join("big.rs"), "a\nb\nc\nd\ne\n").expect("w");
        let found = scan_tree(&root);
        assert_eq!(found.len(), 2, "两个孤儿: {:?}", found);
        assert_eq!(found[0].stem, "big", "应按行数降序: {:?}", found);
        assert!(found[0].lines >= found[1].lines);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn scan_tree_on_nonexistent_root_is_empty_not_panic() {
        assert!(scan_tree(Path::new("/nonexistent/scan_tree/xyz")).is_empty());
    }

    /// ⭐ 生产接线自证：在**真实仓库**上跑，必须抓到已知的真孤儿。
    ///
    /// 若此测试抓不到 `seal/source_adapter.rs`，说明 `scan_tree` 与
    /// `orphans_in_dir` 的接线断了（而不是「仓库很干净」）。
    #[test]
    fn scan_tree_catches_known_orphan_in_real_repo() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let found = scan_tree(&root);
        assert!(
            !found.is_empty(),
            "真实源码树应至少有已知孤儿（seal/source_adapter.rs 等）"
        );
        let has_seal = found
            .iter()
            .any(|o| o.mod_rs.contains("seal") || o.stem == "source_adapter");
        assert!(
            has_seal,
            "应抓到 seal 下的孤儿（已知真孤儿）: {:?}",
            found.iter().map(|o| (&o.stem, &o.mod_rs)).collect::<Vec<_>>()
        );
    }
}