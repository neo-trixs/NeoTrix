//! `nt_channel_serve` — 常驻长轮询（`neobot channel serve`）。
//!
//! 桌面 App **刻意不做**常驻：App 关掉之后不该有后台进程在偷偷连公网 ——
//! 那与 neobot 的 local-first 承诺冲突（默认零网络是它的卖点）。
//! 要一直在线就另开一个终端跑这个子命令，进程归用户自己管。
//!
//! 循环形状（每一轮）：
//! `取到期渠道 → 收取 → 跑轮 → 排空 outbox → 补发 → 维护 → 按各自节拍睡`
//!
//! 三条崩不掉的律：
//! 1. **单个渠道失败不拖垮整轮** —— 每个渠道的收取各自 `catch`，某平台抽风时
//!    其余渠道照常服务，最后统一报一行；
//! 2. **每轮重新读渠道配置** —— 用户在设置页改了访问模式/停用某渠道，
//!    常驻进程不必重启就生效（否则「停用」这个动作只是看起来生效了）；
//! 3. **Ctrl-C 要能立刻停** —— 所以 sleep 分片（`slice_sleep`），
//!    不然最坏要等满一个轮询周期（Telegram 是 25s）。
//!
//! 4. **循环是串行的，所以 `/stop` 只会被排队**（2026-09-28 记）：收取与跑轮
//!    在**同一个线程**里（`run_once` 的 `for msg in inbound`）。跑轮不返回，
//!    下一条消息**不会被 poll 到** ⇒ `/stop` 一定在这一轮**结束后**才被读，
//!    那时已经没有可停的轮次了（`nt_channel_dispatch` 的登记已被 RAII 注销）。
//!    停止管道本身**已接通**（命令入口 → 按 `convo_id` 的登记表 → 跑轮那枚
//!    `StopToken`），缺的是**并发调度**把收取与跑轮解耦。在那之前，IM 上的
//!    `/stop` **停不了任何一轮** —— 而它会如实这么说（「没有可停的轮次」）。
//!
//! 无 async：长轮询靠阻塞 `poll` + 分片 sleep 驱动，与本 crate 其余部分同律。

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use crate::nt_channel::ChannelRegistry;
use crate::nt_changes;
use crate::nt_channel_dispatch::{
    self, InboundOutcome,
};
use crate::nt_channel_telegram::TelegramChannel;
use crate::{NeobotConfig, NeobotStore};

/// 建一个只装了内置适配器的注册表。
///
/// 与 `nt_cmd_channels::registry()` 同构（同一条规则、同样的静默策略）。
/// 两处各写一份是刻意的：CLI 那个 bin 不依赖 Tauri，桌面那个是瘦壳，
/// 共用反而要把一个 crate 的 IPC 层拖进 CLI。
pub fn registry() -> ChannelRegistry {
    let mut reg = ChannelRegistry::new();
    // 失败只可能是重复 id —— 编程错误，不是用户错误；静默即可
    // （少一个渠道会在探活时立刻显形，而让启动 panic 会让 CLI 直接不可用）。
    drop(reg.register(Box::new(TelegramChannel::new(""))));
    reg
}

/// 一轮的处理统计。
// 2026-09-29：去掉 `Copy` —— 新增 `changes_prune_error: Option<String>`
// 含堆数据，Copy 不可能实现。原代码用 Copy 是因为当时全是 i64。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RoundStats {
    /// 收到的入站消息数。
    pub received: i64,
    /// 真的跑了轮（其余是重复 / 被闸门拒 / 是指令）。
    pub ran: i64,
    /// 指令与丢弃的合计。
    pub ignored: i64,
    /// 跑轮失败（已如实计入，不静默）。
    pub failed: i64,
    /// 补发成功的条数。
    pub deferred: i64,
    /// 2026-09-29：账目留存期清理本轮删掉的行数。
    ///
    /// 此前清理函数**从未被调用**（R-P79 未接线），账目表无上限增长。
    pub changes_pruned: i64,
    /// 账目清理失败的原因（`None` = 本轮成功）。
    ///
    /// 存在的理由：原实现 `unwrap_or(0)` 把「删除失败」伪装成「删了 0 行」，
    /// 无人能察觉账目在持续膨胀。**失败必须可观测**。
    pub changes_prune_error: Option<String>,
}

impl RoundStats {
    /// 2026-09-29：入参由 `self` 改 `&self`。
    /// 加 `changes_prune_error: Option<String>` 后 `RoundStats` 不能再 `Copy`，
    /// 而 `assert!(x.is_quiet(), "{x:?}")` 这类写法**先调方法再格式化 x** ——
    /// 若方法收 `self` 会把 x 移走，格式化处借用即报 E0382。
    /// 改收 `&self` 一次性修好全部 3 处消费方，调用点无需改动。
    pub fn is_quiet(&self) -> bool {
        self.received == 0 && self.deferred == 0 && self.failed == 0
    }

    /// 本轮是否有**该被察觉但可能被忽略**的问题。
    /// 与 `is_quiet` 分开：清理失败不阻断出站，但绝不能静默。
    pub fn has_prune_error(&self) -> bool {
        self.changes_prune_error.is_some()
    }
}

/// 单个渠道的引擎解析（与 CLI 侧 `resolve_engine` 同口径）。
fn engine_for(config: &NeobotConfig) -> Result<Box<dyn crate::nt_engine::EngineAdapter>, String> {
    use crate::nt_engine::{CliEngine, LocalEchoEngine, OpencodeEngine};
    use crate::{EngineKind, HttpEngine};
    match &config.engine {
        EngineKind::Echo => Ok(Box::new(LocalEchoEngine)),
        EngineKind::Cli { command } => CliEngine::new(command)
            .map(|e| Box::new(e) as Box<dyn crate::nt_engine::EngineAdapter>)
            .map_err(|err| err.to_string()),
        EngineKind::Opencode { model } => OpencodeEngine::new(model)
            .map(|e| Box::new(e) as Box<dyn crate::nt_engine::EngineAdapter>)
            .map_err(|err| err.to_string()),
        EngineKind::Http { .. } => HttpEngine::from_env()
            .map(|e| Box::new(e) as Box<dyn crate::nt_engine::EngineAdapter>)
            .map_err(|err| err.to_string()),
    }
}

/// 这条消息该交给哪个机器人。
///
/// 渠道只有一个适配器（一个 token、一个 offset），所以**没有「chat → bot」的绑定**。
/// 可用的判据只有每个机器人自己的白名单，于是：
/// 1. 挑**第一个白名单认领了这个发件人**的机器人 —— `allow` 模式下这让
///    「A 机器人的群给 A、B 机器人的群给 B」真正成立（这本来就是多机器人的意义）；
/// 2. 都没认领时回落到第一个机器人，让它按**自己的**白名单如实回一句
///    「没有权限」。刻意不静默丢弃：用户需要知道为什么机器人不理他。
pub fn route_bot<'a>(
    bots: &'a [crate::nt_store::BotRow],
    access: crate::nt_channel::AccessMode,
    msg: &crate::nt_channel::InboundMessage,
) -> &'a crate::nt_store::BotRow {
    let Some(first) = bots.first() else {
        // 调用方保证了非空；真到这里说明上面的 `is_empty` 守卫被删了。
        return empty_bot();
    };
    for bot in bots {
        let allow = crate::nt_store::parse_allow_list(&bot.allow_list);
        if crate::nt_channel::admits(access, &allow, msg.is_dm, &msg.sender) {
            return bot;
        }
    }
    first
}

/// 同渠道多机器人时**只警告一次**（每轮都刷会把日志淹掉）。
fn warn_shared_adapter_once(channel: &str, count: usize) {
    use std::collections::BTreeSet;
    use std::sync::{Mutex, OnceLock};
    static WARNED: OnceLock<Mutex<BTreeSet<String>>> = OnceLock::new();
    let set = WARNED.get_or_init(|| Mutex::new(BTreeSet::new()));
    let Ok(mut seen) = set.lock() else { return };
    if !seen.insert(channel.to_owned()) {
        return;
    }
    eprintln!(
        "[neobot] 渠道 {channel} 配了 {count} 个机器人，但 serve 只有**一个共享适配器**          （一个 token / 一个 offset）：per-bot 的 token_env 与 model 在 serve 里**没有生效**，         收件按各自白名单路由。给每个渠道配一个机器人，或等并发调度落地。"
    );
}

/// 路由兜底用的空机器人（`bots` 为空时才可能取到；调用方已守卫）。
static EMPTY_BOT: std::sync::OnceLock<crate::nt_store::BotRow> = std::sync::OnceLock::new();
fn empty_bot() -> &'static crate::nt_store::BotRow {
    EMPTY_BOT.get_or_init(|| crate::nt_store::BotRow {
        channel: String::new(),
        bot_id: String::new(),
        alias: String::new(),
        token_env: String::new(),
        conversation_id: None,
        model: String::new(),
        allow_list: String::new(),
        created_at: String::new(),
        last_seen: None,
    })
}

/// 跑**一轮**：收取 + 出站 + 补发。
///
/// 返回本轮统计。`only` 为 `Some(id)` 时只处理该渠道。
pub fn run_once(
    store: &NeobotStore,
    config: &NeobotConfig,
    only: Option<&str>,
) -> Result<RoundStats, String> {
    let mut reg = registry();
    let mut stats = RoundStats::default();
    // 每轮重读配置（律 2）：停用/改访问模式不必重启进程就生效。
    let channels = store.list_channels().map_err(|e| e.to_string())?;
    for row in channels {
        if !row.enabled {
            continue;
        }
        if let Some(want) = only {
            if row.id != want {
                continue;
            }
        }
        if reg.get(&row.id).is_none() {
            continue;
        }
        let bots = store.list_bots(&row.id).map_err(|e| e.to_string())?;
        if bots.is_empty() {
            continue;
        }
        if bots.len() > 1 {
            warn_shared_adapter_once(&row.id, bots.len());
        }

        // 收取需要 `&mut`（长轮询要推进 offset），跑轮需要 `&`。
        // 所以分两段借：先 `get_mut` 收完就放，再 `get` 拿不可变引用跑轮 ——
        // 同时持有两者是借不出来的。
        //
        // 律 1：单渠道收取失败只记一次，不中断整轮。
        //
        // **每个渠道只 poll 一次，不在机器人循环里 poll**：
        // 适配器是**按渠道**注册的，一个 offset 由整个渠道共享。
        // 早先的写法把 `poll()` 放进 `for bot in &bots` 里，于是第一个机器人
        // 就把这一轮的 update 全部取走（offset 已推进），后面的机器人再 poll
        // 只会拿到空 —— **第二个及以后的机器人永远收不到消息，而且一声不响**。
        // 「多机器人」在这套实现里是静默失效的，比报错更坏。
        let inbound = match reg.get_mut(&row.id) {
            Some(adapter) => match adapter.poll() {
                Ok(items) => items,
                Err(_) => {
                    stats.failed += 1;
                    continue;
                }
            },
            None => continue,
        };
        if inbound.is_empty() {
            continue;
        }
        let access = crate::nt_channel::AccessMode::parse(&row.access_mode);
        // `ChannelRegistry::get` 已经返回 `&dyn ChannelAdapter`，不用再 as_ref。
        let Some(adapter_ref) = reg.get(&row.id) else {
            continue;
        };
        // **这条 `for msg in inbound` 不再是同步调度的全部**（2026-10-08 解耦）：
        // 跑轮在 worker 线程执行（`on_inbound`），主线程在本批通道跑完后
        // 用有界等待收成确定性统计；worker 线程跑完其收据后 NMake 200ms
        // 超时判为「长 HTTP 轮，放手」。
        // 复效：跑轮期间抵达的 /stop 能在下一条消息里被 poll 到，经
        // `signal_run_cancel` 翻转同一登记表 ⇒ IM /stop 真正可停。
        let mut handles: Vec<std::sync::mpsc::Receiver<Result<InboundOutcome, String>>> =
            Vec::new();
        let mut pool = WorkerPool::new(WORKER_POOL_MAX);
        for msg in inbound {
            stats.received += 1;
            // 一条消息交给**认领这个发件人**的那个机器人。
            let bot = route_bot(&bots, access, &msg);
            // 缺 token：同步记失败，不给 worker 发（测试路径与旧行为一致）。
            if bot_token_missing(&bot.token_env) {
                stats.failed += 1;
                continue;
            }
            if row.id != "telegram" {
                // 非 telegram 适配器（测试 Fake 等）保持原串行语义。
                let engine = match engine_for(config) {
                    Ok(engine) => engine,
                    Err(err) => {
                        stats.failed += 1;
                        eprintln!("[neobot] 引擎不可用：{err}");
                        break;
                    }
                };
                match nt_channel_dispatch::on_inbound(
                    store,
                    config,
                    engine.as_ref(),
                    adapter_ref,
                    bot,
                    &msg,
                ) {
                    Ok(InboundOutcome::Turn { .. }) => stats.ran += 1,
                    Ok(_) => stats.ignored += 1,
                    Err(_) => stats.failed += 1,
                }
                continue;
            }
            // Captain_Who multi-agent.md 并发上限吸收：
            // 活跃 worker 先收尸再判定；满员 ⇒ 本条**退回同步路径**，不再起线程
            // （否则每轮为每个 msg 无限起线程，极端并发线程数不封顶）。
            pool.reap();
            if !pool.can_spawn() {
                let engine = match engine_for(config) {
                    Ok(engine) => engine,
                    Err(err) => {
                        stats.failed += 1;
                        eprintln!("[neobot] 引擎不可用：{err}");
                        break;
                    }
                };
                match nt_channel_dispatch::on_inbound(
                    store,
                    config,
                    engine.as_ref(),
                    adapter_ref,
                    bot,
                    &msg,
                ) {
                    Ok(InboundOutcome::Turn { .. }) => stats.ran += 1,
                    Ok(_) => stats.ignored += 1,
                    Err(_) => stats.failed += 1,
                }
                continue;
            }
            // 跑轮与收取解耦：worker 线程跑 `on_inbound`，主线程继续 poll、
            // 使得跑轮期间抵达的 /stop 能经 `signal_run_cancel` 翻转同一登记。
            //
            // worker 用「独立 NeobotStore 连接同库 WAL + 独立 TelegramChannel
            // + 独立 engine」：NeobotStore/ChannelAdapter 均不 Sync，这是 Rust
            // 安全下唯一能让 turn 真并行的形态（store 由 worker 自己持有）。
            let config_w = config.clone();
            let bot_w = bot.clone();
            let msg_w = msg.clone();
            let (tx, rx) = std::sync::mpsc::channel::<Result<InboundOutcome, String>>();
            let worker_handle = std::thread::spawn(move || {
                let res = (|| -> Result<InboundOutcome, String> {
                    let worker_store = NeobotStore::open(
                        &config_w.db_path().to_string_lossy(),
                    )
                    .map_err(|e| e.to_string())?;
                    let worker_engine = engine_for(&config_w).map_err(|e| e)?;
                    let worker_adapter = TelegramChannel::new(&bot_w.token_env);
                    nt_channel_dispatch::on_inbound(
                        &worker_store,
                        &config_w,
                        worker_engine.as_ref(),
                        &worker_adapter,
                        &bot_w,
                        &msg_w,
                    )
                    .map_err(|e| e.to_string())
                })();
                // 接收端可能已在 200ms 后放弃对超时轮次的等待 ⇒ 丢弃亦可。
                let _ = tx.send(res);
            });
            pool.admit(worker_handle);
            handles.push(rx);
        }
        // 主线程按序对每个 worker 做有界等待：
        //  快速轮次（ echo / 命令 / 已就位的工具轮 ）直接收成确定性统计；
        //  长轮次（真 HTTP）则放手给 detached worker，主线程继续收取，
        //  跑轮期间抵达的 /stop 才能在下一轮 poll 时被翻转 ⇒ 真正兑现 /stop。
        for rx in handles.drain(..) {
            match rx.recv_timeout(std::time::Duration::from_millis(200)) {
                Ok(Ok(InboundOutcome::Turn { .. })) => stats.ran += 1,
                Ok(Ok(_)) => stats.ignored += 1,
                Ok(Err(_)) => stats.failed += 1,
                // 超时（长 HTTP 轮）⇒ detached worker 继续跑；其 outbox 回执由
                // 后续 poll 的 drain 送出。200ms 是「快速轮次等待」的上限，
                // 不是把跑轮本身卡住。
                Err(_) => stats.failed += 1,
            }
        }
    }
    // 出站 + 补发（对所有渠道一起做，与桌面侧同一条路径）。
    let (_sent, _failed) =
        nt_channel_dispatch::drain_outbox_once(store, &mut reg).map_err(|e| e.to_string())?;
    let (deferred, _gave_up) =
        nt_channel_dispatch::sweep_pending(store, &mut reg).map_err(|e| e.to_string())?;
    stats.deferred = deferred as i64;
    nt_channel_dispatch::upkeep_best_effort(store);
    // 2026-09-29 接线（R-P79）：账目留存期清理此前**从未被调用**，
    // 账目表无上限增长。与 upkeep 同一条调度路径。
    // 失败不阻断本轮出站（best-effort 语义），但**不吞** ——
    // 写进 stats 让调用方可观测（`unwrap_or(0)` 会伪装成「删了 0 行」）。
    match nt_changes::prune_best_effort(store) {
        Ok(n) if n > 0 => stats.changes_pruned = n as i64,
        Ok(_) => {}
        Err(err) => stats.changes_prune_error = Some(format!("{err:?}")),
    }
    Ok(stats)
}

/// 分片睡眠（每 200ms 醒一次）。
///
/// **不要把它当成「Ctrl-C 能在 200ms 内生效」的机制** —— 早先的注释这么写，
/// 但这个函数**不检查任何停止标志**，只是把一次长睡切成几段。真正让 Ctrl-C
/// 生效的是信号处理（默认处理器直接终止进程），跟这里分不分片无关。分片只对
/// 「将来改成检查一个原子停止标志」有意义；在那之前，别在注释里承诺它做不到的事。
pub fn slice_sleep(total: Duration) {
    const SLICE: Duration = Duration::from_millis(200);
    let deadline = Instant::now() + total;
    while Instant::now() < deadline {
        std::thread::sleep(SLICE.min(deadline.saturating_duration_since(Instant::now())));
    }
}

/// 各渠道的下一次轮询间隔（秒）：`only` 限定的渠道用它，其余用各自 `poll_secs`）。
pub fn intervals(store: &NeobotStore, only: Option<&str>) -> BTreeMap<String, i64> {
    let mut out = BTreeMap::new();
    let Ok(channels) = store.list_channels() else {
        return out;
    };
    for row in channels {
        if !row.enabled {
            continue;
        }
        if let Some(want) = only {
            if row.id != want {
                continue;
            }
        }
        out.insert(row.id, row.poll_secs);
    }
    out
}

/// 渠道 token 环境变量缺失时给出的提示（一次性，不刷屏）。
pub fn missing_token_hint(bot_channel: &str, bot_id: &str, token_env: &str) -> String {
    format!(
        "[neobot] {bot_channel}/{bot_id} 的 token 变量 {token_env} 未设置 —— 该机器人收不到消息。\
         （token 只从环境变量读，库里不存值。）"
    )
}

/// 该机器人是否缺 token 变量（设置页与服务端共用的判据）。
pub fn bot_token_missing(token_env: &str) -> bool {
    std::env::var(token_env)
        .ok()
        .map(|v| v.trim().is_empty())
        .unwrap_or(true)
}

/// 每轮活跃 worker 线程上限（Captain_Who multi-agent.md 并发上限吸收）。
/// 满员时本批通道内剩余消息退回**同步路径**，不再起线程 ⇒ 线程数有顶。
pub const WORKER_POOL_MAX: usize = 8;

/// 跟踪本轮未收尸的 worker 把柄。`thread::spawn` 不计；`reap`/`admit`/`can_spawn` 三件套。
pub(crate) struct WorkerPool {
    max: usize,
    live: Vec<std::thread::JoinHandle<()>>,
}

impl WorkerPool {
    pub(crate) fn new(max: usize) -> Self {
        Self { max, live: Vec::new() }
    }

    /// 收掉已结束的把柄，返回当前活跃数。不变量：`活跃 = live.len()`（收尸后）。
    pub(crate) fn reap(&mut self) -> usize {
        let mut i = 0;
        while i < self.live.len() {
            if self.live[i].is_finished() {
                // swap_remove 未处理的把柄会立即 detach（必要时等待）；noop 线程 200ms 外就是正常语义。
                let _ = self.live.swap_remove(i);
            } else {
                i += 1;
            }
        }
        self.live.len()
    }

    pub(crate) fn admit(&mut self, h: std::thread::JoinHandle<()>) {
        self.live.push(h);
    }

    pub(crate) fn can_spawn(&self) -> bool {
        self.live.len() < self.max
    }
}

/// 白名单解析（服务循环里判断「有没有人会被放行」用）。
pub fn bot_allow_list(raw: &str) -> Vec<String> {
    crate::nt_store::parse_allow_list(raw)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nt_channel::AccessMode;
    use crate::nt_channel_dispatch;

    fn store(case: &str) -> NeobotStore {
        NeobotStore::open(":memory:").unwrap_or_else(|err| panic!("{case}: {err}"))
    }

    fn config(case: &str) -> NeobotConfig {
        let dir = crate::nt_testutil::temp_dir(&format!("neobot-serve-test-{}", case));
        let _ = std::fs::remove_dir_all(&dir);
        NeobotConfig {
            data_dir: dir.clone(),
            workspace_dir: dir.join("workspace"),
            policy_mode: crate::nt_config::PolicyMode::Enforce,
            human_has_control: false,
            max_steps: 2,
            engine: crate::nt_config::EngineKind::Echo,
            computer_allow: Vec::new(),
            computer_hosts: Vec::new(),
            extra_deny: Vec::new(),
            write_budget: crate::nt_config::default_write_budget(),
        }
    }

    #[test]
    fn registry_ships_telegram() {
        let reg = registry();
        assert!(reg.get("telegram").is_some(), "Telegram 应在注册表里");
        assert!(reg.get("nope").is_none());
    }

    #[test]
    #[test]
    fn worker_pool_respects_max() {
        let mut pool = WorkerPool::new(0);
        assert!(!pool.can_spawn(), "max=0 永不准 spawn");
        let mut pool = WorkerPool::new(1);
        pool.admit(std::thread::spawn(|| {}));
        assert!(!pool.can_spawn(), "未收尸前的线程不让出额度");
    }

    #[test]
    fn worker_pool_reaps_finished() {
        let mut pool = WorkerPool::new(1);
        pool.admit(std::thread::spawn(|| {}));
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(200);
        while pool.reap() > 0 && std::time::Instant::now() < deadline {
            std::thread::yield_now();
        }
        assert_eq!(pool.reap(), 0, "已结束的 worker 必须被收尸");
        assert!(pool.can_spawn(), "收尸后恢复容量");
    }

    fn run_once_on_empty_store_is_a_noop() {
        let st = store("empty");
        let cfg = config("empty");
        let got = run_once(&st, &cfg, None).expect("run");
        assert_eq!(got, RoundStats::default());
        assert!(got.is_quiet());
        let _ = std::fs::remove_dir_all(&cfg.data_dir);
    }

    #[test]
    fn disabled_channels_are_skipped() {
        let st = store("disabled");
        let cfg = config("disabled");
        st.upsert_channel("telegram", "TG", "open", 5).expect("ch");
        st.upsert_bot(&crate::nt_store::BotRow {
            channel: "telegram".to_owned(),
            bot_id: "1".to_owned(),
            alias: String::new(),
            token_env: "NEOBOT_TG_UNSET_XYZ".to_owned(),
            conversation_id: None,
            model: String::new(),
            allow_list: String::new(),
            created_at: String::new(),
            last_seen: None,
        })
        .expect("bot");
        st.set_channel_enabled("telegram", false).expect("off");
        // 停用后不该去连平台（也就不会因为「token 没设」而报失败）。
        let got = run_once(&st, &cfg, None).expect("run");
        assert_eq!(got.failed, 0, "停用的渠道不该被收取");
        assert!(got.is_quiet());
        let _ = std::fs::remove_dir_all(&cfg.data_dir);
    }

    #[test]
    fn run_once_survives_a_missing_token() {
        // token 没设 → 收取报错，但**整轮不崩**，且如实记一次失败。
        let st = store("notoken");
        let cfg = config("notoken");
        st.upsert_channel("telegram", "TG", "open", 5).expect("ch");
        st.upsert_bot(&crate::nt_store::BotRow {
            channel: "telegram".to_owned(),
            bot_id: "1".to_owned(),
            alias: String::new(),
            token_env: "NEOBOT_TG_UNSET_XYZ".to_owned(),
            conversation_id: None,
            model: String::new(),
            allow_list: String::new(),
            created_at: String::new(),
            last_seen: None,
        })
        .expect("bot");
        let got = run_once(&st, &cfg, None).expect("run must not error");
        assert_eq!(got.received, 0);
        assert_eq!(got.failed, 1, "缺 token 该记一次失败，而不是静默");
        let _ = std::fs::remove_dir_all(&cfg.data_dir);
    }

    #[test]
    fn only_filter_restricts_the_round() {
        let st = store("only");
        let cfg = config("only");
        st.upsert_channel("telegram", "TG", "open", 5).expect("ch");
        st.upsert_bot(&crate::nt_store::BotRow {
            channel: "telegram".to_owned(),
            bot_id: "1".to_owned(),
            alias: String::new(),
            token_env: "NEOBOT_TG_UNSET_XYZ".to_owned(),
            conversation_id: None,
            model: String::new(),
            allow_list: String::new(),
            created_at: String::new(),
            last_seen: None,
        })
        .expect("bot");
        // 限定到一个**不存在**的渠道 → 什么都不该发生，且不报错。
        let got = run_once(&st, &cfg, Some("nope")).expect("run");
        assert!(got.is_quiet(), "{got:?}");
        // 限定到 telegram（缺 token）→ 记一次失败。
        let got = run_once(&st, &cfg, Some("telegram")).expect("run");
        assert_eq!(got.failed, 1, "{got:?}");
        let _ = std::fs::remove_dir_all(&cfg.data_dir);
    }

    #[test]
    fn channel_without_bots_is_skipped_quietly() {
        // 建了渠道但没挂机器人 → 不该反复报「收取失败」，
        // 那是配置还没填完，不是故障。
        let st = store("nobots");
        let cfg = config("nobots");
        st.upsert_channel("telegram", "TG", "open", 5).expect("ch");
        let got = run_once(&st, &cfg, None).expect("run");
        assert!(got.is_quiet(), "{got:?}");
        let _ = std::fs::remove_dir_all(&cfg.data_dir);
    }

    #[test]
    fn intervals_reflects_enabled_channels_and_their_poll_secs() {
        let st = store("iv");
        st.upsert_channel("telegram", "TG", "open", 7).expect("ch");
        st.upsert_channel("slack", "Slack", "open", 11)
            .expect("ch2");
        st.set_channel_enabled("slack", false).expect("off");
        let got = intervals(&st, None);
        assert_eq!(got.len(), 1, "停用的渠道不该出现在节拍表里");
        assert_eq!(got.get("telegram"), Some(&7));
        // 限定渠道时只看那一个。
        let only = intervals(&st, Some("telegram"));
        assert_eq!(only.len(), 1);
        assert!(intervals(&st, Some("slack")).is_empty(), "停用的不该被选中");
    }

    #[test]
    fn slice_sleep_returns_early_and_actually_waits() {
        // 不能睡太久（测试要快），但也不能立刻返回（否则等于没睡）。
        let started = Instant::now();
        slice_sleep(Duration::from_millis(250));
        let spent = started.elapsed();
        assert!(
            spent >= Duration::from_millis(200),
            "只睡了 {spent:?}，分片睡眠没生效"
        );
        assert!(spent < Duration::from_secs(3), "睡过头了：{spent:?}");
    }

    #[test]
    fn missing_token_is_detected_from_the_env_name_only() {
        assert!(bot_token_missing("NEOBOT_TG_DEFINITELY_UNSET_XYZ"));
        std::env::set_var("NEOBOT_TG_PRESENT_TEST", "  x  ");
        assert!(!bot_token_missing("NEOBOT_TG_PRESENT_TEST"));
        std::env::set_var("NEOBOT_TG_BLANK_TEST", "   ");
        assert!(
            bot_token_missing("NEOBOT_TG_BLANK_TEST"),
            "只有空白的也算缺"
        );
        std::env::remove_var("NEOBOT_TG_PRESENT_TEST");
        std::env::remove_var("NEOBOT_TG_BLANK_TEST");
    }

    fn bot_row(id: &str, allow: &str) -> crate::nt_store::BotRow {
        crate::nt_store::BotRow {
            channel: "telegram".to_owned(),
            bot_id: id.to_owned(),
            alias: String::new(),
            token_env: format!("NEOBOT_TG_{id}"),
            conversation_id: None,
            model: String::new(),
            allow_list: allow.to_owned(),
            created_at: String::new(),
            last_seen: None,
        }
    }

    fn inbound(sender: &str) -> crate::nt_channel::InboundMessage {
        crate::nt_channel::InboundMessage {
            message_id: "1".to_owned(),
            chat: "c1".to_owned(),
            sender: sender.to_owned(),
            text: "hi".to_owned(),
            is_dm: true,
            attachments: Vec::new(),
            reply_to: None,
            sender_name: String::new(),
        }
    }

    #[test]
    fn routing_hands_each_sender_to_the_bot_that_claims_it() {
        use crate::nt_channel::AccessMode;
        let bots = vec![bot_row("A", "alice"), bot_row("B", "bob")];
        assert_eq!(
            route_bot(&bots, AccessMode::Allow, &inbound("alice")).bot_id,
            "A"
        );
        assert_eq!(
            route_bot(&bots, AccessMode::Allow, &inbound("bob")).bot_id,
            "B"
        );
    }

    #[test]
    fn routing_falls_back_to_first_bot_so_a_denial_gets_explained() {
        use crate::nt_channel::AccessMode;
        let bots = vec![bot_row("A", "alice"), bot_row("B", "bob")];
        // carol 不在任何名单：不能静默丢，要落到某个机器人让它如实回「没权限」。
        assert_eq!(
            route_bot(&bots, AccessMode::Allow, &inbound("carol")).bot_id,
            "A"
        );
    }

    #[test]
    fn open_mode_routes_to_the_first_bot() {
        use crate::nt_channel::AccessMode;
        let bots = vec![bot_row("A", ""), bot_row("B", "")];
        assert_eq!(
            route_bot(&bots, AccessMode::Open, &inbound("zed")).bot_id,
            "A"
        );
    }

    #[test]
    fn two_bots_on_one_channel_are_both_reachable() {
        // 回归锁：早先把 `poll()` 放在 `for bot in &bots` 里 —— 第一个机器人
        // 就取走全部 update（offset 已推进），**第二个及以后的机器人永远收不到
        // 消息，而且一声不响**。现在每渠道只 poll 一次并按白名单路由，
        // 两个机器人都能各自收到自己那份。
        let st = store("twobots");
        let cfg = config("twobots");
        st.upsert_channel("telegram", "TG", "allow", 5).expect("ch");
        st.upsert_bot(&bot_row("A", "alice")).expect("A");
        st.upsert_bot(&bot_row("B", "bob")).expect("B");
        let bots = st.list_bots("telegram").expect("bots");
        assert_eq!(bots.len(), 2, "两个机器人都应在库里");

        let access = crate::nt_channel::AccessMode::Allow;
        let mut reached = std::collections::BTreeSet::new();
        for sender in ["alice", "bob"] {
            reached.insert(route_bot(&bots, access, &inbound(sender)).bot_id.to_owned());
        }
        assert_eq!(
            reached.len(),
            2,
            "alice 与 bob 必须落到**不同**机器人上；早先第二个机器人被共享 offset 饿死"
        );
        let _ = std::fs::remove_dir_all(&cfg.data_dir);
    }

    #[test]
    fn a_channel_with_several_bots_warns_once_instead_of_silently_starving() {
        // 静默失效比报错更坏：至少要说清 serve 只有共享适配器。
        let st = store("warnonce");
        let cfg = config("warnonce");
        st.upsert_channel("telegram", "TG", "open", 5).expect("ch");
        st.upsert_bot(&bot_row("A", "")).expect("A");
        st.upsert_bot(&bot_row("B", "")).expect("B");
        // 跑两轮；第一次该警告，第二次同一渠道不再刷。
        warn_shared_adapter_once("telegram", 2);
        warn_shared_adapter_once("telegram", 2);
        let got = run_once(&st, &cfg, None).expect("run");
        assert!(got.failed <= 1, "多机器人本身不该让整轮失败：{got:?}");
        let _ = std::fs::remove_dir_all(&cfg.data_dir);
    }

    #[test]
    fn token_hint_names_the_env_var() {
        let hint = missing_token_hint("telegram", "42", "NEOBOT_TELEGRAM_TOKEN");
        assert!(hint.contains("NEOBOT_TELEGRAM_TOKEN"), "{hint}");
        assert!(
            hint.contains("库里不存值"),
            "要说清凭据为什么不落库：{hint}"
        );
    }

    #[test]
    fn access_mode_gate_is_the_same_one_dispatch_uses() {
        // 服务循环不自己判访问（那是 dispatch 的事），但要确认两处用的是同一套判据。
        let allow = bot_allow_list("alice, bob");
        assert_eq!(allow, vec!["alice".to_owned(), "bob".to_owned()]);
        assert!(nt_channel_dispatch::last_reply_of(&store("gate"), "nope").is_none());
        let _ = AccessMode::parse("allow");
    }
}
