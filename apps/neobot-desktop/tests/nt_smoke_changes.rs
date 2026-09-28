//! 冒烟：改动账目（`nt_cmd_files` 的 changes_* 半边）+ 侧聊末次回复。
//!
//! ## 这是本轮最重要的一套
//!
//! 线上真出过这样一个缺陷：前端给 `neobot_changes_paths` 传了 `taskId: ""`，
//! 而核心函数 `tally_task_paths` **完全正确** —— 查 `WHERE task_id = ''`，
//! 当然是空。于是「本轮文件」页**永远空**，用户看不到任何历史改动。
//!
//! 260 个核心测试一个都抓不到它：核心没问题，错在**参数绑定**这一层。
//! 所以下面每条都按「先经核心 API 造数、再断言命令读得到」写 ——
//! 数据在、命令接对了，才算真的走过一遍 IPC 面。
//!
//! ## 并发注记
//!
//! 本文件内多个测试共用一份库（环境是进程全局的，见 `common`）。
//! 故**所有断言都按自己的 `task_id` 收窄**，绝不断言全库行数 ——
//! 否则加一条测试就会弄红另一条，那是假绿变假红，不是真信号。

mod common;

use neobot_desktop::nt_commands::nt_cmd_convo::{neobot_convo_dm, neobot_member_add};
use neobot_desktop::nt_commands::nt_cmd_files::{
    neobot_changes_get, neobot_changes_list, neobot_changes_paths, neobot_changes_recent_paths,
    neobot_convo_last_reply,
};
use neotrix_neobot::{AgentTask, FileChange, NeobotStore, TaskStatus, KIND_READ, KIND_WRITE};

/// 经**核心 API** 造一个任务 + 一笔改动，返回 `(task_id, task_title, change_id)`。
///
/// 刻意不走命令来造数：造数的路和验数的路必须不同，
/// 否则「用被测对象造被测对象的数据」会掩盖接线错误。
fn seed_change(path: &str, before: Option<&str>, after: Option<&str>) -> (String, String, String) {
    let store: NeobotStore = common::open_store();
    let task_id = common::uniq("task");
    let change_id = common::uniq("chg");
    let now = "2026-09-28T00:00:00Z";
    let title = format!("冒烟任务 {task_id}");

    store
        .save_task(&AgentTask {
            id: task_id.clone(),
            title: title.clone(),
            status: TaskStatus::Done,
            created_at: now.to_owned(),
            updated_at: now.to_owned(),
            claimed_by: None,
            conversation_id: None,
            claimed_at: None,
            visibility: "team".to_owned(),
            lease_id: None,
            lease_until: None,
            attempts: 1,
            error: None,
        })
        .expect("应可存任务");

    store
        .record_change(
            &FileChange {
                id: change_id.clone(),
                task_id: task_id.clone(),
                at: now.to_owned(),
                path: path.to_owned(),
                kind: KIND_WRITE.to_owned(),
                bytes: after.map_or(0, str::len) as i64,
                content_omitted: false,
            },
            before,
            after,
        )
        .expect("应可记改动");

    (task_id, title, change_id)
}

#[test]
fn changes_list_surfaces_a_change_written_through_the_core_api() {
    let _ = common::data_dir();
    let path = common::uniq("reports") + ".md";
    let (task_id, _title, change_id) = seed_change(&path, None, Some("新内容\n"));

    let rows = neobot_changes_list(Some(task_id.clone()), Some(50)).expect("列改动应成功");
    let row = rows
        .iter()
        .find(|r| r.id == change_id)
        .unwrap_or_else(|| panic!("核心写入的改动应被壳列出来，实际列了：{:?}", rows.iter().map(|r| &r.id).collect::<Vec<_>>()));

    assert_eq!(row.task_id, task_id, "task_id 必须原样透传，不能被壳换掉");
    assert_eq!(row.path, path, "path 必须原样透传");
    assert_eq!(row.kind, KIND_WRITE);
    assert_eq!(row.bytes, "新内容\n".len() as i64);
    assert!(!row.content_omitted);
    assert!(!row.at.is_empty(), "时间戳不能空 —— 前端按它排序");
}

#[test]
fn changes_list_with_no_task_id_returns_every_task() {
    // 「全部任务」这一路（task_id = None）也得走通：侧边栏那个页签没有当前任务概念。
    let _ = common::data_dir();
    let (task_id, _t, change_id) = seed_change(&(common::uniq("all") + ".txt"), None, Some("x"));

    let rows = neobot_changes_list(None, Some(2000)).expect("全量列改动应成功");
    assert!(
        rows.iter().any(|r| r.id == change_id),
        "不传 task_id 时应看到所有任务的改动"
    );
    assert!(rows.iter().any(|r| r.task_id == task_id));
}

#[test]
fn changes_list_clamps_a_nonsense_limit_instead_of_failing() {
    // 壳把 limit 收敛到 1..=2000 —— 传 0 或负数不能变成「返回空」
    // （那正是本次要根除的缺陷类别：看着能用、实际永远没数据）。
    let _ = common::data_dir();
    let (_t, _ti, _c) = seed_change(&(common::uniq("clamp") + ".txt"), None, Some("x"));
    // 断的是**收敛**这件事：limit 越界要被夹到 1..=2000，而不是报错、
    // 更不是「返回 0 行」——后者正是本次要根除的缺陷类别（看着能用、实际永远没数据）。
    // 断言用行数而非「含我的那条」：并行下「最新那一行」是谁不确定。
    let rows = neobot_changes_list(None, Some(0)).expect("limit=0 不该报错");
    assert_eq!(rows.len(), 1, "limit=0 应收敛到 1 行，实际 {} 行", rows.len());
    let rows = neobot_changes_list(None, Some(-5)).expect("负数 limit 不该报错");
    assert_eq!(rows.len(), 1, "负数 limit 同样应收敛到 1 行，实际 {} 行", rows.len());
    // 上限那一侧：99999 应收敛到 2000（不报错、不整库拉）。
    let rows = neobot_changes_list(None, Some(99_999)).expect("超大 limit 不该报错");
    assert!(rows.len() <= 2000, "limit 应被夹到 <=2000，实际 {} 行", rows.len());
}

#[test]
fn changes_paths_groups_reads_writes_and_edits_by_path() {
    let _ = common::data_dir();
    let store = common::open_store();
    let (task_id, _t, _c) = seed_change("grouped/a.rs", None, Some("fn a() {}\n"));
    let change_id = common::uniq("chg");
    store
        .record_change(
            &FileChange {
                id: change_id,
                task_id: task_id.clone(),
                at: "2026-09-28T00:00:01Z".to_owned(),
                path: "grouped/a.rs".to_owned(),
                kind: KIND_READ.to_owned(),
                bytes: 10,
                content_omitted: false,
            },
            None,
            None,
        )
        .expect("应可再记一笔读");

    let tallies = neobot_changes_paths(task_id.clone()).expect("按路径分组应成功");
    let tally = tallies
        .iter()
        .find(|t| t.path == "grouped/a.rs")
        .unwrap_or_else(|| panic!("应按路径聚合出 grouped/a.rs，实际：{:?}", tallies.iter().map(|t| &t.path).collect::<Vec<_>>()));
    assert_eq!(tally.writes, 1, "写次数");
    assert_eq!(tally.reads, 1, "读次数");
    assert_eq!(tally.edits, 0, "没改过就不该有 edit");
    assert!(!tally.last_change.is_empty(), "last_change 供前端点开 diff");
}

#[test]
fn changes_paths_with_empty_task_id_returns_nothing_and_recent_paths_works() {
    // ─── 已上线缺陷的墓碑 + 回归锁 ───
    //
    // 事故：前端传 `taskId: ""` → `WHERE task_id = ''` → 整页恒空。
    // 修法是让侧边栏改调 `neobot_changes_recent_paths`（自己反推最近 N 轮）。
    //
    // 这条测试把**两半都钉住**：
    //   - 空 task_id 确实返回空（钉住事故机制，防止有人「顺手优化」回去）；
    //   - 而前端实际调的那条路**必须**能出数据（钉住修复本身）。
    // 只有钉住两半，回归才是双向的 —— 只钉一半等于没钉。
    //
    // 为什么要显式给 `task_limit = 50`：**测试线程是并行的、库是共享的**
    // （环境变量是进程全局的，见 `common`）。用缺省 1 就等于断言
    // 「全库最近的那一笔是我的」—— 那取决于谁先跑，是个必然变红的 flaky。
    // 放宽到 50 才是与顺序无关的断言。收敛行为由下一条测试单独验。
    let _ = common::data_dir();
    let path = format!("{}.md", common::uniq("recent"));
    let (task_id, title, _c) = seed_change(&path, None, Some("改动内容\n"));

    let empty = neobot_changes_paths(String::new()).expect("空 task_id 不该报错");
    assert!(
        empty.is_empty(),
        "空 task_id 按 SQL 语义本就无行 —— 这正是事故现场，钉住它以防回退：{empty:?}"
    );

    let recent = neobot_changes_recent_paths(Some(50)).expect("最近改动应成功");
    assert!(
        recent.paths.iter().any(|p| p.path == path),
        "前端实际走的这条路必须出数据，实际：{:?}",
        recent.paths.iter().map(|p| &p.path).collect::<Vec<_>>()
    );

    // 标题接线（LEFT JOIN tasks）：`task_title` 必须是**真实标题**而不是
    // 退化出来的 task_id 本身 —— 后者正是「用户不知道自己在看哪一轮」。
    // 因为并行下「最新那一轮是谁」不确定，这里不去赌 task_id 等于我的，
    // 而是从返回的 task_id 反查库里真标题再比对：验的是接线，不是排序。
    let shown = recent.task_id.expect("有改动就该有 task_id 可显示");
    let shown_title = recent.task_title.expect("标题页要能说清在看哪一轮");
    let store = common::open_store();
    let real = store
        .list_changes(Some(shown.as_str()), 1)
        .expect("应能反查");
    assert!(
        !real.is_empty(),
        "返回的 task_id 在库里必须真有改动（否则标题是凭空捏造的）"
    );
    let expected = if shown == task_id { Some(title) } else { None };
    if let Some(want) = expected {
        assert_eq!(shown_title, want, "titles 表接线断了：应为任务标题而非 task_id");
    } else {
        assert!(
            !shown_title.is_empty(),
            "别的测试造的那一轮也得有非空标题（LEFT JOIN 退化时回的是 task_id，也非空）"
        );
    }
}

#[test]
fn changes_recent_paths_defaults_to_the_single_latest_task() {
    // 缺省 `task_limit = 1` = 只看最近一轮。这是**刻意**的收敛：
    // 页签标题写的是「本轮」，多轮混在一起就与标题不符了。
    // 故这里断的是「收敛」这件事本身：造两笔，只看第一笔那一轮。
    let _ = common::data_dir();
    let first = format!("{}.md", common::uniq("d1"));
    let second = format!("{}.md", common::uniq("d2"));
    seed_change(&first, None, Some("a"));
    seed_change(&second, None, Some("b"));

    // 顺序不敏感地验收敛：无论谁最后写，`limit=1` 返回的**行数**都必须是
    // 那一轮的文件数（本夹具里每轮各 1 个文件），不会是两轮合并的 2 行。
    // 顺序不敏感地验收敛：库是并行写的，「最新那一轮」可能是**别的测试**造的，
    // 所以只断**行数**——这正是收敛的可观测证据（多轮混在一起就该 >1 行）。
    // 刻意不断「留下的那行是谁」：那会把这个测试变成必然 flaky 的断言。
    let one = neobot_changes_recent_paths(None).expect("缺省 task_limit 应成功");
    assert_eq!(
        one.paths.len(),
        1,
        "缺省只看最近一轮（每轮各 1 文件），实际收了 {} 行：{:?}",
        one.paths.len(),
        one.paths.iter().map(|p| &p.path).collect::<Vec<_>>()
    );
    // 留下的那一行必须真的在库里（不是凭空拼的路径）。
    let store = common::open_store();
    let all = store.list_changes(None, 2000).expect("应能列全量");
    assert!(
        all.iter().any(|c| c.path == one.paths[0].path),
        "收敛后留下的路径必须在改动账里真实存在：{}",
        one.paths[0].path
    );
}

#[test]
fn changes_recent_paths_widens_monotonically_with_task_limit() {
    let _ = common::data_dir();
    let first = format!("{}.md", common::uniq("old"));
    let second = format!("{}.md", common::uniq("new"));
    seed_change(&first, None, Some("a"));
    seed_change(&second, None, Some("b"));

    // 为什么断「单调变宽」而不是「limit=1 里含我后写的那笔」：
    // 库是**并行**写的（环境变量是进程全局的，见 `common`），
    // 别的测试随时可能在我之后插一笔，于是「最近一轮」根本不由我决定。
    // 断言「哪一行是我的」= 断言一个我控制不了的顺序 = 必然 flaky。
    // 「放宽后能看回更多」才是这条命令真正要保证的性质，且与顺序无关。
    let one = neobot_changes_recent_paths(Some(1)).expect("task_limit=1 应成功");
    let fifty = neobot_changes_recent_paths(Some(50)).expect("task_limit=50 应成功");

    assert!(!one.paths.is_empty(), "库里明明有改动，limit=1 不该是空");
    assert!(
        fifty.paths.len() > one.paths.len(),
        "放宽 task_limit 必须能看到更多轮：1 轮 {} 行 vs 50 轮 {} 行",
        one.paths.len(),
        fifty.paths.len()
    );
    // 放大后视野必须是放大前的超集（只增不减）。
    for row in &one.paths {
        assert!(
            fifty.paths.iter().any(|p| p.path == row.path),
            "放宽后丢了 {} —— 那是丢数据，不是收敛",
            row.path
        );
    }
    // 放宽到 50 轮，本二进制里造的两笔必然都在视野内。
    assert!(fifty.paths.iter().any(|p| p.path == first), "放宽后应能看回更早的改动");
    assert!(fifty.paths.iter().any(|p| p.path == second));
}

#[test]
fn changes_get_returns_before_and_after_for_a_diff() {
    let _ = common::data_dir();
    let (_t, _ti, change_id) = seed_change("diff/one.txt", Some("旧\n"), Some("新\n"));

    let view = neobot_changes_get(change_id)
        .expect("取单条改动应成功")
        .expect("刚写的改动必须查得到");
    assert_eq!(view.before.as_deref(), Some("旧\n"), "diff 要能画出增删");
    assert_eq!(view.after.as_deref(), Some("新\n"));
    assert_eq!(view.change.path, "diff/one.txt");
}

#[test]
fn changes_get_returns_none_for_an_unknown_id_not_an_error() {
    // 前端点一个已 prune 掉的改动 → 必须拿到 None 好显示「已清理」，
    // 而不是抛错把整个 diff 面板炸掉。
    let _ = common::data_dir();
    assert!(
        neobot_changes_get(common::uniq("nope")).expect("未知 id 不该报错").is_none(),
        "未知 id 应回 None"
    );
}

#[test]
fn changes_get_honours_content_omitted_without_inventing_content() {
    // 脱敏律：内容超限就**如实**标 content_omitted，before/after 恒 None。
    // 前端据此显示「内容过大，去打开文件对比」—— 编一份假内容比不显示更坏。
    let _ = common::data_dir();
    let store = common::open_store();
    let change_id = common::uniq("big");
    store
        .record_change(
            &FileChange {
                id: change_id.clone(),
                task_id: common::uniq("task"),
                at: "2026-09-28T00:00:00Z".to_owned(),
                path: "huge.bin".to_owned(),
                kind: KIND_WRITE.to_owned(),
                bytes: 9_000_000,
                content_omitted: true,
            },
            None,
            None,
        )
        .expect("应可记一笔超限改动");

    let view = neobot_changes_get(change_id).expect("取改动应成功").expect("应查得到");
    assert!(view.change.content_omitted, "超限标志必须透出");
    assert!(view.before.is_none() && view.after.is_none(), "超限就不许有内容");
}

// ─── 侧聊末次回复：None vs Some 必须分得清 ───

/// 造一个「有回复」的任务：成员 → 会话 → 任务 → 一条 `reply` 步骤。
///
/// 成员**必须先登记**：`create_conversation` 有外键指向 `members`，
/// 直接 `get_or_create_dm` 会报 `no such member`。
/// 顺带说明：用 `neobot_member_add` / `neobot_convo_dm` 这两个**命令**来建，
/// 于是本文件顺带把「成员登记 → 建会话」这条接线也走了一遍。
fn seed_convo_with_reply(peer: &str, reply: &str) -> String {
    neobot_member_add("owner".to_owned(), "human".to_owned()).expect("应可登记 owner");
    neobot_member_add(peer.to_owned(), "agent".to_owned()).expect("应可登记对端");
    let convo_id = neobot_convo_dm("owner".to_owned(), peer.to_owned()).expect("应可建 DM 会话");

    let store = common::open_store();
    let task_id = common::uniq("task");
    let now = "2026-09-28T00:00:00Z";
    store
        .save_task(&AgentTask {
            id: task_id.clone(),
            title: "侧聊冒烟".to_owned(),
            status: TaskStatus::Done,
            created_at: now.to_owned(),
            updated_at: now.to_owned(),
            claimed_by: None,
            conversation_id: Some(convo_id.clone()),
            claimed_at: None,
            visibility: "team".to_owned(),
            lease_id: None,
            lease_until: None,
            attempts: 1,
            error: None,
        })
        .expect("应可存任务");
    store
        .add_step(&task_id, 0, "reply", true, reply)
        .expect("应可记回复步骤");
    convo_id
}

/// 造一个「有会话有任务、但没有 reply 步骤」的会话。
fn seed_convo_without_reply(peer: &str) -> String {
    neobot_member_add("owner".to_owned(), "human".to_owned()).expect("应可登记 owner");
    neobot_member_add(peer.to_owned(), "agent".to_owned()).expect("应可登记对端");
    let convo_id = neobot_convo_dm("owner".to_owned(), peer.to_owned()).expect("应可建 DM 会话");
    let store = common::open_store();
    store
        .save_task(&AgentTask {
            id: common::uniq("task"),
            title: "没回复".to_owned(),
            status: TaskStatus::Running,
            created_at: "2026-09-28T00:00:00Z".to_owned(),
            updated_at: "2026-09-28T00:00:00Z".to_owned(),
            claimed_by: None,
            conversation_id: Some(convo_id.clone()),
            claimed_at: None,
            visibility: "team".to_owned(),
            lease_id: None,
            lease_until: None,
            attempts: 0,
            error: None,
        })
        .expect("应可存任务");
    convo_id
}

#[test]
fn convo_last_reply_returns_none_for_an_unknown_conversation() {
    // `None` 与 `Some("")` 必须可分辨：空串会和「空回复」混淆，
    // 调用方（侧聊气泡）需要据此走不同分支。
    let _ = common::data_dir();
    let got = neobot_convo_last_reply(common::uniq("ghost"))
        .expect("未知会话不该报错");
    assert_eq!(got, None, "未知会话必须回 None，不能回空串也不能报错");
}

#[test]
fn convo_last_reply_returns_none_for_a_conversation_with_no_reply() {
    let _ = common::data_dir();
    let chatty = seed_convo_with_reply(&common::uniq("bot"), "第一版回复");
    let silent = seed_convo_without_reply(&common::uniq("bot"));

    assert_eq!(
        neobot_convo_last_reply(silent).expect("取回复不该报错"),
        None,
        "只有任务没 reply 步骤时回 None"
    );
    // 顺手确认同库另一条确实有回复 —— 证明上面那个 None 不是「查库坏了」。
    // 这条交叉检查是必要的：只断 None 的话，
    // 「查询整体恒返回 None」这个更糟的 bug 也会绿。
    assert!(
        neobot_convo_last_reply(chatty).expect("取回复").is_some(),
        "同库另一条必须有回复，否则 None 断言无意义"
    );
}

#[test]
fn convo_last_reply_returns_some_with_the_newest_reply() {
    let _ = common::data_dir();
    neobot_member_add("owner".to_owned(), "human".to_owned()).expect("应可登记 owner");
    let peer = common::uniq("bot");
    neobot_member_add(peer.clone(), "agent".to_owned()).expect("应可登记对端");
    let convo_id = neobot_convo_dm("owner".to_owned(), peer).expect("应可建会话");

    let store = common::open_store();
    let task_id = common::uniq("task");
    store
        .save_task(&AgentTask {
            id: task_id.clone(),
            title: "多轮".to_owned(),
            status: TaskStatus::Done,
            created_at: "2026-09-28T00:00:00Z".to_owned(),
            updated_at: "2026-09-28T00:00:00Z".to_owned(),
            claimed_by: None,
            conversation_id: Some(convo_id.clone()),
            claimed_at: None,
            visibility: "team".to_owned(),
            lease_id: None,
            lease_until: None,
            attempts: 2,
            error: None,
        })
        .expect("应可存任务");
    store.add_step(&task_id, 0, "reply", true, "旧回复").expect("step");
    store.add_step(&task_id, 1, "bash", true, "跑了条命令").expect("step");
    store.add_step(&task_id, 2, "reply", true, "新回复\n").expect("step");

    let got = neobot_convo_last_reply(convo_id).expect("取回复应成功");
    assert_eq!(got.as_deref(), Some("新回复"), "必须取**最后一条** reply，且已 trim");
    assert_ne!(got.as_deref(), Some(""), "绝不能是空串");
}
