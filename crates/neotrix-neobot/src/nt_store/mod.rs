//! `nt_store` — SQLite 持久化：通用 KV/认领存储 + 调用账本 +
//! 实时 outbox（事务内 enqueue + 租约 drain），全部收敛到单文件
//! SQLite，无 Postgres/Redis.

use rusqlite::{Connection, params};

use crate::nt_error::NtBotError;

/// 认领默认 TTL 秒（认领过期自动释放：5 分钟）。
pub const CLAIM_TTL_SECS: i64 = 300;

/// 单次补发最多试几次（到顶就不再自动重试，防重复发送）。
pub const MAX_SEND_ATTEMPTS: i64 = 3;

/// 会话行（`conversations` 表 + 聚合；IM 语义的容器）。
#[derive(Debug, Clone)]
pub struct Conversation {
    pub id: String,
    /// dm | group
    pub kind: String,
    pub title: String,
    pub created_at: String,
    pub members: Vec<String>,
    pub task_count: i64,
    pub last_active: String,
    /// 免打扰（不亮未读、不打扰）。
    pub muted: bool,
    /// 未读数（已读水位本地版：水位之后新建任务数）。
    pub unread: i64,
    /// 侧聊的母会话 id（`None` = 顶层会话；见 `nt_side_chat`）。
    pub parent_id: Option<String>,
    /// `chat`（顶层，主列表可见）| `sidebar`（侧聊，只在侧边栏可见）。
    pub origin: String,
}

/// 晶体核心配对行（`core_pair` 单行表；灵魂嵌入状态：配对即嵌入，删除即摘除）。
#[derive(Debug, Clone)]
pub struct CorePair {
    pub base_url: String,
    pub model: String,
    pub paired_at: String,
    pub last_ok_at: String,
    pub last_latency_ms: i64,
    /// 通道：http（OpenAI 兼容直连）| cli（本机 opencode CLI）。
    pub via: String,
    /// token 环境变量名（永不存 token 值；缺省 `CRYSTAL_TOKEN`）。
    pub token_env: String,
}

/// 附件行（`attachments` 表；文件本体在 `<data_dir>/attachments/`）。
#[derive(Debug, Clone)]
pub struct Attachment {
    pub id: String,
    pub convo_id: String,
    /// image | video | text | file
    pub kind: String,
    pub name: String,
    pub path: String,
    pub size: i64,
    pub created_at: String,
}

/// 按扩展名判附件类型（未知走 file；大小写不敏感）。
pub fn classify_attachment(name: &str) -> &'static str {
    let lower = name.to_ascii_lowercase();
    let ext = lower.rsplit('.').next().unwrap_or("");
    match ext {
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "svg" | "heic" => "image",
        "mp4" | "mov" | "webm" | "mkv" | "m4v" => "video",
        "txt" | "md" | "json" | "csv" | "log" | "rs" | "ts" | "js" | "py" => "text",
        _ => "file",
    }
}

/// 本轮文件改动账目行（`file_changes` 表；**不含** before/after）。
///
/// 前后内容是重物（两侧各 256 KiB 上限），故不进列表行、按需走
/// [`FileChangeView`] 取 —— 列一百行账不该拖一百份文件内容。
#[derive(Debug, Clone)]
pub struct FileChange {
    pub id: String,
    pub task_id: String,
    pub at: String,
    /// 相对工作区根的路径（即便工作区整体搬走，历史账仍读得懂）。
    pub path: String,
    /// `read` | `write` | `edit`（见 `nt_changes::KIND_*`）。
    pub kind: String,
    /// 读=读到的长度；写/改=**改后**内容长度（真实增删行数由前端 diff 算）。
    pub bytes: i64,
    /// 任一侧超 `nt_changes::CHANGE_CONTENT_CAP`，内容整笔略去。
    pub content_omitted: bool,
}

/// 账目行 + 前后内容（渲染 diff 的唯一入口）。
#[derive(Debug, Clone)]
pub struct FileChangeView {
    pub change: FileChange,
    /// `None` = 新建（此前不存在）或内容被略去；靠 `content_omitted` 区分。
    pub before: Option<String>,
    pub after: Option<String>,
}

/// 一任务内按文件分组的汇总（「本轮文件」列表的一行）。
#[derive(Debug, Clone)]
pub struct PathTally {
    pub path: String,
    pub reads: i64,
    pub writes: i64,
    pub edits: i64,
    /// 最近一次改动的字节数。
    pub bytes: i64,
    pub last_at: String,
    /// 最近一条账的 id（点开即取 before/after）。
    pub last_change: String,
}

/// 账本行（模型调用记录本地版）。
#[derive(Debug, Clone)]
pub struct LedgerEntry {    pub id: String,
    pub at: String,
    pub engine: String,
    pub model: String,
    pub actor: String,
    pub purpose: String,
    pub in_tokens: i64,
    pub out_tokens: i64,
    pub cost_usd: f64,
    pub measured: bool,
    pub status: String,
    pub latency_ms: i64,
    pub error: Option<String>,
}

/// 渠道行（`channels` 表；token 值永不落库，只有 `token_env` 变量名）。
#[derive(Debug, Clone)]
pub struct ChannelRow {
    pub id: String,
    pub title: String,
    pub enabled: bool,
    /// 缺省访问模式（新机器人继承）。
    pub access_mode: String,
    pub poll_secs: i64,
    pub created_at: String,
}

/// 机器人行（`channel_bots` 表；别名/白名单/会话**各机器人独立**）。
///
/// **`model` 与 `token_env` 存了但 serve 不用**：一个渠道只注册一个适配器，
/// 所以 per-bot 的 token 与模型在跑轮里**没有生效**（跑轮只用全局设置）。
/// 别在这里写「模型各自独立绑定」——那会让读代码的人以为多模型分发已经可用。
#[derive(Debug, Clone)]
pub struct BotRow {
    pub channel: String,
    pub bot_id: String,
    /// 显示别名（空 = 用平台原名）。
    pub alias: String,
    /// token 的**环境变量名**（绝不是值）。
    pub token_env: String,
    /// 绑定的 neobot 会话 id。
    pub conversation_id: Option<String>,
    /// 覆盖用模型（空 = 跟主设置）。
    pub model: String,
    /// 独立白名单（逗号分隔）。
    pub allow_list: String,
    pub created_at: String,
    pub last_seen: Option<String>,
}

/// 待补发的结果行（`pending_deliveries` 表；超时后补发）。
#[derive(Debug, Clone)]
pub struct PendingDelivery {
    pub id: String,
    pub channel: String,
    pub bot_id: String,
    pub chat: String,
    pub origin_message: String,
    pub text: String,
    pub task_id: String,
    pub attempts: i64,
    pub created_at: String,
}

/// 任务行 13 元组（`get_task`/`list_tasks` 共用，压 type_complexity）。
pub(crate) type TaskRow = (
    String,
    String,
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    String,
    Option<String>,
    Option<String>,
    i64,
    Option<String>,
    Option<String>,
);

/// 账本聚合行（engine, model, in, out, cost）。
pub type LedgerSum = (String, String, i64, i64, f64);
/// 账本聚合行（engine, model, actor, in, out, cost）。
pub type LedgerActorSum = (String, String, String, i64, i64, f64);


/// SQLite store (单文件, `bundled` 特性本地编译, 无外部服务).
pub struct NeobotStore {
    conn: Connection,
}

impl NeobotStore {
    /// 打开 (内存 `:memory:` 或文件), 幂等建表.
    pub fn open(path: &str) -> Result<Self, NtBotError> {
        let conn = Connection::open(path)?;
        // P0 审计 F4：CLI 与桌面 App 双进程同库——5s 忙等待 + WAL，
        // 并发写不再直接 `database is locked`。
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        let _journal: String = conn
            .query_row("PRAGMA journal_mode=WAL", [], |r| r.get(0))
            .unwrap_or_else(|_| "memory".to_owned());
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    /// WAL 检查点（导出前落盘，保证拷贝的 db 文件自包含）。
    pub fn checkpoint(&self) -> Result<(), NtBotError> {
        self.conn
            .query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |r| r.get::<_, i64>(0))
            .map(|_| ())?;
        Ok(())
    }

    /// 导出用计数（tasks, conversations, ledger 行数）。
    pub fn export_counts(&self) -> Result<(i64, i64, i64), NtBotError> {
        let tasks: i64 =
            self.conn.query_row("SELECT COUNT(*) FROM tasks", [], |r| r.get(0))?;
        let convos: i64 =
            self.conn.query_row("SELECT COUNT(*) FROM conversations", [], |r| r.get(0))?;
        let ledger: i64 =
            self.conn.query_row("SELECT COUNT(*) FROM ledger", [], |r| r.get(0))?;
        Ok((tasks, convos, ledger))
    }

    fn migrate(&self) -> Result<(), NtBotError> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS tasks(
               id TEXT PRIMARY KEY, title TEXT NOT NULL, status TEXT NOT NULL,
               created_at TEXT NOT NULL, updated_at TEXT NOT NULL,
               claimed_by TEXT, claimed_at TEXT,
               visibility TEXT NOT NULL DEFAULT 'team',
               lease_id TEXT, lease_until TEXT,
               attempts INTEGER NOT NULL DEFAULT 0, error TEXT);
             CREATE TABLE IF NOT EXISTS steps(
               id INTEGER PRIMARY KEY AUTOINCREMENT, task_id TEXT NOT NULL,
               n INTEGER NOT NULL, tool TEXT NOT NULL, ok INTEGER NOT NULL,
               output TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS audit(
               id TEXT PRIMARY KEY, at TEXT NOT NULL, actor TEXT NOT NULL,
               tool TEXT NOT NULL, decision TEXT NOT NULL,
               rule TEXT, detail TEXT NOT NULL);
             CREATE TABLE IF NOT EXISTS ledger(
               id TEXT PRIMARY KEY, at TEXT NOT NULL, engine TEXT NOT NULL,
               model TEXT NOT NULL, actor TEXT NOT NULL DEFAULT 'bot',
               purpose TEXT NOT NULL DEFAULT 'agent-turn',
               in_tokens INTEGER NOT NULL, out_tokens INTEGER NOT NULL,
               cost_usd REAL NOT NULL, measured INTEGER NOT NULL DEFAULT 0,
               status TEXT NOT NULL DEFAULT 'ok',
               latency_ms INTEGER NOT NULL DEFAULT 0, error TEXT);
              CREATE TABLE IF NOT EXISTS outbox(
                id TEXT PRIMARY KEY, topic TEXT NOT NULL,
                payload TEXT NOT NULL, claimed INTEGER NOT NULL DEFAULT 0,
                attempts INTEGER NOT NULL DEFAULT 0,
                available_at TEXT NOT NULL DEFAULT '1970-01-01T00:00:00Z');
              CREATE TABLE IF NOT EXISTS members(
                id TEXT PRIMARY KEY, kind TEXT NOT NULL,
                created_at TEXT NOT NULL);
              CREATE TABLE IF NOT EXISTS control(
                id INTEGER PRIMARY KEY CHECK (id = 1),
                holder TEXT NOT NULL, updated_at TEXT NOT NULL);
              CREATE TABLE IF NOT EXISTS providers(
                name TEXT PRIMARY KEY, base_url TEXT NOT NULL,
                key_env TEXT NOT NULL DEFAULT '', model TEXT NOT NULL DEFAULT '',
                enabled INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL);
              CREATE TABLE IF NOT EXISTS core_pair(
                id INTEGER PRIMARY KEY CHECK(id=1),
                base_url TEXT NOT NULL, model TEXT NOT NULL,
                paired_at TEXT NOT NULL, last_ok_at TEXT NOT NULL,
                last_latency_ms INTEGER NOT NULL DEFAULT -1,
                via TEXT NOT NULL DEFAULT 'http',
                token_env TEXT NOT NULL DEFAULT 'CRYSTAL_TOKEN');
              CREATE TABLE IF NOT EXISTS conversations(
                id TEXT PRIMARY KEY, kind TEXT NOT NULL,
                title TEXT NOT NULL, created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL, muted INTEGER NOT NULL DEFAULT 0);
              CREATE TABLE IF NOT EXISTS conversation_members(
                convo_id TEXT NOT NULL, member_id TEXT NOT NULL,
                PRIMARY KEY (convo_id, member_id));
              CREATE TABLE IF NOT EXISTS attachments(
                id TEXT PRIMARY KEY, convo_id TEXT NOT NULL, kind TEXT NOT NULL,
                name TEXT NOT NULL, path TEXT NOT NULL,
                size INTEGER NOT NULL DEFAULT 0, created_at TEXT NOT NULL);
              CREATE TABLE IF NOT EXISTS routines(
                name TEXT PRIMARY KEY, interval_secs INTEGER NOT NULL,
                instruction TEXT NOT NULL, owner TEXT NOT NULL,
                failures INTEGER NOT NULL DEFAULT 0,
                disabled INTEGER NOT NULL DEFAULT 0,
                next_run_at INTEGER NOT NULL DEFAULT 0,
                last_run_at INTEGER NOT NULL DEFAULT 0,
                last_error TEXT);
              CREATE TABLE IF NOT EXISTS file_changes(
                id TEXT PRIMARY KEY, task_id TEXT NOT NULL, at TEXT NOT NULL,
                path TEXT NOT NULL, kind TEXT NOT NULL,
                bytes INTEGER NOT NULL DEFAULT 0,
                content_omitted INTEGER NOT NULL DEFAULT 0,
                before TEXT, after TEXT);
              CREATE INDEX IF NOT EXISTS file_changes_task
                ON file_changes(task_id, at);
              CREATE INDEX IF NOT EXISTS file_changes_path
                ON file_changes(path, at);
            CREATE TABLE IF NOT EXISTS channels(
              id TEXT PRIMARY KEY, title TEXT NOT NULL,
              enabled INTEGER NOT NULL DEFAULT 1,
              access_mode TEXT NOT NULL DEFAULT 'allow',
              poll_secs INTEGER NOT NULL DEFAULT 5,
              created_at TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS channel_bots(
              channel TEXT NOT NULL, bot_id TEXT NOT NULL,
              alias TEXT NOT NULL DEFAULT '',
              token_env TEXT NOT NULL DEFAULT '',
              conversation_id TEXT, model TEXT NOT NULL DEFAULT '',
              allow_list TEXT NOT NULL DEFAULT '',
              created_at TEXT NOT NULL, last_seen TEXT,
              PRIMARY KEY (channel, bot_id));
            CREATE TABLE IF NOT EXISTS channel_seen(
              dedup_key TEXT PRIMARY KEY, at TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS pending_deliveries(
              id TEXT PRIMARY KEY, channel TEXT NOT NULL, bot_id TEXT NOT NULL,
              chat TEXT NOT NULL, origin_message TEXT NOT NULL DEFAULT '',
              text TEXT NOT NULL, task_id TEXT NOT NULL DEFAULT '',
              attempts INTEGER NOT NULL DEFAULT 0, created_at TEXT NOT NULL,
              -- ⛔ 下面两列**必须追加在末尾**：`delivery_row()` 按 `r.get(0..8)`
              --    取列，若插在 `created_at` 之前会**静默错位**
              --    （类型都是 String/i64，`r.get` 不报错 ⇒ 编译过、测试可能也绿、
              --      线上读出垃圾 —— R-SCAN-1 家族的事故形态）。
              -- 与 outbox 的同名列**保持一致**，两条投递路径策略统一。
              available_at TEXT NOT NULL DEFAULT '1970-01-01T00:00:00Z',
              claimed INTEGER NOT NULL DEFAULT 0);",
        )?;
        // 存量库补列（新库建表已含；ALTER 重复报错吞掉，保证幂等；
        // 单机本地库，补列失败不影响本次调用——返回时统一 Ok）。
        for alter in [
            "ALTER TABLE tasks ADD COLUMN conversation_id TEXT",
            "ALTER TABLE tasks ADD COLUMN claimed_by TEXT",
            "ALTER TABLE tasks ADD COLUMN claimed_at TEXT",
            "ALTER TABLE tasks ADD COLUMN visibility TEXT NOT NULL DEFAULT 'team'",
            "ALTER TABLE tasks ADD COLUMN lease_id TEXT",
            "ALTER TABLE tasks ADD COLUMN lease_until TEXT",
            "ALTER TABLE tasks ADD COLUMN attempts INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE tasks ADD COLUMN error TEXT",
            "ALTER TABLE ledger ADD COLUMN actor TEXT NOT NULL DEFAULT 'bot'",
            "ALTER TABLE ledger ADD COLUMN purpose TEXT NOT NULL DEFAULT 'agent-turn'",
            "ALTER TABLE ledger ADD COLUMN measured INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE ledger ADD COLUMN status TEXT NOT NULL DEFAULT 'ok'",
            "ALTER TABLE ledger ADD COLUMN latency_ms INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE ledger ADD COLUMN error TEXT",
            "ALTER TABLE outbox ADD COLUMN attempts INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE outbox ADD COLUMN available_at TEXT NOT NULL DEFAULT '1970-01-01T00:00:00Z'",
            "ALTER TABLE routines ADD COLUMN next_run_at INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE routines ADD COLUMN last_run_at INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE routines ADD COLUMN last_error TEXT",
            "ALTER TABLE routines ADD COLUMN owner TEXT NOT NULL DEFAULT 'owner'",
            "ALTER TABLE routines ADD COLUMN instruction TEXT NOT NULL DEFAULT ''",
            "ALTER TABLE conversations ADD COLUMN muted INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE conversations ADD COLUMN parent_id TEXT",
            "ALTER TABLE conversations ADD COLUMN origin TEXT NOT NULL DEFAULT 'chat'",
            "ALTER TABLE core_pair ADD COLUMN via TEXT NOT NULL DEFAULT 'http'",
            "ALTER TABLE core_pair ADD COLUMN token_env TEXT NOT NULL DEFAULT 'CRYSTAL_TOKEN'",
            // ⛔ 存量库补列：`CREATE TABLE IF NOT EXISTS` 对**已有表静默跳过**，
            //    SQLite 又不支持 `ADD COLUMN IF NOT EXISTS` ⇒ 只有 ALTER 一条路。
            //    照抄上面 outbox 的同款语句；重复执行报 duplicate column，
            //    由下面的 `.ok()` 吞掉 ⇒ 幂等成立（同 mod.rs:361-362 的先例）。
            "ALTER TABLE pending_deliveries ADD COLUMN available_at TEXT NOT NULL DEFAULT '1970-01-01T00:00:00Z'",
            "ALTER TABLE pending_deliveries ADD COLUMN claimed INTEGER NOT NULL DEFAULT 0",
        ] {
            let _applied: Option<usize> = self.conn.execute(alter, []).ok();
        }
        // 已读水位表（时间戳版；缺省水位=会话创建时间）。
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS read_marks(
               convo_id TEXT PRIMARY KEY, last_read_at TEXT NOT NULL)",
            [],
        )?;
        // 聊天消息表（会话内问答落库；见 nt_store_messages）。
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS messages(
               id TEXT PRIMARY KEY, convo_id TEXT NOT NULL, role TEXT NOT NULL,
               text TEXT NOT NULL, created_at TEXT NOT NULL)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_convo
             ON messages(convo_id, created_at)",
            [],
        )?;
        // ⛔⛔ **这里刻意没有 `(convo_id, rowid)` 索引**，而且**加不了**：
        //    `rowid` 是 SQLite 的**隐式** rowid 列，在 `SELECT`/`WHERE`/`ORDER BY`
        //    里可用（别名 `_rowid_`、`oid`），但 **索引定义看不见它** ⇒
        //    `CREATE INDEX … ON messages(convo_id, rowid)` 直接报
        //    `no such column: rowid`（实测：`nt_store_messages` 4 个测试当场红）。
        //    ⭐ 故 `ORDER BY rowid` 仍需 SQLite 排序 —— 但那是**性能**问题，
        //    而游标分页的**正确性**来自「`rowid` 是全序整数」，与此无关。
        //    ⛔ 若日后要索引覆盖，必须新增**真实列** `seq INTEGER` 并把
        //    `rowid` 的值写进去（见 `nt_store_messages.rs` 里 seq 游标的取舍注释）。

        // 存量任务回填默认群组（仅当有无归属任务时；用户删掉全体后不再复活）。
        let now = chrono::Utc::now().to_rfc3339();
        let orphans: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM tasks WHERE conversation_id IS NULL",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if orphans > 0 {
            let _backfilled: Option<usize> = self
                .conn
                .execute(
                    "INSERT OR IGNORE INTO conversations(id,kind,title,created_at,updated_at)
                     VALUES('general','group','全体',?1,?1)",
                    params![now],
                )
                .ok();
            let _updated: Option<usize> = self
                .conn
                .execute(
                    "UPDATE tasks SET conversation_id='general' WHERE conversation_id IS NULL",
                    [],
                )
                .ok();
        }
        // 内置端点自举：neotrix 核心模型池口默认在席（不可达则列表自动跳过）。
        let _seeded: Option<usize> = self
            .conn
            .execute(
                "INSERT OR IGNORE INTO providers(name,base_url,key_env,model,enabled,created_at)
                 VALUES('neotrix','http://127.0.0.1:3000/v1','NEOBOT_API_KEY','neotrix-crystal',1,?1)",
                params![now],
            )
            .ok();
        // 存量 neotrix 行（model 为空的老版本）补默认核心模型名。
        let _patched: Option<usize> = self
            .conn
            .execute(
                "UPDATE providers SET model='neotrix-crystal' WHERE name='neotrix' AND model=''",
                [],
            )
            .ok();
        Ok(())
    }
}

mod nt_store_changes;
mod nt_store_channels;
pub use nt_store_channels::{
    MAX_POLL_SECS, MIN_POLL_SECS, PENDING_RETENTION_DAYS, SEEN_RETENTION_DAYS, parse_allow_list,
};
mod nt_store_convos;
mod nt_store_files;
mod nt_store_ledger;
mod nt_store_messages;
pub use nt_store_messages::{ChatMessage, MESSAGE_MAX_CHARS};
mod nt_store_providers;
mod nt_store_reply_tag;
mod nt_store_routines;
mod nt_store_tasks;
/// 重导出：调用方能用 `.ok` / `.output`，但**写不出**该类型（模块私有），
/// 所以具名类型必须从这里出去，否则调用点只能继续拿位置性元组。
pub use nt_store_tasks::LastStep;
mod nt_store_upkeep;
