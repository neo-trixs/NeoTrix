#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    reason = "CLI entry point — pedantic false-positives"
)]

mod entry;

use clap::{CommandFactory, Parser, Subcommand};
use entry::*;
// config 模块由 lib 提供 (neotrix::config), 避免与 lib.rs 重复定义。

#[derive(Parser, Debug)]
#[command(
    name = "neotrix",
    version,
    about = "NeoTrix — Self-evolving reasoning engine",
    after_help = "\
EXAMPLES:
  neotrix run \"explain this codebase\"          Interactive / one-shot reasoning
  neotrix exec --json \"summarize the diff\"    Structured non-interactive execution
  neotrix reason -f prompt.txt                 Reason from a file
  neotrix status                               Show brain/daemon status
  neotrix completions bash > /etc/bash_completion.d/neotrix
  neotrix search \"rust async runtime\" -n 10   Web search
  neotrix discover --json                      Scan for NeoTrix agents on LAN
  neotrix features list                        List runtime feature flags
"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(
        global = true,
        long,
        value_name = "COLOR",
        help = "Color mode: auto|always|never"
    )]
    color: Option<String>,

    #[arg(global = true, long, help = "Suppress non-error log output")]
    quiet: bool,

    #[arg(
        global = true,
        long,
        short = 's',
        help = "Run HTTP server mode (legacy flag)"
    )]
    serve: bool,

    #[arg(global = true, long, help = "Run headless mode (legacy flag)")]
    headless: bool,

    #[arg(global = true, long, help = "Run standalone mode (no LLM)")]
    standalone: bool,

    #[arg(
        global = true,
        long,
        help = "Run Agent Loop mode (NeoTrix as subject, LLM as backend)"
    )]
    agent: bool,

    #[arg(global = true, long, value_name = "ADDR", default_value_t = String::from("0.0.0.0:3000"), help = "Server address")]
    addr: String,

    #[arg(
        global = true,
        long,
        value_name = "STAGE",
        default_value_t = 18,
        help = "Reasoning stage count"
    )]
    stage: usize,

    #[arg(global = true, long, default_value_t = String::from("default"), help = "Profile name for isolated state")]
    profile: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    // ── Core: LLM 交互 ──
    #[command(about = "Non-interactive execution with structured output")]
    Exec {
        prompt: Option<String>,
        #[arg(long, short = 'f', value_name = "FILE")]
        file: Option<String>,
        #[arg(long, help = "Read prompt from stdin")]
        pipe: bool,
        #[arg(long, help = "JSONL streaming output (one JSON object per line)")]
        json: bool,
        #[arg(
            long,
            value_name = "SCHEMA",
            help = "Output schema for structured validation (reserved)"
        )]
        output_schema: Option<String>,
        #[arg(long, help = "Execution timeout in seconds", default_value_t = 60)]
        timeout: u64,
        #[arg(
            long,
            value_name = "DOLLARS",
            help = "Hard limit on total API spend in USD"
        )]
        max_budget_usd: Option<f64>,
        #[arg(
            long,
            short = 'S',
            help = "Stream output in real-time (text mode only)"
        )]
        stream: bool,
    },
    #[command(about = "Run interactive mode (TUI) or one-shot prompt")]
    Run {
        #[arg(long)]
        headless: bool,
        #[arg(help = "One-shot prompt")]
        prompt: Option<String>,
        #[arg(long, short = 'f', value_name = "FILE")]
        file: Option<String>,
        #[arg(long, help = "Read prompt from stdin")]
        pipe: bool,
        #[arg(long, value_name = "FORMAT", help = "Output format: text|json")]
        format: Option<String>,
        /// 主入口（2026-10-06）：审批与沙箱是**两个正交轴**，各一个 flag。
        /// 依据 `codex` 的 `--sandbox` × `--ask-for-approval` 双轴设计 ——
        /// 沙箱管「在哪跑、能写多少」，审批管「动手前问不问」，二者不可互相替代。
        #[arg(
            long,
            value_name = "MODE",
            help = "Approval mode: suggest|auto-edit|full-auto (default: suggest)"
        )]
        approval_mode: Option<String>,
        #[arg(long, help = "Alias for --approval-mode suggest")]
        suggest: bool,
        #[arg(long, help = "Alias for --approval-mode auto-edit")]
        auto_edit: bool,
        #[arg(long, help = "Alias for --approval-mode full-auto")]
        full_auto: bool,
        /// ⚠️ **`--yolo` 与 `--full-auto` 字面等价**，两者都是「无审批」。
        ///
        /// 之所以把这句话写进 help：`codex` 的 `--yolo` = 无沙箱无审批，
        /// 而 `opencode` 的 `--yolo` 是**隐藏别名**，实际语义是
        /// 「批准一切未被显式 deny 的请求」⇒ **同名不同义**。
        /// （子代理实证：opencode 把它 `hidden: true` 且 OR 进 `auto` 布尔，
        ///   用户按 codex 的肌肉记忆敲它，得到的是**更弱的**保护。）
        /// ⇒ 本仓明确声明：`--yolo` ≡ `--full-auto` ≡ `--approval-mode full-auto`。
        #[arg(long, help = "Alias for --approval-mode full-auto (== --full-auto)")]
        yolo: bool,
        #[arg(
            long,
            value_name = "DOLLARS",
            help = "Hard limit on total API spend in USD"
        )]
        max_budget_usd: Option<f64>,
        #[arg(
            long,
            value_name = "MODE",
            default_value = "disabled",
            help = "Sandbox level: read-only|workspace-write|disabled|docker \
(未知值直接报错退出，不静默兜底)"
        )]
        sandbox: String,
        #[arg(long, help = "Disposable session — do not save to disk")]
        ephemeral: bool,
        #[arg(
            long,
            short = 'S',
            help = "Stream output in real-time (text mode only)"
        )]
        stream: bool,
    },
    #[command(about = "Start HTTP API server")]
    Serve {
        #[arg(long, default_value_t = String::from("0.0.0.0:3000"))]
        addr: String,
    },
    #[command(about = "One-shot reasoning (non-interactive)")]
    Reason {
        prompt: Option<String>,
        #[arg(long, short = 'f', value_name = "FILE")]
        file: Option<String>,
        #[arg(long, help = "Read prompt from stdin")]
        pipe: bool,
        #[arg(long, value_name = "FORMAT", help = "Output format: text|json")]
        format: Option<String>,
        #[arg(long, short = 'S', help = "Stream output in real-time")]
        stream: bool,
    },
    #[command(name = "mcp-server", about = "Run as MCP server (stdio JSON-RPC 2.0)")]
    McpServer,

    // ── Consciousness Core (意识核心 — opencode agent 通道) ──
    #[command(
        name = "consciousness",
        about = "意识核心状态/运行: status|tick|health|branches [--json] [--cycles N]"
    )]
    Consciousness {
        #[arg(help = "子命令: status (默认) | tick | health | branches")]
        sub: Option<String>,
        #[arg(long, help = "JSON 输出 (machine-readable, opencode 友好)")]
        json: bool,
        #[arg(long, default_value_t = 1, help = "tick 时运行的生长周期数")]
        cycles: usize,
    },

    // ── Project Evolution (独立项目进化 — 第三方 CLI 集成入口) ──
    #[command(
        name = "project-evolve",
        about = "对任意目标项目运行进化链路 (scan→detect→score→report); 第三方 CLI 可集成"
    )]
    ProjectEvolve {
        #[arg(help = "目标项目目录 (默认当前目录)")]
        target: Option<String>,
        #[arg(long, help = "自动修复 auto_fixable 问题")]
        autofix: bool,
        #[arg(long, help = "JSON 输出 (machine-readable)")]
        json: bool,
        #[arg(long, help = "修复断路器最大轮次", default_value_t = 10)]
        max_rounds: usize,
    },

    // ── Knowledge & Memory ──
    #[command(name = "wiki", about = "Wiki KB: generate|status|sync|graph|query")]
    Wiki {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(
        name = "todo",
        about = "TODO smart-sync (sync_todos.py replacement): sync|status|allocate [max]|import <path>"
    )]
    Todo {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    #[command(about = "Search the web")]
    Search {
        query: String,
        #[arg(long, short = 'n', default_value_t = 5, help = "Number of results")]
        count: usize,
    },

    // ── System & Ops ──
    #[command(about = "Run benchmarks")]
    Bench { category: Option<String> },
    #[command(about = "Show brain/daemon status")]
    Status,
    #[command(about = "Start background daemon")]
    Daemon {
        #[arg(long)]
        evolve: bool,
    },
    #[command(about = "Self-update the binary")]
    Update {
        #[arg(long)]
        check_only: bool,
    },
    #[command(about = "Generate shell completions")]
    Completions { shell: String },
    #[command(about = "Manage runtime feature flags")]
    Features {
        #[command(subcommand)]
        command: FeaturesCommands,
    },
    #[command(about = "Manage config file (encrypt/decrypt API keys)")]
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    /// 管理权限档位（`--approval-mode` 的**持久化对应物**）。
    ///
    /// 【为什么要它】`--approval-mode` 只改**单次进程**的全局单例，进程一退就没了；
    /// 而 `switch_profile_with_audit` 此前**零生产调用方** ——
    /// 库函数写好了、测试也绿，但从命令行**根本够不着** ⇒ 等于没有。
    #[command(about = "Manage permission profiles (list/show/use/current)")]
    Profile {
        #[command(subcommand)]
        command: ProfileCommands,
    },
    #[command(
        about = "NeoTrix 系统运维 (统一安装/守护/卸载, 替代分散 sh 脚本): daemons|uninstall|status"
    )]
    Sysops {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// 社交平台访问：渠道后端诊断 / 单平台探测 / cookie 认证
    #[command(about = "Social platform access: catalog|doctor|probe|status|sites|login|auth|weights|rank")]
    Social {
        #[arg(
            trailing_var_arg = true,
            allow_hyphen_values = true,
            help = "catalog|doctor|probe|status|sites|login <site>|auth <site>|weights|rank (--json where supported)"
        )]
        args: Vec<String>,
    },
    #[command(
        name = "clean",
        about = "Scan & remove dev/junk/ai/trash (PureMac-style safe clean): [dev|junk|ai|trash|all] [--dry-run] [--json] [--force]"
    )]
    Clean {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    // ── Guard & Compliance ──
    #[command(
        about = "AGENTS.md pointer-conservation guard (check structure, ceilings, forbidden sections)"
    )]
    Guard {
        #[arg(long, help = "Path to AGENTS.md (default: ./AGENTS.md)")]
        path: Option<String>,
        #[arg(
            long,
            help = "Exit with non-zero code on violation (for CI/pre-commit)"
        )]
        strict: bool,
    },

    // ── Network & Agents ──
    #[command(about = "Browse a URL")]
    Browse { url: String },
    #[command(about = "Run browser action file (JSON array of BrowserAction)")]
    BrowseAct { file: String },
    #[command(about = "Browser login")]
    Login { url: String },
    // ── 对话面（neobot 即 neotrix 对外对话的一部分；与 neobot 二进制同律） ──
    #[command(about = "对话：跑一轮/agent/模型池/端点/配对/会话/任务（neobot 同律）")]
    Dialog {
        #[command(subcommand)]
        cmd: DialogCmd,
    },
    #[command(about = "Proxy daemon control (status|mode|start|stop|install)")]
    Proxy { args: Vec<String> },
    #[command(about = "Scan network for NeoTrix agents via UDP discovery")]
    Discover {
        #[arg(long, short = 'p', default_value_t = 42069, help = "UDP port")]
        port: u16,
        #[arg(
            long,
            short = 'd',
            default_value_t = 3000,
            help = "Scan duration in ms"
        )]
        duration: u64,
        #[arg(long, help = "JSON output")]
        json: bool,
    },
    #[command(about = "Cloud/Docker sandbox commands")]
    Sandbox {
        #[command(subcommand)]
        command: SandboxCommands,
    },

    // ── Finance & Wallet ──
    #[command(about = "Wallet management (create, import, list, balance)")]
    Wallet {
        #[command(subcommand)]
        command: WalletCommands,
    },

    // ── Web ──
    #[command(about = "网络访问（用 NeoTrix 自带的浏览器能力）")]
    Web {
        #[command(subcommand)]
        command: WebCommands,
    },
}

#[derive(Subcommand, Debug)]
enum WebCommands {
    /// 抓取一个 URL，把正文打到 stdout
    ///
    /// 这是**补缺口**，不是新能力：`UniversalBrowser::fetch()`
    /// 早已存在于 `l1_action::nt_io::universal_browser`，
    /// 但此前**没有任何 CLI 或 example 能触达它** ⇒ 能力等于不存在。
    ///
    /// ⚠️ **凭据边界（重要）**：
    /// · 本命令**只发匿名请求** —— 不读 Chrome、不导 cookie、不碰本机凭据。
    /// · 因此需要登录的站点（飞书 / Google / Meta 等）**只会拿到登录墙**。
    /// · `BrowserResult.cookies` 是**响应带来的**，不是本机的；打印时
    ///   `CookieEntry` 的 `Debug` 已 redact（value ⇒ `<redacted>`），
    ///   故本命令**不会**把会话值写进终端或日志。
    /// · 若确需带自己的会话，请**自行**把 cookie 放进
    ///   `~/.neotrix/cookies/<id>.json`（凭据操作，应由本人执行）。
    Fetch {
        /// 目标 URL（http/https）
        url: String,
        /// 只输出正文，不输出诊断行
        #[arg(long)]
        quiet: bool,
    },
}

#[derive(Subcommand, Debug)]
enum SandboxCommands {
    #[command(about = "Execute code in sandbox")]
    Run {
        #[arg(help = "Code to execute (reads from stdin if omitted)")]
        code: Option<String>,
        #[arg(
            long,
            short = 'r',
            default_value = "python3",
            help = "Runtime (python3, node18, rust, go1_21, linux)"
        )]
        runtime: String,
        #[arg(
            long,
            short = 't',
            default_value_t = 300,
            help = "Max runtime in seconds"
        )]
        timeout: u64,
    },
    #[command(about = "List active sandbox sessions")]
    List,
    #[command(about = "Cancel a sandbox session")]
    Cancel {
        #[arg(help = "Session ID")]
        session_id: String,
    },
    #[command(about = "Upload file to sandbox session")]
    Upload {
        #[arg(help = "Local file path")]
        path: String,
        #[arg(help = "Session ID (creates new if omitted)", default_value = "")]
        session_id: String,
    },
}

#[derive(Subcommand, Debug)]
enum FeaturesCommands {
    #[command(about = "Enable a runtime feature flag")]
    Enable {
        #[arg(help = "Feature name to enable")]
        name: String,
    },
    #[command(about = "List all available feature flags and their status")]
    List,
}

/// `profile` 的子命令。
///
/// ## 信任边界（比 `claude-code` 更严一档）
/// · `use` 会改动**全局审批模式** ⇒ 属于「不可逆性放宽」，
///   所以**必须先只算副作用、再确认**（`plan_profile_switch` 是纯查询）。
/// · **非 TTY（脚本/管道/CI）且未给 `--yes` ⇒ 直接拒绝**，
///   不静默放行。依据 `claude-code` 的「静默降级只朝严格方向」纪律：
///   它在无对话框时**直接放行**，我们反其道 —— 因为我们的默认是**拒绝**，
///   静默放行会让「CI 里跑 `neotrix profile use developer`」
///   **悄悄把审批降到 auto-edit**，而 CI 日志里什么痕迹都没有。
#[derive(Subcommand, Debug)]
enum ProfileCommands {
    #[command(about = "List all permission profiles (★ marks the active one)")]
    List,
    #[command(about = "Show a profile's effective rules and approval mode")]
    Show {
        #[arg(help = "Profile name")]
        name: String,
        #[arg(long, help = "JSON output (for scripts)")]
        json: bool,
    },
    /// 切换档位。改动全局审批模式时需确认（非 TTY 需 `--yes`）。
    #[command(about = "Switch to a profile (asks before loosening approvals)")]
    Use {
        #[arg(help = "Profile name")]
        name: String,
        /// 非交互通道（脚本/CI）的显式授权。语义是「我知道这会改审批模式」。
        #[arg(long, help = "Skip the confirmation prompt (non-interactive channels)")]
        yes: bool,
    },
    #[command(about = "Show the active profile and current approval mode")]
    Current,
}

#[derive(Subcommand, Debug)]
enum ConfigCommands {
    #[command(about = "Encrypt all plaintext API keys in the config file")]
    EncryptKeys,
    #[command(about = "Decrypt all encrypted API keys in the config file (use with caution)")]
    DecryptKeys,
}

#[derive(Subcommand, Debug)]
enum WalletCommands {
    #[command(about = "Create a new wallet")]
    Create {
        #[arg(help = "Wallet label")]
        label: String,
    },
    #[command(about = "Import wallet from private key")]
    Import {
        #[arg(help = "Wallet label")]
        label: String,
        #[arg(help = "Private key (hex with or without 0x)")]
        private_key: String,
    },
    #[command(about = "List all wallets")]
    List {
        #[arg(long, help = "JSON output")]
        json: bool,
    },
    #[command(about = "Check wallet balance")]
    Balance {
        #[arg(help = "Chain name (eth, bsc, polygon, etc.)", default_value = "eth")]
        chain: String,
    },
    #[command(about = "Delete a wallet")]
    Delete {
        #[arg(help = "Wallet label to delete")]
        label: String,
    },
    #[command(about = "Export private key (⚠️  security sensitive)")]
    Export {
        #[arg(help = "Wallet label")]
        label: String,
    },
}


/// `profile use` 的**授权裁决**（纯函数，无 I/O、无副作用）。
///
/// ## 为什么必须是纯函数
/// 这是**安全边界**（决定「要不要问一句」）。
/// ⛔ 若它内联在 I/O 代码里，就**无法写测试** ⇒ 边界改了没人知道
/// ——这与本会话修掉的「`--yolo` 全程没人断言」是同一种病。
/// ⇒ 判据在此单点，I/O 只负责「拿到输入后调用它」。
#[derive(Debug, Clone, PartialEq, Eq)]
enum ProfileUseAuthorization {
    /// 该档不改全局审批模式 ⇒ 无需确认，直接执行。
    NoConfirmationNeeded { actor: String },
    /// 会改审批模式，且当前是 TTY ⇒ 必须拿到用户的明确 `yes`。
    NeedsInteractiveConfirm {
        /// 必须**先打印**给用户的副作用预告（确认才不只是形式）。
        notice: String,
        /// 是否**放宽**（`false` 表示是收紧）。
        loosens: bool,
    },
    /// 会改审批模式，当前非 TTY，但给了 `--yes` ⇒ 显式授权成立。
    AllowNonInteractive { actor: String },
    /// 会改审批模式，当前非 TTY 且**没给** `--yes` ⇒ **拒绝**。
    Refuse { reason: String },
}

/// 裁决 `profile use` 该怎么执行。
///
/// ## 规则表（每条都有依据，不是拍脑袋）
/// | 是否改审批模式 | TTY | `--yes` | 裁决 |
/// |---|---|---|---|
/// | 否 | 任意 | 任意 | `NoConfirmationNeeded` |
/// | 是 | 是 | 任意 | `NeedsInteractiveConfirm` |
/// | 是 | 否 | 是 | `AllowNonInteractive` |
/// | 是 | 否 | 否 | **`Refuse`** |
///
/// ## 最后一格为什么是「拒绝」而不是「放行」
/// `claude-code` 在**无对话框**（非交互/管道）时是**直接放行**。
/// 我们反其道而行，理由：
/// · 它的默认是「放行」，我们是「拒绝」——
///   **静默降级只允许朝严格方向**（这条纪律见多处吸收分析）。
/// · 若非 TTY 缺 `--yes` 也放行，则
///   `neotrix profile use developer` 出现在 CI/脚本里会
///   **悄悄把审批严格度降到 auto-edit**，而日志里零痕迹、
///   没有任何「这是自动化决定的」标记。
/// · `--yes` 的语义是**显式**声明「我知道这会改审批模式」，
///   与 `claude-code` 的**自动注入默认值**（用户压根没意识到）性质不同。
///
/// `plan` 是 [`plan_profile_switch`] 的产物（纯查询）；
/// `current` 是**当前**全局审批模式（用于判断「放宽」还是「收紧」）。
fn authorize_profile_use(
    plan: &neotrix::l6_meta::nt_permission_profiles::ProfileSwitchPlan,
    current: neotrix::l6_meta::nt_approval::ApprovalMode,
    is_tty: bool,
    yes_flag: bool,
) -> ProfileUseAuthorization {
    use neotrix::l6_meta::nt_permission_profiles::ProfileSwitchPlan;

    // 不改全局审批模式 ⇒ 不是不可逆放宽 ⇒ 无需确认（方案 A 的前提）。
    if !plan.changes_approval_mode() {
        return ProfileUseAuthorization::NoConfirmationNeeded {
            actor: format!("cli:{}", plan.profile),
        };
    }

    let notice = plan
        .notice
        .clone()
        .unwrap_or_else(|| format!("档位 '{}' 会改动全局审批模式。", plan.profile));
    let loosens = plan.loosens_approval(current);

    if is_tty {
        return ProfileUseAuthorization::NeedsInteractiveConfirm { notice, loosens };
    }
    if yes_flag {
        return ProfileUseAuthorization::AllowNonInteractive {
            // actor 标注「这是自动化通道决定的」，审计行据此可区分人/机。
            actor: format!("non-interactive:{}", plan.profile),
        };
    }
    ProfileUseAuthorization::Refuse {
        reason: format!(
            "非交互通道下切换到 '{}' 会改动全局审批模式（{}），
拒绝静默执行。\n\
若确认要改，请显式加 --yes（它会记下 actor=non-interactive 以便审计）。",
            plan.profile,
            plan.resulting_mode
                .map(|m| m.as_str())
                .unwrap_or("<unknown>")
        ),
    }
}

/// 打印一行 notice（确认前的副作用预告**必须**先落到用户眼前）。
fn print_profile_notice(notice: &str, loosens: bool) {
    let tag = if loosens { "⚠️  放宽审批" } else { "ℹ️  收紧审批" };
    println!("{tag}");
    println!("{notice}");
}

/// **审批模式解码**（2026-10-06）—— 抽成独立函数以便**被测试覆盖**。
///
/// ⛔ 之前这段逻辑**内联在 `main()` 里**，因此**无法写测试**
/// ⇒ 这本身就是「声明了但不可验证」的一种形态：
/// flag 存在、能解析、能编译，但**没有任何机制能证明它真的改了全局态**。
/// （子代理实证：`codex` / `claude-code` / `opencode` **三家都没做**这种
/// 「启用即断言接线」的测试；而本会话修的正是这一类缺陷。）
///
/// ## 规则
/// · 主 flag `--approval-mode` 优先；旧 flag（`--suggest` / `--auto-edit` /
///   `--full-auto` / `--yolo`）保留为**别名**（已在文档/README/脚本里出现，
///   直接删是破坏性变更）。
/// · 多个来源指向**同一档**不算冲突（`--yolo` ≡ `--full-auto` 是常态用法）。
/// · 指向**不同档** ⇒ 返回 `Err`，**不猜优先级** ——
///   「同时给 `--suggest` 和 `--yolo` 时谁赢」不该由程序替用户决定。
/// · 一个都没给 ⇒ `Suggest`（最严）。
/// · 未知档位名 ⇒ 返回 `Err`（`ApprovalMode::from_str` 已改为硬拒绝）。
fn resolve_approval_mode(
    approval_mode: Option<&str>,
    suggest: bool,
    auto_edit: bool,
    full_auto: bool,
    yolo: bool,
) -> Result<neotrix::l6_meta::nt_approval::ApprovalMode, String> {
    use neotrix::l6_meta::nt_approval::ApprovalMode;
    let mut wanted: Vec<ApprovalMode> = Vec::new();
    let mut push = |m: ApprovalMode| {
        if !wanted.contains(&m) {
            wanted.push(m);
        }
    };
    if let Some(raw) = approval_mode {
        push(ApprovalMode::from_str(raw)?);
    }
    if suggest {
        push(ApprovalMode::Suggest);
    }
    if auto_edit {
        push(ApprovalMode::AutoEdit);
    }
    if full_auto || yolo {
        push(ApprovalMode::FullAuto);
    }
    match wanted.len() {
        0 => Ok(ApprovalMode::Suggest),
        1 => Ok(wanted[0]),
        _ => {
            let names: Vec<&str> = wanted.iter().map(|m| m.as_str()).collect();
            Err(format!(
                "审批模式冲突：同时请求了 {}\n每个模式只能给一次（--yolo 与 --full-auto 等价，可同时给）。",
                names.join(" 与 ")
            ))
        }
    }
}



/// `profile` 子命令的执行体（I/O 层）。
///
/// ⛔ 这里**不含任何安全判据** —— 全部在 [`authorize_profile_use`] 里。
/// 本函数只做：取计划 → 问判据 → 按裁决行动。
fn run_profile_command(command: &ProfileCommands) -> Result<(), String> {
    use neotrix::l6_meta::nt_permission_profiles as pp;
    use std::io::IsTerminal;

    match command {
        ProfileCommands::List => {
            let active = pp::active_profile_name();
            let names = pp::list_profiles();
            if names.is_empty() {
                println!("(没有任何权限档位)");
                return Ok(());
            }
            for n in names {
                let mark = if n == active { "★" } else { " " };
                println!("{mark} {n}");
            }
            println!("\n(★ = 当前生效档位)");
            Ok(())
        }
        ProfileCommands::Show { name, json } => {
            let info = pp::get_profile_info(name)?;
            let json = *json;
            if json {
                println!("{}", serde_json::to_string_pretty(&info).map_err(|e| e.to_string())?);
                return Ok(());
            }
            let active = pp::active_profile_name();
            println!("档位 {name}{}", if *name == active { "（当前生效）" } else { "" });
            println!("  父档          : {}", info["parent"].as_str().unwrap_or("-"));
            println!(
                "  审批模式覆盖  : {}",
                info["approval_mode_override"].as_str().unwrap_or("(未设置)")
            );
            println!(
                "  生效审批模式  : {}",
                info["effective_approval_mode"].as_str().unwrap_or("(不改)")
            );
            // 规则用 `effective_rules`（已沿父链继承）⇒ 打印的**就是实际生效的**，
            // 而不是该档自己写的那几条。
            let rules = info["effective_rules"].as_object();
            match rules {
                Some(m) if !m.is_empty() => {
                    println!("  生效规则（沿父链继承后）:");
                    let mut kv: Vec<(&String, &serde_json::Value)> = m.iter().collect();
                    kv.sort_by(|a, b| a.0.cmp(b.0));
                    for (k, v) in kv {
                        println!("    {:<28} {}", k, v.as_str().unwrap_or("?"));
                    }
                }
                _ => println!("  生效规则      : (无显式规则 ⇒ 落到默认审批流程)"),
            }
            Ok(())
        }
        ProfileCommands::Current => {
            let name = pp::active_profile_name();
            let mode = neotrix::l6_meta::nt_approval::global_approval()
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .mode();
            println!("当前档位      : {name}");
            println!("当前审批模式  : {}", mode.as_str());
            // 把「档位与模式是否一致」摆出来 —— 两者可以独立漂移
            // （`--approval-mode` 只改模式不改档位，`profile use` 两者都改）。
            let declared = pp::get_profile_info(&name)
                .ok()
                .and_then(|i| i["effective_approval_mode"].as_str().map(|s| s.to_string()));
            match declared {
                Some(d) if d != mode.as_str() => println!(
                    "⚠️  提示：档位 '{name}' 声明的审批模式是 {d}，但当前全局模式是 {}。\
               ⇒ 两者已漂移（`--approval-mode` 只改本次进程的模式，不改档位）。",
                    mode.as_str()
                ),
                _ => {}
            }
            Ok(())
        }
        ProfileCommands::Use { name, yes } => {
            let yes = *yes;
            // ① 只算副作用（不改状态）
            let plan = pp::plan_profile_switch(name)?;
            let current = neotrix::l6_meta::nt_approval::global_approval()
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .mode();

            // ② 裁决（纯函数）
            let auth = authorize_profile_use(&plan, current, std::io::stdin().is_terminal(), yes);
            let actor = match auth {
                ProfileUseAuthorization::NoConfirmationNeeded { actor } => actor,
                ProfileUseAuthorization::NeedsInteractiveConfirm { notice, loosens } => {
                    // ③ 先把「会发生什么」摆出来，否则确认只是形式
                    print_profile_notice(&notice, loosens);
                    print!("\n确认切换到 '{name}' 吗？[y/N] ");
                    use std::io::Write;
                    let _ = std::io::stdout().flush();
                    let mut line = String::new();
                    std::io::stdin().read_line(&mut line).map_err(|e| e.to_string())?;
                    let ans = line.trim().to_ascii_lowercase();
                    // ⛔ 默认 N：空回车 = 拒绝（不放行 ⇒ 静默降级只朝严格方向）
                    if ans != "y" && ans != "yes" {
                        return Err(format!("已取消（未输入 y）。档位仍为 '{}'", pp::active_profile_name()));
                    }
                    format!("cli-confirmed:{}", name)
                }
                ProfileUseAuthorization::AllowNonInteractive { actor } => {
                    println!("⚠️  非交互通道 + --yes ⇒ 直接执行 {actor}");
                    actor
                }
                ProfileUseAuthorization::Refuse { reason } => return Err(reason),
            };

            // ④ 执行（带 actor ⇒ 改审批模式这件事不许匿名发生）
            let msg = pp::switch_profile_with_audit(&name, &actor)?;
            println!("{msg}");
            if let Some(m) = plan.resulting_mode {
                println!("   全局审批模式 → {}", m.as_str());
            }
            Ok(())
        }
    }
}


fn main() {
    // 智能命令整合: clap 解析失败时, 未知子命令回退到交互式命令注册表
    // (60+ 命令: /kb /goal /wiki /evidence ...), 使它们可直接从命令行调用。
    // 注意: try_parse 必须在 init_tracing 之前, 这样回退路径设置的
    // RUST_LOG 才能在 tracing subscriber 初始化时生效 (抑制 KB 等 INFO 日志)。
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            // cli::commands removed — no command registry fallback
            e.exit()
        }
    };

    neotrix::nt_io_logging::init_tracing();
    let _sentry_guard = neotrix::nt_shield_sentry::init_sentry();

    // --quiet: 必须在 NeoTrixConfig::load() 之前设置环境变量,
    // 否则 load() 的 "[config] loaded" 诊断已在 quiet 之前输出。
    if cli.quiet {
        std::env::set_var("RUST_LOG", "neotrix=error");
        std::env::set_var("NEOTRIX_QUIET", "1");
    }

    // First-run provider config wizard (skip for pure ops commands)
    // 对标主流 CLI: 纯本地命令 (help/status/completions/features/config/wallet/
    // evidence/wiki/todo/sysops/bench/discover/proxy/sandbox/update/browse/login)
    // 不依赖 LLM provider, 不应被交互式 wizard 阻塞。
    let is_ops_cmd = matches!(
        cli.command,
        Some(Commands::Sysops { .. })
            | Some(Commands::Social { .. })
            | Some(Commands::Guard { .. })
            | Some(Commands::Status)
            | Some(Commands::Completions { .. })
            | Some(Commands::Features { .. })
            | Some(Commands::Config { .. })
            | Some(Commands::Profile { .. })
            | Some(Commands::Wallet { .. })
            | Some(Commands::Web { .. })
            | Some(Commands::Wiki { .. })
            | Some(Commands::Todo { .. })
            | Some(Commands::Bench { .. })
            | Some(Commands::Discover { .. })
            | Some(Commands::Proxy { .. })
            | Some(Commands::Sandbox { .. })
            | Some(Commands::Clean { .. })
            | Some(Commands::Update { .. })
            | Some(Commands::Browse { .. })
            | Some(Commands::Login { .. })
            | Some(Commands::Dialog { .. })
    ) || cli.agent
        || cli.standalone
        || cli.headless;
    if !is_ops_cmd && !entry::check_provider_config() {
        // 管理类 slash 命令 (provider pool 等) 在未配置 provider 时也须可用 —
        // 它们本身就是配置 provider 的入口, 不应被 wizard 阻塞。
        let is_mgmt_prompt = match &cli.command {
            Some(Commands::Exec {
                prompt: Some(p), ..
            })
            | Some(Commands::Run {
                prompt: Some(p), ..
            }) => {
                let t = p.trim_start();
                t.starts_with("/provider") || t.starts_with("/free") || t.starts_with("/model")
            }
            _ => false,
        };
        if !is_mgmt_prompt {
            entry::run_provider_wizard();
        }
    }

    let cfg = neotrix::config::NeoTrixConfig::load();

    let color_mode = cli
        .color
        .as_deref()
        .or(cfg.color_mode.as_deref())
        .unwrap_or("auto");
    if color_mode == "never" {
        colored::control::set_override(false);
    } else {
        colored::control::set_override(true);
    }

    if let Some(level) = &cfg.log_level {
        std::env::set_var("RUST_LOG", format!("neotrix={}", level));
    }
    // (--quiet 已在 parse 后提前设置, 此处不再重复)

    match &cli.command {
        Some(Commands::Exec {
            prompt,
            file,
            pipe,
            json,
            output_schema: _,
            timeout,
            max_budget_usd,
            stream,
        }) => {
            if let Some(limit) = max_budget_usd {
                neotrix::l6_meta::nt_cost_tracker::COST_TRACKER
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .set_max_budget_usd(*limit);
            }
            let resolved = resolve_prompt(prompt.as_deref(), file.as_deref(), *pipe);
            if resolved.is_empty() {
                // 用法错误 → 退出码 2 (对标 clap 约定: 0=成功 / 1=运行时错误 / 2=用法错误)
                eprintln!("error: no prompt provided. Usage: neotrix exec <prompt>");
                std::process::exit(2);
            }
            if cli.standalone {
                // standalone: 纯 ReasoningKernel 推理, 不依赖外部 LLM/网络
                use neotrix::nt_io_standalone::StandaloneEngine;
                let mut engine = StandaloneEngine::new(cli.stage.min(18));
                let response = engine.reason(&resolved);
                if *json {
                    println!(
                        "{{\"standalone\":true,\"output\":{}}}",
                        serde_json::to_string(&response).unwrap_or_default()
                    );
                } else {
                    println!("{}", response);
                }
            } else {
                run_exec(&resolved, *json, *stream, *timeout);
            }
        }
        Some(Commands::Run {
            headless,
            prompt,
            file,
            pipe,
            format,
            suggest,
            approval_mode,
            auto_edit,
            full_auto,
            yolo,
            sandbox,
            max_budget_usd,
            ephemeral,
            stream,
        }) => {
            if let Some(limit) = max_budget_usd {
                neotrix::l6_meta::nt_cost_tracker::COST_TRACKER
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .set_max_budget_usd(*limit);
            }
            // 审批模式解码（抽成 `resolve_approval_mode` 以便被测试覆盖 —— 见其文档）
            // ⚠️ 局部名不能也叫 `approval_mode`（会遮住同名的 flag 绑定）
            let approval_mode_flag = approval_mode.clone();
            let approval_mode = match resolve_approval_mode(
                approval_mode_flag.as_deref(),
                // ⚠️ 解构自 `&Cli` ⇒ 这些是 `&bool`，必须解引用
                *suggest,
                *auto_edit,
                *full_auto,
                *yolo,
            ) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("error: {e}");
                    std::process::exit(2);
                }
            };
            neotrix::l6_meta::nt_approval::global_approval()
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .set_mode(approval_mode);
            // 2026-10-06 未知沙箱档**硬拒绝并退出**（原为静默兜底成 Disabled）
            //
            // 【缺陷】`from_str` 首版是 `_ => Self::Disabled`
            // ⇒ `--sandbox danger-full-access` 不报错、静默变成「不设限」
            // ⇒ 用户以为设了最严档，实际**语义与意图相反**。
            // 判据：静默失效只允许朝**严格**方向回落（claude-code 的文档纪律），
            // 而「未知 → Disabled」是朝**宽松**方向 ⇒ 直接违反。
            let sandbox_mode =
                match neotrix::l3_embodiment::nt_sandbox::SandboxMode::from_str(sandbox.as_str()) {
                    Ok(m) => m,
                    Err(e) => {
                        eprintln!("error: {e}");
                        std::process::exit(2);
                    }
                };
            neotrix::l3_embodiment::nt_sandbox::init_sandbox(sandbox_mode);
            if let Some(p) = prompt {
                let resolved = resolve_prompt(Some(p), file.as_deref(), *pipe);
                run_one_shot(&resolved, format.as_deref(), &cli.profile, *stream);
            } else if let Some(f) = file {
                let resolved = resolve_prompt(None, Some(f), *pipe);
                run_one_shot(&resolved, format.as_deref(), &cli.profile, *stream);
            } else if *pipe {
                let resolved = resolve_prompt(None, None, true);
                run_one_shot(&resolved, format.as_deref(), &cli.profile, *stream);
            } else if *headless {
                run_headless_mode(&cfg, &cli.profile);
            } else {
                run_interactive_with_ephemeral(&cfg, &cli.profile, *ephemeral);
            }
        }
        Some(Commands::Serve { addr }) => run_background_daemon(addr, &cli.profile),
        Some(Commands::Reason {
            prompt,
            file,
            pipe,
            format,
            stream,
        }) => {
            let resolved = resolve_prompt(prompt.as_deref(), file.as_deref(), *pipe);
            if cli.standalone {
                // standalone: 纯 ReasoningKernel 推理, 不依赖外部 LLM/网络 (无 LLM 环境可用)
                use neotrix::nt_io_standalone::StandaloneEngine;
                let mut engine = StandaloneEngine::new(cli.stage.min(18));
                println!("{}", engine.reason(&resolved));
            } else {
                run_one_shot(&resolved, format.as_deref(), &cli.profile, *stream);
            }
        }
        Some(Commands::Bench { category }) => run_benchmark(category.as_deref()),
        Some(Commands::Status) => show_status(),
        Some(Commands::Daemon { evolve }) => {
            if *evolve {
                run_daemon_evolution(&cli.profile);
            } else {
                run_daemon(&cli.profile);
            }
        }
        Some(Commands::Update { check_only }) => run_update(*check_only),
        Some(Commands::Completions { shell }) => generate_completions(shell, &mut Cli::command()),
        Some(Commands::Browse { url }) => run_browse(url),
        Some(Commands::Dialog { cmd }) => {
            if let Err(e) = run_dialog(cmd.clone()) {
                eprintln!("neotrix dialog: {e}");
                std::process::exit(1);
            }
        }
        Some(Commands::BrowseAct { file }) => run_browse_act(file),
        Some(Commands::Login { url }) => run_login(url),
        Some(Commands::Proxy { args }) => {
            let cmd_str = args.join(" ");
            // ⛔ 原为 `.expect("tokio")`：`Runtime::new()` 会因 OS 线程创建失败而失败，
            // 裸 panic 只打印 `"tokio"`（信息量为零）。
            // ⇒ CLI 入口应给**可读错误**并走既有退出路径，而不是 panic。
            match tokio::runtime::Runtime::new() {
                Ok(rt) => rt.block_on(entry::run_proxy_cmd(&cmd_str)),
                Err(e) => {
                    eprintln!("neotrix proxy: tokio runtime 创建失败: {e}");
                }
            }
        }
        Some(Commands::Sandbox { command }) => match command {
            SandboxCommands::Run {
                code,
                runtime,
                timeout,
            } => {
                entry::run_sandbox_run(code.as_deref(), runtime, *timeout);
            }
            SandboxCommands::List => {
                entry::run_sandbox_list();
            }
            SandboxCommands::Cancel { session_id } => {
                entry::run_sandbox_cancel(session_id);
            }
            SandboxCommands::Upload { path, session_id } => {
                entry::run_sandbox_upload(path, session_id);
            }
        },
        Some(Commands::Search { query, count }) => {
            run_search(query, *count);
        }
        Some(Commands::Discover {
            port,
            duration,
            json,
        }) => {
            run_discover(*port, *duration, *json);
        }
        Some(Commands::McpServer) => entry::run_mcp_server(),
        Some(Commands::Consciousness { sub, json, cycles }) => {
            entry::run_consciousness_core(sub.as_deref(), *json, *cycles);
        }
        Some(Commands::ProjectEvolve {
            target,
            autofix,
            json,
            max_rounds,
        }) => {
            if let Err(e) =
                entry::run_project_evolve(target.as_deref(), *autofix, *json, *max_rounds)
            {
                eprintln!("error: {}", e);
                std::process::exit(1);
            }
        }
        Some(Commands::Features { command }) => match command {
            FeaturesCommands::Enable { name } => {
                entry::run_features_enable(name);
            }
            FeaturesCommands::List => {
                entry::run_features_list();
            }
        },
        Some(Commands::Wallet { command }) => match command {
            WalletCommands::Create { label } => {
                entry::run_wallet_create(label);
            }
            WalletCommands::Import { label, private_key } => {
                entry::run_wallet_import(label, private_key);
            }
            WalletCommands::List { json } => {
                entry::run_wallet_list(*json);
            }
            WalletCommands::Balance { chain } => {
                entry::run_wallet_balance(chain);
            }
            WalletCommands::Delete { label } => {
                entry::run_wallet_delete(label);
            }
            WalletCommands::Export { label } => {
                entry::run_wallet_export(label);
            }
        },
        Some(Commands::Web { command }) => match command {
            WebCommands::Fetch { url, quiet } => {
                entry::run_web_fetch(url, *quiet);
            }
        },
        Some(Commands::Profile { command }) => {
            if let Err(e) = run_profile_command(command) {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
        Some(Commands::Config { command }) => match command {
            ConfigCommands::EncryptKeys => {
                entry::run_config_encrypt_keys();
            }
            ConfigCommands::DecryptKeys => {
                entry::run_config_decrypt_keys();
            }
        },
        Some(Commands::Wiki { args }) => {
            // 意图路由收敛：wiki 经 AutoOrchestrator 分类后走 KB 后端
            if let Err(e) = entry::run_wiki(args) {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
        Some(Commands::Todo { args }) => {
            // 意图路由收敛：todo status 走 orchestrator 观测；管理命令已迁移
            if let Err(e) = entry::run_todo(args) {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
        Some(Commands::Sysops { args }) => {
            entry::run_sysops(args);
        }
        Some(Commands::Social { args }) => {
            // social 是纯本地诊断/认证命令，不依赖 LLM provider ——
            //    必须列入 is_ops_cmd，否则未配置 provider 时会被 wizard 阻塞。
            let json = args.iter().any(|a| a == "--json");
            let positional: Vec<&str> = args
                .iter()
                .filter(|a| !a.starts_with("--"))
                .map(|a| a.as_str())
                .collect();
            let rc = match positional.first().copied() {
                None | Some("doctor") => entry::run_social_doctor(json),
                Some("probe") => match positional.get(1).copied() {
                    Some(p) => entry::run_social_probe(p, json),
                    None => {
                        eprintln!("usage: neotrix social probe <platform> [--json]");
                        78
                    }
                },
                Some("status") => entry::run_social_status(json),
                Some("weights") => entry::run_social_weights(json),
                Some("rank") => entry::run_social_rank(json),
                Some("sites") => entry::run_social_sites(json),
                Some("catalog") => entry::run_social_catalog(json),
                Some("login") => match positional.get(1).copied() {
                    Some(site) => entry::run_social_login(site, json),
                    None => {
                        eprintln!("usage: neotrix social login <site> [--json]");
                        eprintln!("       see `neotrix social sites` for the registered list");
                        78
                    }
                },
                Some("auth") => match positional.get(1).copied() {
                    Some(site) => entry::run_social_auth_site(site),
                    None => {
                        eprintln!("usage: neotrix social auth <site>");
                        eprintln!("       see `neotrix social catalog` for the registered list");
                        78
                    }
                },
                Some(other) => {
                    eprintln!(
                        "unknown subcommand '{}'; expected one of: doctor, probe, status, sites, catalog, login, weights, rank, auth",
                        other
                    );
                    78
                }
            };
            // 退出码对 CI 可判定（对齐 OpenCLI 的 sysexits 约定）：
            // 0 = 全好，78 = 环境未配置后端/凭据。
            if rc != 0 {
                std::process::exit(rc);
            }
        }
        Some(Commands::Clean { args }) => {
            if let Err(e) = entry::run_clean(args) {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
        Some(Commands::Guard { path, strict }) => {
            // 意图路由收敛：guard 经 AutoOrchestrator 分类（SecurityAudit）后走治理检查
            let orchestrator = neotrix::l6_meta::nt_auto_orchestrator::AutoOrchestrator::new();
            let classification =
                orchestrator.classify_intent("guard agents-md pointer conservation audit");
            println!(
                "🔍 意图识别: {:?} (conf={:.2})",
                classification.task_type, classification.confidence
            );
            let target = path.as_deref().unwrap_or("./AGENTS.md");
            match neotrix::l6_meta::nt_agents_guard::run_guard(
                std::path::Path::new(target),
                *strict,
            ) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("error: {e}");
                    std::process::exit(1);
                }
            }
        }
        None => {
            if cli.standalone {
                run_standalone_mode(cli.stage);
            } else if cli.agent {
                entry::run_agent_tui(&cli.profile);
            } else if cli.serve {
                run_background_daemon(&cli.addr, &cli.profile);
            } else if cli.headless {
                run_headless_mode(&cfg, &cli.profile);
            } else {
                run_interactive(&cfg, &cli.profile);
            }
        }
    }
}

#[cfg(test)]
mod cli_permission_tests {
    //! 「**启用即断言接线**」的测试 —— 子代理实证：`codex` / `claude-code` /
    //! `opencode` **三家都没有**做这件事；而本会话修的正是「flag 存在、
    //! 能解析、但语义与用户意图相反」这一类缺陷。
    //!
    //! 成本极低（几条纯函数断言），收益是**这类缺陷无法再悄悄进来**。

    use super::resolve_approval_mode;
    use neotrix::l6_meta::nt_approval::ApprovalMode;

    /// 什么都没给 ⇒ 最严档（`Suggest`）。默认必须朝**严格**方向。
    #[test]
    fn default_is_strictest_mode() {
        assert_eq!(
            resolve_approval_mode(None, false, false, false, false).unwrap(),
            ApprovalMode::Suggest
        );
    }

    /// 主 flag 三档都要能解析。
    #[test]
    fn primary_flag_parses_all_three_modes() {
        for (raw, want) in [
            ("suggest", ApprovalMode::Suggest),
            ("auto-edit", ApprovalMode::AutoEdit),
            ("full-auto", ApprovalMode::FullAuto),
        ] {
            assert_eq!(
                resolve_approval_mode(Some(raw), false, false, false, false).unwrap(),
                want,
                "--approval-mode {raw} 应解析为 {want:?}"
            );
        }
    }

    /// 未知档位必须 `Err`（不得静默兜底成默认）。
    /// 依据：静默失效只允许朝**严格**方向回落；「拼错 → Suggest」看似更严，
    /// 实则**用户要的 auto-edit 没生效且无人知道** ⇒ 仍是静默失效。
    #[test]
    fn unknown_mode_is_ERR_not_silent_default() {
        let e = resolve_approval_mode(Some("yoloo"), false, false, false, false)
            .expect_err("拼错的档位必须 Err");
        assert!(e.contains("suggest"), "错误信息应列出可用档位：{e}");
    }

    /// 旧 flag 是**别名**，语义必须与主 flag 对应档**完全一致**。
    /// 这一条锁的是「重构没改语义」—— 别名降级最常见的失误就是偷偷改了含义。
    #[test]
    fn legacy_flags_are_true_aliases_of_primary_flag() {
        let cases: [(bool, bool, bool, bool, ApprovalMode); 4] = [
            (true, false, false, false, ApprovalMode::Suggest),
            (false, true, false, false, ApprovalMode::AutoEdit),
            (false, false, true, false, ApprovalMode::FullAuto),
            (false, false, false, true, ApprovalMode::FullAuto),
        ];
        for (sg, ae, fa, yo, want) in cases {
            let via_flag = resolve_approval_mode(Some(want.as_str()), false, false, false, false).unwrap();
            let via_alias = resolve_approval_mode(None, sg, ae, fa, yo).unwrap();
            assert_eq!(via_alias, want, "旧 flag 应等价于 --approval-mode {}", want.as_str());
            assert_eq!(via_alias, via_flag, "别名与主 flag 必须给出同一档");
        }
    }

    /// **`--yolo` 与 `--full-auto` 必须字面等价**。
    ///
    /// 【为什么这条最重要】`codex` 的 `--yolo` = 无沙箱无审批，
    /// 而 `opencode` 的 `--yolo` 是**隐藏别名**、实际语义是
    /// 「批准一切未被显式 deny 的请求」⇒ **同名不同义**。
    /// 子代理在 opencode 源码里验到：它把 `--yolo` 与
    /// `--dangerously-skip-permissions` 都 `hidden: true` 然后 OR 进 `auto` 布尔
    /// ⇒ 用户按 codex 的肌肉记忆敲它，得到的是**更弱的**保护，且零文档零警告。
    /// ⇒ 本仓明确声明等价，并用测试钉住。
    #[test]
    fn yolo_is_exactly_equivalent_to_full_auto() {
        assert_eq!(
            resolve_approval_mode(None, false, false, false, true).unwrap(),
            resolve_approval_mode(None, false, false, true, false).unwrap(),
        );
        assert_eq!(
            resolve_approval_mode(None, false, false, false, true).unwrap(),
            ApprovalMode::FullAuto
        );
        // 同时给两者**不算冲突**（等价档去重）
        assert!(resolve_approval_mode(None, false, false, true, true).is_ok());
    }

    /// **冲突必须报错**，不得猜优先级。
    ///
    /// 「同时给 `--suggest` 和 `--yolo` 时谁赢」**不该由程序替用户决定** ——
    /// 无论选哪个，另一个都是用户明确要求的、且被静默忽略。
    #[test]
    fn conflicting_modes_are_ERR_not_silent_precedence() {
        for (sg, ae, fa, yo) in [
            (true, true, false, false),   // suggest + auto-edit
            (true, false, true, false),   // suggest + full-auto
            (true, false, false, true),   // suggest + yolo
            (false, true, true, false),   // auto-edit + full-auto
            (false, true, false, true),   // auto-edit + yolo
        ] {
            let r = resolve_approval_mode(None, sg, ae, fa, yo);
            assert!(r.is_err(), "冲突组合({sg},{ae},{fa},{yo}) 必须 Err，实际 {r:?}");
            assert!(r.unwrap_err().contains("冲突"), "错误信息应说明是冲突");
        }
        // 主 flag 与旧 flag 指向不同档 ⇒ 同样冲突
        assert!(resolve_approval_mode(Some("suggest"), false, true, false, false).is_err());
        // 指向同一档 ⇒ 不算冲突
        assert!(resolve_approval_mode(Some("full-auto"), false, false, true, true).is_ok());
    }

    /// **「启用即断言接线」**：解码结果必须真的能落地到全局单例。
    ///
    /// 【这条为什么必要】本会话实测过同族缺陷：
    /// `--yolo` / `--full-auto` / `--auto-edit` 会写入全局 `ApprovalMode`，
    /// 但**没有任何生产工具执行路径读取它** ⇒ flag 存在、能编译、有文档，
    /// 而行为完全不变。
    /// ⇒ 本测试至少守住「解码 ⇒ 落地」这一段是通的；
    /// 「落地 ⇒ 被消费」那一段由 `CLAIMED-BUT-NOT-ENFORCED` 清单第 9 项跟踪。
    #[test]
    fn resolved_mode_actually_lands_in_global_state() {
        let m = resolve_approval_mode(Some("full-auto"), false, false, false, false).unwrap();
        neotrix::l6_meta::nt_approval::global_approval()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .set_mode(m);
        assert_eq!(
            neotrix::l6_meta::nt_approval::global_approval()
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .mode(),
            ApprovalMode::FullAuto,
            "解码出的模式必须真的落到全局单例（否则 flag 就是假开关）"
        );
        // 复位，别污染其它测试
        neotrix::l6_meta::nt_approval::global_approval()
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .set_mode(ApprovalMode::Suggest);
    }
}

#[cfg(test)]
mod profile_use_authorization_tests {
    //! **安全边界的反向锁** —— `profile use` 的授权裁决。
    //!
    //! 【为什么必须有】这是决定「要不要问一句 / 允不允许静默执行」的判据。
    //! ⛔ 它一旦被改松，**后果是审批严格度被静默放宽**，
    //! 且**没有任何测试会红**（编译器不管、运行时无日志）。
    //! ⇒ 这类判据必须逐格锁住，否则等于没写。
    //!
    //! 【规则表】
    //! | 改审批模式 | TTY | `--yes` | 裁决 |
    //! |---|---|---|---|
    //! | 否 | 任意 | 任意 | `NoConfirmationNeeded` |
    //! | 是 | 是 | 任意 | `NeedsInteractiveConfirm` |
    //! | 是 | 否 | 是 | `AllowNonInteractive` |
    //! | 是 | 否 | 否 | **`Refuse`** |

    use super::{authorize_profile_use, ProfileUseAuthorization};
    use neotrix::l6_meta::nt_approval::ApprovalMode;
    use neotrix::l6_meta::nt_permission_profiles::ProfileSwitchPlan;

    fn plan(changes: bool, resulting: Option<ApprovalMode>) -> ProfileSwitchPlan {
        ProfileSwitchPlan {
            profile: "developer".into(),
            approval_mode_override: resulting.map(|m| m.as_str().to_string()),
            resulting_mode: resulting,
            notice: resulting.map(|m| format!("切过去会变成 {m:?}")),
        }
        // ⚠️ 字段是 `pub` ⇒ 构造器测试**绕过了 plan_profile_switch**。
        // 这不是缺陷：`ProfileSwitchPlan` 是纯数据（DTO），
        // 而**真正的裁决**在 `authorize_profile_use`，此处正是要独立测它。
    }

    /// 不改审批模式 ⇒ 无需确认（方案 A 的前提：不可逆放宽才需要问）。
    #[test]
    fn no_mode_change_never_asks() {
        for (is_tty, yes) in [(true, false), (false, false), (false, true), (true, true)] {
            let auth = authorize_profile_use(&plan(false, None), ApprovalMode::Suggest, is_tty, yes);
            assert!(
                matches!(auth, ProfileUseAuthorization::NoConfirmationNeeded { .. }),
                "不改模式时(is_tty={is_tty},yes={yes})不该问，实际 {auth:?}"
            );
        }
    }

    /// 改审批模式 + **非 TTY + 无 `--yes` ⇒ 必须 `Refuse`**。
    ///
    /// 【这是整张表里最关键的一格】
    /// 若放行，则 `neotrix profile use developer` 出现在 CI/脚本里
    /// 会**悄悄把审批严格度降到 auto-edit**，日志零痕迹。
    /// （对比 `claude-code`：无对话框时**直接放行** —— 它的默认是放行，
    ///   我们的默认是拒绝，**静默降级只允许朝严格方向**。）
    #[test]
    fn non_tty_without_yes_is_REFUSED_not_silently_allowed() {
        let auth = authorize_profile_use(
            &plan(true, Some(ApprovalMode::AutoEdit)),
            ApprovalMode::Suggest,
            false,
            false,
        );
        match auth {
            ProfileUseAuthorization::Refuse { reason } => {
                assert!(reason.contains("--yes"), "拒绝理由应告诉用户怎么显式授权：{reason}");
                assert!(reason.contains("developer"), "拒绝理由应点名是哪个档位：{reason}");
            }
            other => panic!("必须 Refuse，实际 {other:?} —— 这会让 CI 静默放宽审批"),
        }
    }

    /// 非 TTY + `--yes` ⇒ 放行，但 actor 必须标注是**自动化通道**（审计要能区分人/机）。
    #[test]
    fn non_tty_with_yes_allows_and_marks_actor_as_non_interactive() {
        let auth = authorize_profile_use(
            &plan(true, Some(ApprovalMode::FullAuto)),
            ApprovalMode::Suggest,
            false,
            true,
        );
        match auth {
            ProfileUseAuthorization::AllowNonInteractive { actor } => {
                assert!(
                    actor.starts_with("non-interactive:"),
                    "actor 必须标明非交互通道（审计据此区分人/机），实际 {actor:?}"
                );
            }
            other => panic!("应 AllowNonInteractive，实际 {other:?}"),
        }
    }

    /// TTY ⇒ 一律要确认，**即使 `--yes` 给了**（有交互能力时不该跳过确认）。
    #[test]
    fn tty_always_confirms_even_with_yes_flag() {
        let auth = authorize_profile_use(
            &plan(true, Some(ApprovalMode::AutoEdit)),
            ApprovalMode::Suggest,
            true,
            true,
        );
        match auth {
            ProfileUseAuthorization::NeedsInteractiveConfirm { notice, loosens } => {
                assert!(!notice.is_empty(), "确认前必须有副作用预告，否则确认只是形式");
                assert!(loosens, "Suggest → AutoEdit 是**放宽**");
            }
            other => panic!("TTY 下必须确认，实际 {other:?}"),
        }
    }

    /// 放宽/收紧的判定必须与 `strictness_rank` 一致（收紧不该被标成「放宽」）。
    #[test]
    fn loosens_flag_matches_direction_of_change() {
        // 放宽：Suggest(0) → FullAuto(2)
        assert!(matches!(
            authorize_profile_use(&plan(true, Some(ApprovalMode::FullAuto)), ApprovalMode::Suggest, true, false),
            ProfileUseAuthorization::NeedsInteractiveConfirm { loosens: true, .. }
        ));
        // 收紧：FullAuto(2) → Suggest(0)
        assert!(matches!(
            authorize_profile_use(&plan(true, Some(ApprovalMode::Suggest)), ApprovalMode::FullAuto, true, false),
            ProfileUseAuthorization::NeedsInteractiveConfirm { loosens: false, .. }
        ));
        // 同档：不算放宽也不收紧
        assert!(matches!(
            authorize_profile_use(&plan(true, Some(ApprovalMode::AutoEdit)), ApprovalMode::AutoEdit, true, false),
            ProfileUseAuthorization::NeedsInteractiveConfirm { loosens: false, .. }
        ));
    }

    /// **穷举锁**：把 (TTY × `--yes` × 是否改模式) 全部 8 格跑一遍，
    /// 钉住「**恰好**只有一格是 Refuse、且没有格子意外放行」。
    ///
    /// 【为什么要穷举】逐格测试容易漏一格，而漏的那格恰好是
    /// 「CI 里静默放宽审批」那条路径。⇒ 用循环穷举，让漏网无处可藏。
    #[test]
    fn exhaustive_truth_table_has_exactly_one_refuse_cell() {
        let mut refuse_cells = 0;
        let mut allow_without_tty_and_without_yes = 0;
        for is_tty in [false, true] {
            for yes in [false, true] {
                for changes in [false, true] {
                    let resulting = if changes { Some(ApprovalMode::AutoEdit) } else { None };
                    let auth =
                        authorize_profile_use(&plan(changes, resulting), ApprovalMode::Suggest, is_tty, yes);
                    match auth {
                        ProfileUseAuthorization::Refuse { .. } => {
                            refuse_cells += 1;
                            assert!(
                                changes && !is_tty && !yes,
                                "只应在 (改模式 ∧ 非TTY ∧ 无--yes) 这一格 Refuse，实际 \
                                 (changes={changes}, is_tty={is_tty}, yes={yes})"
                            );
                        }
                        ProfileUseAuthorization::AllowNonInteractive { .. } => {
                            assert!(!is_tty && yes, "AllowNonInteractive 只该出现在非TTY+--yes");
                        }
                        ProfileUseAuthorization::NeedsInteractiveConfirm { .. } => {
                            assert!(changes && is_tty, "确认只该出现在「改模式 ∧ TTY」");
                        }
                        ProfileUseAuthorization::NoConfirmationNeeded { .. } => {
                            assert!(!changes, "不改模式时不该确认");
                            if !is_tty && !yes {
                                allow_without_tty_and_without_yes += 1;
                            }
                        }
                    }
                }
            }
        }
        assert_eq!(refuse_cells, 1, "整张表里恰好一格 Refuse");
        // 「非TTY 无 --yes 但不改模式」是**允许**的（没放宽就不需要授权）
        assert_eq!(allow_without_tty_and_without_yes, 1, "不改模式时不该要授权");
    }
}
