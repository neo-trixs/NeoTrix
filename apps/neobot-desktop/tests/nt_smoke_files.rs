//! 冒烟：文件工作台（`nt_cmd_files` 的 fs_* 半边）+ **越狱拒绝**。
//!
//! ## 覆盖的缺陷类别
//!
//! 「参数绑定接错」：命令把 `rel` 传给了核心的**另一个**位置、或 jail 校验被
//! 悄悄绕过。核心层测不出来 —— 核心函数是干净的，错的是壳这一层的接线。
//!
//! ## 关键断言
//!
//! 越狱必须被**拒**（`Err`），而不是「静默重新定根到工作区」。
//! 后者更阴险：用户以为读到了 `/etc/passwd`，其实读到的是工作区里的同名文件。
//! 见 `nt_workspace::check_rel` 的注释——「报错比骗人好」。

mod common;

use neobot_desktop::nt_commands::nt_cmd_files::{
    neobot_fs_find, neobot_fs_list, neobot_fs_read, neobot_fs_root, neobot_fs_write,
};
use neotrix_neobot::nt_workspace::SearchCap;

#[test]
fn fs_write_then_read_roundtrips_through_the_workspace_root() {
    let _ = common::data_dir();
    let rel = common::write_in_workspace("notes/hello.md", "第一行\n第二行\n");

    let written = neobot_fs_write(rel.clone(), "第一行\n第二行\n第三行\n".to_owned())
        .expect("写工作区文件应成功");
    assert_eq!(written.rel, rel, "写回 DTO 的 rel 必须与传入一致");
    assert!(written.bytes > 0, "字节数不能是 0，否则前端进度条不动");

    let read = neobot_fs_read(rel.clone()).expect("读工作区文件应成功");
    assert_eq!(read.rel, rel);
    assert_eq!(read.content, "第一行\n第二行\n第三行\n", "内容必须逐字回来");
    assert!(!read.binary, "纯文本不能被标成二进制");
    assert!(!read.truncated, "这么点内容不该被截断");
    assert!(read.size > 0);
}

#[test]
fn fs_write_creates_missing_parent_dirs() {
    // 前端「新建文件」会把整条路径传下来，父目录常常还不存在。
    let _ = common::data_dir();
    let written = neobot_fs_write("deep/a/b/c.txt".to_owned(), "x".to_owned())
        .expect("应自动补建父目录");
    assert!(written.created, "首次落盘应报 created=true（前端据此提示「新文件」）");
    assert!(
        common::workspace_dir().join("deep/a/b/c.txt").exists(),
        "文件应真的落在工作区里"
    );
}

#[test]
fn fs_list_sees_written_files_and_reports_parent() {
    let _ = common::data_dir();
    common::write_in_workspace("listed/one.txt", "1");
    common::write_in_workspace("listed/two.txt", "2");

    let root = neobot_fs_list(String::new()).expect("列根目录应成功");
    assert!(
        root.entries.iter().any(|e| e.rel == "listed"),
        "根目录清单里应看到 listed/，实际：{:?}",
        root.entries.iter().map(|e| &e.rel).collect::<Vec<_>>()
    );
    assert!(root.parent.is_none(), "根目录没有上级");

    let inside = neobot_fs_list("listed".to_owned()).expect("列子目录应成功");
    let names: Vec<&str> = inside.entries.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"one.txt") && names.contains(&"two.txt"), "实际：{names:?}");
    assert_eq!(inside.parent.as_deref(), Some(""), "子目录的上级应为空串（= 根）");
}

#[test]
fn fs_find_locates_a_file_by_name() {
    let _ = common::data_dir();
    common::write_in_workspace("haystack/needle.md", "找到我");

    // 签名从 `Vec<SearchHit>` 改成 `SearchResults`（带 `truncated`），
    // 故这里补一个 `.hits`；下面每条断言与改前逐字一致。
    let hits = neobot_fs_find("needle".to_owned())
        .expect("搜索应成功")
        .hits;
    assert!(
        hits.iter().any(|hit| hit.rel == "haystack/needle.md" && !hit.is_dir),
        "应命中刚写的文件，实际命中：{:?}",
        hits.iter().map(|h| &h.rel).collect::<Vec<_>>()
    );
}

#[test]
fn fs_find_reports_hit_cap_truncation_over_ipc() {
    // 这一条盯的是**壳这一层**：命令的返回 DTO 里必须带着 `truncated`。
    // 核心层的 `truncated` 测得再好看，壳这里若把它丢掉（回裸数组），
    // 前端就又回到「靠 hits.length >= 200 猜」的老路 —— 那是原缺陷的成因。
    let _ = common::data_dir();
    // 201 个匹配 > MAX_SEARCH_HITS(200)：第 201 个必然被砍。
    // `find` 匹配的是**文件名**（不是路径），所以查询词必须落在文件名里；
    // 父目录取名 `caps` 就是为了不让它自己也算一个命中，把计数搞脏。
    for i in 0..201 {
        common::write_in_workspace(&format!("caps/smokecap-{i:04}.txt"), "");
    }

    let got = neobot_fs_find("smokecap".to_owned()).expect("搜索应成功");
    assert_eq!(
        got.hits.len(),
        200,
        "应正好停在 MAX_SEARCH_HITS 上（夹具自身要先数对：{:?}）",
        got.hits.iter().map(|h| &h.rel).collect::<Vec<_>>()
    );
    assert!(
        got.truncated,
        "命中数被上限砍过，IPC 必须如实回报（否则前端无从知道结果不完整）"
    );
    assert_eq!(got.truncated_by, Some(SearchCap::Hits));
}

#[test]
fn fs_find_reports_no_truncation_when_nothing_was_cut() {
    // 反向的一条（防「永远返回 true」）：没撞任何上限时必须报 false，
    // 否则前端会对着一个完整结果也喊「已截断」，用户白搜一遍。
    // 用一个谁也不会命中的查询，故不受同二进制里其它测试造的文件影响。
    let _ = common::data_dir();
    let got = neobot_fs_find("zzz-no-such-name-zzz".to_owned()).expect("搜索应成功");
    assert!(got.hits.is_empty(), "这个查询本就不该有命中");
    assert!(!got.truncated, "没撞上限就不能报截断");
    assert_eq!(got.truncated_by, None, "没截断就不该报出上限名");
}

#[test]
fn fs_root_reports_the_injected_workspace_not_the_real_home() {
    // 这条直接验「壳真的读了 NEOBOT_DATA_DIR」——
    // 如果命令偷偷用了 `~/.neobot/workspace`，这里就会暴露。
    let _ = common::data_dir();
    let root = neobot_fs_root().expect("取工作区根应成功");
    assert_eq!(
        root,
        common::workspace_dir().to_string_lossy().into_owned(),
        "工作区根必须跟着 NEOBOT_DATA_DIR 走"
    );
    assert!(!root.contains("/.neobot/workspace/"), "不该指向真实 home：{root}");
}

// ─── 越狱：必须拒，不能「静默重新定根」 ───

#[test]
fn fs_read_refuses_parent_directory_escape() {
    let _ = common::data_dir();
    let err = neobot_fs_read("../secrets.txt".to_owned())
        .expect_err("`..` 越狱必须被拒");
    assert!(
        err.contains("workspace-jail") || err.contains("越狱") || err.contains("escape"),
        "拒绝原因应指向 workspace-jail 规则，实际：{err}"
    );
}

#[test]
fn fs_read_refuses_absolute_path() {
    let _ = common::data_dir();
    // 绝对路径**拒**而不是重定根 —— 否则用户以为读到了 /etc/passwd。
    neobot_fs_read("/etc/passwd".to_owned())
        .expect_err("绝对路径必须被拒（静默重定根 = 骗人）");
}

#[test]
fn fs_read_refuses_tilde_and_backslash_forms() {
    let _ = common::data_dir();
    neobot_fs_read("~/.ssh/id_rsa".to_owned()).expect_err("`~` 开头必须被拒");
    neobot_fs_read("..\\..\\windows\\system32".to_owned()).expect_err("反斜杠必须被拒");
}

#[test]
fn fs_write_cannot_escape_the_jail_either() {
    let _ = common::data_dir();
    neobot_fs_write("../escaped.txt".to_owned(), "pwned".to_owned())
        .expect_err("写越狱必须被拒（只读命令测了不算，写才是真要命的那面）");
    assert!(
        !common::data_dir().join("escaped.txt").exists(),
        "越狱写必须一个字节都没落盘"
    );
}

#[test]
fn fs_read_refuses_symlink_pointing_outside_the_workspace() {
    // 第二道 jail（canonicalize 真实路径）才拦得住这一种 ——
    // 词法判定看见的 `link/passwd` 完全合法。这一条证明两道都在。
    let _ = common::data_dir();
    let outside = common::data_dir().join("outside-secret.txt");
    std::fs::write(&outside, "不该被读到").expect("应可在工作区外造诱饵");
    let link = common::workspace_dir().join("link");
    std::os::unix::fs::symlink(&outside, &link).expect("应可建软链接");

    neobot_fs_read("link".to_owned()).expect_err("软链接越狱必须被拒");
    neobot_fs_list("link".to_owned()).expect_err("列软链接越狱目录也必须被拒");
}
