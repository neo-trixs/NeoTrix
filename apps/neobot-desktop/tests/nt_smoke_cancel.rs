//! 冒烟：桌面「真停止」（停止键 + `/stop`）与本轮 task id 的端到端。
//!
//! ## 这一套在钉什么
//!
//! 1. **停止键真的停**：起一轮，跑轮**途中**翻令牌，断言任务行真的以
//!    `TaskStatus::Cancelled` 收场（回库核对，不看令牌），且**不再发起第二次
//!    模型调用**。
//! 2. **取消失败必须报错**：没有在跑的轮次时 `neobot_run_cancel` 回 `Err`。
//!    「静默成功」= 用户以为停了而它还在烧 token，是本项目最该避免的那类缺陷。
//! 3. **取消不跨会话**：B 的取消请求不许停掉 A 那一轮。
//! 4. **`/stop` 也真的停**：斜杠指令那条路（`intercept_local_command`）会去翻
//!    同一枚令牌，**且不改变事件形状**（core 的回执文本与既有冒烟测试按字节
//!    钉死，本文件不碰它一个字）。
//! 5. **`task_id` 是本轮那一个**，不是「库里最新的那条」；`collect_labels`
//!    的工具 chips 同理（否则会挂出**别人**调过的工具）。
//! 6. **跑完即注销**：轮次结束后停止键如实报「没有在跑」。
//! 7. **UI 侧失败必须有可见反馈**（静态钉，见文件末尾那条）。
//!
//! ## 为什么不用 echo 引擎跑「跑轮途中取消」这条
//!
//! echo 那一轮**只有一个跳**且不带任何 tool_calls，跑完就 break；而四个取消
//! 检查点分别在「跳开头（C1）」「同跳内两个工具之间（C2）」「sleep 之前（C3）」
//! 「入轮清旗（C0）」——**没有一个在单跳无工具的跑轮里会被走到**。所以
//! 「跑轮途中翻令牌 ⇒ 任务 Cancelled」用 echo **物理上测不出来**（翻了也只会
//! 正常跑完）。要测就得有一轮**会继续**的跑轮，故本文件起一个**本机假模型
//! 端点**（SSE，按脚本回 tool_calls），并用「逐个放行」的闸把「翻令牌」与
//! 「这一跳的响应」之间的先后关系变成测试定的，而不是抢出来的。
//!
//! ## 网络
//!
//! 不碰外网：假端点绑 `127.0.0.1:0`（内核给的随机端口）。引擎由
//! `NEOBOT_ENGINE=http` + `NEOBOT_BASE_URL` 指向它；其余用例钉回 echo。

mod common;

use std::collections::HashSet;
use std::io::{BufRead as _, BufReader, Read as _, Write as _};
use std::net::TcpListener;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use neobot_desktop::nt_commands::nt_cmd_run::{
    LOCAL_COMMAND_STATUS, StreamEvent, neobot_run_cancel, neobot_run_stream,
};
use neobot_desktop::nt_commands::NeobotRunResult;
use neotrix_neobot::{NeobotStore, TaskStatus};
use serde_json::Value;

// ─── 进程级环境串行闸 ───
//
// 这些用例都改**进程级**环境（`NEOBOT_ENGINE` / `NEOBOT_BASE_URL` / …），
// 而 cargo 默认在同一个进程里**并行**跑一个二进制里的各条测试。共用一把锁
// = 每个用例独占环境。不用 `Once` 是因为每个用例要**不同**的环境。
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// 取串行闸（中毒也照用：前一条挂了不该让后面全部报毒）。
fn lock_env() -> MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner())
}

/// 引擎钉回 echo（默认路径：不碰网络）。
fn use_echo_engine() {
    let _ = common::data_dir();
    std::env::set_var("NEOBOT_ENGINE", "echo");
}

/// 引擎指向本机那个假端点。
fn use_fake_http(base_url: &str) {
    let _ = common::data_dir();
    std::env::set_var("NEOBOT_ENGINE", "http");
    std::env::set_var("NEOBOT_BASE_URL", base_url);
    std::env::set_var("NEOBOT_MODEL", "fake-model");
    // 空 key ⇒ 不发 Authorization 头（假端点也不校验，纯粹少一件事）。
    std::env::remove_var("NEOBOT_API_KEY");
}

// ─── 假模型端点 ───

/// 假端点的可调状态。
struct FakeState {
    /// 收到的请求数（一跳 = 一次 `POST /chat/completions`）。
    requests: usize,
    /// 已被**显式放行**的请求下标集合。
    released: HashSet<usize>,
    /// 下一个响应的 SSE 正文。
    body: String,
}

/// 本机假模型端点（OpenAI 形状 + SSE）。
///
/// 「按脚本 + 按下标逐个放行」是为了**确定性**：模型调用到达时先停住，等测试
/// 放行。于是「翻停止令牌」与「这一跳拿到响应」谁先谁后由测试说了算 ——
/// 取消测试的整个意义就在这个顺序上。
struct FakeModel {
    base_url: String,
    state: Arc<(Mutex<FakeState>, Condvar)>,
}

impl FakeModel {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("应能绑本机回环端口");
        let port = listener
            .local_addr()
            .expect("应拿得到本地地址")
            .port();
        let state = Arc::new((
            Mutex::new(FakeState {
                requests: 0,
                released: HashSet::new(),
                body: plain_body("neobot(fake): 答完了"),
            }),
            Condvar::new(),
        ));
        let worker = Arc::clone(&state);
        std::thread::spawn(move || {
            for conn in listener.incoming() {
                let Ok(conn) = conn else { continue };
                let state = Arc::clone(&worker);
                // 每个连接一个线程：被闸拦住的连接不能挡住别人的（并发跑两轮
                // 时两个请求会同时在飞）。
                std::thread::spawn(move || serve_one(conn, &state));
            }
        });
        Self {
            base_url: format!("http://127.0.0.1:{port}/v1"),
            state,
        }
    }

    /// 脚本：此后**被放行**的请求都用这个正文回应（改脚本不影响已到达的请求）。
    fn script(&self, body: &str) {
        self.state.0.lock().expect("假端点状态锁").body = body.to_owned();
    }

    /// 已到达的请求数。
    fn requests(&self) -> usize {
        self.state.0.lock().expect("假端点状态锁").requests
    }

    /// 等到第 `n` 个请求到达（含）为止；超时即测试卡住，直接红。
    fn wait_requests(&self, n: usize) {
        let deadline = Instant::now() + Duration::from_secs(20);
        let (lock, cv) = &*self.state;
        let mut st = lock.lock().expect("假端点状态锁");
        while st.requests < n {
            let now = Instant::now();
            assert!(now < deadline, "等第 {n} 个模型请求等了 20s（测试卡住了）");
            let (guard, _) = cv.wait_timeout(st, deadline - now).expect("假端点状态锁");
            st = guard;
        }
    }

    /// 放行第 `idx` 个请求（按下标，不是按下标序 —— 并发跑两轮时要能挑）。
    fn release(&self, idx: usize) {
        let (lock, cv) = &*self.state;
        lock.lock().expect("假端点状态锁").released.insert(idx);
        cv.notify_all();
    }
}

/// 处理一次请求：读掉它 → 等放行 → 回 SSE。
fn serve_one(mut conn: std::net::TcpStream, state: &Arc<(Mutex<FakeState>, Condvar)>) {
    let mut reader = match conn.try_clone() {
        Ok(clone) => BufReader::new(clone),
        Err(_) => return,
    };
    // 读头到空行，再按 Content-Length 读 body（ureq 的 `send_json` 给长度）。
    let mut length = 0usize;
    let mut line = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line).unwrap_or(0) == 0 {
            return;
        }
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            break;
        }
        if let Some((key, value)) = trimmed.split_once(':') {
            if key.trim().eq_ignore_ascii_case("content-length") {
                length = value.trim().parse().unwrap_or(0);
            }
        }
    }
    if length > 0 {
        let mut request = vec![0u8; length];
        if reader.read_exact(&mut request).is_err() {
            return;
        }
    }
    // 闸：登记自己（拿到自己的请求下标），然后等测试点名放行。
    let (lock, cv) = &**state;
    let reply = {
        let mut st = lock.lock().expect("假端点状态锁");
        let request_idx = st.requests;
        st.requests += 1;
        cv.notify_all();
        let deadline = Instant::now() + Duration::from_secs(30);
        while !st.released.contains(&request_idx) {
            let now = Instant::now();
            if now >= deadline {
                return;
            }
            let (guard, _) = cv
                .wait_timeout(st, deadline - now)
                .expect("假端点状态锁");
            st = guard;
        }
        st.body.clone()
    };
    // `Connection: close` ⇒ 一条连接一次请求（否则客户端连接池会把第二次请求
    // 塞进上一条连接，上面「一次连接一次请求」的读法就失效了）。
    let head = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        reply.len()
    );
    // 写失败只说明客户端已经走了（测试里就是「这一轮被取消后连接被弃」），
    // 不值得报 —— 故按仓库惯例显式 `drop` 掉 Result。
    drop(conn.write_all(head.as_bytes()));
    drop(conn.write_all(reply.as_bytes()));
    drop(conn.flush());
}

/// 一次「纯回复」的 SSE（无 `tool_calls` ⇒ 这一跳结束就是终态）。
fn plain_body(text: &str) -> String {
    let chunk = serde_json::json!({ "choices": [{ "delta": { "content": text } }] });
    format!("data: {chunk}\n\ndata: [DONE]\n\n")
}

/// 一次「要调工具」的 SSE（带 `tool_calls` ⇒ 跑轮会继续往下走）。
fn tool_body(tools: &[(&str, &str)], content: &str) -> String {
    let calls: Vec<Value> = tools
        .iter()
        .enumerate()
        .map(|(idx, (name, args))| {
            serde_json::json!({
                "index": idx,
                "id": format!("call-{idx}"),
                "function": { "name": name, "arguments": args },
            })
        })
        .collect();
    let say = serde_json::json!({ "choices": [{ "delta": { "content": content } }] });
    let act = serde_json::json!({ "choices": [{ "delta": { "tool_calls": calls } }] });
    format!("data: {say}\n\ndata: {act}\n\ndata: [DONE]\n\n")
}

// ─── 共用夹具 ───

/// 一个只属于本测试的会话（免得任务/会话断言与别的测试串味）。
fn fresh_convo(title: &str) -> String {
    common::open_store()
        .create_conversation("dm", title, &[])
        .expect("应能建测试会话")
}

/// 起一条真 `tauri::ipc::Channel`，把事件按**线上形状**收进 `Vec`。
fn collecting_channel() -> (tauri::ipc::Channel<StreamEvent>, Arc<Mutex<Vec<Value>>>) {
    let seen: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&seen);
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

/// 真调一次 `neobot_run_stream`（**不是**直接调内层函数），返回
/// `(Channel 上的全部事件, 命令返回值)`。
fn send(text: &str, convo: &str) -> (Vec<Value>, NeobotRunResult) {
    let (channel, seen) = collecting_channel();
    let out = tauri::async_runtime::block_on(neobot_run_stream(
        text.chars().take(24).collect::<String>(),
        text.to_owned(),
        "neo".to_owned(),
        Some(convo.to_owned()),
        None,
        None,
        channel,
    ))
    .expect("跑轮命令应成功返回");
    let events = seen.lock().expect("收集器锁").clone();
    (events, out)
}

/// 事件序列的 `kind` 列表（前端 `ev.kind` 那一份）。
fn kinds(events: &[Value]) -> Vec<String> {
    events
        .iter()
        .map(|ev| ev["kind"].as_str().unwrap_or_default().to_owned())
        .collect()
}

/// 本轮任务行（按 id 读；读不到就红 —— 那是「这一轮没建任务」）。
fn task_of(store: &NeobotStore, task_id: &str) -> neotrix_neobot::AgentTask {
    assert!(!task_id.is_empty(), "跑过轮就必须给出本轮 task id");
    store
        .get_task(task_id)
        .expect("读任务行")
        .expect(&format!("库里没有这个任务：{task_id}（task_id 认错了）"))
}

// ─── 1. 取消失败必须如实报错 ───

/// 没有在跑的轮次时，停止键**回错**而不是静默成功。
///
/// 反向断言：静默成功 = 前端会给用户一个假回执（「以为停了，其实还在跑」）。
/// 前端那条失败分支的可见反馈就是靠这个 `Err` 驱动的（见文件末尾那条静态锁）。
#[test]
fn cancel_without_a_running_turn_reports_the_truth_instead_of_pretending() {
    let _guard = lock_env();
    use_echo_engine();
    let convo = fresh_convo(&common::uniq("cancel-idle"));

    let err = tauri::async_runtime::block_on(neobot_run_cancel(Some(convo.clone())))
        .expect_err("没有在跑的轮次时必须报错，不能静默成功");
    assert!(
        err.contains("没有在跑"),
        "要说清是「没有在跑」（可能已结束/还没开始），而不是含糊其辞：{err}"
    );

    // 连会话都没带 ⇒ 更无从知道停哪一轮，同样必须报错而不是成功。
    let err_none =
        tauri::async_runtime::block_on(neobot_run_cancel(None)).expect_err("没带会话标识必须报错");
    assert!(!err_none.trim().is_empty(), "报错必须带可读原因");
    assert!(
        !err_none.contains("已停"),
        "回执里不许出现「已停」这种既成事实的措辞：{err_none}"
    );
}

// ─── 2. 端到端：跑轮途中取消，轮次真的以 Cancelled 收场 ───

/// 主线用例：起一轮 → 跑轮途中翻令牌 → 这一轮**真的**被停掉。
///
/// 断言四件事（都落在可核实的地方，不看令牌、不看推理）：
/// ① 任务行 `status == Cancelled` 且 `error` 说得出是用户叫停的；
/// ② `NeobotRunResult.task_id` **就是本轮那一个**（按 id 读回库里交叉核对，
///    并核它的标题/会话就是本轮这一条）；
/// ③ `cancelled == true`（回执自报，回库核对过）；
/// ④ 只发生**一次**模型调用 —— 被停之后不许再发起新的（省钱的 C1 那条律）。
#[test]
fn a_running_turn_really_ends_cancelled_and_reports_its_own_task_id() {
    let _guard = lock_env();
    let model = FakeModel::start();
    use_fake_http(&model.base_url);
    let convo = fresh_convo(&common::uniq("cancel-e2e"));
    // 这一跳回**两个** tool_calls ⇒ 跑轮会进工具循环，于是 C2（同跳内不再执行
    // 下一个工具）就是拦得住它的那道检查点，且**一个工具都不会真被执行**。
    model.script(&tool_body(
        &[
            ("read_file", r#"{"path":"a.txt"}"#),
            ("write_file", r#"{"path":"b.txt","content":"x"}"#),
        ],
        "我要动手了",
    ));

    let worker = {
        let convo = convo.clone();
        std::thread::spawn(move || send("跑一轮然后停我", &convo))
    };
    // 模型请求到达 = 这一轮真的在跑（任务已建、轮次在跑的那一段）。
    model.wait_requests(1);

    let ack = tauri::async_runtime::block_on(neobot_run_cancel(Some(convo.clone())))
        .expect("有在跑的轮次时必须能停");
    assert!(ack.signalled, "回执必须自报停止信号已递到跑轮");
    assert_eq!(ack.convo_id, convo, "回执要回显它停的确实是本会话那一轮");

    // 现在才让这一跳拿到响应 —— 于是「令牌已置位」发生在「跑轮看到工具调用」之前。
    model.release(0);
    let (events, out) = worker.join().expect("跑轮线程不该 panic");

    let store = common::open_store();
    // ① 真停。
    let task = task_of(&store, &out.task_id);
    assert_eq!(
        task.status,
        TaskStatus::Cancelled,
        "这一轮必须以 cancelled 收场（而不是正常跑完）"
    );
    let error = task.error.clone().unwrap_or_default();
    assert!(
        error.contains("stopped by user"),
        "任务行要说得出是用户叫停的：{error}"
    );
    assert!(
        task.lease_id.is_none() && task.lease_until.is_none(),
        "取消必须清租约钥匙（不清则十分钟内没人碰它）"
    );
    // ② `task_id` 是**本轮**那一个：按 id 读回来的行，标题/会话都得是本轮这条。
    assert_eq!(task.title, "跑一轮然后停我", "task_id 指错了任务");
    assert_eq!(
        task.conversation_id.as_deref(),
        Some(convo.as_str()),
        "task_id 指的不是本轮这个会话的任务"
    );
    assert!(out.cancelled, "回执必须自报「这一轮真的被停住了」");
    assert_eq!(
        out.status, "waiting",
        "被叫停的轮次公开状态是 waiting（人可接手），不是 done"
    );
    // ③ 被停之后不许再发起新的模型调用。
    assert_eq!(
        model.requests(),
        1,
        "叫停后不该再问模型（省钱的 C1 那条律）：已发生 {} 次调用",
        model.requests()
    );
    // ④ 事件流语义不变：增量与 Done 都在；一个工具都没执行 ⇒ 没有 step 事件。
    let seen = kinds(&events);
    assert!(seen.contains(&"delta".to_owned()), "增量事件应照发：{seen:?}");
    assert!(seen.contains(&"done".to_owned()), "Done 事件应照发：{seen:?}");
    assert!(
        !seen.contains(&"step".to_owned()),
        "被 C2 拦下时一个工具都没执行，不该有 step 事件：{seen:?}"
    );
}

// ─── 3. 取消不跨会话 ───

/// 取消键只停**它自己那个会话**在跑的那一轮。
///
/// 反向断言：B 的取消请求**必须报错**（B 下没有登记任何在跑的轮次），
/// 且它**不许**顺手把 A 那一轮停掉 —— 「停错了轮次」和「停不了」一样是缺陷。
#[test]
fn cancel_only_reaches_the_turn_of_its_own_conversation() {
    let _guard = lock_env();
    let model = FakeModel::start();
    use_fake_http(&model.base_url);
    let convo_a = fresh_convo(&common::uniq("cancel-scope-a"));
    let convo_b = fresh_convo(&common::uniq("cancel-scope-b"));
    model.script(&tool_body(
        &[
            ("read_file", r#"{"path":"a.txt"}"#),
            ("write_file", r#"{"path":"b.txt","content":"x"}"#),
        ],
        "我要动手了",
    ));

    let worker = {
        let convo = convo_a.clone();
        std::thread::spawn(move || send("A 那一轮", &convo))
    };
    model.wait_requests(1);

    let err = tauri::async_runtime::block_on(neobot_run_cancel(Some(convo_b.clone())))
        .expect_err("B 会话下没有在跑的轮次，必须报错");
    assert!(err.contains("没有在跑"), "要说清 B 下没有可停的轮次：{err}");

    // A 还没被停（刚才那次取消打偏了）：此刻停 A 仍然成功。
    tauri::async_runtime::block_on(neobot_run_cancel(Some(convo_a.clone())))
        .expect("A 那一轮还在跑，必须能停");
    model.release(0);
    let (_events, out) = worker.join().expect("跑轮线程不该 panic");

    let store = common::open_store();
    let task = task_of(&store, &out.task_id);
    assert_eq!(
        task.status,
        TaskStatus::Cancelled,
        "只有 A 那一轮该被停掉"
    );
    assert_eq!(task.conversation_id.as_deref(), Some(convo_a.as_str()));
    assert!(
        store
            .list_tasks(200)
            .unwrap_or_default()
            .iter()
            .all(|t| t.conversation_id.as_deref() != Some(convo_b.as_str())),
        "B 会话不该凭空多出任务"
    );
}

// ─── 4. `/stop` 也真的停（且不改变事件形状）───

/// 聊天框里打 `/stop`：它**真的**去翻那一轮的令牌。
///
/// 断言两半：
/// ① `/stop` 这一趟的事件形状**一字不变**（恰好 `[delta, done]`）—— core 的
///    回执文本归 `nt_channel_cmd::local_reply` 所有且被既有冒烟测试按字节钉死，
///    本切片不碰它；「只加一条失败 delta」会撞那条测试，故也不加。
/// ② 真正被证明的是：**那一轮真的以 `Cancelled` 收场了**。回执那句「停不了」
///    在这一刻是陈旧的 —— 回执该怎么改是核验者的决定。
#[test]
fn slash_stop_flips_the_token_of_the_running_turn_and_adds_no_extra_events() {
    let _guard = lock_env();
    let model = FakeModel::start();
    use_fake_http(&model.base_url);
    let convo = fresh_convo(&common::uniq("slash-stop-real"));
    model.script(&tool_body(
        &[
            ("read_file", r#"{"path":"a.txt"}"#),
            ("write_file", r#"{"path":"b.txt","content":"x"}"#),
        ],
        "我要动手了",
    ));

    let worker = {
        let convo = convo.clone();
        std::thread::spawn(move || send("先跑着", &convo))
    };
    model.wait_requests(1);

    // 用户在聊天框里打 `/stop` —— 走**同一个**命令（`intercept_local_command`）。
    let (events, out) = send("/stop", &convo);
    assert_eq!(
        kinds(&events),
        vec!["delta".to_owned(), "done".to_owned()],
        "`/stop` 的事件形状不许变（core 回执 + 一条 done）：{events:?}"
    );
    assert_eq!(
        out.status, LOCAL_COMMAND_STATUS,
        "`/stop` 仍走本地分支（不进跑轮）"
    );
    assert!(
        out.task_id.is_empty(),
        "本地指令一个任务都不建，task_id 必须如实为空而不是编一个：{}",
        out.task_id
    );
    assert!(!out.cancelled, "本地指令那一趟没有跑轮可停");

    model.release(0);
    let (_events, out) = worker.join().expect("跑轮线程不该 panic");
    let store = common::open_store();
    let task = task_of(&store, &out.task_id);
    assert_eq!(
        task.status,
        TaskStatus::Cancelled,
        "`/stop` 必须**真的**停掉在跑的那一轮（这是本用例的全部意义）"
    );
    assert!(out.cancelled, "被 `/stop` 停掉的轮次收尾时也必须自报 cancelled");
}

// ─── 5. `task_id` 是本轮那一个，不是「库里最新的」───

/// 端到端钉 `task_id` 的口径：它指**本轮**的任务，不随别人的轮次变。
///
/// 做法：连跑两轮（**不同会话**，各建一个任务），第一条的结果必须仍指回第一条。
/// 若实现是「取库里最新的那条」，第一条的结果会指向第二条 —— 这条会立刻红。
#[test]
fn run_result_task_id_is_this_turn_not_the_newest_task_in_the_store() {
    let _guard = lock_env();
    use_echo_engine();
    let convo_a = fresh_convo(&common::uniq("taskid-a"));
    let convo_b = fresh_convo(&common::uniq("taskid-b"));

    let (_, first) = send("第一条", &convo_a);
    let (_, second) = send("第二条", &convo_b);

    assert!(!first.task_id.is_empty(), "第一轮也要给出 task id");
    assert!(
        first.task_id != second.task_id,
        "两轮是两个任务，不该是同一个 id"
    );
    let store = common::open_store();
    // 前提自检：此刻库里最新的确实是第二条。
    let newest = store
        .list_tasks(1)
        .expect("列任务")
        .into_iter()
        .next()
        .expect("库里至少有两个任务");
    assert_eq!(
        newest.id, second.task_id,
        "前提自检：最新那条是第二轮"
    );
    // 交叉断言：第一轮的结果指回**它自己**那一行。
    let first_row = task_of(&store, &first.task_id);
    assert_eq!(first_row.title, "第一条", "task_id 指错了任务");
    assert_eq!(first_row.conversation_id.as_deref(), Some(convo_a.as_str()));
    assert_ne!(
        first.task_id, newest.id,
        "第一轮的 task_id 不能是「库里最新的那条」"
    );
    assert!(!first.cancelled && !second.cancelled, "没人叫停过，不该报 cancelled");
}

// ─── 6. 跑完即注销 ───

/// 轮次结束后停止键如实报「没有在跑」—— 即登记表在守卫析构时真的清了。
///
/// 反向断言：残留一枚令牌会让用户在一个**早已结束**的轮次上得到「已递停止信号」
/// 的回执，而实际上什么都没得停（而且下一次跑轮会被 C0 的「入轮清旗」抹掉，
/// 于是这枚残留令牌永远停在「已翻」的状态，污染它覆盖到的任何一轮）。
#[test]
fn a_finished_turn_unregisters_its_token_so_cancel_reports_the_truth() {
    let _guard = lock_env();
    use_echo_engine();
    let convo = fresh_convo(&common::uniq("unregister"));
    let (_, out) = send("跑完就注销", &convo);
    assert!(!out.task_id.is_empty());

    let err = tauri::async_runtime::block_on(neobot_run_cancel(Some(convo.clone())))
        .expect_err("跑完之后必须报「没有在跑」");
    assert!(err.contains("没有在跑"), "要说清没有可停的轮次：{err}");
}

// ─── 7. 工具 chips 取本轮的 steps，不取「库里最新那条」───

/// `collect_labels` 的 `list_tasks(1).first()` 是「库里最新那条任务」的 steps，
/// 于是并发的另一轮跑完时，本轮的 chips 会列出**别人**调过的工具。
///
/// 复现（确定性，靠假端点逐个放行）：A 在它的**第 2 跳**上被拦住时，让 B（纯
/// 回复、无工具）跑完 —— 此刻库里最新的是 B。然后放行 A，A 收尾。
/// 断言：A 的 chips 里有 A 真的调过的 `read_file`，而 B 的 chips 为空。
/// 若还按「库里最新那条」取 steps，A 会拿到 B 的（空）⇒ 这条会红。
#[test]
fn the_tools_chip_comes_from_this_turn_not_from_a_concurrently_finished_turn() {
    let _guard = lock_env();
    let model = FakeModel::start();
    use_fake_http(&model.base_url);
    let convo_a = fresh_convo(&common::uniq("tools-a"));
    let convo_b = fresh_convo(&common::uniq("tools-b"));

    // A 的第 1 跳要一个真会被执行的工具（`read_file` 默认放行，路径在工作区内）。
    model.script(&tool_body(&[("read_file", r#"{"path":"a.txt"}"#)], "先读个文件"));
    let a = {
        let convo = convo_a.clone();
        std::thread::spawn(move || send("A 那一轮", &convo))
    };
    model.wait_requests(1);
    model.release(0); // A 执行 read_file → 进第 2 跳 → 请求 1 到达
    model.wait_requests(2); // A 停在第 2 跳上（还没放行）

    // B：纯回复、无工具。此刻 B 跑完 ⇒ 库里最新的是 B 的任务。
    model.script(&plain_body("neobot(fake): B 答完了"));
    let b = {
        let convo = convo_b.clone();
        std::thread::spawn(move || send("B 那一轮", &convo))
    };
    model.wait_requests(3);
    model.release(2); // 只放行 B
    let (_events_b, out_b) = b.join().expect("B 跑轮线程不该 panic");

    model.release(1); // 再放行 A 的第 2 跳
    let (_events_a, out_a) = a.join().expect("A 跑轮线程不该 panic");

    let store = common::open_store();
    // 前提自检：B 的任务确实是库里最新的，且它一个工具都没调。
    let newest = store
        .list_tasks(1)
        .expect("列任务")
        .into_iter()
        .next()
        .expect("库里至少有两个任务");
    assert_eq!(newest.id, out_b.task_id, "前提自检：最新那条是 B 的");
    assert!(out_b.labels.tools.is_empty(), "前提自检：B 没调工具");
    let b_steps = store.list_step_tools(&out_b.task_id).expect("读 B 的 steps");
    assert!(
        !b_steps.iter().any(|tool| tool == "read_file"),
        "前提自检：B 的 steps 里不该有工具（纯回复轮只有一行 reply）：{b_steps:?}"
    );
    // 真正的断言：A 的 chips 来自 A 自己的 steps。
    assert!(
        store
            .list_step_tools(&out_a.task_id)
            .expect("读 A 的 steps")
            .iter()
            .any(|tool| tool == "read_file"),
        "前提自检：A 真的调过 read_file"
    );
    assert!(
        out_a.labels.tools.iter().any(|tool| tool == "read_file"),
        "A 的工具 chips 必须是 A 自己的 steps，而「库里最新那条」是 B 的（空的）：{:?}",
        out_a.labels.tools
    );
}

// ─── 8. UI 侧：取消失败必须有可见反馈（静态锁）───

/// 极小的 `ntInvoke("x"` / `invoke("x"` 探测（不引 `regex` 依赖）。
///
/// 只为让上面的静态锁同时覆盖 typed wrapper 与裸 invoke 两种写法。
fn regex_lite_find_nt_invoke(source: &str, needle: &str) -> bool {
    let mut from = 0usize;
    while let Some(rel) = source[from..].find(needle) {
        let at = from + rel;
        let before = &source[..at];
        if before.ends_with("ntInvoke(") || before.ends_with("invoke(") {
            return true;
        }
        from = at + needle.len();
    }
    false
}

/// 停止键的失败分支**必须**把失败显示出来，且乐观隐藏要能被撤销。
///
/// 为什么是静态锁：这个仓库的 UI 侧机器检查是 `selftest.ts`（纯函数），
/// `main.ts` 的 DOM 交互没有测试位。故这里钉住三件**会退化**的事：
/// ① 停止键不再只是「置个 flag 让渲染跳过增量」（那是旧行为，等于没停）；
/// ② 失败有一行进对话流（`pushSys`）＋ 一个 toast；
/// ③ 取消失败时把乐观的「收起」撤销（`stopRequested` 清回、运行态还回去）。
#[test]
fn the_stop_key_calls_the_backend_and_shows_failures_instead_of_silently_hiding() {
    let source = include_str!("../frontend/src/main.ts");

    // ① 真的去调后端取消命令（不是只置 flag）。
    //
    // 注意匹配的是 **typed wrapper 的两种入口**（`ntInvoke` / 裸 `invoke`），
    // 而**不是**某一个字面量：上一轮这条曾被 `ntInvoke("neobot_run_cancel"`
    // 弄红过一次 —— 原因是它把「**实现细节**（裸 invoke）」当成了契约。
    // 契约是「去调后端取消命令」，裸 invoke 只是其中一种写法（上一轮还曾是
    // 绕过类型表的一条旁路，现已并回 `invoke.ts`）。用带 `N?` 前缀的正则
    // 同时覆盖两种入口，测试才对着行为而不是对着字面量。
    let calls_backend = {
        let re = regex_lite_find_nt_invoke(&source, r#""neobot_run_cancel""#);
        re
    };
    assert!(
        calls_backend,
        "停止键必须调后端的取消命令（`ntInvoke` 或裸 `invoke` 都算）；只置 shell.stopRequested 等于没停"
    );
    assert!(
        !source.contains("if (shell.running) { shell.stopRequested = true;"),
        "旧的「只置 flag 就当停了」那条分支不许复活 —— 那是本缺陷的原来形态"
    );

    // ② 取消失败 ⇒ 可见反馈（对话流一行 + toast）。
    assert!(
        source.contains("停止失败，这一轮没有停住："),
        "取消失败必须有一行进对话流（toast 会消失，这一行会留在记录里）"
    );
    assert!(
        source.contains(r#"toast("停止失败", "err")"#),
        "取消失败也必须弹一个 toast"
    );

    // ③ 乐观隐藏可撤销 + 两个终态都有回执。
    assert!(
        source.contains("shell.stopRequested = false;\n        setRunning(true);"),
        "取消失败时要把「收起」撤销并把运行态还回去（这一轮还在跑）"
    );
    assert!(
        source.contains("已停（这一轮被叫停了"),
        "真停住了也要有一行回执（依据是后端回库核对过的 cancelled）"
    );
    assert!(
        source.contains("停止请求发出时这一轮已经跑完了（没停住）"),
        "「信号递到了但没停住」也必须有回执 —— 否则用户会以为停了"
    );
}
