//! 冒烟：桌面聊天框的斜杠指令分流（`nt_cmd_run` 的「斜杠指令分流」一节）。
//!
//! ## 要修的缺陷
//!
//! 桌面聊天走 `neobot_run_stream`，**不经过** `nt_channel_cmd::parse`
//! （`DESIGN-CHANNEL-DISPATCH.md` §0.4 第 2 条已查实）。于是用户在聊天框里
//! 打 `/stop`，这串字符会被**当成字面文本发给模型**，模型回一句「我没有停止
//! 功能」—— 把控制指令当自然语言，UX 上等于这四个命令在桌面不存在。
//!
//! ## 覆盖的缺陷类别
//!
//! 1. **控制指令漏进跑轮**：本地指令必须**不进跑轮**。断言不是「有回复」，
//!    而是「引擎没被调用」—— 用**跑轮一定会写 task 行**这个事实当探针
//!    （`NeobotStore::list_tasks`），外加 `Channel` 上**没有任何 `step` 事件**。
//! 2. **回执撒谎**：`/stop` 做不到（跑轮同步、无 stop hook），回执必须说清
//!    停不了，且**不得**出现「已停止」这类虚假承诺。这条是**反向断言**
//!    （断言坏措辞**不在**），不是断言好措辞在场。
//! 3. **`/` 开头的普通词被吞**：用户发个路径（`/usr/bin/env`）不该被当指令。
//!    桌面必须与 IM **同一口径**，故直接拿 `nt_channel_cmd::parse` 做等价断言。
//! 4. **两端口径漂移**：`/help` 与 `/stop` 的文案必须与 IM 同源。`/help` 的
//!    「同源」是**三方钉死**的：同一渠道名下两端逐字同文（各走各的生产入口）、
//!    渠道名不同则差异**只在**自报家门那一行、且 help 自述的每条动词都真能被
//!    `parse` 认得。`/stop` 拿 `execute` 的**真输出**逐字比。注意 `help` 两端
//!    **本就不该**逐字相等（各自自报家门），把它当相等断言来测就是恒真。
//!
//! ## 网络
//!
//! 全程不碰网络。夹具把引擎钉死 `echo`（`tests/common/mod.rs`），
//! 所以「进了跑轮」与「没进跑轮」都能从输出里读出来，不需要真模型。

mod common;

use std::sync::{Arc, Mutex};

use neobot_desktop::nt_commands::nt_cmd_run::{
    DESKTOP_CHANNEL_NAME, LOCAL_COMMAND_STATUS, StreamEvent, intercept_local_command,
    neobot_run_stream,
};
use neotrix_neobot::nt_channel_cmd::{self, Command};
use neotrix_neobot::nt_store::BotRow;
use neotrix_neobot::NeobotStore;
use serde_json::Value;

/// 起一条真 `tauri::ipc::Channel`，把每条事件按**线上形状**（`serde_json::Value`）
/// 收进 `Vec`。
///
/// 收 `Value` 而不是收回 `StreamEvent`：`StreamEvent` 只 `derive(Serialize)`，
/// 且真正要验的是**前端看到的那份**（`ev.kind === "delta"`），故按线上 JSON 断言。
fn collecting_channel() -> (tauri::ipc::Channel<StreamEvent>, Arc<Mutex<Vec<Value>>>) {
    let seen: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = seen.clone();
    let channel = tauri::ipc::Channel::new(move |body| {
        if let tauri::ipc::InvokeResponseBody::Json(raw) = body {
            if let Ok(value) = serde_json::from_str::<Value>(&raw) {
                sink.lock().expect("收集器锁").push(value);
            }
        }
        Ok(())
    });
    (channel, seen)
}

/// 真调一次 `neobot_run_stream`（**不是**直接调分流函数），
/// 返回 `(Channel 上的全部事件, 命令返回值)`。
fn send(text: &str, convo_id: &str) -> (Vec<Value>, neobot_desktop::nt_commands::NeobotRunResult) {
    let (channel, seen) = collecting_channel();
    let out = tauri::async_runtime::block_on(neobot_run_stream(
        text.chars().take(24).collect::<String>(),
        text.to_owned(),
        "neo".to_owned(),
        Some(convo_id.to_owned()),
        None,
        None,
        channel,
    ))
    .expect("跑轮命令应成功返回（本地指令分支不得报错）");
    let events = seen.lock().expect("收集器锁").clone();
    (events, out)
}

/// 事件序列的 `kind` 列表（前端 `ev.kind` 的那一份）。
fn kinds(events: &[Value]) -> Vec<String> {
    events
        .iter()
        .map(|ev| ev["kind"].as_str().unwrap_or_default().to_owned())
        .collect()
}

/// 把所有 `delta` 的正文拼起来（前端 `streamRaw` 的累积结果）。
fn deltas(events: &[Value]) -> String {
    events
        .iter()
        .filter(|ev| ev["kind"] == "delta")
        .filter_map(|ev| ev["text"].as_str().map(str::to_owned))
        .collect()
}

/// 这个会话里所有 task 的标题（跑轮**一定**会写一行 task，标题取自调用方给的
/// `title`，而桌面那个 `title` 就是消息前 24 字 —— 故它是「这一轮确实跑过」的
/// 精确探针，且与别的测试不串味）。
fn task_titles(store: &NeobotStore, convo_id: &str) -> Vec<String> {
    store
        .list_tasks(200)
        .unwrap_or_default()
        .into_iter()
        .filter(|task| task.conversation_id.as_deref() == Some(convo_id))
        .map(|task| task.title)
        .collect()
}

/// 一个只属于本测试的会话（免得 task/会话断言与别的测试串味）。
fn fresh_convo(title: &str) -> String {
    let _ = common::data_dir();
    let store = common::open_store();
    store
        .create_conversation("dm", title, &[])
        .expect("应能建测试会话")
}

/// `BotRow` 夹具：**只在测试里**造。
///
/// 存在的唯一理由是第 4 条缺陷 —— 拿 `nt_channel_cmd::execute` 的**真输出**
/// 逐字钉住桌面 `/stop` 的文案，防止两端口径漂移。
/// 生产代码里**不许**出现 `BotRow`（见 `nt_cmd_run.rs` 的设计律 3）。
fn bot_fixture() -> BotRow {
    BotRow {
        channel: "tg".to_owned(),
        bot_id: "1".to_owned(),
        alias: String::new(),
        token_env: "NEOBOT_T".to_owned(),
        conversation_id: None,
        model: String::new(),
        allow_list: String::new(),
        created_at: String::new(),
        last_seen: None,
    }
}

/// 语料：四个动词 + 中文别名 + 大小写 + **不该被当指令的普通词**。
///
/// 最后四个是重点：`/usr/bin/env`、`/newfile`、`/status.txt`、`a /new`。
/// 用户发个路径/文件名不该被吞 —— 这条若坏，用户的命令行输出会凭空消失。
const CORPUS: [&str; 18] = [
    "/help",
    "/帮助",
    "/HELP",
    "/status",
    "/状态",
    "/stop",
    "/Stop",
    "/new",
    "/new  重构侧边栏",
    "/usr/bin/env",
    "/newfile",
    "/status.txt",
    "/unknown",
    "/",
    "a /new",
    "not/a/command",
    "  ",
    "hello",
];

// ─── 缺陷 1：本地指令不进跑轮 ───

/// 桌面 `/help`、`/status` 走本地处理，**不进跑轮**。
///
/// 断言是「引擎没被调用」而不是「有回复」：
/// ① `Channel` 上恰好两条事件且**没有 `step`**（没调任何工具）；
/// ② 库里**没有**这一轮的任务行 —— 而跑轮必然写一行（`echo` 引擎也写），
///    所以「没有」就是「跑轮没跑过」的实证，不是推测。
#[test]
fn slash_commands_are_answered_locally_and_never_reach_the_engine() {
    let convo = fresh_convo("slash-local");
    for text in ["/help", "/status", "/stop", "/new", "/帮助", "/状态", "/NEW", "/Stop"] {
        let (events, out) = send(text, &convo);
        assert_eq!(
            kinds(&events),
            vec!["delta".to_owned(), "done".to_owned()],
            "{text} 应只产生一条本地 delta + 一条 done（不得有任何 step 事件）：{events:?}"
        );
        assert_eq!(out.status, LOCAL_COMMAND_STATUS, "{text} 的 status 口径不对");
        assert_eq!(
            out.labels.model, "本地指令",
            "{text} 这一轮没有模型，标签不能编一个模型名"
        );
        assert!(out.labels.tools.is_empty(), "{text} 没调工具，标签不该有工具");
        assert_eq!(
            out.labels.usage.input_tokens, 0,
            "{text} 没跑模型，零耗；非零即说明账本被动了"
        );
        let store = common::open_store();
        assert!(
            !task_titles(&store, &convo).contains(&text.chars().take(24).collect::<String>()),
            "{text} **进了跑轮**：库里出现了这一轮的任务行，而本地指令一个 token 都不该烧"
        );
    }
}

/// 每个被识别的动词（含中文别名与大小写变体）都本地作答。
///
/// 与上一条分工：那条盯「不进跑轮」，这条盯「**都**被认出来了」——
/// 只回 `/help` 而把 `/状态` 当普通文本发出去，是同一个缺陷的另外半边。
#[test]
fn every_recognised_verb_and_alias_is_answered_locally() {
    let convo = fresh_convo("slash-alias");
    for (text, verb) in [
        ("/help", "help"),
        ("/帮助", "help"),
        ("/status", "status"),
        ("/状态", "status"),
        ("/stop", "stop"),
        ("/new", "new"),
        ("/new  查日志", "new"),
    ] {
        let (events, _) = send(text, &convo);
        assert_eq!(kinds(&events).first().map(String::as_str), Some("delta"), "{text} 没被本地接住");
        let outcome = intercept_local_command(text, Some(&convo), None, None)
            .expect("分流不应报错")
            .expect("应被识别为指令");
        assert_eq!(outcome.command, verb, "{text} 命中的动词不对");
        assert!(!outcome.reply.trim().is_empty(), "{text} 回执不能是空串");
    }
}

// ─── 缺陷 2：`/stop` 不许撒谎（反向断言）───

/// 桌面 `/stop` 的回执**不含**任何虚假承诺，且说清为什么停不了。
///
/// 这条是**反向断言**：先列禁用措辞再断言它们**不在**，只断言「有回复」
/// 的话，改成「已请求停止当前回合」照样全绿 —— 而那正是本缺陷。
#[test]
fn stop_reply_does_not_promise_a_stop_that_cannot_happen() {
    let _ = common::data_dir();
    let reply = intercept_local_command("/stop", None, None, None)
        .expect("分流不应报错")
        .expect("/stop 应被识别")
        .reply;

    // 虚假承诺：这一轮**停不了**（跑轮同步、无 stop hook，见
    // DESIGN-CHANNEL-DISPATCH.md §0.5 与 nt_channel_cmd.rs 的 Command::Stop）。
    for lie in [
        "已停止",
        "已停止当前",
        "已请求停止",
        "正在停止",
        "中止成功",
        "已打断",
        "已终止",
    ] {
        assert!(
            !reply.contains(lie),
            "桌面 /stop 回执在撒谎（出现「{lie}」）：{reply}"
        );
    }
    // 也不能把用户指向一个**不取消任何东西**的操作（IM 侧那条假建议，
    // nt_channel_cmd.rs 的测试已按动作短语把它钉死）。
    for advice in ["按发送键", "它会变停止键", "请到桌面 App 按"] {
        assert!(
            !reply.contains(advice),
            "桌面 /stop 回执在把用户指向一个不终止运行的操作（{advice}）：{reply}"
        );
    }
    // 但也不能只说一句「不行」：要说清原因 + 给出**真能生效**的替代路径。
    assert!(reply.contains("停不了"), "要讲清结论：{reply}");
    assert!(reply.contains("同步"), "要讲清原因：{reply}");
    assert!(reply.contains("退出"), "要给出真正能生效的替代路径：{reply}");
    // 桌面这个键仍然只是收起输出 —— 回执必须承认它不终止运行。
    assert!(
        reply.contains("不终止运行"),
        "桌面用户就在这个 App 里，回执必须说明停止键只收起输出：{reply}"
    );
}

// ─── 缺陷 3：`/` 开头的普通词不该被吞，且两端同口径 ───

/// 桌面与 IM **同一口径**：认不认得，全看 `nt_channel_cmd::parse`。
///
/// 逐条拿核心那个函数做等价断言（`intercept` 说「不是指令」⇔ `parse` 说 `None`），
/// 所以「桌面自创了一套动词表」这种漂移**不可能**悄悄发生。
#[test]
fn desktop_and_im_agree_on_which_slash_words_are_commands() {
    let _ = common::data_dir();
    for text in CORPUS {
        let desktop = intercept_local_command(text, None, None, None).expect("分流不应报错");
        let im = nt_channel_cmd::parse(text);
        assert_eq!(
            desktop.is_none(),
            im.is_none(),
            "「{text}」两端口径不一致：桌面认={}，IM 认={}",
            desktop.is_some(),
            im.is_some()
        );
    }
}

/// 语料自检：CORPUS 里**既有**该认的也有不该认的。
///
/// 没有这条，上一条可能因为「一个都没认出来」而空转 —— 那是这类等价断言
/// 最常见的假绿，必须自己堵上。
#[test]
fn the_corpus_actually_contains_both_kinds() {
    let _ = common::data_dir();
    let commands: Vec<&str> = CORPUS
        .iter()
        .copied()
        .filter(|t| nt_channel_cmd::parse(t).is_some())
        .collect();
    let plain: Vec<&str> = CORPUS
        .iter()
        .copied()
        .filter(|t| nt_channel_cmd::parse(t).is_none())
        .collect();
    assert_eq!(commands.len(), 9, "语料里该认的应恰好 9 条：{commands:?}");
    assert_eq!(plain.len(), 9, "语料里不该认的应恰好 9 条：{plain:?}");
}

/// **普通文本里以 `/` 开头但不是已知指令，照常发给模型（不被吞）。**
///
/// 具体到用户真会敲的东西：`/usr/bin/env`、`/newfile`、`/status.txt`。
/// 若这条坏了，用户贴一段路径进聊天框就凭空消失，且没有任何提示。
#[test]
fn unknown_slash_words_reach_the_model_instead_of_being_swallowed() {
    let convo = fresh_convo("slash-unknown");
    let store = common::open_store();
    for text in ["/usr/bin/env", "/newfile", "/status.txt", "/unknown", "/"] {
        assert!(
            intercept_local_command(text, Some(&convo), None, None)
                .expect("分流不应报错")
                .is_none(),
            "{text} 不是已知指令，不该被本地接住"
        );
        let (events, out) = send(text, &convo);
        let body = deltas(&events);
        assert!(
            body.contains("neobot(echo)"),
            "{text} 没发给模型（echo 引擎会给每条消息回一句 `neobot(echo): …`）：{body}"
        );
        assert!(
            body.contains(text.trim_end_matches('/')),
            "{text} 到模型手里已经不是原文了：{body}"
        );
        assert_ne!(
            out.status, LOCAL_COMMAND_STATUS,
            "{text} 走的是本地分支，status 不该是 {LOCAL_COMMAND_STATUS}"
        );
        assert!(
            task_titles(&store, &convo)
                .contains(&text.chars().take(24).collect::<String>()),
            "{text} 真的跑了一轮就该有任务行；没有说明它被吞了"
        );
    }
}

// ─── 普通文本行为逐字不变 ───

/// 非 `/` 开头的消息行为**完全不变**（含正文里含 `/stop` 的情况）。
///
/// 三个信号一起看：状态是跑轮状态（不是本地状态）、正文是 echo 引擎的原样回声、
/// 库里多了一行任务 —— 即「与本缺陷之前逐字一致」。
#[test]
fn ordinary_text_still_goes_through_the_run_turn_unchanged() {
    let _ = common::data_dir();
    let store = common::open_store();
    for text in [
        "hello",
        "帮我看下 /stop 这条指令在桌面怎么走",
        "路径是 /usr/bin/env，不是指令",
        "1/2 + 1/2 等于几",
    ] {
        let convo = fresh_convo("slash-plain");
        let (events, out) = send(text, &convo);
        let body = deltas(&events);
        assert!(
            body.contains("neobot(echo)"),
            "{text} 应当照常发给模型：{body}"
        );
        assert!(
            body.contains(text),
            "{text} 到模型手里必须仍是原文（一个字都不能被改写）：{body}"
        );
        assert_ne!(
            out.status, LOCAL_COMMAND_STATUS,
            "{text} 是普通文本，status 不该是本地指令状态"
        );
        assert!(
            !out.labels.model.is_empty() && out.labels.model != "本地指令",
            "{text} 走了跑轮，模型标签该是引擎名而不是「本地指令」"
        );
        assert_eq!(
            task_titles(&store, &convo),
            vec![text.chars().take(24).collect::<String>()],
            "{text} 应当恰好跑一轮并留下任务行"
        );
    }
}

// ─── 两端口径不漂移 ───

/// `/help` 的两端**同一口径**，但「一致」的准确含义必须说清：**不是逐字相等**。
///
/// `help_text(channel_name)` 把渠道名插进第一行自报家门（`我是 {name}，…`），
/// 而两端传的**本来就是不同的名字**：桌面传 `DESKTOP_CHANNEL_NAME`，IM 传
/// 自己的渠道名。所以「桌面 help 文案 == IM help 文案」这句话在**字面上是
/// 假的** —— 真正成立、也真正值得守的是下面三条：
/// 1. **同一渠道名下两端逐字同文**。两侧各走**各自的生产入口**（IM 侧
///    `execute`、桌面侧 `intercept_local_command`），不是同一个函数算两遍 ——
///    日后谁让 `execute` 不再委托 `local_reply`、或桌面传错渠道名，这里都会红。
/// 2. **渠道名不同，差异恰在自报家门那一行**：其余各行必须与渠道无关。
/// 3. **help 自述的每条指令都真能被 `parse` 认得**（且反过来一条不漏）——
///    「自述里写着的命令存在」是 help 的**功能契约**，与文案措辞无关。
///
/// 早先这条测试比的是 `help_text(X)` vs `help_text(X)`（桌面改成直调
/// `help_text` 之后），**恒真、没有牙**：实测把 `help_text` 里的 `/status`
/// 改成 `/sttus`，本文件 15 条测试仍然全绿。
#[test]
fn help_text_is_byte_identical_to_the_im_help_text() {
    let _ = common::data_dir();
    let store = common::open_store();
    // IM 侧的真实入口。**故意传桌面那个渠道名**：自报家门那行被钉成同一份，
    // 于是剩下的差异只能是「谁在生成」。
    let im = nt_channel_cmd::execute(&store, &Command::Help, &bot_fixture(), DESKTOP_CHANNEL_NAME)
        .expect("IM 侧 /help 应能执行");
    let desktop = intercept_local_command("/help", None, None, None)
        .expect("分流不应报错")
        .expect("/help 应被识别")
        .reply;
    assert_eq!(
        desktop, im.reply,
        "同一渠道名下，桌面 /help 必须与 IM 的 execute 逐字同文"
    );
    // 端到端：前端真正收到的那条 delta 也是这一份。
    let convo = fresh_convo("slash-help");
    let (events, _) = send("/help", &convo);
    assert_eq!(deltas(&events), im.reply, "前端收到的正文必须与 IM 的 help 同文");
    // help 必须把不可用的 `/stop` 标出来，否则等于把停不了的命令说成能用。
    assert!(im.reply.contains("当前不可用"), "help 不能把停不了的命令说成能用：{}", im.reply);

    // ── 前提钉死：渠道名**确实**流进了文案 ──
    // 少了这条，上面那句 `assert_eq` 在「help 忽略渠道名」时会退化成恒真。
    let im_as_tg = nt_channel_cmd::execute(&store, &Command::Help, &bot_fixture(), "TG")
        .expect("IM 侧 /help 应能执行")
        .reply;
    assert_ne!(
        im_as_tg, im.reply,
        "help 的自报家门必须随渠道名变 —— 否则「两端同文」是在比同一个值"
    );
    // 差异**只在**自报家门那一行：其余各行与渠道无关。
    let desktop_lines: Vec<&str> = desktop.lines().collect();
    let tg_lines: Vec<&str> = im_as_tg.lines().collect();
    assert_eq!(
        desktop_lines.len(),
        tg_lines.len(),
        "help 的行数不该随渠道名变"
    );
    let differing: Vec<usize> = (0..desktop_lines.len())
        .filter(|i| desktop_lines[*i] != tg_lines[*i])
        .collect();
    assert_eq!(
        differing,
        vec![0],
        "除自报家门那一行外，help 文案必须与渠道无关（多出的差异行：{differing:?}）"
    );
    assert!(
        desktop_lines[0].contains(DESKTOP_CHANNEL_NAME) && tg_lines[0].contains("TG"),
        "唯一该变的那行必须是自报家门：桌面 {:?} / IM {:?}",
        desktop_lines[0],
        tg_lines[0]
    );

    // ── help 自述 ⇔ `parse` 的双向等价 ──
    // 自述里写着一个**用不了**的动词 = 骗用户去敲；少说一条 = 用户永远不知道
    // 它存在。两个方向都要钉，且一律问 `parse` 本身（不抄第二份动词表）。
    for verb in ["/new", "/stop", "/help", "/status"] {
        assert!(desktop.contains(verb), "help 必须自述 {verb}：{desktop}");
        assert!(
            nt_channel_cmd::parse(verb).is_some(),
            "help 自述了 {verb}，但 nt_channel_cmd::parse 认不出它 —— 自述与实现漂移了"
        );
    }
    for command in [
        Command::Help,
        Command::Stop,
        Command::Status,
        Command::New {
            title: String::new(),
        },
    ] {
        assert!(
            desktop.contains(&format!("/{}", command.as_str())),
            "core 认得 /{} 但 help 没自述它 —— 用户无从知道它存在：{desktop}",
            command.as_str()
        );
    }
}

/// 桌面 `/stop` 与 IM `/stop` **逐字同文**。
///
/// `/stop` 的回执在核心里是纯文本（不读 `store` 也不读 `bot`），但 `execute`
/// 的**签名**强制要 `BotRow`，而桌面不许造假 —— 故桌面留了一份常量。
/// 这条测试就是那份常量的**锁**：核心改了文案，这里会红，那是故意的。
#[test]
fn stop_reply_is_byte_identical_to_the_im_reply() {
    let _ = common::data_dir();
    let store = common::open_store();
    let im = nt_channel_cmd::execute(&store, &Command::Stop, &bot_fixture(), "TG")
        .expect("IM 侧 /stop 应能执行");
    let desktop = intercept_local_command("/stop", None, None, None)
        .expect("分流不应报错")
        .expect("/stop 应被识别")
        .reply;
    assert_eq!(
        desktop, im.reply,
        "桌面 /stop 与 IM /stop 文案漂移了（core 改文案时这里会红 —— 那正是它该红的时候）"
    );
    // 端到端：前端收到的也是这一份。
    let convo = fresh_convo("slash-stop");
    let (events, _) = send("/stop", &convo);
    assert_eq!(deltas(&events), im.reply, "前端收到的 /stop 回执必须与 IM 同文");
}

// ─── `/new`：桌面不支持，且**如实说** ───

/// `/new` 是**被认出来但被有意拒绝**，不是解析漏了。
///
/// 先证伪「漏解析」：核心 `parse("/new")` 确实认得它。再证桌面没有偷偷建会话 ——
/// IM 的 `/new` 会 `create_conversation`，桌面**一个都不建**。
#[test]
fn new_is_recognised_but_refused_and_creates_nothing() {
    let _ = common::data_dir();
    assert_eq!(
        nt_channel_cmd::parse("/new"),
        Some(Command::New { title: String::new() }),
        "前提自检：/new 确实是被认识的一条指令"
    );
    let store = common::open_store();
    let convo = fresh_convo("slash-new");

    // 两条都试：带标题与不带标题 —— IM 的 `/new` 会建出这两种标题的会话
    // （`nt_channel_cmd::execute` 里空标题落成「新会话」）。
    let mut replies = String::new();
    for text in ["/new", "/new  查日志"] {
        let (events, _) = send(text, &convo);
        replies.push_str(&deltas(&events));
        assert!(
            !task_titles(&store, &convo)
                .contains(&text.chars().take(24).collect::<String>()),
            "{text} 不该进跑轮"
        );
    }
    let reply = replies.as_str();

    // 「不支持」必须说出口。
    assert!(
        reply.contains("做不了") || reply.contains("不支持"),
        "桌面 /new 必须如实说做不了：{reply}"
    );
    // **反向断言**：不许回一句假的「已重置 / 已开新会话」。
    // 哪怕是否定句式（`「我不会说已重置」`）也不行 —— 用户是在气泡里扫一眼
    // 找结论的，那几个字一出现，扫过去的人读到的就是「已重置」。
    for lie in ["已重置", "已开新会话", "已新建", "后续消息都进这里", "已切换", "开好了。"] {
        assert!(
            !reply.contains(lie),
            "桌面 /new 在撒谎（出现「{lie}」）：{reply}"
        );
    }
    // 也不许**偷偷建会话** —— 建了却不切换，等于给用户一个永远进不去的会话。
    // 按 IM 那两个**确切标题**查（不去数总会话数：并行测试共用一个库，数数会串）。
    let titles: Vec<String> = store
        .list_conversations()
        .unwrap_or_default()
        .into_iter()
        .map(|convo| convo.title)
        .collect();
    for fingerprint in ["新会话", "查日志"] {
        assert!(
            !titles.contains(&fingerprint.to_owned()),
            "桌面 /new 建了题为「{fingerprint}」的会话 —— 建了却切不过去，\
             用户会拿到一个永远进不去的会话。现有：{titles:?}"
        );
    }
}

// ─── `/status`：只报桌面**真有**的状态 ───

/// `/status` 报的是**桌面的真状态**（当前会话标题 + 选中模型 + 本机数据目录），
/// 且不照抄 IM 那些在桌面恒为假的行。
#[test]
fn status_reports_the_real_desktop_state() {
    let title = common::uniq("slash-status");
    let convo = fresh_convo(&title);
    let (events, _) = send("/status", &convo);
    let reply = deltas(&events);

    assert!(reply.contains(&title), "/status 必须报当前会话标题：{reply}");
    assert!(reply.contains("模型："), "/status 必须报模型行：{reply}");
    assert!(
        reply.contains(DESKTOP_CHANNEL_NAME),
        "/status 要说清自己是什么界面：{reply}"
    );
    assert!(reply.contains("~/.neobot"), "/status 要讲清数据只在本机：{reply}");
    // IM 专属那行「访问模式：见设置页（白名单 / 私聊 / 开放）」在桌面是**假的**
    // （桌面没有白名单/私聊这层），照抄就是撒谎。
    assert!(
        !reply.contains("见设置页"),
        "/status 照抄了 IM 的访问模式行，而桌面没有白名单/私聊这层：{reply}"
    );
    assert!(
        reply.contains("没有白名单/私聊那套"),
        "/status 应如实说清桌面没有访问闸门：{reply}"
    );
}

/// `/status` 认得「绑了但会话没了」，且**不**说成「没绑」—— 与 IM 同律。
#[test]
fn status_tells_a_dead_conversation_apart_from_no_conversation() {
    let _ = common::data_dir();
    let dead = intercept_local_command("/status", Some("convo-that-was-deleted"), None, None)
        .expect("分流不应报错")
        .expect("/status 应被识别")
        .reply;
    assert!(dead.contains("已不存在"), "绑了但会话没了 ≠ 没绑：{dead}");
    assert!(
        !dead.contains("\n当前会话：无归属会话"),
        "含糊其辞会让用户以为设置没生效：{dead}"
    );
    let none = intercept_local_command("/status", None, None, None)
        .expect("分流不应报错")
        .expect("/status 应被识别")
        .reply;
    assert!(none.contains("无归属会话"), "真的没绑要说清：{none}");
    assert_ne!(dead, none, "「会话没了」与「没绑」是**两件事**，不能给同一句话");
}

// ─── 两端**结构同源**（不再是「两份拷贝 + 一条相等断言」）───

/// `/help` 与 `/stop` 的桌面回执**就是** core `local_reply` 的输出本身。
///
/// 此前桌面在 `nt_cmd_run.rs` 里**抄了一份** `/stop` 的回执（`STOP_REPLY`），
/// 只靠「与 IM 字节相等」防漂移 —— 那只证明**此刻**一致，证明不了日后改一处
/// 会不会忘了改另一处。现在那条拷贝**已删**，桌面与 IM（`execute` 委托同一个
/// `local_reply`）**结构同源**。
///
/// 故这条断言测的是「两端确实走同一个出口」，不是「两份文本碰巧一样」。
#[test]
fn help_and_stop_come_straight_out_of_the_shared_local_reply() {
    let _ = common::data_dir();
    for (text, command) in [("/help", Command::Help), ("/stop", Command::Stop)] {
        let shared = nt_channel_cmd::local_reply(&command, DESKTOP_CHANNEL_NAME)
            .expect("help/stop 属本地回执范围")
            .reply;
        let desktop = intercept_local_command(text, None, None, None)
            .expect("分流不应报错")
            .expect("应被识别为指令")
            .reply;
        assert_eq!(
            desktop, shared,
            "{text} 的桌面回执不是 local_reply 的原样输出 —— 又抄了一份？"
        );
    }
}

/// 桌面那份**拷贝真的没了**：源码里不再有第二份 `/stop` 文案。
///
/// 上一条断言的是**行为**（跑出来的文本一样），这条断言的是**结构**
/// （源码里字面量只有一处）。上一条在结构同源之后已恒成立，所以还需要这条
/// 静态锁：有人日后「顺手」把文案抄回桌面时，这条会红，而上一条不会。
#[test]
fn the_desktop_source_no_longer_carries_a_second_copy_of_the_stop_reply() {
    let source = include_str!("../src/nt_commands/nt_cmd_run.rs");
    // 抄回执时一定会带上的那个开头半句。
    assert!(
        !source.contains("停不了：跑轮是同步的"),
        "nt_cmd_run.rs 里又出现了 /stop 回执的拷贝 —— 走 local_reply，不要抄"
    );
    // 被删掉的那个常量本身。
    assert!(
        !source.contains("STOP_REPLY"),
        "STOP_REPLY 这个拷贝常量不该复活 —— 它的内容由 core 的 local_reply 持有"
    );
}

/// 桌面 `/help` 回执**不含 markdown 强调记号**。
///
/// 两端都不渲染 markdown：IM 是纯文本直发（用户看到的就是字面的星号），
/// 桌面侧前端 `esc()` 之后同样按字面显示。故正确做法是**文案里不放 markdown**，
/// 而不是给桌面加渲染器。早先 help 里是 `（**当前不可用**：…）`。
///
/// 反向断言（盯坏记号不在）+ 正向断言（结论还在）：去掉星号时不能顺手
/// 把「当前不可用」这个对用户最重要的结论一起去掉。
#[test]
fn help_reply_carries_no_markdown_emphasis() {
    let _ = common::data_dir();
    let reply = intercept_local_command("/help", None, None, None)
        .expect("分流不应报错")
        .expect("/help 应被识别")
        .reply;
    for mark in ["**", "__", "*当前不可用*"] {
        assert!(
            !reply.contains(mark),
            "桌面 /help 带了 markdown 记号「{mark}」，而两端都不渲染 markdown：{reply}"
        );
    }
    // 结论必须还在，且 IM（纯文本）读起来仍然通顺。
    assert!(
        reply.contains("（当前不可用：跑轮同步"),
        "去掉星号时别把「当前不可用」这个结论一起去掉：{reply}"
    );
    assert!(reply.contains("/stop"), "help 仍要留着 /stop（用户会试）：{reply}");
}
