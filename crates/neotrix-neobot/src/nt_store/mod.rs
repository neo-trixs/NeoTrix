//! `nt_store` — SQLite 持久化：通用 KV/认领存储 + 调用账本 +
//! 实时 outbox（事务内 enqueue + 租约 drain），全部收敛到单文件
//! SQLite，无 Postgres/Redis.

use rusqlite::{Connection, params};

use crate::nt_error::NtBotError;

/// 认领默认 TTL 秒（认领过期自动释放：5 分钟）。
pub const CLAIM_TTL_SECS: i64 = 300;

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

/// 账本行（模型调用记录本地版）。
#[derive(Debug, Clone)]
pub struct LedgerEntry {
    pub id: String,
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
                last_error TEXT);",
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
            "ALTER TABLE core_pair ADD COLUMN via TEXT NOT NULL DEFAULT 'http'",
            "ALTER TABLE core_pair ADD COLUMN token_env TEXT NOT NULL DEFAULT 'CRYSTAL_TOKEN'",
        ] {
            let _applied: Option<usize> = self.conn.execute(alter, []).ok();
        }
        // 已读水位表（时间戳版；缺省水位=会话创建时间）。
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS read_marks(
               convo_id TEXT PRIMARY KEY, last_read_at TEXT NOT NULL)",
            [],
        )?;
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

mod nt_store_convos;
mod nt_store_files;
mod nt_store_ledger;
mod nt_store_providers;
mod nt_store_reply_tag;
mod nt_store_routines;
mod nt_store_tasks;
mod nt_store_upkeep;
