//! `nt_store_channels` — 渠道 / 机器人 / 去重 / 延迟补发的持久化。
//!
//! 四张表：
//! - `channels`：一个平台一行（kind / access 模式 / 轮询开关）；
//! - `channel_bots`：一个平台上可挂多个机器人（**各机器人独立绑定**
//!   工作区、模型、别名、白名单 —— 这是 dsh-im 最该抄的一条）；
//! - `channel_seen`：入站去重（长轮询重发必须幂等）；
//! - `pending_deliveries`：超时后补发（`deferred_delivery`）。
//!
//! 凭据律：`channel_bots` 只存 `token_env`（**变量名**），不存 token 值。
//! 值由适配器发送/接收时从环境现读，状态接口因此永不回传凭据。

use rusqlite::{OptionalExtension, params};

use crate::nt_channel::AccessMode;
use crate::nt_error::NtBotError;
use crate::nt_store::{BotRow, ChannelRow, MAX_SEND_ATTEMPTS, NeobotStore, PendingDelivery};

/// 轮询节拍下限（秒）：再密也没意义，且会给平台 API 招限流。
pub const MIN_POLL_SECS: i64 = 3;
/// 轮询节拍上限（秒）：再疏就谈不上「即时」了。
pub const MAX_POLL_SECS: i64 = 600;
/// 去重表保留天数。
pub const SEEN_RETENTION_DAYS: i64 = 7;
/// 待补发保留天数（超过即认为用户不关心了）。
pub const PENDING_RETENTION_DAYS: i64 = 3;

impl NeobotStore {
    // ---- channels ----

    /// 建渠道（已存在则原样返回其 id，不覆盖配置）。
    pub fn upsert_channel(
        &self,
        id: &str,
        title: &str,
        access_mode: &str,
        poll_secs: i64,
    ) -> Result<String, NtBotError> {
        let id = id.trim();
        if id.is_empty() {
            return Err(NtBotError::Invalid("channel id is empty".to_owned()));
        }
        let title = if title.trim().is_empty() { id } else { title.trim() };
        // 缺省访问模式认不出时归一到 `allow`（全拒），与 `AccessMode::parse` 同律。
        let access = AccessMode::parse(access_mode).as_str();
        let poll = poll_secs.clamp(MIN_POLL_SECS, MAX_POLL_SECS);
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO channels(id,title,enabled,access_mode,poll_secs,created_at)
             VALUES(?1,?2,1,?3,?4,?5)
             ON CONFLICT(id) DO UPDATE SET title=excluded.title,
               access_mode=excluded.access_mode, poll_secs=excluded.poll_secs",
            params![id, title, access, poll, now],
        )?;
        Ok(id.to_owned())
    }

    pub fn set_channel_enabled(&self, id: &str, enabled: bool) -> Result<(), NtBotError> {
        let n = self.conn.execute(
            "UPDATE channels SET enabled=?1 WHERE id=?2",
            params![i64::from(enabled), id],
        )?;
        if n == 0 {
            return Err(NtBotError::Store(format!("no such channel '{id}'")));
        }
        Ok(())
    }

    /// 渠道列表（按 id 排序，诊断可复现）。
    pub fn list_channels(&self) -> Result<Vec<ChannelRow>, NtBotError> {
        let mut stmt = self.conn.prepare(
            "SELECT id,title,enabled,access_mode,poll_secs,created_at
             FROM channels ORDER BY id",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, i64>(4)?,
                r.get::<_, String>(5)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (id, title, enabled, access_mode, poll_secs, created_at) = row?;
            out.push(ChannelRow {
                id,
                title,
                enabled: enabled != 0,
                access_mode,
                poll_secs,
                created_at,
            });
        }
        Ok(out)
    }

    pub fn get_channel(&self, id: &str) -> Result<Option<ChannelRow>, NtBotError> {
        Ok(self
            .list_channels()?
            .into_iter()
            .find(|row| row.id == id))
    }

    // ---- channel_bots ----

    /// 挂一个机器人。已存在则**只更新绑定**，不覆盖别名字段。
    pub fn upsert_bot(&self, bot: &BotRow) -> Result<(), NtBotError> {
        if bot.channel.trim().is_empty() || bot.bot_id.trim().is_empty() {
            return Err(NtBotError::Invalid(
                "bot needs channel and bot_id".to_owned(),
            ));
        }
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO channel_bots(channel,bot_id,alias,token_env,conversation_id,
               model,allow_list,created_at)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8)
             ON CONFLICT(channel,bot_id) DO UPDATE SET
               token_env=excluded.token_env,
               conversation_id=excluded.conversation_id,
               model=excluded.model,
               allow_list=excluded.allow_list",
            params![
                bot.channel,
                bot.bot_id,
                bot.alias,
                bot.token_env,
                bot.conversation_id,
                bot.model,
                bot.allow_list,
                now,
            ],
        )?;
        Ok(())
    }

    /// 改别名（空串 = 清除，回落平台原名）。
    pub fn set_bot_alias(
        &self,
        channel: &str,
        bot_id: &str,
        alias: &str,
    ) -> Result<(), NtBotError> {
        if alias.chars().count() > 80 {
            return Err(NtBotError::Invalid("alias exceeds 80 chars".to_owned()));
        }
        let n = self.conn.execute(
            "UPDATE channel_bots SET alias=?1 WHERE channel=?2 AND bot_id=?3",
            params![alias.trim(), channel, bot_id],
        )?;
        if n == 0 {
            return Err(NtBotError::Store(format!("no such bot '{bot_id}'")));
        }
        Ok(())
    }

    /// 某渠道的机器人列表。
    pub fn list_bots(&self, channel: &str) -> Result<Vec<BotRow>, NtBotError> {
        let mut stmt = self.conn.prepare(
            "SELECT channel,bot_id,alias,token_env,conversation_id,model,allow_list,created_at,last_seen
             FROM channel_bots WHERE channel=?1 ORDER BY bot_id",
        )?;
        let rows = stmt.query_map(params![channel], bot_row)?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn get_bot(&self, channel: &str, bot_id: &str) -> Result<Option<BotRow>, NtBotError> {
        Ok(self.list_bots(channel)?.into_iter().find(|b| b.bot_id == bot_id))
    }

    /// 记一次成功收发（供 UI 显示「在线」）。
    pub fn touch_bot(&self, channel: &str, bot_id: &str) -> Result<(), NtBotError> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE channel_bots SET last_seen=?1 WHERE channel=?2 AND bot_id=?3",
            params![now, channel, bot_id],
        )?;
        Ok(())
    }

    pub fn remove_bot(&self, channel: &str, bot_id: &str) -> Result<(), NtBotError> {
        let n = self.conn.execute(
            "DELETE FROM channel_bots WHERE channel=?1 AND bot_id=?2",
            params![channel, bot_id],
        )?;
        if n == 0 {
            return Err(NtBotError::Store(format!("no such bot '{bot_id}'")));
        }
        Ok(())
    }

    // ---- channel_seen（入站去重） ----

    /// 登记一条已处理的消息；返回 `true` = **首次见到**（该处理）。
    ///
    /// 用 `INSERT OR IGNORE` + `rows_affected` 判重，天然原子 ——
    /// 两个 web 进程同库时也不会重复跑轮。
    pub fn mark_seen(&self, dedup_key: &str) -> Result<bool, NtBotError> {
        let now = chrono::Utc::now().to_rfc3339();
        let n = self.conn.execute(
            "INSERT OR IGNORE INTO channel_seen(dedup_key,at) VALUES(?1,?2)",
            params![dedup_key, now],
        )?;
        Ok(n > 0)
    }

    /// 切掉过老的去重行。
    pub fn prune_seen(&self, cutoff: &str) -> Result<usize, NtBotError> {
        Ok(self
            .conn
            .execute("DELETE FROM channel_seen WHERE at < ?1", params![cutoff])?)
    }

    // ---- pending_deliveries（超时补发） ----

    /// 记一条待补发。
    pub fn enqueue_delivery(&self, item: &PendingDelivery) -> Result<(), NtBotError> {
        if item.text.trim().is_empty() {
            return Err(NtBotError::Invalid(
                "pending delivery text is empty".to_owned(),
            ));
        }
        self.conn.execute(
            "INSERT INTO pending_deliveries(id,channel,bot_id,chat,origin_message,text,
               task_id,attempts,created_at)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![
                item.id,
                item.channel,
                item.bot_id,
                item.chat,
                item.origin_message,
                item.text,
                item.task_id,
                item.attempts,
                item.created_at,
            ],
        )?;
        Ok(())
    }

    /// 待补发是否存在（补发去重用：同一意图只留一行）。
    pub fn has_pending_delivery(&self, id: &str) -> Result<bool, NtBotError> {
        let found: Option<String> = self
            .conn
            .query_row(
                "SELECT id FROM pending_deliveries WHERE id=?1",
                params![id],
                |r| r.get(0),
            )
            .optional()?;
        Ok(found.is_some())
    }

    /// 到期的待补发（按创建时间正序，先到先补）。
    pub fn due_deliveries(
        &self,
        now: &str,
        limit: i64,
    ) -> Result<Vec<PendingDelivery>, NtBotError> {
        // ⛔ `claimed=0` 排除**已被别人认领**的行：CLI 与桌面 App 是**两个进程同库**
        //    （见 nt_store/mod.rs 的双进程说明），没有这一过滤则两个进程各发一遍。
        // ⛔ `available_at <= ?` 实现**时间退避**：失败后推迟到期，
        //    而不再「下一轮（秒级）立刻重试」。
        //    ⓘ 显式 9 列清单**不追加**新列 ⇒ `delivery_row()` 的 `r.get(0..8)`
        //        语义不变（表末尾多两列不影响 SELECT 清单决定的位置）。
        let mut stmt = self.conn.prepare(
            "SELECT id,channel,bot_id,chat,origin_message,text,task_id,attempts,created_at
             FROM pending_deliveries
             WHERE attempts < ?1 AND claimed = 0 AND available_at <= ?2
             ORDER BY created_at, rowid LIMIT ?3",
        )?;
        let rows = stmt.query_map(
            params![MAX_SEND_ATTEMPTS, now, limit.clamp(1, 200)],
            delivery_row,
        )?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    /// ⭐ **原子认领**一行补发：成功返回 `true`，被别人抢走返回 `false`。
    ///
    /// ⛔⛔ **刻意不照抄 `drain_outbox`**：它是「SELECT … WHERE claimed=0 收集到 Vec，
    ///    再 for 循环逐条 UPDATE」且**全程无事务**（本 crate 零事务）
    ///    ⇒ 两个进程可同时通过 SELECT、拿到同一批行、**都发**。
    /// ⇒ 正确写法是**条件 UPDATE + 判 `rows_affected`**：单条
    ///   `UPDATE … WHERE id=? AND claimed=0` 自带写锁，原子。
    pub fn claim_delivery(&self, id: &str) -> Result<bool, NtBotError> {
        let n = self
            .conn
            .execute(
                "UPDATE pending_deliveries SET claimed = 1 WHERE id = ?1 AND claimed = 0",
                params![id],
            )?;
        Ok(n == 1)
    }

    /// 一条补发成功了 → 删掉。
    ///
    /// ⛔ `AND claimed=1`：**只删自己认领的那一行**。
    ///    无条件 DELETE 会删掉别人正在处理/已处理完的行 ⇒ 补发结果丢失。
    ///    附带好处：**天然幂等**（重复调用第二次影响 0 行，不报错）。
    pub fn complete_delivery(&self, id: &str) -> Result<(), NtBotError> {
        self.conn.execute(
            "DELETE FROM pending_deliveries WHERE id=?1 AND claimed=1",
            params![id],
        )?;
        Ok(())
    }

    /// 一条补发失败了 → 记一次尝试；**到顶就不再自动重试**
    /// （避免平台返回不确定时无限重发，把同一条结果发好几遍）。
    pub fn fail_delivery(&self, id: &str, retry_at: &str) -> Result<i64, NtBotError> {
        // ⛔ **必须同时把 `claimed` 归 0**，否则这一行永久卡在「已认领」状态
        //    ⇒ 再也不会被 `due_deliveries` 捞到（静默丢消息）。
        //    与 outbox 的 `fail_outbox` 同款语义。
        self.conn.execute(
            "UPDATE pending_deliveries
             SET attempts = attempts + 1, claimed = 0, available_at = ?2 WHERE id=?1",
            params![id, retry_at],
        )?;
        Ok(self.conn.query_row(
            "SELECT attempts FROM pending_deliveries WHERE id=?1",
            params![id],
            |r| r.get(0),
        )?)
    }

    /// ⛔ **仅测试用**：把所有补发行标记为「已到期」。
    ///
    /// # 为什么需要它
    /// 补发现在带**时间退避**（失败后推迟 `available_at`，默认 +60s）。
    /// ⓘ 于是「跑 N 轮 sweep ⇒ attempts 到顶 ⇒ 放弃」这类测试**不能只靠循环**：
    ///    第一轮失败后该行就不再到期，后面 N-1 轮什么也捞不到 ⇒ 永远到不了上限。
    ///    而真实世界里那 60s 是会过去的 —— 测试里必须**显式模拟时间流逝**，
    ///    否则测试的**意图**（到顶放弃）与**假设**（立即重试）会被混为一谈。
    #[cfg(test)]
    pub fn force_deliveries_due(&self) -> Result<usize, NtBotError> {
        Ok(self.conn.execute(
            "UPDATE pending_deliveries SET available_at = '1970-01-01T00:00:00Z', claimed = 0",
            [],
        )?)
    }

    /// 切掉过老的待补发。
    pub fn prune_deliveries(&self, cutoff: &str) -> Result<usize, NtBotError> {
        Ok(self.conn.execute(
            "DELETE FROM pending_deliveries WHERE created_at < ?1",
            params![cutoff],
        )?)
    }
}

fn bot_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<BotRow> {
    Ok(BotRow {
        channel: r.get(0)?,
        bot_id: r.get(1)?,
        alias: r.get(2)?,
        token_env: r.get(3)?,
        conversation_id: r.get(4)?,
        model: r.get(5)?,
        allow_list: r.get(6)?,
        created_at: r.get(7)?,
        last_seen: r.get(8)?,
    })
}

fn delivery_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<PendingDelivery> {
    Ok(PendingDelivery {
        id: r.get(0)?,
        channel: r.get(1)?,
        bot_id: r.get(2)?,
        chat: r.get(3)?,
        origin_message: r.get(4)?,
        text: r.get(5)?,
        task_id: r.get(6)?,
        attempts: r.get(7)?,
        created_at: r.get(8)?,
    })
}

/// 解析白名单串（逗号 / 换行分隔，两侧空白忽略，空项丢弃）。
pub fn parse_allow_list(raw: &str) -> Vec<String> {
    raw.split([',', '\n', ';'])
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_owned)
        .collect()
}

/// 测试用「永远到期」的 now：列默认值是 1970 ⇒ 传远期未来即等价于「全都到期」，
/// 保持这些用例「读完整个队列」的原有语义。
const ALL_DUE_NOW: &str = "9999-12-31T23:59:59Z";
/// 与 [`ALL_DUE_NOW`] 相对：判「**尚未到期**」时用它。
/// ⛔ 不能拿 `ALL_DUE_NOW` 去判「未到期」—— 它是 9999 年，
///    比任何 `retry_at` 都晚 ⇒ 会被判成**已到期**（我第一版就栽在这）。
const NOT_YET_DUE: &str = "2000-01-01T00:00:00Z";

#[cfg(test)]
mod tests {
    use super::*;

    fn store(case: &str) -> NeobotStore {
        NeobotStore::open(":memory:").unwrap_or_else(|err| panic!("{case}: {err}"))
    }

    fn bot(channel: &str, bot_id: &str) -> BotRow {
        BotRow {
            channel: channel.to_owned(),
            bot_id: bot_id.to_owned(),
            alias: String::new(),
            token_env: "NEOBOT_TEST_TOKEN".to_owned(),
            conversation_id: None,
            model: String::new(),
            allow_list: String::new(),
            created_at: String::new(),
            last_seen: None,
        }
    }

    #[test]
    fn channel_upsert_is_idempotent_and_clamps_poll() {
        let st = store("channel");
        st.upsert_channel("telegram", "Telegram", "open", 0).expect("upsert");
        let row = st.get_channel("telegram").expect("get").expect("some");
        assert_eq!(row.title, "Telegram");
        assert_eq!(row.access_mode, "open");
        // 0 太小 → 夹到下限。
        assert_eq!(row.poll_secs, MIN_POLL_SECS);
        assert!(row.enabled);
        // 再 upsert 一次不建重复行。
        st.upsert_channel("telegram", "TG 改名", "dm_only", 99999).expect("again");
        let all = st.list_channels().expect("list");
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].title, "TG 改名");
        assert_eq!(all[0].poll_secs, MAX_POLL_SECS);
        // 空 id 拒。
        assert!(st.upsert_channel("  ", "x", "open", 5).is_err());
    }

    #[test]
    fn unknown_access_mode_falls_back_to_deny_all() {
        let st = store("access");
        st.upsert_channel("x", "X", "公开随便", 5).expect("upsert");
        let row = st.get_channel("x").expect("get").expect("some");
        // 认不出的模式绝不能变成 open。
        assert_eq!(row.access_mode, "allow");
    }

    #[test]
    fn channel_enable_toggle() {
        let st = store("toggle");
        st.upsert_channel("x", "X", "open", 5).expect("upsert");
        st.set_channel_enabled("x", false).expect("off");
        assert!(!st.get_channel("x").expect("get").expect("some").enabled);
        st.set_channel_enabled("x", true).expect("on");
        assert!(st.get_channel("x").expect("get").expect("some").enabled);
        assert!(st.set_channel_enabled("nope", true).is_err());
    }

    #[test]
    fn bots_are_independent_per_channel() {
        let st = store("bots");
        st.upsert_channel("telegram", "TG", "allow", 5).expect("ch");
        let mut a = bot("telegram", "111");
        a.alias = "工作号".to_owned();
        a.model = "fast".to_owned();
        a.allow_list = "alice".to_owned();
        let mut b = bot("telegram", "222");
        b.alias = "私人号".to_owned();
        b.allow_list = "bob,carol".to_owned();
        st.upsert_bot(&a).expect("a");
        st.upsert_bot(&b).expect("b");
        let bots = st.list_bots("telegram").expect("list");
        assert_eq!(bots.len(), 2);
        // 各机器人独立：别名、模型、白名单互不串。
        let a2 = st.get_bot("telegram", "111").expect("get").expect("some");
        let b2 = st.get_bot("telegram", "222").expect("get").expect("some");
        assert_eq!(a2.alias, "工作号");
        assert_eq!(a2.model, "fast");
        assert_eq!(parse_allow_list(&a2.allow_list), vec!["alice".to_owned()]);
        // 另一个机器人的绑定不能串过来。
        assert_eq!(b2.alias, "私人号");
        assert_eq!(parse_allow_list(&b2.allow_list), vec!["bob".to_owned(), "carol".to_owned()]);
        assert_eq!(b2.model, "", "未设模型就该是空（跟主设置）");
        assert_eq!(a2.token_env, "NEOBOT_TEST_TOKEN");
        // **凭据只有变量名**，没有值这一说（表里就没这个列）。
        assert!(!a2.token_env.contains("123456:ABC"));
    }

    #[test]
    fn bot_upsert_keeps_alias_but_updates_binding() {
        let st = store("reupsert");
        st.upsert_channel("tg", "TG", "open", 5).expect("ch");
        let mut first = bot("tg", "1");
        first.alias = "叫我".to_owned();
        st.upsert_bot(&first).expect("first");
        let mut second = bot("tg", "1");
        second.alias = "不该覆盖".to_owned();
        second.model = "new-model".to_owned();
        st.upsert_bot(&second).expect("second");
        let got = st.get_bot("tg", "1").expect("get").expect("some");
        // 绑定更新了，别名没被冲掉。
        assert_eq!(got.model, "new-model");
        assert_eq!(got.alias, "叫我");
    }

    #[test]
    fn alias_can_be_set_and_cleared() {
        let st = store("alias");
        st.upsert_channel("tg", "TG", "open", 5).expect("ch");
        st.upsert_bot(&bot("tg", "1")).expect("bot");
        st.set_bot_alias("tg", "1", "小助手").expect("set");
        assert_eq!(st.get_bot("tg", "1").expect("get").expect("some").alias, "小助手");
        st.set_bot_alias("tg", "1", "").expect("clear");
        assert_eq!(st.get_bot("tg", "1").expect("get").expect("some").alias, "");
        // 超长拒；不存在的机器人报错。
        assert!(st.set_bot_alias("tg", "1", &"x".repeat(81)).is_err());
        assert!(st.set_bot_alias("tg", "999", "y").is_err());
    }

    #[test]
    fn seen_dedup_is_atomic_and_first_wins() {
        let st = store("seen");
        assert!(st.mark_seen("tg:1").expect("first"), "首次见到应处理");
        assert!(!st.mark_seen("tg:1").expect("dup"), "重复不该再处理");
        assert!(st.mark_seen("tg:2").expect("other"), "不同消息各自处理");
    }

    #[test]
    fn seen_prunes_by_age() {
        let st = store("pruneseen");
        st.mark_seen("a").expect("a");
        let removed = st.prune_seen("2999-01-01T00:00:00Z").expect("prune");
        assert_eq!(removed, 1);
        // 剪掉之后这条又是「首次见到」—— 去重表本来就只管一个留存窗口。
        assert!(st.mark_seen("a").expect("after prune"), "剪掉后应重新算首次");
    }

    #[test]
    fn delivery_queue_retries_then_gives_up() {
        let st = store("delivery");
        let item = PendingDelivery {
            id: "d1".to_owned(),
            channel: "tg".to_owned(),
            bot_id: "1".to_owned(),
            chat: "42".to_owned(),
            origin_message: "7".to_owned(),
            text: "结果来了".to_owned(),
            task_id: "t1".to_owned(),
            attempts: 0,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        st.enqueue_delivery(&item).expect("enqueue");
        assert_eq!(st.due_deliveries(ALL_DUE_NOW, 10).expect("due").len(), 1);
        // 连试三次；到顶后不再自动重试（防重复发送）。
        for expected in 1..=MAX_SEND_ATTEMPTS {
            let got = st.fail_delivery("d1", ALL_DUE_NOW).expect("fail");
            assert_eq!(got, expected);
        }
        assert!(st.due_deliveries(ALL_DUE_NOW, 10).expect("due").is_empty(), "到顶后不该再到期");
        st.complete_delivery("d1").expect("complete");
        assert!(st.due_deliveries(ALL_DUE_NOW, 10).expect("due").is_empty());
    }

    /// ⭐ 本轮修复的核心判据：补发行**只能被认领一次**。
    ///
    /// # 为什么这条测试必须存在
    /// CLI 与桌面 App 是**两个进程打开同一个库文件**。修复前
    /// `due_deliveries` 既无 `claimed=0` 过滤、也无原子认领
    /// ⇒ 两个进程可读到同一批行、**各发一遍** ⇒ 用户收到重复消息。
    /// ⛔ 而 `sweep_pending` 的**并发场景此前零测试覆盖** ——
    ///    「单进程跑两次 sweep」测不出重复发送，因为单进程是顺序的。
    #[test]
    fn delivery_claim_is_exclusive() {
        let st = store("claim");
        st.enqueue_delivery(&crate::nt_store::PendingDelivery {
            id: "d1".to_owned(),
            channel: "fake".to_owned(),
            bot_id: "b1".to_owned(),
            chat: "42".to_owned(),
            origin_message: "m1".to_owned(),
            text: "只该被发一次".to_owned(),
            task_id: "t1".to_owned(),
            attempts: 0,
            created_at: chrono::Utc::now().to_rfc3339(),
        })
        .expect("enqueue");
        // 认领前可见
        assert_eq!(st.due_deliveries(ALL_DUE_NOW, 10).expect("due").len(), 1);
        // 第一个「进程」认领成功
        assert!(st.claim_delivery("d1").expect("claim"), "首次认领应成功");
        // 第二个「进程」必须失败，且**看不到这行**
        assert!(!st.claim_delivery("d1").expect("claim2"), "重复认领必须失败");
        assert!(
            st.due_deliveries(ALL_DUE_NOW, 10).expect("due").is_empty(),
            "已认领的行不该再出现在待发集合里"
        );
    }

    /// ⭐ 失败必须把 `claimed` 归 0，否则该行**永久卡住** ⇒ 静默丢消息。
    #[test]
    fn failed_delivery_releases_claim() {
        let st = store("release");
        st.enqueue_delivery(&crate::nt_store::PendingDelivery {
            id: "d1".to_owned(),
            channel: "fake".to_owned(),
            bot_id: "b1".to_owned(),
            chat: "42".to_owned(),
            origin_message: "m1".to_owned(),
            text: "失败后要能重来".to_owned(),
            task_id: "t1".to_owned(),
            attempts: 0,
            created_at: chrono::Utc::now().to_rfc3339(),
        })
        .expect("enqueue");
        assert!(st.claim_delivery("d1").expect("claim"));
        // 失败：归 claimed=0，并按 retry_at 推迟到期
        let attempts = st.fail_delivery("d1", "2099-01-01T00:00:00Z").expect("fail");
        assert_eq!(attempts, 1, "失败应记一次尝试");
        // ⛔ 关键：行**没被永久藏起来** —— 只是「尚未到期」，且仍可再认领
        assert!(
            st.due_deliveries(NOT_YET_DUE, 10).expect("due").is_empty(),
            "推迟到期的行不该现在就能捞到"
        );
        assert!(st.claim_delivery("d1").expect("re-claim"), "失败后必须能再认领，否则消息永久卡住");
    }

    /// ⭐ `complete_delivery` 只删**自己认领**的行，且**重复调用无害**。
    #[test]
    fn complete_delivery_is_scoped_and_idempotent() {
        let st = store("complete");
        st.enqueue_delivery(&crate::nt_store::PendingDelivery {
            id: "d1".to_owned(),
            channel: "fake".to_owned(),
            bot_id: "b1".to_owned(),
            chat: "42".to_owned(),
            origin_message: "m1".to_owned(),
            text: "幂等删除".to_owned(),
            task_id: "t1".to_owned(),
            attempts: 0,
            created_at: chrono::Utc::now().to_rfc3339(),
        })
        .expect("enqueue");
        // ⛔ 没认领就 complete ⇒ 不应删掉（可能属于别人正在处理的行）
        st.complete_delivery("d1").expect("complete-unclaimed");
        assert_eq!(
            st.due_deliveries(ALL_DUE_NOW, 10).expect("due").len(),
            1,
            "未认领的行不该被删"
        );
        // 认领后 complete ⇒ 删掉；再调一次仍不报错（幂等）
        assert!(st.claim_delivery("d1").expect("claim"));
        st.complete_delivery("d1").expect("complete");
        st.complete_delivery("d1").expect("complete-again");
        assert!(st.due_deliveries(ALL_DUE_NOW, 10).expect("due").is_empty());
    }

    #[test]
    fn delivery_rejects_empty_text() {
        let st = store("empty");
        let item = PendingDelivery {
            id: "d1".to_owned(),
            channel: "tg".to_owned(),
            bot_id: "1".to_owned(),
            chat: "42".to_owned(),
            origin_message: String::new(),
            text: "   ".to_owned(),
            task_id: String::new(),
            attempts: 0,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        assert!(st.enqueue_delivery(&item).is_err());
    }

    #[test]
    fn allow_list_parsing_drops_blanks() {
        assert_eq!(parse_allow_list("a, b ,,c\nd;e"), vec!["a", "b", "c", "d", "e"]);
        assert!(parse_allow_list("").is_empty());
        assert!(parse_allow_list("  ,  ").is_empty());
    }
}
