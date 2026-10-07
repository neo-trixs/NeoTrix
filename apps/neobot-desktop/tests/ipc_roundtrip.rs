//! IPC **真实分发**往返测试 —— 抓「前端载荷键 ↔ Rust 形参」不匹配。
//!
//! ## 为什么静态门不够（2026-10-02 P0 复盘）
//!
//! `scripts/ops/neobot-check-ipc-keys.mjs` 是**静态**门：它从 Rust 源码抽形参名、
//! 从 TS 源码抽载荷键，然后**自己按 `to_lower_camel_case` 算**期望键。
//! ⛔ 但它证明不了「Tauri 运行时真的这么取键」。
//! 本文件补的就是这一层：走 **Tauri 自己的命令分发 + 自己的反序列化**
//! （`tauri-2.12.0/src/ipc/command.rs:109` 的 `v.get(self.key)`、
//! 宏生成的 camelCase 键），**没有桩**。
//!
//! ## 为什么所有 UI 门都抓不到那个 P0
//!
//! `neobot-ui-smoke.mjs` / `neobot-check-*.mjs` 各自
//! `invoke = async (cmd, args) => …` **直接读 `args.xxx` 返 fixture**
//! ⇒ 桩是 JS，键名怎么写都能读到 ⇒ **结构上就抓不到键名不匹配**。
//! `neobot-ui-smoke` 的 `when_not` 也写明了「改 Rust 后端或 Tauri 桥时不要用它」。
//!
//! ## ⛔ 为什么必须在 `tests/` 独立二进制（本目录注释抄自 `data_dir_env.rs`）
//!
//! ① `NEOBOT_DATA_DIR` 是**进程级**环境变量；放在 `#[cfg(test)]` 里会让
//!    **所有并行跑的测试**看见我们的临时目录。
//! ② 本二进制内多组测试共享同一进程 ⇒ 还需 `static LOCK` 串行化
//!    （`data_dir_env.rs` 的教训：「进程隔离解决的是别的测试看不见，
//!    解决不了本进程内的互相踩」）。
//!
//! ## 为什么这套测试**零网络**
//!
//! `neobot_send` 的落库（`commands.rs:247-253` 的问 + `:263-268` 的答）
//! **早于** `agent_run` 的真实 HTTP；而未配对时 `agent_run` 必返
//! `Err("core unpaired (pair first)")`。
//! ⇒ **对键与错键的返回值完全相同**，差别只在 `neobot.db` 里有没有那行 `user`。
//! 这正是「界面正常显示回复、但一条都没落库」这个最毒的形态的最小自动化复现。

use std::sync::Mutex;

use neobot_desktop::commands::AppState;
use serde_json::json;
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{get_ipc_response, mock_builder, mock_context, noop_assets, INVOKE_KEY};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
use tauri::webview::InvokeRequest;

// 本二进制内全部测试串行（理由见文件头 ②）。
static LOCK: Mutex<()> = Mutex::new(());

fn serial<T>(f: impl FnOnce() -> T) -> T {
    let _g = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    f()
}

/// 官方示例要求 `url` 必须是 `tauri://localhost`
/// （`tauri-2.12.0/src/test/mod.rs:220-224`）。传错会走 ACL 拒绝分支。
const LOCAL_URL: &str = "tauri://localhost";

/// ⛔ `get_ipc_response` **无超时**（`src/test/mod.rs:309` 的 `rx.recv()`）：
///    `cmd`/`invoke_key` 不对时 `on_message` 直接 `return`、**不回调**
///    ⇒ 测试会**永久挂起**而不是失败。所以每个请求都用本函数发，
///    且每条测试**第一条**请求都必须是「已知能通」的命令。
fn req(cmd: &str, body: InvokeBody) -> InvokeRequest {
    InvokeRequest {
        cmd: cmd.into(),
        callback: CallbackFn(0),
        error: CallbackFn(1),
        url: LOCAL_URL.parse().expect("本地 URL"),
        body,
        headers: Default::default(),
        invoke_key: INVOKE_KEY.to_owned(),
    }
}

/// 造一个带 `AppState` 的 mock app + 一个可发起调用的窗口。
///
/// 必须 `.manage(AppState)` —— 命令签名里的
/// `tauri::State<'_, AppState>` 靠 `try_state()` 取（`tauri/src/state.rs:59-67`），
/// 不 manage 时所有命令都会返回
/// `state not managed for field 'state'`。
/// ⛔ **刻意不注册** autostart / clipboard / opener / notification 四个插件：
///    它们会写真实 LaunchAgent plist、改用户真实剪贴板、真开浏览器/真弹通知。
///    不注册时 `app.autolaunch()` 之类返回可读的 `plugin not found`，
///    这本身就是可断言的行为，且零副作用。
fn mock_app() -> tauri::App<tauri::test::MockRuntime> {
    let app = mock_builder()
        .manage(AppState::default())
        // 注册表：这里**刻意只注册被测的 4 条**，而不是 `neobot_commands!()` 全量。
        //
        // ⛔⛔ 为什么不能直接用全量（实测踩到）：带 `AppHandle` / `tauri::WebviewWindow`
        //    形参的命令（如 `pet.rs:299 move_pet_window(app: AppHandle, …)`）
        //    在 `AppHandle` **不带泛型参数**时签名被固定成 `AppHandle<Wry>`
        //    ⇒ 而 `CommandArg for AppHandle<R>`（`tauri-2.12.0/src/app.rs:487`）
        //    要求 `R = MockRuntime` ⇒ **编译期 E0277**。
        //    解法本应是给命令签名加 `<R: Runtime>` 泛型 —— 那是动 74 条命令签名的
        //    生产改动，超出本轮范围。⇒ 本轮只注册**不依赖窗口/句柄**的命令。
        //
        // 「注册漏了会红」这个保证由 `注册表含全部被测命令` 一条**源码级**断言补上。
        .invoke_handler(tauri::generate_handler![
            neobot_desktop::commands::neobot_api_specs,
            neobot_desktop::commands::neobot_member_add,
            neobot_desktop::commands::neobot_convo_group,
            neobot_desktop::commands::neobot_convo_messages_page,
            neobot_desktop::commands::neobot_send,
        ])
        .build(mock_context(noop_assets()))
        .expect("mock app");
    let w = WebviewWindowBuilder::new(&app, "main", WebviewUrl::App("index.html".into()))
        .build()
        .expect("主窗口");
    // ⓘ mock runtime 下 `set_position` 是空操作、`outer_position()` 恒 `(0,0)`
    // （`tauri-2.12.0/src/test/mock_runtime.rs:739-741,1049-1051`），
    // ⇒ 窗口位置类断言只能验「调通了 + 落库值」，
    //    不能验「窗口真的动了」。这条限制已写进对应测试注释。
    let _ = w;
    app
}

// ⓘ 关于上下文：本文件用官方的 `mock_context(noop_assets())`
//   （`tauri-2.12.0/src/test/mod.rs:185` 的 `mock_app()` 就是这么造的），
//   ⛔ **不用** `generate_context!()`：那会去读 `tauri.conf.json` 的 `frontendDist`，
//   而本 harness 只验 IPC 键名，不需要真实前端产物。
//   ⛔ 代价（诚实声明）：mock context 的 `runtime_authority` 使 `has_app_acl=false`
//   （`src/test/mod.rs:143`）⇒ **ACL 校验整条被跳过**
//   ⇒ 本 harness **抓不到**「命令没进 capabilities」这类缺陷，那需要真 context。

/// 起一个独立数据目录并设 `NEOBOT_DATA_DIR`。
///
/// 依据 `crates/neotrix-neobot/src/nt_config.rs:104-111`：
/// 设了它就把 `neobot.db` / `usage.json` / `MEMORY.md` / `app-config.json`
/// 全部重定向。⇒ **不需要**（也无法）用 `app.path()` 重定向。
fn with_temp_data_dir<T>(f: impl FnOnce(&std::path::Path) -> T) -> T {
    serial(|| {
        let dir = tempdir::TempDir::new("nb-ipc").expect("临时目录");
        let prev = std::env::var("NEOBOT_DATA_DIR").ok();
        std::env::set_var("NEOBOT_DATA_DIR", dir.path());
        let out = f(dir.path());
        match prev {
            Some(v) => std::env::set_var("NEOBOT_DATA_DIR", v),
            None => std::env::remove_var("NEOBOT_DATA_DIR"),
        }
        out
    })
}

/// canary：先用一条**无参**命令证明「这个 harness 能走通真实分发」。
///
/// ⛔⛔ 这条测试是**整套的生死判据**：`get_ipc_response` 无超时，
///    若 harness 配错（如 `invoke_key`/`url` 不对），每个测试都会**永久挂起**
///    而不是失败 ⇒ CI 上表现为「卡住」，极难诊断。
/// ⇒ 所以把它写成一条独立测试：它红 ⇒ 说明问题在 harness，不在业务命令。
#[test]
fn canary_无参命令能走通真实分发() {
    with_temp_data_dir(|_| {
        let app = mock_app();
        let w = app.get_webview_window("main").expect("窗口");
        let resp = get_ipc_response(&w, req("neobot_api_specs", InvokeBody::Json(json!({}))));
        let body = resp.expect("无参命令不应失败");
        let specs: serde_json::Value = body.deserialize().expect("反序列化");
        // ⓘ 实测它返回 `{summary, specs, upstream_unlisted}` 对象，**不是**裸数组。
        let arr = specs
            .get("specs")
            .and_then(|v| v.as_array())
            .unwrap_or_else(|| panic!("应含 `specs` 数组，实际：{specs}"));
        assert!(!arr.is_empty(), "`specs` 数组不应为空");
        // 再断一条：注册表里确实含本文件要测的命令（证明接的是真注册表）。
        let names: Vec<&str> = arr
            .iter()
            .filter_map(|v| v.get("name").and_then(|n| n.as_str()))
            .collect();
        assert!(
            names.contains(&"neobot_send") && names.contains(&"neobot_convo_messages_page"),
            "契约清单应含被测命令，实际含：{names:?}"
        );
    })
}

/// P0 回归：`**载荷键写错 ⇒ 消息不落库**`，而返回值看不出差别。
///
/// 复现 `2026-10-02` 修掉的 P0：`neobot_send(convo_id: Option<String>, text: String)`
/// 的 `convo_id` 是 `Option` ⇒ 键名错时 Tauri 走
/// `deserialize_option` 的 `visitor.visit_none()`（`ipc/command.rs:145-153`）
/// ⇒ **静默变 None、不报错**，于是 `commands.rs:247-253` 的问落库被跳过。
#[test]
fn neobot_send_键名错则问不落库且返回值不变() {
    with_temp_data_dir(|dir| {
        let app = mock_app();
        let w = app.get_webview_window("main").expect("窗口");
        make_convo(&w, "t");
        let convo = read_convo_id(dir);

        // ① ⛔ 错键：snake_case。期望 `convoId`，这里发 `convo_id`。
        let bad = get_ipc_response(
            &w,
            req(
                "neobot_send",
                InvokeBody::Json(json!({ "convo_id": convo, "text": "hi" })),
            ),
        );
        let bad_err = bad.expect_err("未配对时 agent_run 必失败");

        // ② 对键：camelCase。
        let good = get_ipc_response(
            &w,
            req(
                "neobot_send",
                InvokeBody::Json(json!({ "convoId": convo, "text": "hi" })),
            ),
        );
        let good_err = good.expect_err("未配对时 agent_run 必失败");

        // 核心断言：**两者返回值完全相同** ⇒ 界面看不出差别。
        assert_eq!(
            bad_err, good_err,
            "错键与对键的返回值必须相同（这正是它当初能带着全部门绿活下来的原因）；\
             若不同，说明行为已变，本测试的对比基准要重估"
        );

        // 然后才是真正的差别：**落库了没有**。
        let users = count_user_rows(dir, &convo);
        assert_eq!(
            users, 1usize,
            "camelCase 键 ⇒ 问必须已落库；实际 user 行数 = {users}"
        );

        // ⛔ 单独再发一次错键，断言它**不**落库（正向可观测的差异）。
        let before = count_user_rows(dir, &convo);
        let _ = get_ipc_response(
            &w,
            req(
                "neobot_send",
                InvokeBody::Json(json!({ "convo_id": convo, "text": "hi again" })),
            ),
        );
        assert_eq!(
            count_user_rows(dir, &convo),
            before,
            "snake_case 键 ⇒ convo_id 静默 None ⇒ 问必须**不**落库（这正是那个 P0）"
        );
    })
}

/// 非 `Option` 形参缺键 ⇒ **硬失败**，错误串里带期望键名。
///
/// 与上一个测试构成对照：`Option` 形参**静默**、非 `Option` **吵**。
#[test]
fn neobot_convo_messages_page_键名错则硬失败并点名期望键() {
    with_temp_data_dir(|dir| {
        let app = mock_app();
        let w = app.get_webview_window("main").expect("窗口");
        make_convo(&w, "t2");
        let convo = read_convo_id(dir);

        // ① 对键 ⇒ 成功（哪怕是空列表）
        let ok = get_ipc_response(
            &w,
            req(
                "neobot_convo_messages_page",
                InvokeBody::Json(json!({ "convoId": convo })),
            ),
        )
        .expect("camelCase 键应成功");
        let page: serde_json::Value = ok.deserialize().expect("反序列化");
        // ⚠️ **返回形状是 `{messages,hasMore,nextSeq}`，不是裸数组。**
        //
        // ⛔⛔ 这条断言在分页命令落地后**一直红着**，直到本轮才被发现 ——
        //    而它红着的方式恰恰是最坏的一种：**测试本身过期**。
        //    它断言 `rows.is_array()`，可 `MessagePage` 从来不是数组
        //    （`api.rs:128` 早就把这记成「旧契约写 `ChatMessage[]`
        //    ⇒ **形状本身就是错的**」并改了契约表，**唯独漏了这个测试**）。
        //    ⇒ 这是 `api.rs:123` 那条「改名后下游没跟」的**第 5 次复发**，
        //    只不过这次漏的下游是**测试自己**。
        //
        //    ⛔ 危害不止「2 条红」：`cargo test -p neobot-desktop` 因此
        //    **在 HEAD 上就已经是红的**（实测 2026-10-07）⇒ 一个长期红的
        //    套件会让人习惯性忽略它的输出，于是它**真正该抓的东西**
        //    （键名不匹配落库）也跟着一起被忽略。
        //    ⇒ 所以这里修的不是断言，是**这个套件的可用性**。
        assert!(
            page.get("messages").and_then(|v| v.as_array()).is_some(),
            "应返回 `{{messages,hasMore,nextSeq}}`，实际：{page}"
        );
        assert!(page.get("hasMore").is_some(), "缺 `hasMore`：{page}");
        assert!(page.get("nextSeq").is_some(), "缺 `nextSeq`：{page}");

        // ② ⛔ 错键 ⇒ 硬失败，且错误串**点名期望的键**。
        let bad = get_ipc_response(
            &w,
            req(
                "neobot_convo_messages_page",
                InvokeBody::Json(json!({ "convo_id": convo })),
            ),
        );
        let msg = bad.expect_err("snake_case 键应失败");
        let text = msg.as_str().unwrap_or_default().to_owned();
        assert!(
            text.contains("convoId"),
            "错误信息应点名期望键 `convoId`，实际：{text}"
        );
    })
}

/// `seq` 必须真的出现在 IPC 返回里（否则前端算不出下一页游标）。
#[test]
fn 消息返回带seq游标() {
    with_temp_data_dir(|dir| {
        let app = mock_app();
        let w = app.get_webview_window("main").expect("窗口");
        make_convo(&w, "t3");
        let convo = read_convo_id(dir);
        // 写一条消息（直接用 store，避免依赖 agent_run）。
        {
            let store = neotrix_neobot::NeobotStore::open(&db_path(dir)).expect("开库");
            store
                .append_message(&convo, "user", "hello")
                .expect("写消息");
        }
        let ok = get_ipc_response(
            &w,
            req(
                "neobot_convo_messages_page",
                InvokeBody::Json(json!({ "convoId": convo })),
            ),
        )
        .expect("读历史");
        let page: serde_json::Value = ok.deserialize().expect("反序列化");
        // ⚠️ 同上：形状是 `{messages,…}`，不是裸数组（这正是上一条修的那个坑）。
        let rows = page
            .get("messages")
            .and_then(|v| v.as_array())
            .unwrap_or_else(|| panic!("应含 `messages` 数组，实际：{page}"));
        assert_eq!(rows.len(), 1, "应有一条消息");
        assert!(
            rows[0].get("seq").is_some(),
            "ChatMessage 必须带 `seq`（游标），实际：{}",
            rows[0]
        );
    })
}

// ── 小工具 ──

/// 库路径口径**照抄**  的 `open_store()`
///    （`dir.join("neobot.db")`）—— 必须与命令侧指向**同一个文件**，
///    否则测试会读到一个空库、并误判「没落库」。
fn db_path(dir: &std::path::Path) -> String {
    dir.join("neobot.db")
        .to_str()
        .expect("路径是合法 UTF-8")
        .to_owned()
}

/// 建一个会话（先登记成员，否则库侧拒 `no such member`）。
///
/// 走**真实 IPC** 而不是直接调 store ⇒ 顺带覆盖了 `neobot_convo_group` 的取键。
fn make_convo(w: &tauri::WebviewWindow<tauri::test::MockRuntime>, title: &str) {
    let _ = get_ipc_response(
        w,
        req(
            "neobot_member_add",
            InvokeBody::Json(json!({ "id": "neo", "kind": "human" })),
        ),
    )
    .expect("登记成员");
    let r = get_ipc_response(
        w,
        req(
            "neobot_convo_group",
            InvokeBody::Json(json!({ "title": title, "members": ["neo"] })),
        ),
    );
    assert!(r.is_ok(), "建会话应成功，实际：{:?}", r.err());
}

/// 从库里读出唯一那个会话 id。
fn read_convo_id(dir: &std::path::Path) -> String {
    let store = neotrix_neobot::NeobotStore::open(&db_path(dir)).expect("开库");
    let list = store.list_conversations().expect("列会话");
    list.first()
        .map(|c| c.id.clone())
        .expect("应至少有一个会话")
}

/// 数某个会话里 `role='user'` 的消息条数 —— 这是「问有没有落库」的**唯一**可观测信号。
///
/// ⛔ 刻意**不**直接查 SQLite：`rusqlite` 不是本 crate 的依赖，
///    加一条 dev-dependency 只为测试去绕过库 API 不划算。
///    ⓰ 用 `list_messages` 当基准的代价是它属本次改动过的读侧；
///    但本测试断言的是「**有没有写进库**」，与分页逻辑无关 ⇒ 可接受。
fn count_user_rows(dir: &std::path::Path, convo_id: &str) -> usize {
    let store = neotrix_neobot::NeobotStore::open(&db_path(dir)).expect("开库");
    store
        .list_messages(convo_id)
        .expect("读回")
        .iter()
        .filter(|m| m.role == "user")
        .count()
}

/// 补上「子集注册表」丢掉的保证：被测的 4 条命令**必须仍在全量注册表里**。
///
/// ⛔ 为什么要源码级断言而不是直接展开 `neobot_commands!()`：
///    见 `mock_app()` 里那条注释 —— 全量展开在 `MockRuntime` 下编译不过。
///    ⇒ 这里读 `lib.rs` 的宏体，断言命令名**逐字**在列。
///    ⓰ 行号/格式漂移会让本测试假红，所以断言的是**命令名子串**，不是行号。
#[test]
fn 注册表含全部被测命令() {
    let lib = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/lib.rs"
    ))
    .expect("读 lib.rs");
    let start = lib.find("macro_rules! neobot_commands").expect("宏存在");
    let body = &lib[start..];
    for cmd in [
        "neobot_api_specs",
        "neobot_convo_group",
        "neobot_convo_messages_page",
        "neobot_send",
    ] {
        assert!(
            body.contains(cmd),
            "⛔ `{cmd}` 已不在 `neobot_commands!` 注册表里 ⇒ 真 app 会报 \
             `Command {cmd} not found`，而本 harness 的子集注册表仍在测它（假绿）"
        );
    }
}
