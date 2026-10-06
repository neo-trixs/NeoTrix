//! 工具**调用**侧的持久化（`tool_calls` 表）—— 配平的另一半。
//!
//! # 为什么已经有 `steps.tool_call_id` 还要这张表
//!
//! `ToolCallJoinCheck`（`neotrix-core/…/nt_core_artifact_verdict.rs`）是**双向**
//! 的：孤儿调用（call 无 result）与孤儿结果（result 无 call）都算破。
//! 上一位 agent 补齐了**结果**侧（`steps.tool_call_id` + `add_step_with_call` +
//! `list_steps`）。**调用**侧此前完全没有落库来源：`TranscriptItem.tool_call_id`
//! 只活在 `nt_agent` 的内存 `Vec` 里 ⇒ 进程一退，调用侧证据归零，
//! join 对真实落库数据**根本无法表达**，检查只能诚实回 `NOT_EVALUABLE`。
//! 本表就是给那半边证据一个家。
//!
//! # 为什么不是 `messages` 的一列
//!
//! `messages` **结构上装不下调用**，三条理由（详见 `mod.rs::migrate` 同段注释）：
//! ① `append_message` 硬拒 `role` 非 `user`/`assistant`
//!    （`nt_store_messages.rs:69-71`），而调用在 OpenAI 兼容形状里正是
//!    `tool` role 的东西 —— 借道它就得先废掉一条前端渲染依赖的校验；
//! ② 基数不同（一次 assistant 消息带**若干**调用），一行一消息装不下；
//! ③ 调用有自己的字段（工具名、跳数），`messages` 补一列给不出。
//!
//! # ⛔ `call_id` 不唯一 —— 所以有代理键
//!
//! 本表的行**键**是自增的 `seq`，`call_id` 上只有**非唯一**索引。实测两个
//! 生产 id 源都会让同一个 id 反复出现：
//!
//! | 源 | id 形态 | 后果 |
//! |---|---|---|
//! | `nt_engine.rs:272` CLI 引擎 | 常量 `"cli-status-1"` | **每次** CLI 调用同名 |
//! | `nt_http_engine.rs:560-564` HTTP 引擎 | provider 空 id → `format!("call-{name}")` | 所有 `bash` 调用同名 `"call-bash"`；同一响应里两个并行同工具调用**当场撞名** |
//!
//! 用 `PRIMARY KEY(call_id)` 会怎样？实测（`sqlite3` CLI）：第二行插入直接
//! `UNIQUE constraint failed` ⇒ 那不是「重复时报个警」，而是**正常业务写不进去**；
//! 若错误被 `.ok()` 吞掉，就退化成**静默丢调用** —— 比重复严重得多。
//!
//! 反过来，`call_id` 重复**本身就是一条要报的结论**：
//! `nt_core_artifact_verdict.rs:640-641` 的 `CODE_DUPLICATE_JOIN_KEY`（判定
//! `INVALID_ARTIFACT`，理由「配平键失去单射性」）就是为它准备的。
//! ⇒ 重复**必须能被记下来**给检查读，**绝不能**由主键在入库前掐掉。
//!
//! # 本 crate 零事务
//!
//! 与 `nt_store_channels.rs:285-289` / `nt_store_ledger.rs:116-120` /
//! `nt_store_messages.rs:46-51` 同款纪律：`record_tool_calls` 逐条 `execute`，
//! **无 `BEGIN`/`COMMIT`/`transaction()`**。批量的失败形态是「写了一半」，
//! 即**部分调用已记、部分没记**；这是可接受的，因为每条调用各自独立成行、
//! 配平检查读的是全集，而少数缺记的调用会诚实地变成**孤儿结果**（`steps`
//! 侧有、调用侧无）—— 被检查看见，而不是被藏起来。

use rusqlite::params;

use crate::nt_error::NtBotError;
use crate::nt_store::NeobotStore;
use crate::nt_types::ToolCall;

/// `tool_calls` 一行（读口返回）。
///
/// ⛔ **`call_id` 是 `String` 而非 `Option<String>`，且本表无 `NULL` 行。**
/// 这与 `steps.tool_call_id` 的可空语义**故意相反**，别照抄那边的读口：
/// * `steps.tool_call_id` 可空，因为那张表**有存量行**，`NULL` = 「本列存在之前
///   写的旧行」，是合法且必须保留的历史状态；
/// * `tool_calls` 是**整表新建**的，没有任何一行早于它存在 ⇒ `call_id` 恒有值。
///   空白键在写口就被拒（见 [`NeobotStore::record_tool_calls`]），
///   因为空白键会与其它空白键互相配平（`CODE_BLANK_JOIN_KEY` 判其为畸形产物）。
///
/// ⚠️ 而「有值」**不等于**「唯一」：同一 `call_id` 出现多行是**正常可记**的
/// 现象（见模块头），判定方要自己数行数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCallRow {
    /// 代理键（`AUTOINCREMENT`；写入序的判据）。**不是** `call_id`。
    pub seq: i64,
    /// 所属任务（与 `steps.task_id` 同一命名域，配平时可交叉引用）。
    pub task_id: String,
    /// 第几跳（0 基；与 `steps.n` 对齐）。
    pub n: i64,
    /// 配平键。与 `steps.tool_call_id` 取值同一形态。
    ///
    /// ⛔ **不保证唯一**：重复是允许记下的**发现**，不是要拦的错。
    pub call_id: String,
    /// 工具名（`ToolCall::name.as_str()`）。
    pub tool: String,
    /// 所属会话；`None` = **调用方此刻拿不到**（`run_loop` 无 `convo_id`），
    /// **不是**「不属于任何会话」。
    pub convo_id: Option<String>,
    /// RFC3339 落库时刻。
    pub created_at: String,
}

impl NeobotStore {
    /// 记一整跳里模型发出的**全部**工具调用，返回落库行（写入序）。
    ///
    /// * `task_id` —— 所属任务（调用方 `run_loop` 手上就有）。
    /// * `n` —— 第几跳（与 `steps.n` 对齐，使「哪个调用配哪条结果」有第二判据）。
    /// * `convo_id` —— `run_loop` 作用域里没有 ⇒ 传 `None`。**不反查
    ///   `tasks.conversation_id` 补一个** —— 那会把「不知道归属」写成「归属于某会话」。
    /// * `calls` —— 本跳引擎回的全部调用（`turn.tool_calls`）。空切片即无写入。
    ///
    /// ## 拒绝什么
    ///
    /// ⛔ **`call_id` 为空白（`trim().is_empty()`）即拒**：空白键配不上任何东西，
    ///   且两行空白键会互相「配平」⇒ 造出假配平。与 `steps` 侧不同，
    ///   这里**没有** `Some("")` 的合法用法（本表无存量行），故直接拒而不是存下来。
    ///
    /// **重复 `call_id` 不拒**：它是一条**发现**（模块头 + `CODE_DUPLICATE_JOIN_KEY`），
    ///   在这里被拦掉就等于把证据销毁。本函数**不**去重、不覆盖。
    ///
    /// ## 零事务
    ///
    /// 逐条 `execute`，**不包 `BEGIN`/`COMMIT`**（见模块头）。一批里中途失败
    /// ⇒ 前面几条已落、后面几条没有；每条独立成行，检查读全集时缺的那几条
    /// 会诚实地呈现为**孤儿结果**，而不是被回滚吞掉。
    pub fn record_tool_calls(
        &self,
        task_id: &str,
        n: i64,
        convo_id: Option<&str>,
        calls: &[ToolCall],
    ) -> Result<Vec<ToolCallRow>, NtBotError> {
        if task_id.trim().is_empty() {
            return Err(NtBotError::Invalid("task_id is empty".to_owned()));
        }
        // 同一跳的调用出自**同一次响应** ⇒ 共用一个时刻，逐条各取一次时间
        // 反而会把「同时发生」写成「先后发生」。
        let now = chrono::Utc::now().to_rfc3339();
        let mut out = Vec::with_capacity(calls.len());
        for call in calls {
            let call_id = call.id.trim();
            if call_id.is_empty() {
                return Err(NtBotError::Invalid(format!(
                    "tool call '{}' has a blank call_id: a blank join key pairs with every \
                     other blank key, so it would fabricate a match",
                    call.name.as_str()
                )));
            }
            let tool = call.name.as_str();
            self.conn.execute(
                "INSERT INTO tool_calls(task_id,n,call_id,tool,convo_id,created_at)
                 VALUES(?1,?2,?3,?4,?5,?6)",
                params![task_id, n, call_id, tool, convo_id, now],
            )?;
            out.push(ToolCallRow {
                seq: self.conn.last_insert_rowid(),
                task_id: task_id.to_owned(),
                n,
                call_id: call_id.to_owned(),
                tool: tool.to_owned(),
                convo_id: convo_id.map(str::to_owned),
                created_at: now.clone(),
            });
        }
        Ok(out)
    }

    /// 某任务的**全部**调用行（按写入序），join 的读口之一。
    ///
    /// 与 [`NeobotStore::list_steps`] 对称：`steps` 给出结果侧全集，
    /// 本函数给出调用侧全集，**两侧按 `call_id` 对齐**即可判双向孤儿。
    pub fn list_tool_calls(&self, task_id: &str) -> Result<Vec<ToolCallRow>, NtBotError> {
        let mut stmt = self.conn.prepare(
            "SELECT seq, task_id, n, call_id, tool, convo_id, created_at FROM tool_calls
             WHERE task_id=?1 ORDER BY seq",
        )?;
        let rows = stmt.query_map(params![task_id], read_tool_call_row)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    /// 按配平键取**全部**匹配行（跨任务）。
    ///
    /// **必须返回 `Vec` 而不是 `Option`** —— 因为 `call_id` 可以重复
    /// （模块头），「有几行」本身就是判定所需的信息：
    /// * `0` 行 ⇒ 若结果侧有这条键 ⇒ **孤儿结果**；
    /// * `1` 行 ⇒ 正常配平；
    /// * `≥2` 行 ⇒ **重复键**（`CODE_DUPLICATE_JOIN_KEY` ⇒ `INVALID_ARTIFACT`），
    ///   此时「哪个结果配哪个调用」无从谈起，判定方应据此停手而不是挑一行。
    ///
    /// 跨任务是**刻意的**：同一个 id 在不同任务里撞名（CLI 引擎常量 id 下必然如此）
    /// 同样是键失去单射性，限定在单任务内查会把它藏起来。
    pub fn find_tool_calls_by_id(&self, call_id: &str) -> Result<Vec<ToolCallRow>, NtBotError> {
        let mut stmt = self.conn.prepare(
            "SELECT seq, task_id, n, call_id, tool, convo_id, created_at FROM tool_calls
             WHERE call_id=?1 ORDER BY seq",
        )?;
        let rows = stmt.query_map(params![call_id], read_tool_call_row)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    /// 某任务内**重复**的配平键（`call_id` → 出现次数，**只含 ≥2 次的**）。
    ///
    /// 这是「重复键」这一类发现的直读口：让判定方不必把全集拉回自己内存里数。
    /// ⛔ 只查 `steps` 侧之外的本表 —— `steps` 侧的重复由判定方按**方向**分别数
    /// （`nt_core_artifact_verdict.rs:696-699` 记录了为什么：同一键本就出现两次，
    /// 一次 Call 一次 Result，共用一个计数桶会把**每一对正常配平**都误判成重复）。
    pub fn duplicate_call_keys(&self, task_id: &str) -> Result<Vec<(String, i64)>, NtBotError> {
        let mut stmt = self.conn.prepare(
            "SELECT call_id, COUNT(*) FROM tool_calls
             WHERE task_id=?1 GROUP BY call_id HAVING COUNT(*) > 1
             ORDER BY call_id",
        )?;
        let rows = stmt.query_map(params![task_id], |r| Ok((r.get(0)?, r.get(1)?)))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }
}

/// 按列序读一行（`seq, task_id, n, call_id, tool, convo_id, created_at`）。
///
/// ⛔⛔ 取列**按位置**，故 SELECT 列表与此处 `r.get` 的序号必须同进同出 ——
/// 往表尾追加列是安全的（`get` 取不到不存在的序号就不该用），
/// 往中间插列会让读口**静默错位**（`mod.rs` `pending_deliveries` 那条事故同款）。
/// `convo_id` 用 `r.get::<_, Option<String>>` ⇒ **SQL `NULL` 读成 `None`、
/// 空串读成 `Some("")`**，两者可区分（实测 `LENGTH(NULL)=-1` vs `LENGTH('')=0`）。
fn read_tool_call_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<ToolCallRow> {
    Ok(ToolCallRow {
        seq: r.get(0)?,
        task_id: r.get(1)?,
        n: r.get(2)?,
        call_id: r.get(3)?,
        tool: r.get(4)?,
        convo_id: r.get(5)?,
        created_at: r.get(6)?,
    })
}

#[cfg(test)]
mod tests {
    use crate::nt_store::NeobotStore;
    use crate::nt_types::{ToolCall, ToolName};

    fn call(id: &str, name: &str) -> ToolCall {
        ToolCall {
            id: id.to_owned(),
            name: ToolName::parse(name),
            args: serde_json::json!({}),
        }
    }

    fn temp_db(tag: &str) -> std::path::PathBuf {
        let dir = crate::nt_testutil::temp_dir(tag);
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir.join("store.sqlite")
    }

    /// 带 id 写入 ⇒ 原样读回，且读口能枚举全集（join 要的是全集）。
    #[test]
    fn tool_call_roundtrips() {
        let store = NeobotStore::open(":memory:").expect("open memory db");
        let written = store
            .record_tool_calls("t1", 0, Some("c9"), &[call("call_a", "bash")])
            .expect("record");
        assert_eq!(written.len(), 1, "写一行读一行");
        let rows = store.list_tool_calls("t1").expect("list");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].call_id, "call_a", "配平键必须原样读回");
        assert_eq!(rows[0].tool, "bash");
        assert_eq!(rows[0].task_id, "t1");
        assert_eq!(rows[0].n, 0);
        assert_eq!(rows[0].convo_id.as_deref(), Some("c9"));
        assert_eq!(
            written[0].seq, rows[0].seq,
            "返回行的代理键必须与读回的一致"
        );
        // 未知任务回空，不炸。
        assert!(store.list_tool_calls("nope").expect("list").is_empty());
    }

    /// ⛔ `NULL` 与空串**必须可区分**：`None` = 不知道归属（`run_loop` 的常态），
    /// `Some("")` = 有一个（已知为空串的）会话。折成同一个值就是撒谎。
    #[test]
    fn null_convo_id_is_distinguishable_from_empty_string() {
        let store = NeobotStore::open(":memory:").expect("open memory db");
        store
            .record_tool_calls("t1", 0, None, &[call("call_a", "bash")])
            .expect("no convo");
        store
            .record_tool_calls("t2", 0, Some(""), &[call("call_b", "bash")])
            .expect("empty convo");
        assert_eq!(
            store.list_tool_calls("t1").expect("list")[0].convo_id,
            None,
            "没给会话 ⇒ 必须落成真 NULL（SQL NULL，不是空串）"
        );
        assert_eq!(
            store.list_tool_calls("t2").expect("list")[0]
                .convo_id
                .as_deref(),
            Some(""),
            "显式给空串 ⇒ 必须与 NULL 读回不同值"
        );
        assert_ne!(
            store.list_tool_calls("t1").expect("list")[0].convo_id,
            store.list_tool_calls("t2").expect("list")[0].convo_id,
            "NULL 与空串混同 ⇒ 判定方会把「不知道归属」读成「归属于空名会话」"
        );
    }

    /// **重复 `call_id` 必须能记下来**（不是被主键拦掉，也不是被去重）。
    ///
    /// 这条守的是一条**实测存在**的同款行为：`nt_engine.rs:272` 的 CLI 引擎
    /// 把 id 写死成常量 `"cli-status-1"`，故「同一个 id 出现两次」是它的**常态**。
    /// 若本表用 `PRIMARY KEY(call_id)`，第二次写就会 `UNIQUE constraint failed`
    /// （实测）—— 要么正常业务写不进去，要么被 `.ok()` 吞成静默丢调用。
    /// 重复同时是 `CODE_DUPLICATE_JOIN_KEY`（`INVALID_ARTIFACT`）的证据来源，
    /// 入库前掐掉等于销毁证据。
    #[test]
    fn duplicate_call_ids_are_recorded_not_rejected() {
        let store = NeobotStore::open(":memory:").expect("open memory db");
        store
            .record_tool_calls("t1", 0, None, &[call("cli-status-1", "set_turn_status")])
            .expect("第一次写");
        store
            .record_tool_calls("t1", 1, None, &[call("cli-status-1", "set_turn_status")])
            .expect("同名第二次写必须成功（否则就是静默丢调用）");
        let rows = store.find_tool_calls_by_id("cli-status-1").expect("find");
        assert_eq!(rows.len(), 2, "重复键两行都要在");
        assert_ne!(
            rows[0].seq, rows[1].seq,
            "代理键必须把两行区分开，否则「哪个结果配哪个调用」无从谈起"
        );
        assert_eq!(rows[0].n, 0);
        assert_eq!(rows[1].n, 1);
        // 重复要能被**读出来**报给检查，而不必把全集拉回内存自己数。
        assert_eq!(
            store.duplicate_call_keys("t1").expect("dups"),
            vec![("cli-status-1".to_owned(), 2)],
            "重复键必须可直读，且只列 ≥2 次的"
        );
    }

    /// 不重复的键**不该**被重复口列出来（防该口退化成「全表清单」）。
    #[test]
    fn unique_call_ids_are_not_reported_as_duplicates() {
        let store = NeobotStore::open(":memory:").expect("open memory db");
        store
            .record_tool_calls(
                "t1",
                0,
                None,
                &[call("call_a", "bash"), call("call_b", "read_file")],
            )
            .expect("record");
        assert!(
            store.duplicate_call_keys("t1").expect("dups").is_empty(),
            "两个不同的键不构成重复"
        );
        assert_eq!(
            store.find_tool_calls_by_id("call_a").expect("find").len(),
            1
        );
        assert!(
            store
                .find_tool_calls_by_id("call_zz")
                .expect("find")
                .is_empty(),
            "查不到的键回空 ⇒ 0 行正是「孤儿结果」的判据"
        );
    }

    /// 跨任务同名也要能被查到（键失去单射性是**全局**性质，限定单任务会藏起来）。
    #[test]
    fn duplicate_detection_spans_tasks() {
        let store = NeobotStore::open(":memory:").expect("open memory db");
        store
            .record_tool_calls("t1", 0, None, &[call("cli-status-1", "set_turn_status")])
            .expect("t1");
        store
            .record_tool_calls("t2", 0, None, &[call("cli-status-1", "set_turn_status")])
            .expect("t2");
        assert_eq!(
            store
                .find_tool_calls_by_id("cli-status-1")
                .expect("find")
                .len(),
            2,
            "跨任务同名同样是键失去单射性"
        );
        assert!(
            store.duplicate_call_keys("t1").expect("dups").is_empty(),
            "单任务内不重复 ⇒ 按任务查时不该报"
        );
    }

    /// ⛔ 空白 `call_id` 拒收：它会与别的空白键互相配平 ⇒ 假配平。
    #[test]
    fn blank_call_id_is_refused() {
        let store = NeobotStore::open(":memory:").expect("open memory db");
        assert!(
            store
                .record_tool_calls("t1", 0, None, &[call("   ", "bash")])
                .is_err(),
            "空白配平键必须拒收（它配不上任何东西，还会和别的空白键假配平）"
        );
        assert!(
            store.list_tool_calls("t1").expect("list").is_empty(),
            "拒收 ⇒ 不得留下半行"
        );
        assert!(
            store
                .record_tool_calls("", 0, None, &[call("call_a", "bash")])
                .is_err(),
            "空 task_id 必须拒收（否则调用侧无从归属到任何任务）"
        );
    }

    /// 空切片 = 本跳没发调用 ⇒ 不写不炸（`turn.tool_calls` 为空是常态）。
    #[test]
    fn empty_call_slice_writes_nothing() {
        let store = NeobotStore::open(":memory:").expect("open memory db");
        assert!(store
            .record_tool_calls("t1", 0, None, &[])
            .expect("空切片不炸")
            .is_empty());
        assert!(store.list_tool_calls("t1").expect("list").is_empty());
    }

    /// **迁移幂等**：同一个库开三次都不得报错、且数据存活。
    ///
    /// 本表用 `CREATE TABLE/INDEX IF NOT EXISTS`（不是 ALTER）⇒ 幂等性由
    /// SQLite 自己保证，第 2/3 次开库是纯 no-op。**顺带把上一位 agent 那条
    /// `ALTER … .ok()` 路径也重跑一遍**：存量库的 `steps` 补列在每次开库时
    /// 都会撞 duplicate column，若那条 `.ok()` 被删/被改成 `?`，这里当场红。
    #[test]
    fn migration_is_idempotent_across_repeated_opens() {
        let path = temp_db("tool-calls-migrate-twice");
        let p = path.to_str().expect("utf8 path");

        let first = NeobotStore::open(p).expect("第一次 open（建表）");
        first
            .record_tool_calls("t1", 0, Some("c9"), &[call("call_a", "bash")])
            .expect("write before reopen");
        drop(first);

        // 断言的是**「只增不减」**而不是一个常数：开库前（第 1 次）已写 1 行，
        //   之后每轮写 1 行 ⇒ 第 N 次开库应恰好看到 **N−1** 行、写完变 **N** 行。
        //   断言常数会与循环自身的写入自相矛盾 ⇒ 那是**测试的错**，不是幂等
        //   被破坏。生产侧此处三次开库零报错、数据零丢失。
        for round in 2..=3 {
            let n = round as usize;
            let store = NeobotStore::open(p)
                .unwrap_or_else(|e| panic!("第 {round} 次 open 不得报错（幂等）：{e}"));
            let rows = store.list_tool_calls("t1").expect("list after reopen");
            assert_eq!(
                rows.len(),
                n - 1,
                "第 {round} 次 open 后数据必须存活，且不多不少（建表语句不得重置它）"
            );
            // `ORDER BY seq` ⇒ `rows[0]` 恒是最早那行 ⇒ 证明「最早的」没被抹掉。
            assert_eq!(rows[0].call_id, "call_a", "最早的配平键必须存活");
            assert_eq!(rows[0].convo_id.as_deref(), Some("c9"));
            // 旧口径仍可读：结果侧那半不能被本次建表碰坏。
            store
                .add_step_with_call("t1", 0, "bash", true, "out", Some("call_a"))
                .expect("steps 侧仍可写");
            assert_eq!(
                store.list_steps("t1").expect("steps")[0]
                    .tool_call_id
                    .as_deref(),
                Some("call_a")
            );
            // 第 2/3 次开库后新写入仍正常（证明表真的在，不是一次性视图）。
            let new_id = format!("call_r{round}");
            store
                .record_tool_calls("t1", round as i64, None, &[call(&new_id, "read_file")])
                .expect("write after reopen");
            // 新行必须**追加**在尾部（`seq` 单调）而不是覆盖旧行。
            let after = store.list_tool_calls("t1").expect("list after write");
            assert_eq!(after.len(), n, "新行必须追加，不得覆盖");
            assert_eq!(after[n - 1].n, round as i64, "跳数原样落库");
            assert_eq!(after[n - 1].call_id, new_id);
        }
        let _ = std::fs::remove_dir_all(path.parent().expect("parent"));
    }

    /// **存量库迁移**：没有 `tool_calls` 表的旧库必须能开，且**旧
    /// `steps`/`messages` 行全部存活**。
    ///
    /// ⛔ 直接用 `rusqlite` 手搓旧 schema，不用 `NeobotStore::open` ——
    ///    后者会自己建表，那样就测不到「表缺失时能否迁移」了。
    #[test]
    fn legacy_db_without_table_migrates_and_keeps_rows() {
        let path = temp_db("tool-calls-legacy");
        {
            let conn = rusqlite::Connection::open(&path).expect("open legacy db");
            conn.execute_batch(
                "CREATE TABLE steps(
                   id INTEGER PRIMARY KEY AUTOINCREMENT, task_id TEXT NOT NULL,
                   n INTEGER NOT NULL, tool TEXT NOT NULL, ok INTEGER NOT NULL,
                   output TEXT NOT NULL);
                 CREATE TABLE tasks(
                   id TEXT PRIMARY KEY, title TEXT NOT NULL, status TEXT NOT NULL,
                   created_at TEXT NOT NULL, updated_at TEXT NOT NULL,
                   claimed_by TEXT, claimed_at TEXT,
                   visibility TEXT NOT NULL DEFAULT 'team',
                   lease_id TEXT, lease_until TEXT,
                   attempts INTEGER NOT NULL DEFAULT 0, error TEXT);
                 CREATE TABLE conversations(
                   id TEXT PRIMARY KEY, kind TEXT NOT NULL, title TEXT NOT NULL,
                   created_at TEXT NOT NULL, updated_at TEXT NOT NULL,
                   muted INTEGER NOT NULL DEFAULT 0);
                 CREATE TABLE messages(
                   id TEXT PRIMARY KEY, convo_id TEXT NOT NULL, role TEXT NOT NULL,
                   text TEXT NOT NULL, created_at TEXT NOT NULL);
                 INSERT INTO steps(task_id,n,tool,ok,output)
                   VALUES('legacy-task',0,'bash',1,'old output');
                 INSERT INTO tasks(id,title,status,created_at,updated_at)
                   VALUES('legacy-task','legacy','done','1970-01-01T00:00:00Z','1970-01-01T00:00:00Z');
                 INSERT INTO conversations(id,kind,title,created_at,updated_at)
                   VALUES('c9','dm','legacy','1970-01-01T00:00:00Z','1970-01-01T00:00:00Z');
                 INSERT INTO messages(id,convo_id,role,text,created_at)
                   VALUES('m1','c9','user','hi','1970-01-01T00:00:00Z');",
            )
            .expect("seed legacy db");
            // 开之前先确认「表真的不在」—— 否则这条测试可能因为建表语句
            // 已被改动而假绿（这就是它必须自己手搓旧 schema 的原因）。
            let has: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master
                     WHERE type='table' AND name='tool_calls'",
                    [],
                    |r| r.get(0),
                )
                .expect("sqlite_master");
            assert_eq!(has, 0, "夹具前提：旧库不该有该表");
        }

        let store = NeobotStore::open(path.to_str().expect("utf8 path"))
            .expect("旧库必须能开（建表不得炸）");

        // 旧行全部存活。
        let steps = store.list_steps("legacy-task").expect("legacy steps");
        assert_eq!(steps.len(), 1, "旧 steps 行必须存活");
        assert_eq!(steps[0].output, "old output");
        assert_eq!(
            steps[0].tool_call_id, None,
            "旧行的结果侧键仍是 NULL（存量语义不被本次改动破坏）"
        );
        assert_eq!(
            store.list_tasks(10).expect("tasks").len(),
            1,
            "旧 tasks 行存活"
        );
        assert_eq!(
            store.list_messages("c9").expect("messages").len(),
            1,
            "旧 messages 行存活"
        );
        // 旧库**没有**任何调用行 —— 调用侧证据是零，不是被编造出来的。
        assert!(
            store
                .list_tool_calls("legacy-task")
                .expect("calls")
                .is_empty(),
            "旧库不该凭空多出调用行"
        );

        // 迁移后新写入正常（证明表真落到旧库上了）。
        store
            .record_tool_calls("legacy-task", 1, None, &[call("call_z", "bash")])
            .expect("write after migration");
        let rows = store
            .list_tool_calls("legacy-task")
            .expect("list after write");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].call_id, "call_z");
        assert_eq!(rows[0].convo_id, None, "迁移不替调用侧编造会话归属");
        drop(store);
        let _ = std::fs::remove_dir_all(path.parent().expect("parent"));
    }
}
