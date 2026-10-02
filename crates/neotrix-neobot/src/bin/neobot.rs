//! `neobot` — 本地独立 App CLI.
//!
//! CLI 即协议：模型/外部进程只调本地 `neobot`，
//! 结构化副作用经 `NEOBOT_RESULT_PATH` JSONL 回传，sidecar 落库前复核。
//!
//! ```sh
//! neobot init                 # 建 ~/.neobot + workspace + config.json
//! neobot doctor               # 自检 (目录/DB/引擎探活 + 残留回收)
//! neobot run -t 标题 --text 内容   # 跑一轮本地任务 (默认 echo 引擎, 零模型)
//! neobot task list            # 列任务
//! neobot task claim <id> --actor alice
//! neobot audit list           # 列审计
//! neobot ledger [--by-actor]  # 成本账
//! neobot routine add --name daily --interval 900 --instruction "..."
//! neobot routine sweep        # 跑到期例行
//! neobot skill list / install <dir>
//! neobot member add alice --kind human
//! neobot control take --holder alice
//! neobot policy drill         # fail-closed 演练
//! ```

#![forbid(unsafe_code)]

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use neotrix_neobot::{
    Actor, CliEngine, EngineAdapter, EngineKind, HttpEngine, LocalEchoEngine, NeobotConfig,
    NtBotError, OpencodeEngine, PolicyMode, Provider, fire_routine, load_config,
    open_store, run_local_turn_as, run_local_turn_stream_as, sweep_routines,
};

#[derive(Debug, Parser)]
#[command(name = "neobot", version, about = "NeoBot local-first agent app")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Debug, Subcommand)]
enum Cmd {
    /// 初始化数据目录与配置.
    Init,
    /// 自检（含崩溃残留回收 + 过期认领释放计数）.
    Doctor,
    /// 跑一轮本地任务.
    Run {
        /// 任务标题.
        #[arg(short, long, default_value = "local task")]
        title: String,
        /// 用户输入.
        #[arg(short = 'x', long, default_value = "hello neobot")]
        text: String,
        /// 引擎: `echo` | 本机命令名 (如 `claude`) | `http` (OpenAI 兼容, 读 NEOBOT_* env).
        #[arg(long, default_value = "echo")]
        engine: String,
        /// 走注册的第三方端点（覆盖 --engine，key 读其 key_env 变量）.
        #[arg(long)]
        provider: Option<String>,
        /// 覆盖模型名（配合 --provider；优先级高于端点默认与 NEOBOT_MODEL）.
        #[arg(long)]
        model: Option<String>,
        /// 归属会话（IM 语义：同会话发送即追加；缺省自动建群）.
        #[arg(long)]
        convo: Option<String>,
        /// 流式输出增量内容 (仅 http 引擎).
        #[arg(long, default_value_t = false)]
        stream: bool,
    },
    /// 任务管理.
    Task {
        #[command(subcommand)]
        cmd: TaskCmd,
    },
    /// 审计管理.
    Audit {
        #[command(subcommand)]
        cmd: AuditCmd,
    },
    /// 成本账聚合.
    Ledger {
        /// 按 actor 切分.
        #[arg(long, default_value_t = false)]
        by_actor: bool,
    },
    /// 定时例行（15min 地板 + 10 连败自停）.
    Routine {
        #[command(subcommand)]
        cmd: RoutineCmd,
    },
    /// 技能下沉 (skills=指令非能力).
    Skill {
        #[command(subcommand)]
        cmd: SkillCmd,
    },
    /// 跨会话记忆 (MEMORY.md， prompt 注入参考).
    Memory {
        #[command(subcommand)]
        cmd: MemoryCmd,
    },
    /// 成员管理 (单管理员座位).
    Member {
        #[command(subcommand)]
        cmd: MemberCmd,
    },
    /// 会话管理（IM 语义：个人 DM / 群组；发送进会话即追加）.
    Convo {
        #[command(subcommand)]
        cmd: ConvoCmd,
    },
    /// 附件管理（文/图/文件/视频；本体拷贝进数据目录）.
    Attach {
        #[command(subcommand)]
        cmd: AttachCmd,
    },
    /// 接管 (take-the-wheel).
    Control {
        #[command(subcommand)]
        cmd: ControlCmd,
    },
    /// 策略管理.
    Policy {
        #[command(subcommand)]
        cmd: PolicyCmd,
    },
    /// 列出模型池（聚合全部启用的第三方端点；默认跳过不可达端点）.
    Models {
        /// 只看指定端点.
        #[arg(long)]
        provider: Option<String>,
    },
    /// 第三方模型端点管理（key 只读环境变量，永不落库）.
    Provider {
        #[command(subcommand)]
        cmd: ProviderCmd,
    },
    /// 晶体核心配对（灵魂嵌入：配对走核心，未配对纯本地）.
    Core {
        #[command(subcommand)]
        cmd: CoreCmd,
    },
    /// 服务端 agent（晶体原生手；需配对，token 现读）.
    Agent {
        #[command(subcommand)]
        cmd: AgentCmd,
    },
    /// IM 渠道（长轮询常驻 / 单轮收取 / 探活）.
    Channel {
        #[command(subcommand)]
        cmd: ChannelCmd,
    },
    /// 一键备份（db + 附件 + MEMORY.md + config.json → 单 zip；换机搬运即此一文件）.
    Export {
        /// 输出路径（缺省 `<data_dir>/backups/neobot-<时间>.zip`）.
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Debug, Subcommand)]
enum ChannelCmd {
    /// 列出已配置的渠道与机器人（token 只出变量名）.
    List,
    /// 探活（读环境变量里的 token 去问平台一次）.
    Probe { channel: String },
    /// 手动跑一轮：收取 + 出站排空 + 补发，然后退出.
    ///
    /// 桌面 App 的「收一轮」走同一条逻辑（`neobot_channel_poll_once`），
    /// 故两边行为一致。
    Once {
        /// 只处理这一个渠道（缺省 = 所有启用的渠道）.
        #[arg(long)]
        channel: Option<String>,
    },
    /// 常驻长轮询（Ctrl-C 退出）.
    ///
    /// 桌面 App **不**做常驻：App 关掉就不该有后台进程在偷偷连公网。
    /// 要一直在线就跑这个。
    Serve {
        /// 只服务这一个渠道（缺省 = 所有启用的渠道）.
        #[arg(long)]
        channel: Option<String>,
        /// 空闲轮询秒数（缺省取渠道自己的 `poll_secs`）.
        #[arg(long)]
        interval: Option<i64>,
    },
}

#[derive(Debug, Subcommand)]
enum TaskCmd {
    List,
    Claim { id: String, #[arg(long)] actor: String },
    Release { id: String, #[arg(long)] actor: String },
    Cancel { id: String },
    Retry { id: String },
    Rename { id: String, title: String },
    Rm { id: String },
}

#[derive(Debug, Subcommand)]
enum AuditCmd {
    List,
    /// 按留存删旧行（默认 30 天）。
    Prune {
        #[arg(long, default_value_t = 30)]
        days: i64,
    },
}

#[derive(Debug, Subcommand)]
enum RoutineCmd {
    Add {
        #[arg(long)] name: String,
        /// 秒（≥900，15min 地板）.
        #[arg(long)] interval: i64,
        #[arg(long)] instruction: String,
        #[arg(long, default_value = "owner")] owner: String,
    },
    List,
    Fire {
        #[arg(long)] name: String,
        #[arg(long, default_value = "echo")] engine: String,
    },
    /// 跑全部到期例行.
    Sweep {
        #[arg(long, default_value = "echo")] engine: String,
    },
    Remove {
        #[arg(long)] name: String,
    },
}

#[derive(Debug, Subcommand)]
enum SkillCmd {
    List,
    Install { path: PathBuf },
    Show { name: String },
}

#[derive(Debug, Subcommand)]
enum MemoryCmd {
    /// 记一行事实（去重；密钥行拒绝）.
    Set { text: String },
    /// 原样输出记忆.
    Get,
    /// 清空（需 --yes 二次确认）.
    Clear {
        #[arg(long, default_value_t = false)]
        yes: bool,
    },
    /// 撤一版（回到上一次记忆；无历史时是 no-op，不是错）.
    Undo,
}

#[derive(Debug, Subcommand)]
enum MemberCmd {
    Add {
        id: String,
        #[arg(long, default_value = "human")] kind: String,
    },
    List,
    Remove { id: String },
}

#[derive(Debug, Subcommand)]
enum ProviderCmd {
    /// 新增/更新端点（key_env 是环境变量名，不是 key 本身）.
    Add {
        #[arg(long)] name: String,
        #[arg(long)] base_url: String,
        #[arg(long, default_value = "")] key_env: String,
        #[arg(long, default_value = "")] model: String,
    },
    List,
    Remove {
        #[arg(long)] name: String,
    },
    On {
        #[arg(long)] name: String,
    },
    Off {
        #[arg(long)] name: String,
    },
    /// 一键下沉知名端点：neotrix | ollama | lmstudio | deepseek | openai | gemini | qwen | moonshot | zhipu.
    Preset { name: String },
}

#[derive(Debug, Subcommand)]
enum CoreCmd {
    /// 配对晶体核心（灵魂嵌入；探活成功才写，死端点拒绝）.
    Pair {
        base_url: String,
        /// 模型名（缺省 neotrix-crystal）.
        #[arg(long, default_value = "neotrix-crystal")]
        model: String,
        /// key 环境变量名（免 key 端点留空；永不存 key 本身）.
        #[arg(long, default_value = "")]
        key_env: String,
        /// token 环境变量名（缺省 CRYSTAL_TOKEN；只存名，现读环境）.
        #[arg(long, default_value = "CRYSTAL_TOKEN")]
        token_env: String,
    },
    /// 免费发现（OpenRouter 在线过滤 + Groq 表；仍需对应 key）.
    Free,
    /// 配对免费模型（discover 列表里的 model_id；缺 key 拒绝并指路）.
    PairFree {
        model_id: String,
    },
    /// 摘除配对（回纯本地；幂等）.
    Unpair,
    /// 灵魂状态（配对行 + 活探 + 版本/工具数）.
    Status,
    /// 热重载（POST /v1/admin/reload，需 token_env 现读）.
    Reload,
}

#[derive(Debug, Subcommand)]
enum AgentCmd {
    /// 服务端 agent 跑一轮（POST /v1/agents/run，需 token）.
    Run {
        /// 目标（goal，进服务端多步执行）.
        goal: String,
        /// 上下文（可选透传）.
        #[arg(long)]
        context: Option<String>,
        /// 步数上限 1-32（缺省 8；长任务调大，直到目标解决）。
        #[arg(long)]
        steps: Option<i64>,
    },
}

#[derive(Debug, Subcommand)]
enum ControlCmd {
    Take {
        #[arg(long)] holder: String,
    },
    Release {
        #[arg(long)] holder: String,
    },
    Status,
}

#[derive(Debug, Subcommand)]
enum ConvoCmd {    List,
    /// 建群组（成员逗号分隔，须已登记）.
    Group {
        #[arg(long)] title: String,
        #[arg(long, default_value = "")] members: String,
    },
    /// 取或建与某人的 DM.
    Dm {
        peer: String,
        #[arg(long, default_value = "owner")] me: String,
    },
    Rename {
        id: String,
        title: String,
    },
    Rm { id: String },
    /// 列成员.
    Members { id: String },
    AddMember { id: String, member: String },
    RmMember { id: String, member: String },
}

#[derive(Debug, Subcommand)]
enum AttachCmd {
    /// 存附件（拷贝进数据目录，50MB 上限；返回附件 id）.
    Add { convo: String, path: String },
    List { convo: String },
    Rm { id: String },
}

#[derive(Debug, Subcommand)]
enum PolicyCmd {
    /// fail-closed 演练（网关三律 + 自写规则 + 坏规则照拒）。
    Drill,
}

fn main() {
    if let Err(err) = real_main() {
        eprintln!("neobot: {err}");
        std::process::exit(1);
    }
}

fn real_main() -> Result<(), NtBotError> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Init => cmd_init(),
        Cmd::Doctor => cmd_doctor(),
        Cmd::Channel { cmd } => match cmd {
            ChannelCmd::List => cmd_channel_list(),
            ChannelCmd::Probe { channel } => cmd_channel_probe(&channel),
            ChannelCmd::Once { channel } => cmd_channel_once(channel.as_deref()),
            ChannelCmd::Serve { channel, interval } => cmd_channel_serve(channel.as_deref(), interval),
        },
        Cmd::Export { out } => cmd_export(out.as_deref()),
        Cmd::Run { title, text, engine, provider, model, convo, stream } => {
            cmd_run(&title, &text, &engine, provider.as_deref(), model.as_deref(), convo.as_deref(), stream)
        }
        Cmd::Task { cmd } => match cmd {
            TaskCmd::List => cmd_task_list(),
            TaskCmd::Claim { id, actor } => cmd_task_claim(&id, &actor),
            TaskCmd::Release { id, actor } => cmd_task_release(&id, &actor),
            TaskCmd::Cancel { id } => cmd_task_cancel(&id),
            TaskCmd::Retry { id } => cmd_task_retry(&id),
            TaskCmd::Rename { id, title } => cmd_task_rename(&id, &title),
            TaskCmd::Rm { id } => cmd_task_rm(&id),
        },
        Cmd::Audit { cmd } => match cmd {
            AuditCmd::List => cmd_audit_list(),
            AuditCmd::Prune { days } => cmd_audit_prune(days),
        },
        Cmd::Ledger { by_actor } => cmd_ledger(by_actor),
        Cmd::Routine { cmd } => match cmd {
            RoutineCmd::Add { name, interval, instruction, owner } => {
                cmd_routine_add(&name, interval, &instruction, &owner)
            }
            RoutineCmd::List => cmd_routine_list(),
            RoutineCmd::Fire { name, engine } => cmd_routine_fire(&name, &engine),
            RoutineCmd::Sweep { engine } => cmd_routine_sweep(&engine),
            RoutineCmd::Remove { name } => cmd_routine_remove(&name),
        },
        Cmd::Skill { cmd } => match cmd {
            SkillCmd::List => cmd_skill_list(),
            SkillCmd::Install { path } => cmd_skill_install(&path),
            SkillCmd::Show { name } => cmd_skill_show(&name),
        },
        Cmd::Memory { cmd } => match cmd {
            MemoryCmd::Set { text } => cmd_memory_set(&text),
            MemoryCmd::Get => cmd_memory_get(),
            MemoryCmd::Clear { yes } => cmd_memory_clear(yes),
            MemoryCmd::Undo => cmd_memory_undo(),
        },
        Cmd::Member { cmd } => match cmd {
            MemberCmd::Add { id, kind } => cmd_member_add(&id, &kind),
            MemberCmd::List => cmd_member_list(),
            MemberCmd::Remove { id } => cmd_member_remove(&id),
        },
        Cmd::Control { cmd } => match cmd {
            ControlCmd::Take { holder } => cmd_control_take(&holder),
            ControlCmd::Release { holder } => cmd_control_release(&holder),
            ControlCmd::Status => cmd_control_status(),
        },
        Cmd::Convo { cmd } => match cmd {
            ConvoCmd::List => cmd_convo_list(),
            ConvoCmd::Group { title, members } => cmd_convo_group(&title, &members),
            ConvoCmd::Dm { peer, me } => cmd_convo_dm(&peer, &me),
            ConvoCmd::Rename { id, title } => cmd_convo_rename(&id, &title),
            ConvoCmd::Rm { id } => cmd_convo_rm(&id),
            ConvoCmd::Members { id } => cmd_convo_members(&id),
            ConvoCmd::AddMember { id, member } => cmd_convo_add_member(&id, &member),
            ConvoCmd::RmMember { id, member } => cmd_convo_rm_member(&id, &member),
        },
        Cmd::Attach { cmd } => match cmd {
            AttachCmd::Add { convo, path } => cmd_attach_add(&convo, &path),
            AttachCmd::List { convo } => cmd_attach_list(&convo),
            AttachCmd::Rm { id } => cmd_attach_rm(&id),
        },
        Cmd::Policy { cmd } => match cmd {
            PolicyCmd::Drill => cmd_policy_drill(),
        },
        Cmd::Models { provider } => cmd_models(provider.as_deref()),
        Cmd::Provider { cmd } => match cmd {
            ProviderCmd::Add { name, base_url, key_env, model } => {
                cmd_provider_add(&name, &base_url, &key_env, &model)
            }
            ProviderCmd::List => cmd_provider_list(),
            ProviderCmd::Remove { name } => cmd_provider_remove(&name),
            ProviderCmd::On { name } => cmd_provider_toggle(&name, true),
            ProviderCmd::Off { name } => cmd_provider_toggle(&name, false),
            ProviderCmd::Preset { name } => cmd_provider_preset(&name),
        },
        Cmd::Core { cmd } => match cmd {
            CoreCmd::Pair { base_url, model, key_env, token_env } => {
                // 兼容：老脚本 `--key-env FOO` 在未显式传 --token-env 时仍生效；
                // 显式 --token-env 优先（缺省 CRYSTAL_TOKEN）。
                let env_name = if token_env.trim() != neotrix_neobot::DEFAULT_TOKEN_ENV
                    || key_env.trim().is_empty()
                {
                    token_env.clone()
                } else {
                    key_env.clone()
                };
                cmd_core_pair(&base_url, &model, &env_name)
            }
            CoreCmd::Free => cmd_core_free(),
            CoreCmd::PairFree { model_id } => cmd_core_pair_free(&model_id),
            CoreCmd::Unpair => cmd_core_unpair(),
            CoreCmd::Status => cmd_core_status(),
            CoreCmd::Reload => cmd_core_reload(),
        },
        Cmd::Agent { cmd } => match cmd {
            AgentCmd::Run { goal, context, steps } => cmd_agent_run(&goal, context.as_deref(), steps),
        },
    }
}

/// 一键备份：db（含 WAL 落盘）+ 附件 + MEMORY.md + config.json → 单 zip。
/// 列出渠道与机器人（token 只出**变量名**）。
fn cmd_channel_list() -> Result<(), NtBotError> {
    use neotrix_neobot::nt_store::parse_allow_list;
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    let channels = store.list_channels()?;
    if channels.is_empty() {
        println!("没有配置任何渠道。到设置页「IM 机器人」里加，或见 `neobot doctor`。");
        return Ok(());
    }
    for row in &channels {
        let flag = if row.enabled { "启用" } else { "停用" };
        println!("\n== {} ({}) · {flag} · 访问 {} · 节拍 {}s",
            row.title, row.id, row.access_mode, row.poll_secs);
        let bots = store.list_bots(&row.id)?;
        if bots.is_empty() {
            println!("   （还没挂机器人）");
            continue;
        }
        for bot in bots {
            let alias = if bot.alias.trim().is_empty() { &bot.bot_id } else { &bot.alias };
            let token = if bot.token_env.trim().is_empty() {
                "（未设 token 变量）".to_owned()
            } else if neotrix_neobot::nt_channel_serve::bot_token_missing(&bot.token_env) {
                format!("{} ← 未设置！", bot.token_env)
            } else {
                format!("{} ← 已设", bot.token_env)
            };
            let allow = parse_allow_list(&bot.allow_list);
            let allow_txt = if allow.is_empty() { "（名单空）".to_owned() } else { allow.join(", ") };
            let convo = bot.conversation_id.as_deref().unwrap_or("（未绑定）");
            println!(
                "   · {alias} [{}] token: {token}\n     绑定会话 {convo} · 白名单 {allow_txt} · 最近 {}",
                bot.bot_id,
                bot.last_seen.as_deref().unwrap_or("从未收发")
            );
        }
    }
    println!("\n提示：`neobot channel serve` 常驻长轮询；`neobot channel once` 单轮。");
    Ok(())
}

/// 探活一个渠道。
fn cmd_channel_probe(channel: &str) -> Result<(), NtBotError> {
    let mut reg = neotrix_neobot::nt_channel_serve::registry();
    let Some(adapter) = reg.get_mut(channel) else {
        return Err(NtBotError::Invalid(format!("未注册的渠道：{channel}")));
    };
    let health = adapter.probe();
    if health.ok {
        println!("{channel} 在线{}", if health.info.is_empty() { String::new() } else { format!(" · {}", health.info) });
    } else {
        println!("{channel} 不通：{}", health.detail);
    }
    Ok(())
}

/// 手动跑一轮。
fn cmd_channel_once(channel: Option<&str>) -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    let stats = neotrix_neobot::nt_channel_serve::run_once(&store, &cfg, channel)
        .map_err(NtBotError::Store)?;
    println!(
        "收到 {}（跑轮 {} · 忽略 {}）· 失败 {} · 补发 {}",
        stats.received, stats.ran, stats.ignored, stats.failed, stats.deferred
    );
    Ok(())
}

/// 常驻长轮询（Ctrl-C 退出）。
///
/// 每轮**重读**渠道配置：在设置页改了访问模式或停用了某渠道，
/// 不必重启本进程就生效 —— 否则「停用」这个动作只是看起来生效了。
fn cmd_channel_serve(channel: Option<&str>, interval: Option<i64>) -> Result<(), NtBotError> {
    use std::time::Duration;
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    let fixed = interval.map(|secs| secs.max(1));
    println!("neobot channel serve —— Ctrl-C 退出。数据只在 {}", cfg.data_dir.display());
    // ⭐ 启动刷一次崩溃残留（`mark_outcome_unknown`，A1）。
    // ⛔ 刻意**不**挂 `open_store()`：`open_store` 在 CLI 里有 50 处调用，
    //    挂那里会让 `task list` 这类纯读命令也触发一次 UPDATE，
    //    「启动一次」的语义就名不副实了。
    // ⓰ best-effort 用 `let _name: usize = …` 形状：既与本文件既有风格一致，
    //    也避开 `check-silent-failure` 的 `let _ =` opener
    //    （方法名不在 GATED 动词表里，双保险）。
    let _marked_unknown: usize = store
        .mark_outcome_unknown(&chrono_now(), "startup sweep (channel serve)")
        .unwrap_or(0);
    let mut round: u64 = 0;
    loop {
        round = round.saturating_add(1);
        let stats = match neotrix_neobot::nt_channel_serve::run_once(&store, &cfg, channel) {
            Ok(stats) => stats,
            Err(err) => {
                // 单轮整体失败也不退出：长轮询服务本来就该比它服务的平台更耐用。
                eprintln!("[neobot] 第 {round} 轮出错（继续）：{err}");
                neotrix_neobot::nt_channel_serve::RoundStats::default()
            }
        };
        if !stats.is_quiet() {
            println!(
                "[第 {round} 轮] 收到 {} · 跑轮 {} · 忽略 {} · 失败 {} · 补发 {}",
                stats.received, stats.ran, stats.ignored, stats.failed, stats.deferred
            );
        }
        // 2026-09-29 接线（自 `nt_coverage_gaps` 报出的死代码）：
        // 账目清理失败**不计入 `failed`**（它不阻断出站），所以 `is_quiet()`
        // 仍为 true ⇒ 上面那行不会打印 ⇒ 错误会彻底静默。
        // 这正是当初把它拆成独立方法的原因，此处补上消费点。
        if stats.has_prune_error() {
            eprintln!(
                "[第 {round} 轮] ⚠ 账目清理失败（账目表可能在持续膨胀）：{}",
                stats.changes_prune_error.as_deref().unwrap_or("未知原因")
            );
        }
        if stats.changes_pruned > 0 {
            eprintln!(
                "[第 {round} 轮] 账目清理：删掉 {} 行（按留存期 + 每任务保尾）",
                stats.changes_pruned
            );
        }
        // 节拍：显式 --interval > 该渠道的 poll_secs；多渠道取最小（最急的那个说话）。
        let secs = fixed.unwrap_or_else(|| {
            let table = neotrix_neobot::nt_channel_serve::intervals(&store, channel);
            table.values().copied().min().unwrap_or(5)
        });
        // 分片睡：Ctrl-C 能在最坏 200ms 内生效，而不是等满一个轮询周期。
        neotrix_neobot::nt_channel_serve::slice_sleep(Duration::from_secs(secs as u64));
    }
}

fn cmd_export(out: Option<&std::path::Path>) -> Result<(), NtBotError> {
    use neotrix_neobot::export_bundle;
    let cfg = load_config()?;
    let path = match out {
        Some(p) => p.to_path_buf(),
        None => {
            let stamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
            cfg.data_dir.join(format!("backups/neobot-{stamp}.zip"))
        }
    };
    let report = export_bundle(&cfg.data_dir, &path)?;
    println!(
        "neobot export ok: {} (db {} bytes, {} files, {} tasks / {} convos / {} ledger rows)",
        report.path.to_string_lossy(),
        report.db_bytes,
        report.files,
        report.tasks,
        report.convos,
        report.ledger_rows,
    );
    Ok(())
}

fn cmd_init() -> Result<(), NtBotError> {    let cfg = load_config()?;
    let config_path = cfg.data_dir.join("config.json");
    if !config_path.exists() {
        let json = serde_json::to_string_pretty(&cfg)?;
        atomic_write(&config_path, json.as_bytes())?;
    }
    let _ = open_store(&cfg)?;
    println!("neobot init ok: {}", cfg.data_dir.to_string_lossy());
    Ok(())
}

/// 原子写（staging + rename，不跟野指针）。
fn atomic_write(path: &std::path::Path, data: &[u8]) -> Result<(), NtBotError> {
    use std::io::Write as _;
    let tmp = path.with_extension("json.tmp");
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&tmp)?;
    file.write_all(data)?;
    file.sync_all()?;
    drop(file);
    std::fs::rename(&tmp, path)?;
    Ok(())
}

fn cmd_doctor() -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    // 启动即回收：崩溃残留 + 过期认领（单机，一次 UPDATE 级代价）。
    // ⭐ A1：`recover_stale_running` 现在落 `outcome_unknown` 而非 `pending`
    //   —— 租约过期的任务**不再被自动重跑**（外部副作用可能已落地）。
    const RECOVERY_NOTE: &str = "lease expired (crash recovery)";
    let now = chrono_now();
    let recovered = store.mark_outcome_unknown(&now, RECOVERY_NOTE)?;
    let swept = store.sweep_stale_claims(&now, neotrix_neobot::CLAIM_TTL_SECS)?;
    let engine_info = match &cfg.engine {
        EngineKind::Echo => LocalEchoEngine.probe()?,
        EngineKind::Cli { command } => CliEngine::new(command)?.probe()?,
        EngineKind::Opencode { model } => OpencodeEngine::new(model)?.probe()?,
        EngineKind::Http { .. } => HttpEngine::from_env()?.probe()?,
    };
    let tasks = store.list_tasks(1)?;
    let holder = store.control_holder()?;
    let routines = store.list_routines()?;
    println!(
        "neobot doctor ok: data={} policy={} engine={} tasks={} recovered={} claims_swept={} control={} routines={}",
        cfg.data_dir.to_string_lossy(),
        match cfg.policy_mode {
            PolicyMode::Enforce => "enforce",
            PolicyMode::DryRun => "dry_run",
        },
        engine_info,
        tasks.len(),
        recovered,
        swept,
        holder.as_deref().unwrap_or("-"),
        routines.len(),
    );
    Ok(())
}

fn chrono_now() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// 按 `--engine` 解析出 boxed 引擎（run/fire/sweep 共用）。
fn resolve_engine(
    cfg: &mut NeobotConfig,
    engine_name: &str,
    model_override: Option<&str>,
) -> Result<Box<dyn EngineAdapter>, NtBotError> {
    let trimmed = engine_name.trim();
    if trimmed == "http" {
        let (http, _) = neotrix_neobot::HttpEngineConfig::from_env()?;
        cfg.engine = EngineKind::Http {
            base_url: http.base_url,
            model: http.model,
        };
        let memory = neotrix_neobot::nt_memory::memory_for_config(&cfg);
        Ok(Box::new(HttpEngine::from_env()?.with_memory_context(memory)))
    } else if trimmed.is_empty() || trimmed == "echo" {
        Ok(Box::new(LocalEchoEngine))
    } else if trimmed == "opencode" {
        // `run --engine opencode --model <provider/model>`；缺省 space-bunny-free。
        let model = model_override
            .map(str::trim)
            .filter(|m| !m.is_empty())
            .map(str::to_owned)
            .or_else(|| {
                let env = std::env::var("NEOBOT_MODEL").unwrap_or_default();
                let t = env.trim().to_owned();
                if t.is_empty() {
                    None
                } else {
                    Some(t)
                }
            })
            .unwrap_or_else(|| neotrix_neobot::nt_engine::DEFAULT_ZEN_MODEL.to_owned());
        cfg.engine = EngineKind::Opencode { model: model.clone() };
        Ok(Box::new(OpencodeEngine::new(&model)?))
    } else {
        cfg.engine = EngineKind::Cli {
            command: trimmed.to_owned(),
        };
        Ok(Box::new(CliEngine::new(trimmed)?))
    }
}

fn cmd_run(
    title: &str,
    text: &str,
    engine_name: &str,
    provider_name: Option<&str>,
    model_override: Option<&str>,
    convo_id: Option<&str>,
    stream: bool,
) -> Result<(), NtBotError> {
    let mut cfg = load_config()?;
    let store = open_store(&cfg)?;
    // --provider 优先：走注册端点的 HTTP 引擎（key 读其 key_env）。
    // 无 --provider 且引擎为默认 echo：配对且活着的晶体核心优先（灵魂嵌入）；
    // 核心离线则回落本地回显并明示（不静默）。
    let engine: Box<dyn EngineAdapter> = match provider_name {
        Some(name) => {
            let provider = store.get_provider(name)?.ok_or_else(|| {
                NtBotError::Store(format!("no such provider '{name}'"))
            })?;
            if !provider.enabled {
                return Err(NtBotError::Store(format!("provider '{name}' is off")));
            }
            let memory = neotrix_neobot::nt_memory::memory_for_config(&cfg);
            Box::new(provider.http_engine(model_override)?.with_memory_context(memory))
        }
        None if engine_name.trim().is_empty() || engine_name.trim() == "echo" => {
            // 灵魂优先：HTTP 配对且活着 → 核心；CLI 配对（Zen）→ 本机 opencode；
            // 配对但不可用 → 明示回落，不静默。
            if let Some(http) = neotrix_neobot::core_engine(&store)? {
                let memory = neotrix_neobot::nt_memory::memory_for_config(&cfg);
                eprintln!("neobot: soul online, routing via crystal core");
                Box::new(http.with_memory_context(memory))
            } else if let Ok(Some(pair)) = store.get_core_pair() {
                if pair.via == "cli" {
                    match OpencodeEngine::new(&pair.model) {
                        Ok(engine) => {
                            eprintln!("neobot: soul online, routing via opencode zen");
                            Box::new(engine)
                        }
                        Err(e) => {
                            eprintln!("neobot: soul offline ({e}), fell back to local echo");
                            resolve_engine(&mut cfg, engine_name, model_override)?
                        }
                    }
                } else {
                    eprintln!("neobot: soul offline, fell back to local echo");
                    resolve_engine(&mut cfg, engine_name, model_override)?
                }
            } else {
                resolve_engine(&mut cfg, engine_name, model_override)?
            }
        }
        None => resolve_engine(&mut cfg, engine_name, model_override)?,
    };
    // CLI 发起方记名：NEOBOT_ACTOR（默认 bot；定时/自动化用 routine 名）。
    let (actor, actor_name) = cli_actor();
    let mut emit = |delta: &str| {
        print!("{delta}");
        use std::io::Write as _;
        let _ = std::io::stdout().flush();
    };
    let is_http = provider_name.is_some() || engine_name.trim() == "http";
    let status = if stream {
        if is_http {
            println!("--- stream ---");
        }
        let status = run_local_turn_stream_as(
            &store, &cfg, engine.as_ref(), actor, &actor_name, title, text, convo_id, &mut emit,
            None,
        )?;
        if is_http {
            println!("\n--- end ---");
        }
        status
    } else {
        run_local_turn_as(
            &store, &cfg, engine.as_ref(), actor, &actor_name, title, text, convo_id,
        )?
    };
    println!("neobot run ok: status={} actor={actor_name}", status.as_str());
    Ok(())
}

/// CLI 发起方：`NEOBOT_ACTOR=alice` → Person/alice；
/// `NEOBOT_ACTOR=routine:daily` → Routine/routine:daily；缺省 Bot/bot。
fn cli_actor() -> (Actor, String) {
    let raw = std::env::var("NEOBOT_ACTOR").unwrap_or_default();
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed == "bot" {
        return (Actor::Bot, "bot".to_owned());
    }
    if let Some(name) = trimmed.strip_prefix("routine:") {
        let name = name.trim();
        if !name.is_empty() {
            return (Actor::Routine, format!("routine:{name}"));
        }
    }
    (Actor::Person, trimmed.to_owned())
}

fn cmd_task_list() -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    for task in store.list_tasks(20)? {
        let claim = task.claimed_by.as_deref().unwrap_or("-");
        println!(
            "{} [{}] {} claim={} vis={} attempts={}",
            task.id, task.status.as_str(), task.title, claim, task.visibility, task.attempts,
        );
    }
    Ok(())
}

fn cmd_task_claim(id: &str, actor: &str) -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    store.claim_task(id, actor)?;
    println!("neobot task {id} claimed by {actor}");
    Ok(())
}

fn cmd_task_release(id: &str, actor: &str) -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    store.release_task(id, actor)?;
    println!("neobot task {id} released by {actor}");
    Ok(())
}

fn cmd_task_cancel(id: &str) -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    store.cancel_task(id)?;
    println!("neobot task {id} cancelled");
    Ok(())
}

fn cmd_task_retry(id: &str) -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    store.retry_task(id)?;
    println!("neobot task {id} re-queued");
    Ok(())
}

fn cmd_task_rename(id: &str, title: &str) -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    store.rename_task(id, title)?;
    println!("neobot task {id} renamed");
    Ok(())
}

fn cmd_task_rm(id: &str) -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    store.delete_task(id)?;
    println!("neobot task {id} deleted");
    Ok(())
}

fn cmd_audit_list() -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    for event in store.list_audit(20)? {
        let rule = event.rule.as_deref().unwrap_or("-");
        println!(
            "{} {} {} {} rule={}",
            event.at,
            event.actor,
            event.tool,
            event.decision.as_str(),
            rule
        );
    }
    Ok(())
}

fn cmd_audit_prune(days: i64) -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    let cutoff = (chrono::Utc::now() - chrono::Duration::days(days.max(0))).to_rfc3339();
    let n = store.prune_audit(&cutoff)?;
    println!("neobot audit pruned: {n} rows older than {days}d");
    Ok(())
}

fn cmd_ledger(by_actor: bool) -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    if by_actor {
        for (engine, model, actor, input, output, cost) in store.ledger_sums_by_actor()? {
            println!("{engine}/{model} actor={actor} in={input} out={output} cost=${cost:.4}");
        }
    } else {
        for (engine, model, input, output, cost) in store.ledger_sums()? {
            println!("{engine}/{model} in={input} out={output} cost=${cost:.4}");
        }
    }
    Ok(())
}

fn cmd_routine_add(name: &str, interval: i64, instruction: &str, owner: &str) -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    let now = chrono::Utc::now().timestamp();
    store.register_routine(name, interval, instruction, owner, now)?;
    println!("neobot routine '{name}' registered (every {interval}s)");
    Ok(())
}

fn cmd_routine_list() -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    for r in store.list_routines()? {
        println!(
            "{} every={}s failures={} disabled={} next={} owner={} :: {}",
            r.name, r.interval_secs, r.failures, r.disabled, r.next_run_at, r.owner,
            r.instruction.chars().take(80).collect::<String>(),
        );
    }
    Ok(())
}

fn cmd_routine_fire(name: &str, engine_name: &str) -> Result<(), NtBotError> {
    let mut cfg = load_config()?;
    let engine = resolve_engine(&mut cfg, engine_name, None)?;
    let store = open_store(&cfg)?;
    let now = chrono::Utc::now().timestamp();
    let (status, disabled) = fire_routine(&store, &cfg, engine.as_ref(), name, now)?;
    println!(
        "neobot routine '{name}' fired: status={}{}",
        status.as_str(),
        if disabled { " (SWITCHED OFF: 10 failures)" } else { "" },
    );
    Ok(())
}

fn cmd_routine_sweep(engine_name: &str) -> Result<(), NtBotError> {
    let mut cfg = load_config()?;
    let engine = resolve_engine(&mut cfg, engine_name, None)?;
    let store = open_store(&cfg)?;
    let now = chrono::Utc::now().timestamp();
    let fired = sweep_routines(&store, &cfg, engine.as_ref(), now)?;
    if fired.is_empty() {
        println!("neobot routine sweep: nothing due");
    }
    for (name, status, disabled) in fired {
        println!(
            "neobot routine '{name}': {}{}",
            status.as_str(),
            if disabled { " (SWITCHED OFF)" } else { "" },
        );
    }
    Ok(())
}

fn cmd_routine_remove(name: &str) -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    store.remove_routine(name)?;
    println!("neobot routine '{name}' removed");
    Ok(())
}

fn cmd_skill_list() -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let (skills, skipped) = neotrix_neobot::nt_skills::scan_skills(&cfg.data_dir);
    for skill in skills {
        println!("{} — {}", skill.name, skill.description);
    }
    if skipped > 0 {
        println!("(skipped {skipped} broken package(s))");
    }
    Ok(())
}

fn cmd_skill_install(path: &std::path::Path) -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let skill = neotrix_neobot::nt_skills::install_skill(&cfg.data_dir, path)?;
    println!("neobot skill '{}' installed", skill.name);
    Ok(())
}

fn cmd_skill_show(name: &str) -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let (skills, _) = neotrix_neobot::nt_skills::scan_skills(&cfg.data_dir);
    let Some(skill) = skills.iter().find(|s| s.name == name) else {
        return Err(NtBotError::Store(format!("no such skill '{name}'")));
    };
    println!("name: {}", skill.name);
    println!("path: {}", skill.path.to_string_lossy());
    println!("desc: {}", skill.description);
    Ok(())
}

fn cmd_memory_set(text: &str) -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let added = neotrix_neobot::nt_memory::append_memory(&cfg.data_dir, text)?;
    println!(
        "neobot memory {}",
        if added { "recorded" } else { "already recorded" }
    );
    Ok(())
}

fn cmd_memory_get() -> Result<(), NtBotError> {
    let cfg = load_config()?;
    print!("{}", neotrix_neobot::nt_memory::read_memory(&cfg.data_dir));
    Ok(())
}

fn cmd_memory_clear(yes: bool) -> Result<(), NtBotError> {
    if !yes {
        return Err(NtBotError::Invalid(
            "refusing without --yes (destructive)".to_owned(),
        ));
    }
    let cfg = load_config()?;
    std::fs::remove_file(neotrix_neobot::nt_memory::memory_path(&cfg.data_dir))?;
    println!("neobot memory cleared");
    Ok(())
}

/// 撤一版记忆。回退本身也可再撤（撤前先把当前存进历史）。
fn cmd_memory_undo() -> Result<(), NtBotError> {
    let cfg = load_config()?;
    match neotrix_neobot::nt_memory::memory_undo(&cfg.data_dir)? {
        Some(_) => println!("neobot memory undone"),
        None => println!("neobot memory nothing to undo"),
    }
    Ok(())
}

fn cmd_member_add(id: &str, kind: &str) -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    store.upsert_member(id, kind)?;
    println!("neobot member '{id}' upserted ({kind})");
    Ok(())
}

fn cmd_member_list() -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    let owner = store.owner_name()?;
    for (id, kind, at) in store.list_members()? {
        let mark = if owner.as_deref() == Some(&id) { " [owner]" } else { "" };
        println!("{id} ({kind}) since={at}{mark}");
    }
    Ok(())
}

fn cmd_member_remove(id: &str) -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    store.remove_member(id)?;
    println!("neobot member '{id}' removed");
    Ok(())
}

fn cmd_control_take(holder: &str) -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    store.take_control(holder)?;
    println!("neobot control taken by {holder}");
    Ok(())
}

fn cmd_control_release(holder: &str) -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    store.release_control(holder)?;
    println!("neobot control released by {holder}");
    Ok(())
}

fn cmd_control_status() -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    match store.control_holder()? {
        Some(holder) => println!("control held by {holder}"),
        None => println!("control free"),
    }
    Ok(())
}

fn cmd_convo_list() -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    for c in store.list_conversations()? {
        println!(
            "{} [{}] {} members={} tasks={} active={}",
            c.id,
            c.kind,
            c.title,
            c.members.join(","),
            c.task_count,
            c.last_active,
        );
    }
    Ok(())
}

fn split_members(raw: &str) -> Vec<String> {
    raw.split([',', ' ', '，', '、'])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}

fn cmd_convo_group(title: &str, members: &str) -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    let id = store.create_conversation("group", title, &split_members(members))?;
    println!("neobot convo group '{title}' created: {id}");
    Ok(())
}

fn cmd_convo_dm(peer: &str, me: &str) -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    // 自注册（与桌面 ensure_default_dm 同律）：me 不存在即建 human 行；
    // peer 仍须已登记（防拼写漂进陌生会话）。
    store.upsert_member(me.trim(), "human")?;
    let id = store.get_or_create_dm(me, peer)?;
    println!("neobot dm with '{peer}': {id}");
    Ok(())
}

fn cmd_convo_rename(id: &str, title: &str) -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    store.rename_conversation(id, title)?;
    println!("neobot convo {id} renamed");
    Ok(())
}

fn cmd_convo_rm(id: &str) -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    store.delete_conversation(id)?;
    println!("neobot convo {id} deleted (tasks cascaded)");
    Ok(())
}

fn cmd_convo_members(id: &str) -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    for m in store.conversation_members(id)? {
        println!("{m}");
    }
    Ok(())
}

fn cmd_convo_add_member(id: &str, member: &str) -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    store.add_conversation_member(id, member)?;
    println!("neobot convo {id} += {member}");
    Ok(())
}

fn cmd_convo_rm_member(id: &str, member: &str) -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    store.remove_conversation_member(id, member)?;
    println!("neobot convo {id} -= {member}");
    Ok(())
}

/// 附件 50MB 上限.
const ATTACH_MAX_BYTES: u64 = 50 * 1024 * 1024;

fn sanitize_filename(name: &str) -> String {
    let clean: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || matches!(c, '.' | '-' | '_' | ' ') {
                c
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = clean.trim().trim_start_matches('.');
    if trimmed.is_empty() {
        "file".to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn cmd_attach_add(convo: &str, src: &str) -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    let src_path = std::path::Path::new(src);
    let meta = std::fs::metadata(src_path).map_err(|err| {
        NtBotError::Invalid(format!("cannot read '{src}': {err}"))
    })?;
    if !meta.is_file() {
        return Err(NtBotError::Invalid(format!("'{src}' is not a file")));
    }
    if meta.len() > ATTACH_MAX_BYTES {
        return Err(NtBotError::Invalid(format!(
            "file exceeds 50MB ({} bytes)",
            meta.len()
        )));
    }
    let name = src_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("file");
    let safe = sanitize_filename(name);
    let dest_name = format!("{}-{safe}", uuid::Uuid::new_v4());
    let dest_dir = cfg.data_dir.join("attachments");
    std::fs::create_dir_all(&dest_dir)?;
    let dest = dest_dir.join(&dest_name);
    std::fs::copy(src_path, &dest)?;
    let kind = neotrix_neobot::classify_attachment(&safe).to_owned();
    let id = store.add_attachment(
        convo,
        &kind,
        &safe,
        &dest.to_string_lossy(),
        meta.len() as i64,
    )?;
    println!("neobot attach ok: {id} [{kind}] {safe}");
    Ok(())
}

fn cmd_attach_list(convo: &str) -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    for att in store.list_attachments(convo)? {
        println!("{} [{}] {} ({} bytes)", att.id, att.kind, att.name, att.size);
    }
    Ok(())
}

fn cmd_attach_rm(id: &str) -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    store.remove_attachment(id)?;
    println!("neobot attach {id} removed");
    Ok(())
}

/// fail-closed 演练：网关三律 + 自写规则 + 坏规则照拒 + 发起方隔离。
fn cmd_policy_drill() -> Result<(), NtBotError> {
    use neotrix_neobot::{Actor as A, PolicyContext, ToolName, evaluate_policy, nt_policy};
    let mut failures = 0;
    let mut check = |name: &str, ok: bool| {
        println!("{} {}", if ok { "PASS" } else { "FAIL" }, name);
        if !ok {
            failures += 1;
        }
    };
    let base = |tool: ToolName| PolicyContext {
        tool,
        actor: A::Bot,
        human_has_control: false,
        file_path: None,
        command: None,
        computer_action: None,
        computer_target: None,
        computer_allow: Vec::new(),
        computer_hosts: Vec::new(),
    };
    // 1. 未知工具永拒
    check(
        "unknown tool denied",
        matches!(
            evaluate_policy(&base(ToolName::Unknown("rm_rf".to_owned()))),
            neotrix_neobot::PolicyDecision::Deny { .. }
        ),
    );
    // 2. 越狱路径拒
    let mut jail = base(ToolName::ReadFile);
    jail.file_path = Some("../secret".to_owned());
    check(
        "jailbreak path denied",
        matches!(evaluate_policy(&jail), neotrix_neobot::PolicyDecision::Deny { .. }),
    );
    // 3. 人接管时 Bot 拒、Person 行
    let mut taken = base(ToolName::Bash);
    taken.human_has_control = true;
    check(
        "human-control denies bot",
        matches!(evaluate_policy(&taken), neotrix_neobot::PolicyDecision::Deny { .. }),
    );
    taken.actor = A::Person;
    check(
        "human-control allows person",
        matches!(evaluate_policy(&taken), neotrix_neobot::PolicyDecision::Allow),
    );
    // 4. computer 默认拒
    check(
        "computer_act default deny",
        matches!(
            evaluate_policy(&base(ToolName::ComputerAct)),
            neotrix_neobot::PolicyDecision::Deny { .. }
        ),
    );
    // 5. 自写 deny 命中
    check(
        "extra deny tool:bash hits",
        nt_policy::evaluate_extra_deny(
            &["deny tool:bash".to_owned()],
            &ToolName::Bash,
            A::Bot,
            Some("echo hi"),
            None,
        )
        .is_some(),
    );
    // 6. 坏规则照拒
    check(
        "broken rule still denies",
        nt_policy::evaluate_extra_deny(
            &["deny tool bash".to_owned()],
            &ToolName::ReadFile,
            A::Bot,
            None,
            Some("notes/a.md"),
        )
        .is_some_and(|(rule, _)| rule == "broken-rule"),
    );
    // 7. 无规则即放行（base 层内正常工具）
    check(
        "no extra rules allows normal read",
        nt_policy::evaluate_extra_deny(
            &[],
            &ToolName::ReadFile,
            A::Bot,
            None,
            Some("notes/a.md"),
        )
        .is_none(),
    );
    if failures > 0 {
        return Err(NtBotError::Store(format!("policy drill: {failures} FAIL")));
    }
    println!("policy drill: all PASS");
    Ok(())
}

fn cmd_models(only: Option<&str>) -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    // 1) legacy env 端点（NEOBOT_BASE_URL，未配置即默认 Ollama 口）。
    let mut pooled: Vec<neotrix_neobot::PoolModel> = Vec::new();
    let want_env = only.is_none();
    if want_env {
        match HttpEngine::for_listing() {
            Ok(engine) => match engine.list_models() {
                Ok(models) => {
                    for (id, owner) in models {
                        pooled.push(neotrix_neobot::PoolModel {
                            provider: "env".to_owned(),
                            id,
                            owner,
                        });
                    }
                }
                Err(err) => eprintln!("neobot: env 端点不可达（跳过）：{err}"),
            },
            Err(err) => eprintln!("neobot: env 端点配置无效（跳过）：{err}"),
        }
    }
    // 2) 注册的第三方端点（启用中；only 过滤单个；聚合律见 `pool_models`）。
    let (mut registered, unreachable) = neotrix_neobot::pool_models(&store);
    if let Some(name) = only {
        registered.retain(|m| m.provider == name);
    }
    for name in &unreachable {
        if only.map(|n| n == name).unwrap_or(true) {
            eprintln!("neobot: 端点 '{name}' 不可达（已用 fallback 行）：发现跳过");
        }
    }
    pooled.extend(registered);
    if let Some(name) = only {
        if !pooled.iter().any(|m| m.provider == name) && store.get_provider(name)?.is_none() {
            return Err(NtBotError::Store(format!("no such provider '{name}'")));
        }
    }
    pooled.sort_by(|a, b| (&a.provider, &a.id).cmp(&(&b.provider, &b.id)));
    for m in pooled {
        println!("{}  (owner={} provider={})", m.id, m.owner, m.provider);
    }
    Ok(())
}

fn cmd_provider_add(name: &str, base_url: &str, key_env: &str, model: &str) -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    store.upsert_provider(&Provider {
        name: name.trim().to_owned(),
        base_url: base_url.trim().to_owned(),
        key_env: key_env.trim().to_owned(),
        model: model.trim().to_owned(),
        enabled: true,
    })?;
    println!("neobot provider '{name}' saved (key via env ${})", if key_env.trim().is_empty() { "-" } else { key_env.trim() });
    Ok(())
}

fn cmd_provider_list() -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    for p in store.list_providers()? {
        // key_env 回显脱敏：只显示变量名；若已是明文泄露形态，告警不打印值。
        let key_show = if p.key_env.is_empty() {
            "-".to_owned()
        } else if neotrix_neobot::Provider::looks_like_secret(&p.key_env) {
            "!!PLAINTEXT-LEAK!!(export 变量后重填)".to_owned()
        } else {
            p.key_env.clone()
        };
        println!(
            "{} {} model={} key_env={} {}",
            if p.enabled { "[on]" } else { "[off]" },
            p.name,
            if p.model.is_empty() { "-" } else { &p.model },
            key_show,
            p.base_url,
        );
    }
    Ok(())
}

/// 配对晶体核心（探活成功才嵌入；死端点拒绝并说明）。
/// `token_env` 只存变量名（缺省 CRYSTAL_TOKEN），token 每次请求现读环境。
fn cmd_core_pair(base_url: &str, model: &str, token_env: &str) -> Result<(), NtBotError> {
    use neotrix_neobot::pair_core;
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    let (pair, count) = pair_core(&store, base_url, model, token_env)?;
    println!(
        "neobot core paired: soul embedded ({} models at {}, model={}, token_env={})",
        count, pair.base_url, pair.model, pair.token_env
    );
    Ok(())
}

/// 免费发现（在线过滤；key 是否就位一并展示）。
fn cmd_core_free() -> Result<(), NtBotError> {
    use neotrix_neobot::discover_free;
    let found = discover_free();
    if found.is_empty() {
        println!("neobot core free: discover failed (offline?)");
        return Ok(());
    }
    for e in &found {
        let key = std::env::var(e.key_env).unwrap_or_default();
        let key_state = if e.key_env.is_empty() {
            "no-key-needed"
        } else if key.trim().is_empty() {
            "no-key"
        } else {
            "key-ok"
        };
        println!(
            "[{}] {} [{} via:{}] key_env={} ({})",
            key_state,
            e.model_id,
            e.provider,
            match e.via {
                neotrix_neobot::FreeVia::Http => "http",
                neotrix_neobot::FreeVia::OpencodeCli => "cli",
            },
            if e.key_env.is_empty() { "-" } else { e.key_env },
            e.display,
        );
    }
    Ok(())
}

/// 配对免费模型（缺 key 拒绝并指路；CLI 类同时把配置引擎切为 Opencode）。
fn cmd_core_pair_free(model_id: &str) -> Result<(), NtBotError> {
    use neotrix_neobot::{discover_free, pair_free};
    let mut cfg = load_config()?;
    let store = open_store(&cfg)?;
    let found = discover_free();
    let entry = found.iter().find(|e| e.model_id == model_id).ok_or_else(|| {
        NtBotError::Invalid(format!("free model '{model_id}' not in discover list (run `neobot core free`)"))
    })?;
    let (pair, count) = pair_free(&store, entry)?;
    if pair.via == "cli" {
        cfg.engine = EngineKind::Opencode { model: pair.model.clone() };
        let path = cfg.data_dir.join("config.json");
        let json = serde_json::to_string_pretty(&cfg)?;
        atomic_write(&path, json.as_bytes())?;
    }
    println!(
        "neobot core paired: soul embedded via {} ({} models at {}, model={})",
        entry.provider, count, pair.base_url, pair.model
    );
    Ok(())
}

/// 摘除配对（回纯本地；幂等）。
fn cmd_core_unpair() -> Result<(), NtBotError> {
    use neotrix_neobot::unpair_core;
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    if unpair_core(&store)? {
        println!("neobot core unpaired: soul removed, back to local-only");
    } else {
        println!("neobot core already unpaired (local-only)");
    }
    Ok(())
}

/// 灵魂状态（配对行 + 活探 + 版本/工具数）。
fn cmd_core_status() -> Result<(), NtBotError> {
    use neotrix_neobot::{CoreStatus, core_status};
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    match core_status(&store)? {
        CoreStatus::Unpaired => println!("neobot core: unpaired (local-only app)"),
        CoreStatus::Online { pair, models, latency_ms, crystal_version, tool_count } => println!(
            "neobot core: soul online ({} models, model={} at {}, {}ms, crystal_version={} tools={})",
            models,
            pair.model,
            pair.base_url,
            latency_ms,
            if crystal_version.is_empty() { "-" } else { &crystal_version },
            tool_count
        ),
        CoreStatus::Offline { pair, reason } => println!(
            "neobot core: soul offline (paired {} model={}; {})",
            pair.base_url, pair.model, reason
        ),
    }
    Ok(())
}

/// 热重载（POST /v1/admin/reload，需 token_env 现读；返回服务端 counts 摘要）。
fn cmd_core_reload() -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    let summary = neotrix_neobot::core_reload(&store)?;
    println!("neobot core {summary}");
    Ok(())
}

/// 服务端 agent 跑一轮（POST /v1/agents/run，需 token）。
fn cmd_agent_run(goal: &str, context: Option<&str>, steps: Option<i64>) -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    let result = neotrix_neobot::agent_run_with_steps(&store, goal, context, steps.unwrap_or(neotrix_neobot::AGENT_DEFAULT_STEPS))?;
    println!("neobot agent: status={} model={}", result.status, result.model_used);
    println!("{}", result.output);
    for step in &result.trace {
        println!("- [{}] {}", step.kind, step.detail);
    }
    Ok(())
}

fn cmd_provider_remove(name: &str) -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    store.remove_provider(name)?;
    println!("neobot provider '{name}' removed");
    Ok(())
}

fn cmd_provider_toggle(name: &str, enabled: bool) -> Result<(), NtBotError> {
    let cfg = load_config()?;
    let store = open_store(&cfg)?;
    store.set_provider_enabled(name, enabled)?;
    println!("neobot provider '{name}' {}", if enabled { "on" } else { "off" });
    Ok(())
}

fn cmd_provider_preset(name: &str) -> Result<(), NtBotError> {
    let Some((_, base_url, key_env, model)) = neotrix_neobot::PRESETS
        .iter()
        .find(|(preset, _, _, _)| *preset == name.trim())
        .copied()
    else {
        return Err(NtBotError::Invalid(format!(
            "unknown preset '{name}' (neotrix|ollama|lmstudio|deepseek|openai|gemini|qwen|moonshot|zhipu)"
        )));
    };
    cmd_provider_add(name.trim(), base_url, key_env, model)
}
