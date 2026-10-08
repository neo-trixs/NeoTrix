//! `nt_store_quota` — 额度窗口快照（N4：`neobot quota --snapshot` 的落点）。
//!
//! 语义：**快照法**而非增量累加 —— 每次按 `window_kind`（daily/weekly/monthly）
//! 从 `ledger` 重算并覆盖该窗口。代价是每次扫描 ledger（单机库足够），
//! 换来的是「历史被改/删/回填」后重算即真值，不会像增量法那样永久漂移。

use super::NeobotStore;
use crate::nt_error::NtBotError;
use chrono::Datelike;
use rusqlite::params;

/// 一行**人工声明**的额度上限（N6.1 P0）。
///
/// ⛔ **这行不是 API 读来的**：上限只能由人填（`neobot quota set`），
/// `source` 列记声明来源（`manual` / 未来某次探针名）⇒ 出表时能分清
/// 「这是人说的」与「这是供应商说的」。**不许把人工值伪装成探针值。**
#[derive(Debug, Clone, PartialEq)]
pub struct QuotaLimit {
    pub key_env: String,
    pub provider: String,
    /// daily | weekly | monthly。
    pub window_kind: String,
    /// 窗口内 token 输入上限（`None` = 未设）。
    pub limit_in: Option<i64>,
    /// 窗口内 token 输出上限（`None` = 未设）。
    pub limit_out: Option<i64>,
    /// 窗口内费用上限 USD（`None` = 未设）。
    pub limit_cost_usd: Option<f64>,
    /// 上限来源（`manual` = 人工声明；其它值代表探针名）。
    pub source: String,
    pub updated_at: String,
}

/// 一行额度窗口。
#[derive(Debug, Clone, PartialEq)]
pub struct QuotaWindow {
    pub key_env: String,
    pub engine: String,
    pub model: String,
    /// daily | weekly | monthly（自由串，但 `snapshot_quota_windows` 只认这三个）。
    pub window_kind: String,
    /// 窗口起点（ISO 日期；weekly 为该周周一，monthly 为该月 1 号）。
    pub window_start: String,
    pub tokens_in: i64,
    pub tokens_out: i64,
    pub cost_usd: f64,
    pub updated_at: String,
}

impl NeobotStore {
    /// 从 `ledger` 重算并覆盖三类窗口（按 `key_env` 维度）。
    ///
    /// 返回本次写入的窗口行数。
    pub fn snapshot_quota_windows(&self) -> Result<usize, NtBotError> {
        // 窗口起点：daily=今天、weekly=本周一、monthly=本月 1 号。
        // 统一 UTC 口径（与 ledger `at` 同源），避免两套时区算出两个窗口。
        let now = chrono::Utc::now();
        let daily = now.format("%Y-%m-%d").to_string();
        let monday = now.date_naive()
            - chrono::Duration::days(i64::from(
                now.weekday().num_days_from_monday(),
            ));
        let weekly = monday.format("%Y-%m-%d").to_string();
        let monthly_start = now.format("%Y-%m-01").to_string();

        let updated = now.to_rfc3339();
        let mut written = 0usize;
        for (kind, window_start, cutoff) in [
            ("daily", daily.as_str(), format!("{}T00:00:00Z", daily)),
            ("weekly", weekly.as_str(), format!("{}T00:00:00Z", weekly)),
            (
                "monthly",
                monthly_start.as_str(),
                format!("{}T00:00:00Z", monthly_start),
            ),
        ] {
            // 每个窗口**重算**（覆盖语义）⇒ 历史上修/删会自愈。
            let rows = {
                let mut stmt = self.conn.prepare(
                    "SELECT COALESCE(NULLIF(key_env,''),'-') AS ke, engine, model,
                       COALESCE(CAST(SUM(in_tokens) AS INTEGER),0),
                       COALESCE(CAST(SUM(out_tokens) AS INTEGER),0),
                       COALESCE(SUM(cost_usd),0.0)
                     FROM ledger WHERE at >= ?1
                     GROUP BY ke, engine, model",
                )?;
                let mut q = stmt.query(params![cutoff])?;
                let mut acc: Vec<(String, String, String, i64, i64, f64)> = Vec::new();
                while let Some(row) = q.next()? {
                    acc.push((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                    ));
                }
                acc
            };
            for (ke, engine, model, ti, to, c) in rows {
                self.conn.execute(
                    "INSERT INTO quota_windows(key_env,engine,model,window_kind,window_start,
                       tokens_in,tokens_out,cost_usd,updated_at)
                     VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)
                     ON CONFLICT(key_env,engine,model,window_kind,window_start)
                     DO UPDATE SET tokens_in=excluded.tokens_in,
                       tokens_out=excluded.tokens_out, cost_usd=excluded.cost_usd,
                       updated_at=excluded.updated_at",
                    params![ke, engine, model, kind, window_start, ti, to, c, updated],
                )?;
                written += 1;
            }
        }
        Ok(written)
    }

    /// 读额度窗口（按 `window_start` 倒序；`kind` 过滤可选）。
    pub fn list_quota_windows(&self, kind: Option<&str>) -> Result<Vec<QuotaWindow>, NtBotError> {
        let mut stmt = self.conn.prepare(
            "SELECT key_env,engine,model,window_kind,window_start,tokens_in,tokens_out,cost_usd,updated_at
             FROM quota_windows
             WHERE (?1 IS NULL OR window_kind = ?1)
             ORDER BY window_start DESC, key_env ASC, engine ASC, model ASC",
        )?;
        let rows = stmt.query_map(params![kind], |r| {
            Ok(QuotaWindow {
                key_env: r.get(0)?,
                engine: r.get(1)?,
                model: r.get(2)?,
                window_kind: r.get(3)?,
                window_start: r.get(4)?,
                tokens_in: r.get(5)?,
                tokens_out: r.get(6)?,
                cost_usd: r.get(7)?,
                updated_at: r.get(8)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }
    /// 降级计数（`status='degraded'` 的账本行），按 `(engine, status)` 聚合。
    ///
    /// 与 `ledger_sums` 分开：降级**不含** token/cost，它回答的是
    /// 「有多少轮的内容被整体丢弃」，和「花了多少」是两个问题。
    pub fn ledger_degraded_counts(&self) -> Result<Vec<(String, i64)>, NtBotError> {
        let mut stmt = self.conn.prepare(
            "SELECT engine, COUNT(*) FROM ledger WHERE status='degraded' GROUP BY engine",
        )?;
        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    /// 写/覆盖一条**人工声明**的额度上限（N6.1 P0）。
    ///
    /// 入库门：`window_kind` 只认 daily/weekly/monthly；`limit_*` 全空 ⇒
    /// **拒收**（一条「什么都没设」的上限行等于噪声，且会让出表说谎）。
    pub fn upsert_quota_limit(&self, limit: &QuotaLimit) -> Result<(), NtBotError> {
        if limit.key_env.trim().is_empty() {
            return Err(NtBotError::Invalid("quota limit key_env is empty".to_owned()));
        }
        if !matches!(limit.window_kind.trim(), "daily" | "weekly" | "monthly") {
            return Err(NtBotError::Invalid(format!(
                "unknown window_kind '{}' (daily|weekly|monthly)",
                limit.window_kind
            )));
        }
        if limit.limit_in.is_none() && limit.limit_out.is_none() && limit.limit_cost_usd.is_none() {
            return Err(NtBotError::Invalid(
                "quota limit with no limit_* set would be noise — set at least one".to_owned(),
            ));
        }
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO quota_limits(key_env,provider,window_kind,limit_in,limit_out,
               limit_cost_usd,source,updated_at)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8)
             ON CONFLICT(key_env,provider,window_kind) DO UPDATE SET
               limit_in=excluded.limit_in, limit_out=excluded.limit_out,
               limit_cost_usd=excluded.limit_cost_usd, source=excluded.source,
               updated_at=excluded.updated_at",
            params![
                limit.key_env.trim(),
                limit.provider.trim(),
                limit.window_kind.trim(),
                limit.limit_in,
                limit.limit_out,
                limit.limit_cost_usd,
                if limit.source.trim().is_empty() { "manual" } else { limit.source.trim() },
                now,
            ],
        )?;
        Ok(())
    }

    pub fn list_quota_limits(&self) -> Result<Vec<QuotaLimit>, NtBotError> {
        let mut stmt = self.conn.prepare(
            "SELECT key_env,provider,window_kind,limit_in,limit_out,limit_cost_usd,source,updated_at
             FROM quota_limits ORDER BY key_env, provider, window_kind",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(QuotaLimit {
                key_env: r.get(0)?,
                provider: r.get(1)?,
                window_kind: r.get(2)?,
                limit_in: r.get(3)?,
                limit_out: r.get(4)?,
                limit_cost_usd: r.get(5)?,
                source: r.get(6)?,
                updated_at: r.get(7)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn remove_quota_limit(
        &self,
        key_env: &str,
        provider: &str,
        window_kind: &str,
    ) -> Result<(), NtBotError> {
        let n = self.conn.execute(
            "DELETE FROM quota_limits WHERE key_env=?1 AND provider=?2 AND window_kind=?3",
            params![key_env.trim(), provider.trim(), window_kind.trim()],
        )?;
        if n == 0 {
            return Err(NtBotError::Store(format!(
                "no such quota limit '{key_env}/{provider}/{window_kind}'"
            )));
        }
        Ok(())
    }

    /// 出表用：**已用**（窗口快照求和）× 上限 ⇒ 剩余/占比。
    ///
    /// 返回 `(已用 in, 已用 out, 已用 cost, 上限行 Option)`；上限缺失 ⇒
    /// `None` ⇒ 调用方必须说「未设上限」，**不许**当成无限或当成 0。
    pub fn quota_used_vs_limit(
        &self,
        key_env: &str,
        provider: &str,
        window_kind: &str,
    ) -> Result<(i64, i64, f64, Option<QuotaLimit>), NtBotError> {
        let windows = self.list_quota_windows(Some(window_kind))?;
        let key = key_env.trim();
        let in_tok: i64 = windows
            .iter()
            .filter(|w| w.key_env == key)
            .map(|w| w.tokens_in)
            .sum();
        let out_tok: i64 = windows
            .iter()
            .filter(|w| w.key_env == key)
            .map(|w| w.tokens_out)
            .sum();
        let cost: f64 = windows.iter().filter(|w| w.key_env == key).map(|w| w.cost_usd).sum();
        let limit = self
            .list_quota_limits()?
            .into_iter()
            .find(|l| l.key_env == key && l.provider.trim() == provider.trim() && l.window_kind == window_kind.trim());
        Ok((in_tok, out_tok, cost, limit))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nt_store::LedgerEntry;

    fn entry(key_env: &str, cost: f64) -> LedgerEntry {
        LedgerEntry {
            id: format!("l-{key_env}-{cost}"),
            at: chrono::Utc::now().to_rfc3339(),
            engine: "http".to_owned(),
            model: "m".to_owned(),
            actor: "bot".to_owned(),
            purpose: "agent-turn".to_owned(),
            in_tokens: 100,
            out_tokens: 50,
            cost_usd: cost,
            measured: true,
            status: "ok".to_owned(),
            latency_ms: 1,
            error: None,
            session_id: None,
            key_env: if key_env.is_empty() { None } else { Some(key_env.to_owned()) },
        }
    }

    #[test]
    fn snapshot_is_idempotent_and_reflects_ledger() {
        let store = NeobotStore::open(":memory:").expect("open");
        store.record_ledger(&entry("K1", 0.02)).expect("record");
        store.record_ledger(&entry("K2", 0.03)).expect("record");
        let n = store.snapshot_quota_windows().expect("snapshot");
        // 2 个 key × 3 个窗口 = 6 行。
        assert_eq!(n, 6, "rows written");
        let first = store.list_quota_windows(Some("daily")).expect("list");
        assert_eq!(first.len(), 2);
        let total: f64 = first.iter().map(|w| w.cost_usd).sum();
        assert!((total - 0.05).abs() < 1e-9, "daily cost = {total}");
        // 幂等：再跑一次仍是 2 行（覆盖而非累加）。
        store.snapshot_quota_windows().expect("snapshot2");
        assert_eq!(store.list_quota_windows(Some("daily")).expect("list").len(), 2);
        let total2: f64 = store.list_quota_windows(Some("daily")).expect("list").iter().map(|w| w.cost_usd).sum();
        assert!((total2 - 0.05).abs() < 1e-9, "快照法不许漂移：{total2}");
    }

    fn limit(key_env: &str, provider: &str, kind: &str, cost: Option<f64>) -> QuotaLimit {
        QuotaLimit {
            key_env: key_env.to_owned(),
            provider: provider.to_owned(),
            window_kind: kind.to_owned(),
            limit_in: None,
            limit_out: None,
            limit_cost_usd: cost,
            source: "manual".to_owned(),
            updated_at: String::new(),
        }
    }

    #[test]
    fn quota_limit_crud_and_validation() {
        let store = NeobotStore::open(":memory:").expect("open");
        // 全空的「上限行」= 噪声，入库即拒（否则出表只能说谎）。
        assert!(store.upsert_quota_limit(&limit("K1", "deepseek", "daily", None)).is_err());
        // 未知窗口拒收。
        assert!(store.upsert_quota_limit(&limit("K1", "deepseek", "hourly", Some(1.0))).is_err());
        // 覆盖同一 (key_env, provider, window_kind) 不新增行。
        store.upsert_quota_limit(&limit("K1", "deepseek", "daily", Some(5.0))).expect("set");
        store.upsert_quota_limit(&limit("K1", "deepseek", "daily", Some(9.0))).expect("set2");
        let all = store.list_quota_limits().expect("list");
        assert_eq!(all.len(), 1, "同键覆盖而非新增");
        assert_eq!(all[0].limit_cost_usd, Some(9.0));
        assert_eq!(all[0].source, "manual");
        store.remove_quota_limit("K1", "deepseek", "daily").expect("rm");
        assert!(store.remove_quota_limit("K1", "deepseek", "daily").is_err());
    }

    /// 三态出表：① 有上限 ⇒ 算剩余；② 无限额 ⇒ 上限为 `None`（不许当无限/当 0）；
    /// ③ 有人为上限但没有任何用量 ⇒ 已用 0。
    #[test]
    fn used_vs_limit_reports_three_states_honestly() {
        let store = NeobotStore::open(":memory:").expect("open");
        store.record_ledger(&entry("K1", 0.02)).expect("record");
        store.snapshot_quota_windows().expect("snapshot");

        // ① 无限额 ⇒ None。
        let (ti, to, cost, none_limit) =
            store.quota_used_vs_limit("K1", "deepseek", "daily").expect("q");
        assert_eq!(ti, 100);
        assert_eq!(to, 50);
        assert!((cost - 0.02).abs() < 1e-9);
        assert!(none_limit.is_none(), "未设上限必须回 None，调用方才可说「未设上限」");

        // ② 有上限 ⇒ 剩余可算。
        store.upsert_quota_limit(&limit("K1", "deepseek", "daily", Some(1.0))).expect("set");
        let (ti2, _to2, cost2, some_limit) =
            store.quota_used_vs_limit("K1", "deepseek", "daily").expect("q2");
        assert_eq!(ti2, ti);
        let lim = some_limit.expect("limit row");
        let remaining = lim.limit_cost_usd.expect("limit") - cost2;
        assert!((remaining - 0.98).abs() < 1e-9, "剩余 = 上限 - 已用：{remaining}");

        // ③ 有上限但没用量（别的 key 用量不串进来）。
        let (ti3, _to3, cost3, _) = store.quota_used_vs_limit("K2", "x", "daily").expect("q3");
        assert_eq!((ti3, cost3), (0, 0.0), "别的 key 的用量不许串到本 key");
    }

    #[test]
    fn degraded_counts_are_separate_from_cost_sums() {
        use crate::nt_output_distill::is_degraded;
        let store = NeobotStore::open(":memory:").expect("open");
        // 降级判据：塌成 `…` 才算；errors-first 压缩（正常）不算。
        assert!(is_degraded("…"));
        assert!(!is_degraded("## errors\nboom\n## tail\nexit=1"));
        store.record_ledger(&entry("K1", 0.02)).expect("cost row");
        let mut deg = entry("K1", 0.0);
        deg.id = "d1".to_owned();
        deg.status = "degraded".to_owned();
        deg.measured = false;
        deg.error = Some("1 tool output(s) collapsed to placeholder".to_owned());
        store.record_ledger(&deg).expect("degraded row");
        assert_eq!(store.ledger_degraded_counts().expect("counts"), vec![("http".to_owned(), 1)]);
        // 费用聚合**不含**降级行（cost=0 也不该混进「花过的钱」的语境之外的口径）。
        let sums = store.ledger_sums().expect("sums");
        assert_eq!(sums.len(), 1);
        assert!((sums[0].4 - 0.02).abs() < 1e-9, "降级行 cost=0 不改变合计");
    }

    #[test]
    fn empty_ledger_yields_no_windows() {
        let store = NeobotStore::open(":memory:").expect("open");
        let n = store.snapshot_quota_windows().expect("snapshot");
        assert_eq!(n, 0);
        assert!(store.list_quota_windows(None).expect("list").is_empty());
    }
}