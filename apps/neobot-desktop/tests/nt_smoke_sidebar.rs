//! 冒烟：侧边栏注册表（`nt_cmd_sidebar` 的 tabs/viewers/resolve 半边）。
//!
//! ## 覆盖的缺陷类别
//!
//! 前端各处 `if (ext === ...)` 猜扩展名，是**曾经发生过**的事。
//! 页签表与查看器表现在注册在 Rust 侧一处，所以「前端猜」和「Rust 决策」
//! 两条路必须给出**同一个答案** —— 那就要求两者都被执行验证。
//!
//! ## 越狱注记（重要，别误读）
//!
//! `neobot_sidebar_resolve` 走的是 `nt_workspace::check_rel`，
//! 即**只有词法那一道**——它手上没有工作区路径，无法做 canonicalize。
//! 真正落盘的那道在 `neobot_fs_read/write` 里（见 `nt_smoke_files.rs`
//! 的软链接越狱用例）。
//!
//! 所以本文件只断言**词法越狱被拒**；软链接越狱在文件那套里测。
//! 两者缺一不可，合起来才是完整的 jail。

mod common;

use neobot_desktop::nt_commands::nt_cmd_sidebar::{
    neobot_sidebar_resolve, neobot_sidebar_tabs, neobot_sidebar_viewer_for,
    neobot_sidebar_viewers,
};

#[test]
fn sidebar_tabs_are_unique_sorted_and_each_declares_its_data_source() {
    let _ = common::data_dir();
    let tabs = neobot_sidebar_tabs().expect("取页签表应成功");
    assert!(!tabs.is_empty(), "页签表不能空 —— 空表等于整个侧边栏空白");

    let orders: Vec<u8> = tabs.iter().map(|t| t.order).collect();
    let sorted = {
        let mut c = orders.clone();
        c.sort_unstable();
        c
    };
    assert_eq!(orders, sorted, "页签必须按 order 排好序再给前端，否则排序在前端");

    let ids: Vec<&str> = tabs.iter().map(|t| t.id.as_str()).collect();
    let mut dedup = ids.clone();
    dedup.sort_unstable();
    dedup.dedup();
    assert_eq!(dedup.len(), ids.len(), "页签 id 重复：{ids:?}");

    for tab in &tabs {
        assert!(!tab.id.is_empty() && !tab.title.is_empty(), "页签字段不能空：{tab:?}");
        assert!(
            !tab.needs.is_empty(),
            "页签 '{}' 没声明数据源 —— 前端就不知道要不要拉，会白拉或漏拉",
            tab.id
        );
        assert!(
            !tab.icon.is_empty(),
            "页签 '{}' 没声明图标，前端会退化成无意义的裸文字",
            tab.id
        );
    }
}

#[test]
fn sidebar_viewers_claim_extensions_uniquely() {
    let _ = common::data_dir();
    let viewers = neobot_sidebar_viewers().expect("取查看器表应成功");
    assert!(!viewers.is_empty(), "查看器表不能空");

    // 一个扩展名被两个查看器认领 = 前端选到哪个全看顺序，决策不再唯一。
    let mut seen: Vec<(&str, &str)> = Vec::new();
    for v in &viewers {
        assert!(!v.id.is_empty() && !v.label.is_empty(), "查看器字段不能空：{v:?}");
        for ext in &v.extensions {
            assert_eq!(ext, &ext.to_lowercase(), "扩展名须小写：{ext}（否则 macOS/Linux 判定分裂）");
            seen.push((ext.as_str(), v.id.as_str()));
        }
    }
    let mut exts: Vec<&str> = seen.iter().map(|(e, _)| *e).collect();
    exts.sort_unstable();
    let before = exts.len();
    exts.dedup();
    assert_eq!(before, exts.len(), "同一扩展名被多个查看器认领，渲染器选择不再唯一：{seen:?}");
}

#[test]
fn sidebar_viewer_for_returns_a_spec_identical_to_the_one_in_the_table() {
    // 关键：前端拿 `viewer_for` 的结果选渲染器，拿 `viewers` 表决定要不要给保存按钮。
    // 两处若对不上（哪怕只差一个 `editable`），前端就会给只读查看器配上保存按钮。
    // 断言用**整条 spec 逐字相等**，而不是只比 id —— 只比 id 正是这类漂移的漏网口。
    let _ = common::data_dir();
    let table = neobot_sidebar_viewers().expect("取查看器表应成功");

    for (rel, want_id) in [
        ("a.rs", "code"),
        ("b.py", "code"),
        ("c.md", "markdown"),
        ("d.json", "data"),
        ("e.png", "image"),
        ("f.unknownext", "editor"), // 兜底
        ("noext", "editor"),
        ("sub/dir/f.rs", "code"), // 带目录也只看最后一段
    ] {
        let picked = neobot_sidebar_viewer_for(rel.to_owned())
            .expect("选查看器应成功")
            .unwrap_or_else(|| panic!("{rel} 应有查看器兜底，实际 None"));
        let declared = table
            .iter()
            .find(|t| t.id == want_id)
            .unwrap_or_else(|| panic!("表里应有 {want_id}，实际 {:?}", table.iter().map(|t| &t.id).collect::<Vec<_>>()));
        assert_eq!(picked.id, want_id, "{rel} 选错了查看器");
        assert_eq!(&picked, declared, "{rel} 选中的 spec 与表里那条不一致（前端两处会对不上）");
    }
}

#[test]
fn sidebar_viewer_for_marks_editable_and_readonly_differently() {
    let _ = common::data_dir();
    // 源文件可改可存，二进制/图片不可改（否则保存会写坏它）。
    let text = neobot_sidebar_viewer_for("main.rs".to_owned())
        .expect("应可选")
        .expect("源文件该有查看器");
    assert!(text.editable, "源文件应可编辑：{}", text.id);

    let image = neobot_sidebar_viewer_for("shot.png".to_owned())
        .expect("应可选")
        .expect("图片该有查看器");
    assert!(!image.editable, "图片不该可编辑，否则保存即写坏二进制：{}", image.id);
}

#[test]
fn sidebar_resolve_binds_topic_and_target_into_one_open_target() {
    let _ = common::data_dir();
    let open = neobot_sidebar_resolve("files".to_owned(), "src/main.rs".to_owned())
        .expect("解析打开请求应成功");
    assert_eq!(open.topic, "files");
    assert_eq!(open.path, "src/main.rs", "path 必须原样带出（前端按它读文件）");
    assert_eq!(open.viewer.as_deref(), Some("code"), "源文件应命中代码查看器");
}

#[test]
fn sidebar_resolve_rejects_an_unknown_topic() {
    // 模型可以让 `sidebar_open` 提议任意 topic；不在册的必须拒，
    // 否则前端会跳到一个不存在的页签、界面「卡住不动」且无从报错。
    let _ = common::data_dir();
    let err = neobot_sidebar_resolve("nope".to_owned(), "a.txt".to_owned())
        .expect_err("未注册 topic 必须被拒");
    assert!(err.contains("nope"), "错误信息应写明是哪个 topic 被拒：{err}");
}

#[test]
fn sidebar_resolve_refuses_jail_escaping_targets() {
    let _ = common::data_dir();
    // 这些必须在**解析阶段**就拒。若哪天改成「静默重新定根」，
    // 这条会红 —— 而那正是「报错比骗人好」被破掉的那一刻。
    for bad in [
        "../../etc/passwd",
        "/etc/passwd",
        "~/.ssh/id_rsa",
        "a/../../b",
        "..\\..\\windows",
        "C:/windows",
    ] {
        let err = neobot_sidebar_resolve("files".to_owned(), bad.to_owned())
            .expect_err("越狱目标必须被拒（静默重新定根 = 骗人）");
        assert!(
            err.contains("workspace-jail"),
            "拒绝原因应指向 workspace-jail 规则，实际：{err}"
        );
    }
}

#[test]
fn sidebar_resolve_requires_a_target_for_the_files_topic() {
    let _ = common::data_dir();
    // 空 target + files topic = 前端无从知道打开哪；必须拒，而不是回一个空路径。
    neobot_sidebar_resolve("files".to_owned(), String::new())
        .expect_err("files 页签必须带 target");
    // 不需要路径的页签则允许空 target。
    neobot_sidebar_resolve("chat".to_owned(), String::new())
        .expect("chat 页签可无 target");
}
