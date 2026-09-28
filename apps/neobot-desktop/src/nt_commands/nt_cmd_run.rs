//! `nt_cmd_run` — 跑轮（阻塞/流式）+ 引擎解析 + 回复标签 + 斜杠指令分流.
//!
//! 每轮返回 `NeobotRunResult { status, labels, task_id, cancelled }`
//! （前端气泡顶部 chips 直消）：
//! `model` 来自分辨出的引擎模型名，`mode` 为 direct/passthrough/fallback 三态，
//! `tools` 取**本轮 task**（`task_id`）的 `steps` 工具列（`nt_reply_tag` 口径过滤），
//! `usage` 取账本前后差值 + `nt_cost` 同律计价（未知不猜）。
//!
//! 另有一条**在跑轮之前**的分流：桌面聊天框里的斜杠指令
//! （`/help` `/status` `/stop` `/new`）。见文件末尾「斜杠指令分流」一节。

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use super::{EngineSelection, NeobotRunResult, load_config, open_store, resolve_engine};
use neotrix_neobot::nt_agent::{RunContext, run_local_turn_cancellable};
use neotrix_neobot::nt_cancel::StopToken;
use neotrix_neobot::nt_channel_cmd::{self, Command};
use neotrix_neobot::{
    Actor, EngineAdapter, HttpEngine, LocalEchoEngine, NeobotConfig, NeobotStore, OpencodeEngine,
    ReplyMode, TaskStatus, TurnLabels, labels_for_turn,
};

// ═══ 停止令牌注册表（桌面真停止）═══
//
// ## 为什么需要它
//
// `StopToken` 是**跨执行流的信号载体**（`nt_cancel.rs`：`Arc<AtomicBool>` 语义，
// 克隆共享同一面旗）。跑轮那侧要把它**传进去**，停止键那侧要能**找到它**并翻它。
// 二者是两次独立的 IPC 调用（`neobot_run_stream` 与 `neobot_run_cancel`），
// 中间隔着一个进程级的表 —— 没有这张表，停止键就找不到任何东西可翻，
// 而「找不到就假装停了」正是本项目最该避免的那类缺陷。
//
// ## 为什么按 `convo_id` 索引（而不是 `task_id`）
//
// 索引键必须是**调用停止键的那一刻前端已经知道**的东西。按 `task_id` 索引
// 不可行，而且不是「还没做」而是**做不到**：任务行是 `run_local_turn_inner`
// 在函数**内部**建的（`nt_agent.rs`），跑轮期间没有任何回调把 id 递出来
// （`on_delta` / `on_step` 的签名里只有正文与工具名）。而前端在按下停止键的
// 那一刻手里只有 `convoId`（这一轮就是在这个会话里发出去的）。
//
// 前端此前只能用「窗口内新建 ＋ 归属本会话」去**猜**本轮任务（`turn_task.ts`），
// 那正是间接证据的来源。现在 `NeobotRunResult.task_id` 把真 id 端到端补上了，
// 但它**跑完**才到 —— 仍然晚于「用户按停止」那一刻。故取消键只能是 `convo_id`。
//
// ## 注册与注销时机
//
// - **注册**：`neobot_run_stream` / `neobot_run` 的**函数入口**，即
//   `spawn_blocking` **之前**。这一点是刻意的：`run_local_turn` 是
//   「先建任务再跑轮」，若等任务建好再登记，中间那一段就是**没有登记**的
//   （停止键会如实报「找不到」，而用户看到的是「我明明在跑」）。
//   提前登记把「任务已建、轮次在跑」整段连同它前面的一段一起盖住了。
// - **注销**：RAII 守卫（[`RunRegistration`] 的 `Drop`）随命令函数返回而析构，
//   含**正常收尾**与**任何 `?` / `return Err` 早退**。没有「忘了注销」的路径。
//
// ## 并发安全
//
// - 表是 `Mutex<HashMap>`，**只在毫秒级的读改写期间持锁**：不跨 `.await`、
//   不跨跑轮、不跨任何 IO。跑轮本体拿着的是 `StopToken` 的**克隆**
//   （`Arc<AtomicBool>`），不持锁跑完全程 ⇒ 无锁序问题、无跨 await 持锁。
// - **同键覆盖 + 世代号**：新登记的直接盖掉旧的（桌面单窗口一次只跑一轮，
//   `main.ts` 的 `shell.running` 闸使同会话重入实际不可达；真发生了，
//   「能停的」是最新那一轮，这比让两轮都以为自己可停更诚实）。
//   守卫析构时**只在世代号仍是自己**时删 —— 否则先发起的旧轮次收尾时会把
//   新一轮的令牌**误删**，那会让新一轮变得「停不了」而用户毫无察觉。
// - 翻旗本身是 `Relaxed` 原子写（core 那侧的理由：置位与检查之间没有需要
//   保护的其他数据），并发翻同一面旗幂等。
//
// ## 找不到就**如实报错**
//
// `signal_run_cancel` 找不到登记时**返回 `Err`**，绝不静默成功：停止键
// 拿到的必须是「已经递了停止信号」或「没有可停的轮次」二者之一，没有第三种。

/// 登记表的一格：一枚令牌 + 一个世代号。
struct RunSlot {
    token: StopToken,
    seq: u64,
}

static RUN_REGISTRY: OnceLock<Mutex<HashMap<String, RunSlot>>> = OnceLock::new();
static RUN_SEQ: AtomicU64 = AtomicU64::new(0);

fn run_registry() -> &'static Mutex<HashMap<String, RunSlot>> {
    RUN_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

/// 登记键：会话 id。**没有会话就不可取消**（如实不登记，而不是编一个键）。
fn run_key(convo_id: Option<&str>) -> Option<String> {
    convo_id.map(str::trim).filter(|id| !id.is_empty()).map(str::to_owned)
}

/// RAII 登记守卫：析构时按世代号注销。
///
/// 刻意**不**用「无条件 `remove(key)`」：那会让先发起的旧轮次在收尾时删掉
/// 新一轮的令牌（新一轮于是变得「停不了」，而没人知道为什么）。
pub struct RunRegistration {
    key: Option<String>,
    seq: u64,
}

impl Drop for RunRegistration {
    fn drop(&mut self) {
        let Some(key) = self.key.as_ref() else {
            return;
        };
        // 锁毒：登记期间若被毒化，注销失败也不该把 panic 传染出去 ——
        // 残留一格令牌的后果只是「停止键报找不到」，而 panic 会掀翻整轮。
        let Ok(mut table) = run_registry().lock() else {
            return;
        };
        if table.get(key).is_some_and(|slot| slot.seq == self.seq) {
            table.remove(key);
        }
    }
}

/// 为这一轮登记一枚可被外部翻转的令牌，返回守卫（**必须**持有到命令返回）。
fn register_run(convo_id: Option<&str>, token: &StopToken) -> RunRegistration {
    let seq = RUN_SEQ.fetch_add(1, Ordering::Relaxed);
    let key = run_key(convo_id);
    if let Some(key) = key.as_ref() {
        if let Ok(mut table) = run_registry().lock() {
            table.insert(
                key.clone(),
                RunSlot {
                    token: token.clone(),
                    seq,
                },
            );
        }
    }
    RunRegistration { key, seq }
}

/// 翻掉某一轮正在跑的令牌。
///
/// `Err` 是**正常答案的一种**：没有可停的轮次（已经结束、或还没开始）。
/// 它**不**被吞成 `Ok` —— 「假装停了」比「停不了」坏得多。
fn signal_run_cancel(convo_id: Option<&str>) -> Result<String, String> {
    let Some(key) = run_key(convo_id) else {
        return Err("没带会话标识，无从知道该停哪一轮。".to_owned());
    };
    let table = run_registry()
        .lock()
        .map_err(|_| "停止登记表锁坏了，这一轮停不了。".to_owned())?;
    let Some(slot) = table.get(&key) else {
        return Err(format!(
            "这一轮没有在跑（可能已经结束，或还没开始）：会话 {key} 下没有可停的轮次。"
        ));
    };
    slot.token.cancel();
    Ok(key)
}

/// 停止键的回执（前端据此显示「已递停止信号」还是「停不了」）。
#[derive(Debug, Clone, serde::Serialize)]
pub struct NeobotRunCancelResult {
    /// 实际被翻掉令牌的那一轮的会话键（= 传入的 `convo_id`，回显以便前端核对）。
    pub convo_id: String,
    /// 停止信号**已经递到跑轮**（`true` = 令牌已置位）。
    ///
    /// 仍**不是**「这一轮一定停住了」：停止键按下时它可能已经跑完。
    /// 「真停住了」的判据是 `NeobotRunResult.cancelled`（回库核对过）。
    pub signalled: bool,
}

/// 停止键：把**正在跑**的那一枚 `StopToken` 翻掉。
///
/// 与 `neobot_run_stream` 是两次独立 IPC 调用，而后者跑在 `spawn_blocking`
/// 里 —— 桌面天然并发，这正是停止键能**真的**在跑轮途中生效的原因
/// （IM 侧跑轮期间不取新消息，`/stop` 到了也只能排队，轮次结束后才被读到）。
///
/// 找不到可停的轮次时**如实报错**（见 [`signal_run_cancel`]）。
#[tauri::command]
pub async fn neobot_run_cancel(convo_id: Option<String>) -> Result<NeobotRunCancelResult, String> {
    let key = signal_run_cancel(convo_id.as_deref())?;
    Ok(NeobotRunCancelResult {
        convo_id: key,
        signalled: true,
    })
}

/// 读最近若干条任务的 id 集（跑轮**之前**取一次当基线）。
///
/// 失败回空集：那样收尾时「新出现的 id」会被判成候选集**全量**，
/// 而收尾还有一道「必须恰好一个 + 归属本会话」才认 —— 认不出就不填，
/// 不会把错的 id 报给前端。
fn recent_task_ids(store: &NeobotStore) -> std::collections::HashSet<String> {
    store
        .list_tasks(TASK_WINDOW)
        .unwrap_or_default()
        .into_iter()
        .map(|task| task.id)
        .collect()
}

/// 任务 id 认领用的窗口大小。
///
/// 取 200：`list_tasks` 按 `created_at DESC` 排序，故本轮新建的任务必然在
/// 最前面；200 是为了让「基线」覆盖到足够多的存量任务（基线覆盖不全时，
/// 认领会**更严**而不是更松：老任务会被误列成候选，但归属/唯一性两道闸
/// 仍要过 —— 而老任务的 `created_at` 更早、id 也在基线里，正常都被排除）。
const TASK_WINDOW: i64 = 200;

/// 认领**本轮自己的** task id（不是「库里最新的那个」）。
///
/// `run_local_turn` 建的 id 只存在于函数内部、没有任何回调递得出来（见本文件
/// 头「为什么按 convo_id 索引」），所以桌面侧唯一能确证的口径是
/// **集合差**：跑轮前后各读一次 id 集，本轮**新建**的那一条就是本轮的。
/// 收窄条件两条，缺一不认：
/// 1. 恰好**一个**新 id（两个以上 = 有并发的别的跑轮/例程，认不出）；
/// 2. 它的 `conversation_id` 就是本轮会话（后台例程任务不认）。
///
/// 认不出返回**空串** —— 前端据此回落到间接判据（`turn_task.ts`），
/// 而不是拿一个「大概是它」的 id 去挂任务卡。
fn claim_turn_task_id(
    store: &NeobotStore,
    before: &std::collections::HashSet<String>,
    convo_id: Option<&str>,
) -> String {
    let fresh: Vec<String> = store
        .list_tasks(TASK_WINDOW)
        .unwrap_or_default()
        .into_iter()
        .filter(|task| !before.contains(&task.id))
        .filter(|task| match convo_id {
            Some(convo) => task.conversation_id.as_deref() == Some(convo),
            None => true,
        })
        .map(|task| task.id)
        .collect();
    match fresh.as_slice() {
        [only] => (*only).clone(),
        _ => String::new(),
    }
}

/// 本轮**真的**被用户叫停了吗 —— **回库核对**，不看令牌。
///
/// 停止请求送达 ≠ 这一轮被停掉（它可能在请求到达前就跑完了）。前端要拿
/// 「停掉了」这句话去回执用户，故判据必须落在**这一轮任务自己的终态**上。
fn turn_was_cancelled(store: &NeobotStore, task_id: &str) -> bool {
    if task_id.is_empty() {
        return false;
    }
    store
        .get_task(task_id)
        .ok()
        .flatten()
        .is_some_and(|task| task.status == TaskStatus::Cancelled)
}

#[tauri::command]
pub async fn neobot_run(
    title: String,
    text: String,
    actor_name: String,
    convo_id: Option<String>,
    model_provider: Option<String>,
    model_name: Option<String>,
) -> Result<NeobotRunResult, String> {
    // 停止令牌在**进 spawn_blocking 之前**登记（覆盖「任务已建、轮次在跑」
    // 连同它前面那一整段，理由见「停止令牌注册表」一节）。
    let stop = StopToken::new();
    let registration = register_run(convo_id.as_deref(), &stop);
    let out = tauri::async_runtime::spawn_blocking(move || {
        let config = load_config()?;
        let store = open_store(&config)?;
        let me = actor_name.trim();
        let me = if me.is_empty() { "owner" } else { me };
        let convo = convo_id.as_deref();
        let (engine, mode, model) =
            resolve_run_engine(&store, &config, model_provider, model_name)?;
        let before = ledger_totals(&store);
        let task_ids_before = recent_task_ids(&store);
        let ctx = RunContext {
            store: &store,
            config: &config,
            engine: engine.as_ref(),
            actor: Actor::Person,
            actor_name: me,
            title: &title,
            user_text: &text,
            convo_id: convo,
        };
        let status =
            run_local_turn_cancellable(&ctx, None, None, Some(&stop)).map_err(|err| err.to_string())?;
        let task_id = claim_turn_task_id(&store, &task_ids_before, convo);
        let labels = collect_labels(&store, &model, mode, before, &task_id);
        Ok(NeobotRunResult {
            status: status.as_str().to_owned(),
            labels,
            cancelled: turn_was_cancelled(&store, &task_id),
            task_id,
        })
    })
    .await
    .map_err(|err| err.to_string())?;
    drop(registration);
    out
}

fn resolve_run_engine(
    store: &NeobotStore,
    config: &NeobotConfig,
    model_provider: Option<String>,
    model_name: Option<String>,
) -> Result<(Box<dyn EngineAdapter>, ReplyMode, String), String> {
    // 模型走内部（架构律）：配对且在线时，一律先走晶体核心（任务拆解→能力网分发→聚合），
    // 模型名透传由晶体池内解析。例外：`env` 本地直连舱（Ollama/LM Studio，无晶体等价）；
    // 晶体离线/未配对 → 回落旧链（显式端点直连 → CLI 配对 → 配置引擎），不断档。
    // 路由三态：核心路径=passthrough，显式/配置直连=direct，本地回显=fallback。
    let explicit_env = matches!(&model_provider, Some(p) if p == "env");
    if !explicit_env {
        let memory = neotrix_neobot::nt_memory::memory_for_config(config);
        // 选中模型名透传（池子模型由晶体池内解析；空即配对模型）。
        let wanted = model_name.as_deref().filter(|m| !m.trim().is_empty());
        if let Ok(Some(http)) = neotrix_neobot::core_engine_with_model(store, wanted) {
            let name = http.model_name().trim();
            let model = if name.is_empty() {
                wanted
                    .unwrap_or(neotrix_neobot::nt_core::CORE_MODEL)
                    .trim()
                    .to_owned()
            } else {
                name.to_owned()
            };
            return Ok((
                Box::new(http.with_memory_context(memory)),
                ReplyMode::Passthrough,
                model,
            ));
        }
    }
    if let Some((engine, model)) = resolve_model_engine(store, config, model_provider, model_name)? {
        return Ok((engine, ReplyMode::Direct, model));
    }
    let memory = neotrix_neobot::nt_memory::memory_for_config(config);
    // 灵魂嵌入：无显式指定且核心活着 → 走晶体核心（桌面/CLI 同律）。
    if let Ok(Some(http)) = neotrix_neobot::core_engine(store) {
        let name = http.model_name().trim();
        let model = if name.is_empty() {
            neotrix_neobot::nt_core::CORE_MODEL.to_owned()
        } else {
            name.to_owned()
        };
        return Ok((
            Box::new(http.with_memory_context(memory)),
            ReplyMode::Passthrough,
            model,
        ));
    }
    // CLI 通道配对（Zen）：配置引擎之外，配对行即路由（opencode 坏了就地回落）。
    // 本地 CLI 直驱，口径记 direct（非晶体服务端透传）。
    if let Ok(Some(pair)) = store.get_core_pair() {
        if pair.via == "cli" {
            if let Ok(engine) = OpencodeEngine::new(&pair.model) {
                return Ok((Box::new(engine), ReplyMode::Direct, pair.model));
            }
        }
    }
    match resolve_engine(config)? {
        EngineSelection::Echo => Ok((
            Box::new(LocalEchoEngine),
            ReplyMode::Fallback,
            "echo".to_owned(),
        )),
        EngineSelection::Cli(engine) => {
            let model = engine.engine_id().to_owned();
            Ok((Box::new(engine), ReplyMode::Direct, model))
        }
        EngineSelection::Opencode(engine) => {
            let model = engine.model().to_owned();
            Ok((Box::new(engine), ReplyMode::Direct, model))
        }
        EngineSelection::Http(engine) => {
            let model = engine.model_name().to_owned();
            Ok((
                Box::new(engine.with_memory_context(memory)),
                ReplyMode::Direct,
                model,
            ))
        }
    }
}

#[tauri::command]
pub async fn neobot_run_stream(
    title: String,
    text: String,
    actor_name: String,
    convo_id: Option<String>,
    model_provider: Option<String>,
    model_name: Option<String>,
    on_event: tauri::ipc::Channel<StreamEvent>,
) -> Result<NeobotRunResult, String> {
    // ① 斜杠指令分流：在**进入跑轮之前**截住，命中就本地回执，一个 token 都不烧。
    //    （IM 侧同一件事在 `nt_channel_dispatch.rs:84`；桌面这条线此前完全缺失，
    //    于是 `/stop` 被当**字面文本发给模型**，见 `DESIGN-CHANNEL-DISPATCH.md` §0.4。）
    match intercept_local_command(
        &text,
        convo_id.as_deref(),
        model_provider.as_deref(),
        model_name.as_deref(),
    ) {
        Ok(Some(outcome)) => {
            // ①b `/stop` 在这里**真的**去停那一轮（而不是回一句「停不了」）。
            //
            // 桌面这条 `/stop` 是**另一次** IPC 调用（IM 侧则要等渠道取新消息，
            // 跑轮期间取不到 ⇒ 只能排队、轮次结束后才被读到，那时就无轮可停）。
            // 桌面天然并发：上一轮还跑在 `spawn_blocking` 里，这一次进来把它翻掉。
            //
            // **失败（没有可停的轮次）不在这里加话** —— 那份回执由 core 的
            // `local_reply` 持有（`nt_channel_cmd.rs`，**不在本切片所有权内**，
            // 一个字都不改），而它现在的措辞恰好就是「停不了」：**没有可停的
            // 轮次时它字面为真**，有可停的轮次时它才变成陈旧措辞（那是回执
            // 该怎么改的问题，由核验者决定）。另：本地分支的事件形状被既有
            // 冒烟测试钉死为恰好 `[delta, done]`，加一条 delta 就会撞它。
            if outcome.command == Command::Stop.as_str() {
                // 结果刻意**丢掉**（`drop` 而不是 `let _ =`：后者撞
                // `clippy::let_underscore_must_use`）。理由有两条：回执归 core
                // 所有我们不能改；本地分支的事件形状被既有冒烟测试钉死。
                // 唯一说得清真相的地方是**那一轮自己的收尾回执**（`cancelled`）。
                drop(signal_run_cancel(convo_id.as_deref()));
            }
            for event in local_reply_events(&outcome) {
                on_event.send(event).map_err(|err| err.to_string())?;
            }
            return Ok(local_run_result());
        }
        // 不是指令：普通文本**逐字**照原样进跑轮。
        Ok(None) => {}
        // 认得出是指令、却造不出回执（库打不开）：如实报错。
        // 绝不把 `/help` 当字面文本发给模型 —— 那正是本缺陷本身。
        Err(err) => return Err(err),
    }
    // ② 停止令牌登记：放在 `spawn_blocking` **之前**，于是从命令入口起
    //    （含「任务已建、轮次在跑」那一段）到收尾，整段都有一枚可被翻的令牌。
    //    守卫 `registration` 活到函数返回 ⇒ 任何早退路径都会注销（见本文件
    //    「停止令牌注册表」一节）。
    let stop = StopToken::new();
    let registration = register_run(convo_id.as_deref(), &stop);
    let (tx, mut rx) = tokio::sync::mpsc::channel::<StreamEvent>(128);
    let run = tokio::task::spawn_blocking(move || -> Result<NeobotRunResult, String> {
        let config = load_config()?;
        let store = open_store(&config)?;
        let me = actor_name.trim().to_owned();
        let me = if me.is_empty() { "owner".to_owned() } else { me };
        let convo = convo_id.as_deref();
        let (engine, mode, model) =
            resolve_run_engine(&store, &config, model_provider, model_name)?;
        let before = ledger_totals(&store);
        // 本轮任务 id 的基线：必须在跑轮**之前**取（跑轮会新建本轮任务）。
        let task_ids_before = recent_task_ids(&store);
        let mut emit_delta = |delta: &str| {
            let _ = tx.blocking_send(StreamEvent::Delta { text: delta.to_owned() });
        };
        let mut emit_step = |tool: &str, ok: bool, output: &str| {
            let _ = tx.blocking_send(StreamEvent::Step {
                tool: tool.to_owned(),
                ok,
                output: output.to_owned(),
            });
        };
        let ctx = RunContext {
            store: &store,
            config: &config,
            engine: engine.as_ref(),
            actor: Actor::Person,
            actor_name: &me,
            title: &title,
            user_text: &text,
            convo_id: convo,
        };
        // 与 `run_local_turn_stream_as` 走**同一个内层**（差别只有 `stop`：
        // `Some(&stop)` ⇒ 四个取消检查点生效；`None` 时 core 会造一枚无人
        // 能置位的私有令牌，行为与本函数存在之前逐字相同）。
        let status = run_local_turn_cancellable(
            &ctx,
            Some(&mut emit_delta),
            Some(&mut emit_step),
            Some(&stop),
        )
        .map_err(|err| err.to_string())?;
        let _ = tx.blocking_send(StreamEvent::Done { status: status.as_str().to_owned() });
        // 收尾：`task_id` 认**本轮**那一个（不是库里最新的），`cancelled`
        // 回库核对（令牌被翻 ≠ 这一轮真被停掉）。
        let task_id = claim_turn_task_id(&store, &task_ids_before, convo);
        let labels = collect_labels(&store, &model, mode, before, &task_id);
        Ok(NeobotRunResult {
            status: status.as_str().to_owned(),
            labels,
            cancelled: turn_was_cancelled(&store, &task_id),
            task_id,
        })
    });
    while let Some(event) = rx.recv().await {
        on_event.send(event).map_err(|err| err.to_string())?;
    }
    let out = run.await.map_err(|err| err.to_string())?;
    // 这一轮真的结束了才注销：注销之后停止键就会如实报「没有可停的轮次」。
    drop(registration);
    out
}

/// 流式事件（delta 增量 / step 工具行 / done 终态）。
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StreamEvent {
    Delta { text: String },
    Step { tool: String, ok: bool, output: String },
    Done { status: String },
}

/// 账本总量快照（engine+model 全聚合；失败回零，不挡主流程）。
fn ledger_totals(store: &NeobotStore) -> (i64, i64, f64) {
    let sums = store.ledger_sums().unwrap_or_default();
    let mut in_tokens = 0i64;
    let mut out_tokens = 0i64;
    let mut cost = 0.0f64;
    for (_, _, inn, out, c) in sums {
        in_tokens = in_tokens.saturating_add(inn.max(0));
        out_tokens = out_tokens.saturating_add(out.max(0));
        if c.is_finite() && c > 0.0 {
            cost += c;
        }
    }
    (in_tokens, out_tokens, cost)
}

/// 本轮标签组装（账本差值 + **本轮 task** 的步骤工具；任一步失败即降级空工具/零耗）。
///
/// `task_id` 是**调用方认领到的本轮任务 id**（见 [`claim_turn_task_id`]）。
/// 此前这里是 `list_tasks(1).first()` —— 那是「库里**最新**的那条任务」，
/// 与「本轮的」没有必然关系：队内并行、后台例程、别的窗口都会让它取到别人的
/// steps，于是这一轮的气泡 chips 会列出**别人**调过的工具。认不出本轮 id
/// （空串）就交出工具列（空），而不是拿一个「大概是它」的去填。
fn collect_labels(
    store: &NeobotStore,
    model: &str,
    mode: ReplyMode,
    before: (i64, i64, f64),
    task_id: &str,
) -> TurnLabels {
    let after = ledger_totals(store);
    let in_tokens = after.0.saturating_sub(before.0).max(0);
    let out_tokens = after.1.saturating_sub(before.1).max(0);
    let cost = if after.2.is_finite() && before.2.is_finite() {
        (after.2 - before.2).max(0.0)
    } else {
        0.0
    };
    let step_tools = if task_id.is_empty() {
        Vec::new()
    } else {
        store.list_step_tools(task_id).unwrap_or_default()
    };
    labels_for_turn(model, mode, step_tools, in_tokens, out_tokens, cost)
}

fn resolve_model_engine(
    store: &NeobotStore,
    config: &NeobotConfig,
    model_provider: Option<String>,
    model_name: Option<String>,
) -> Result<Option<(Box<dyn EngineAdapter>, String)>, String> {
    let (provider, model) = match (model_provider, model_name) {
        (Some(provider), Some(model)) if !model.trim().is_empty() => (provider, model),
        _ => return Ok(None),
    };
    let memory = neotrix_neobot::nt_memory::memory_for_config(config);
    let display = model.trim().to_owned();
    let engine = if provider == "env" {
        let base = std::env::var("NEOBOT_BASE_URL")
            .unwrap_or_else(|_| neotrix_neobot::nt_http_engine::DEFAULT_BASE_URL.to_owned());
        let key = std::env::var("NEOBOT_API_KEY").unwrap_or_default();
        let http_config = neotrix_neobot::HttpEngineConfig {
            base_url: base.trim().trim_end_matches('/').to_owned(),
            model: model.trim().to_owned(),
            timeout_secs: neotrix_neobot::nt_http_engine::DEFAULT_TIMEOUT_SECS,
        };
        HttpEngine::new(http_config, key).map_err(|err| err.to_string())?
    } else {
        let item = store
            .get_provider(&provider)
            .map_err(|err| err.to_string())?
            .ok_or_else(|| format!("no such provider '{provider}'"))?;
        if !item.enabled {
            return Err(format!("provider '{provider}' is off"));
        }
        item.http_engine(Some(&model)).map_err(|err| err.to_string())?
    };
    Ok(Some((
        Box::new(engine.with_memory_context(memory)),
        display,
    )))
}

// ═══ 斜杠指令分流 ═══
//
// **要修的缺陷**（`DESIGN-CHANNEL-DISPATCH.md` §0.4 第 2 条）：桌面聊天走
// `neobot_run_stream`，**不经过** `nt_channel_cmd::parse`。于是用户在聊天框里
// 打 `/stop`，这串字符会被**当成字面文本发给模型**，模型回一句「我没有停止
// 功能」—— 这是把控制指令当自然语言，UX 上等于这四个命令在桌面不存在。
//
// 三条设计律（改这一节前先读完）：
//
// 1. **复用而非重写**：识别走 `nt_channel_cmd::parse`，回执走
//    `nt_channel_cmd::local_reply`。两端口径**结构同源**，不是两份相似实现。
//    `parse` 只认「开头的 `/` + 已知动词」，所以 `/usr/bin/env`、`/unknown`、
//    `a /new` 一律回 `None` —— 用户发个路径不会被吞，两端同口径。
//
// 2. **不进跑轮**：命中就 `return`。`Channel` 事件路径（`emit_delta`/
//    `emit_step`/`Done`）**一字不动**，本地分支只额外发它自己那两条事件。
//    普通文本消息的行为**逐字不变**。
//
// 3. **只做能诚实做到的**：`nt_channel_cmd::execute` 的签名要
//    `store + &BotRow + channel_name`。`store` 桌面有；**`BotRow` 桌面没有**
//    —— 它是 IM 侧 `channel × bot_id` 的绑定行（Telegram 机器人 / 群绑定），
//    桌面聊天不经过那一层。**本节拒绝为了「跑通」而造一个假 `BotRow`**
//    （那是把 IM 上下文编出来，回执就成了 fiction）。于是：
//    - `/help`、`/stop` 确实与 IM 上下文无关 → 走 core 的 `local_reply`
//      （**同一份源码**，桌面不抄第二份；`DESKTOP_CHANNEL_NAME` 是唯一差异）；
//    - `/status` 只报桌面**真有**的状态 → 做，但换掉两行 IM 专属概念；
//    - `/new` 是「切会话」→ 桌面**不支持**并如实说（决断见 `NEW_UNSUPPORTED`）。
//
// `local_reply` 回 `None` 的意思正是「这条命令需要渠道上下文，别假装能本地答」
// —— 上面后两条就是靠这个信号落到桌面自己的如实答复上的。
//
// 4. **`/stop` 现在真的停**（2026-09-28 切片 C3，与本节原有三条并列）：
//    桌面 `/stop` 是**另一次** IPC 调用，而上一轮还跑在 `spawn_blocking` 里
//    ⇒ 桌面天然并发，那枚 `StopToken` 在这一趟被翻掉，跑轮在下一个检查点
//    （C1 跳边界 / C2 工具边界 / C3 sleep 前）收手并落 `TaskStatus::Cancelled`。
//    **回执文本一个字都没改**（它归 core `nt_channel_cmd::local_reply` 所有，
//    且被既有冒烟测试按字节钉住）—— 于是它在「没有可停的轮次」时**字面为真**，
//    在「真的停掉了」时**变成陈旧措辞**。回执该改成什么，是核验者的决定，
//    不是写这一行的人的（见 `neobot_run_stream` 里 ①b 那段注释）。

/// 桌面自报家门用的名字（`local_reply` 的 `channel_name` 实参，唯一的桌面差异）。
pub const DESKTOP_CHANNEL_NAME: &str = "NeoTrix 桌面";

/// 本地指令分支的 `status`。
///
/// 不进跑轮 ⇒ 这一轮**没有** `TurnStatus`，故它不是 `TurnStatus::as_str()`
/// 里的任何一个值；前端只当不透明串展示（团队会话会拼成「本地指令 · 0s · 0步」）。
pub const LOCAL_COMMAND_STATUS: &str = "本地指令";

/// 气泡顶部 chips 的「模型」标签。**这一轮没有模型**，
/// 故如实写「本地指令」而不是编一个模型名上去。
const LOCAL_COMMAND_MODEL: &str = "本地指令";

/// 一次本地指令的处理结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalCmdOutcome {
    /// 命中的动词（`Command::as_str()` 口径，便于日志与测试）。
    pub command: &'static str,
    /// 回给用户的话（**整段一次发完** —— 这不是模型的增量流）。
    pub reply: String,
}

/// `/new` 在桌面**不支持**，并说清为什么 + 真正的替代路径。
///
/// **产品决断：不做。** IM 的 `/new` 是「建新会话 + 把机器人的绑定切过去」
/// （`CommandOutcome::switch_convo_to`，`nt_channel_dispatch` 侧消费）。而桌面
/// 当前会话由**界面**（左侧会话栏）持有，聊天里发一句话，后端**没有任何通道**
/// 能把界面切到那个新会话上去。
///
/// 硬做只有两条路，都比「如实说不支持」更糟：
/// - 造一个假 `BotRow` 骗过 `execute` → 那是**编造 IM 上下文**，回执变成 fiction；
/// - 建一个新会话却让界面留在原地 → 用户以为切了，下一句其实还进旧会话。
///
/// 所以桌面 `/new` 明确拒绝，并把「真正能开新会话的地方」指出来。
///
/// 注意：回执里**不出现**「已重置 / 已开新会话」这些串 —— 哪怕是否定句式
/// （「我不会说已重置」）也不行：用户是在气泡里扫一眼找结论的，那几个字
/// 一旦出现，扫过去的人读到的就是「已重置」。测试里那条反向断言专门盯这个。
const NEW_UNSUPPORTED: &str = concat!(
    "/new 在桌面这边做不了：它是 IM 侧「把机器人的绑定切到另一个会话」的动作，\n",
    "而桌面的当前会话由界面（左侧会话栏）持有 —— 聊天里发一句话切不过去。\n",
    "所以这里给不出一个真的「开好了」——那会是一句假的。\n",
    "要开新会话：在左侧会话栏新建一个（或点别的会话），接着说就行。",
);

/// 发送前分流：不是已知指令回 `None`，调用方**照原样**把它送进跑轮。
///
/// - `Ok(Some(_))`：本地已处理，**不许**再进跑轮；
/// - `Ok(None)`：普通文本（见 [`nt_channel_cmd::parse`] 的口径），行为一字不变；
/// - `Err(_)`：认得出是指令、却造不出回执（如库打不开）→ 如实报错。
///   刻意**不**降级成「当普通文本发给模型」—— 那正是本缺陷本身。
pub fn intercept_local_command(
    text: &str,
    convo_id: Option<&str>,
    model_provider: Option<&str>,
    model_name: Option<&str>,
) -> Result<Option<LocalCmdOutcome>, String> {
    // **复用 IM 的识别口径**（不重写、不抄一份动词表）。
    let Some(command) = nt_channel_cmd::parse(text) else {
        return Ok(None);
    };
    // `/help` 与 `/stop` 与渠道无关 → 走 core 的 `local_reply`，**不抄第二份**。
    //
    // 这两条的文本**只存在于** `nt_channel_cmd::local_reply` 一处，而 IM 侧的
    // `execute` 也委托给它 —— 故两端口径是**结构同源**，不是「两份相似实现 +
    // 一条字节相等的断言」。`DESKTOP_CHANNEL_NAME` 是唯一差异（help 里自报家门）。
    //
    // `local_reply` 回 `None` = 「这条命令需要渠道上下文」，不是「识别失败」——
    // 故下面两条**各自如实答**，绝不拿 IM 上下文编一句假的。
    if let Some(outcome) = nt_channel_cmd::local_reply(&command, DESKTOP_CHANNEL_NAME) {
        return Ok(Some(LocalCmdOutcome {
            command: command.as_str(),
            reply: outcome.reply,
        }));
    }
    let reply = match &command {
        // `/new`：桌面不支持，如实说（决断见 `NEW_UNSUPPORTED`）。
        Command::New { .. } => NEW_UNSUPPORTED.to_owned(),
        // `/status` 是唯一要开库的（查会话标题）；其余三条**不开库** ——
        // 库坏了 `/help` 照样答得出来。
        Command::Status => {
            let config = load_config()?;
            let store = open_store(&config)?;
            status_reply(&store, convo_id, model_label(model_provider, model_name))
        }
        // 兜底（今天走不到：`local_reply` 已接走 `/help` 与 `/stop`）：core 若加了
        // 一条它认为「不需要 IM 上下文」的命令，而桌面还没登记自己的说法 ——
        // **如实说答不了**，绝不编一句回执。那比一条假回执便宜得多。
        other => format!(
            "/{} 桌面这边答不了：它还需要别的上下文，而这里没有。",
            other.as_str()
        ),
    };
    Ok(Some(LocalCmdOutcome { command: command.as_str(), reply }))
}

/// `/status` 的「模型：」行（与 IM 同一套口径：空即「跟主设置」）。
fn model_label(provider: Option<&str>, model: Option<&str>) -> String {
    let model = model.map(str::trim).unwrap_or_default();
    if !model.is_empty() {
        return model.to_owned();
    }
    let provider = provider.map(str::trim).unwrap_or_default();
    if provider.is_empty() {
        "跟主设置".to_owned()
    } else {
        format!("{provider}（跟主设置）")
    }
}

/// `/status`：只报桌面**真有**的状态。
///
/// 行的标签与顺序沿用 IM 那份（`nt_channel_cmd::execute` 的 `Command::Status`
/// 分支），差异只在两行 IM 专属的概念上 —— 桌面没有「机器人别名 / 访问白名单」
/// 这一层（那是渠道的事），故如实换成桌面自己的话，而不是照抄一个
/// 在桌面永远为假的值。
fn status_reply(store: &NeobotStore, convo_id: Option<&str>, model: String) -> String {
    let (bound_label, title) = match convo_id {
        None => ("无归属会话".to_owned(), "无归属会话".to_owned()),
        Some(id) => {
            // 「绑了但会话没了」与「没绑」是**两件事**（与 IM 同律）：
            // 前者从没绑过，后者是配置指向了不存在的东西。含糊其辞会
            // 让用户以为设置没生效。
            match store.get_convo_title(id) {
                Ok(Some(found)) => (id.to_owned(), found),
                _ => (id.to_owned(), "（绑定指向的会话已不存在）".to_owned()),
            }
        }
    };
    format!(
        "界面：{DESKTOP_CHANNEL_NAME}\n\
         当前会话：{title}（{bound_label}）\n\
         模型：{model}\n\
         访问模式：桌面这边就是本机用户本人，没有白名单/私聊那套\n\
         数据目录：~/.neobot（只在本机）"
    )
}

/// 本地指令在 `Channel` 上产生的**全部**事件。
///
/// 恰好两条：一条 `Delta`（整段回执，一次发完）+ 一条 `Done`。
/// **没有** `Step`（一个工具都没调），**没有**第二条 `Delta`（这不是增量流）。
/// 前端按 `delta` 累积后 `pushMsg`，于是用户看到的就是一条普通助手气泡。
pub fn local_reply_events(outcome: &LocalCmdOutcome) -> Vec<StreamEvent> {
    vec![
        StreamEvent::Delta { text: outcome.reply.clone() },
        StreamEvent::Done { status: LOCAL_COMMAND_STATUS.to_owned() },
    ]
}

/// 本地指令分支的返回值（不进跑轮 ⇒ 零耗、无工具）。
fn local_run_result() -> NeobotRunResult {
    NeobotRunResult {
        status: LOCAL_COMMAND_STATUS.to_owned(),
        // `ReplyMode` 只有 direct/passthrough/fallback 三态，本地指令**不属
        // 于**任何一态；取 `Direct`（=「不经晶体核心」）是三者里最不误导的，
        // 因为另两个会谎称「经核心透传」或「回显兜底」。真正的诚实在
        // `model` 那一格：它写「本地指令」。
        labels: TurnLabels::empty(LOCAL_COMMAND_MODEL, ReplyMode::Direct),
        // 不进跑轮 ⇒ **一个任务都没建** ⇒ 如实交出空 id（前端据此不挂任务卡，
        // 而不是去库里找一个「最像的」）。`cancelled` 同理恒为 false。
        task_id: String::new(),
        cancelled: false,
    }
}
