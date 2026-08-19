//! # AccountPool — Provider 账户池 (open-kritt P7 吸收)
//!
//! 多个 provider 登录账户的健康池化选择层。吸收自:
//! - notes/absorption-20260817-open-kritt.md P7 (workspace.py:1381-1502, 1505-1555):
//!   round-robin 选择跳过 rate-limit/不健康账户 (`provider_home_for_job`);
//!   rate-limit 账户检疫 + 冷却后自动恢复 (`_reconcile_provider_account_limits`);
//!   每账户并发租约 (ENGINE_WORKERS_PER_ACCOUNT=15)。
//!
//! R-P42: 作为 GatewayV2 的选择层接线, 不建平行 provider 系统 —
//! 池只负责"选哪个账户", 实际调用仍走既有 provider 调用路径。

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

/// 账户健康状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountHealth {
    /// 健康 — 参与选择。
    Healthy,
    /// 不健康 (连续失败阈值内) — 仍可尝试但优先级低。
    Unhealthy,
    /// 检疫中 (rate-limited / 连续失败超阈) — 冷却期内不参与选择。
    Quarantined,
}

/// 单账户运行时状态。
#[derive(Debug, Clone)]
struct AccountState {
    provider: String,
    health: AccountHealth,
    in_flight: usize,
    max_concurrent: usize,
    consecutive_failures: u32,
    quarantine_until: Option<Instant>,
    total_calls: u64,
    total_errors: u64,
}

/// 池配置。
#[derive(Debug, Clone)]
pub struct AccountPoolConfig {
    /// 限流检疫默认冷却时长。
    pub quarantine_cooldown: Duration,
    /// 单账户默认并发上限 (ENGINE_WORKERS_PER_ACCOUNT=15, P7 evidence)。
    pub default_max_concurrent: usize,
    /// 连续失败多少次后标记 Unhealthy。
    pub unhealthy_failure_threshold: u32,
}

impl Default for AccountPoolConfig {
    fn default() -> Self {
        Self {
            quarantine_cooldown: Duration::from_secs(60),
            default_max_concurrent: 15,
            unhealthy_failure_threshold: 3,
        }
    }
}

/// 选择失败原因。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountPoolError {
    /// 指定 provider 无任何已注册账户。
    NoAccounts(String),
    /// 所有账户检疫中/不健康/并发饱和 — 返回错误而非 panic。
    NoHealthyAccount(String),
    /// 单账户并发租约已达上限 (in_flight == max_concurrent)。
    Saturated(String),
}

impl std::fmt::Display for AccountPoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoAccounts(p) => write!(f, "no accounts registered for provider '{p}'"),
            Self::NoHealthyAccount(p) => {
                write!(f, "no healthy account available for provider '{p}'")
            }
            Self::Saturated(a) => write!(f, "account '{a}' at concurrency cap"),
        }
    }
}

impl std::error::Error for AccountPoolError {}

/// 并发租约 — RAII: 创建时占用 in_flight 槽, Drop 时释放。
/// 对应 P7 `provider_account_lease` (per-account concurrency gate)。
#[derive(Debug)]
pub struct AccountLease {
    state: Arc<RwLock<HashMap<String, AccountState>>>,
    name: String,
}

impl AccountLease {
    pub fn account_name(&self) -> &str {
        &self.name
    }
}

impl Drop for AccountLease {
    fn drop(&mut self) {
        if let Ok(mut acc) = self.state.write() {
            if let Some(s) = acc.get_mut(&self.name) {
                s.in_flight = s.in_flight.saturating_sub(1);
            }
        }
    }
}

/// Provider 账户池 — 健康感知 round-robin 选择 + 限流检疫 + 并发租约。
pub struct AccountPool {
    accounts: Arc<RwLock<HashMap<String, AccountState>>>,
    cursor: AtomicUsize,
    config: AccountPoolConfig,
}

impl AccountPool {
    pub fn new(config: AccountPoolConfig) -> Self {
        Self {
            accounts: Arc::new(RwLock::new(HashMap::new())),
            cursor: AtomicUsize::new(0),
            config,
        }
    }

    /// 注册账户 (幂等)。同名账户更新 max_concurrent 并复位为健康。
    pub fn register(&self, provider: &str, name: &str, max_concurrent: usize) {
        let mut acc = self.accounts.write().unwrap_or_else(|e| e.into_inner());
        acc.insert(
            name.to_string(),
            AccountState {
                provider: provider.to_string(),
                health: AccountHealth::Healthy,
                in_flight: 0,
                max_concurrent: max_concurrent.max(1),
                consecutive_failures: 0,
                quarantine_until: None,
                total_calls: 0,
                total_errors: 0,
            },
        );
    }

    /// 以默认并发上限注册。
    pub fn register_default(&self, provider: &str, name: &str) {
        self.register(provider, name, self.config.default_max_concurrent);
    }

    /// 取消注册。
    pub fn unregister(&self, name: &str) -> bool {
        self.accounts
            .write()
            .map(|mut acc| acc.remove(name).is_some())
            .unwrap_or(false)
    }

    /// 账户是否存在。
    pub fn contains(&self, name: &str) -> bool {
        self.accounts.read().map(|a| a.contains_key(name)).unwrap_or(false)
    }

    /// 健康账户计数 (指定 provider)。
    pub fn healthy_count(&self, provider: &str) -> usize {
        self.accounts
            .read()
            .map(|a| {
                a.values()
                    .filter(|s| s.provider == provider && s.health == AccountHealth::Healthy)
                    .count()
            })
            .unwrap_or(0)
    }

    /// 检疫中账户计数。
    pub fn quarantined_count(&self, provider: &str) -> usize {
        self.accounts
            .read()
            .map(|a| {
                a.values()
                    .filter(|s| s.provider == provider && s.health == AccountHealth::Quarantined)
                    .count()
            })
            .unwrap_or(0)
    }

    pub fn total_accounts(&self) -> usize {
        self.accounts.read().map(|a| a.len()).unwrap_or(0)
    }

    /// 查询单账户健康状态。
    pub fn health_of(&self, name: &str) -> Option<AccountHealth> {
        self.accounts
            .read()
            .ok()
            .and_then(|a| a.get(name).map(|s| s.health))
    }

    pub fn in_flight_of(&self, name: &str) -> Option<usize> {
        self.accounts
            .read()
            .ok()
            .and_then(|a| a.get(name).map(|s| s.in_flight))
    }

    /// 检疫 (rate-limit 触发): 冷却期内不参与选择。
    pub fn quarantine(&self, name: &str) {
        let mut acc = self.accounts.write().unwrap_or_else(|e| e.into_inner());
        if let Some(s) = acc.get_mut(name) {
            s.health = AccountHealth::Quarantined;
            s.quarantine_until = Some(Instant::now() + self.config.quarantine_cooldown);
            s.consecutive_failures = s.consecutive_failures.saturating_add(1);
            s.total_errors += 1;
        }
    }

    /// 记录成功。
    pub fn record_success(&self, name: &str) {
        let mut acc = self.accounts.write().unwrap_or_else(|e| e.into_inner());
        if let Some(s) = acc.get_mut(name) {
            s.total_calls += 1;
            s.consecutive_failures = 0;
            s.health = AccountHealth::Healthy;
            s.quarantine_until = None;
        }
    }

    /// 记录失败 — 连续失败超阈则标记 Unhealthy。
    pub fn record_failure(&self, name: &str) {
        let mut acc = self.accounts.write().unwrap_or_else(|e| e.into_inner());
        if let Some(s) = acc.get_mut(name) {
            s.total_errors += 1;
            s.total_calls += 1;
            s.consecutive_failures = s.consecutive_failures.saturating_add(1);
            if s.consecutive_failures >= self.config.unhealthy_failure_threshold
                && s.health == AccountHealth::Healthy
            {
                s.health = AccountHealth::Unhealthy;
            }
        }
    }

    /// 手动恢复账户 (冷却未到也可显式恢复)。
    pub fn restore(&self, name: &str) {
        let mut acc = self.accounts.write().unwrap_or_else(|e| e.into_inner());
        if let Some(s) = acc.get_mut(name) {
            s.health = AccountHealth::Healthy;
            s.quarantine_until = None;
            s.consecutive_failures = 0;
        }
    }

    /// 调和 (P7 `_reconcile_provider_account_limits`): 冷却到期的检疫账户自动恢复。
    /// 返回恢复数量。
    pub fn reconcile(&self) -> usize {
        let now = Instant::now();
        let mut acc = self.accounts.write().unwrap_or_else(|e| e.into_inner());
        let mut restored = 0;
        for s in acc.values_mut() {
            if s.health == AccountHealth::Quarantined {
                if let Some(until) = s.quarantine_until {
                    if until <= now {
                        s.health = AccountHealth::Healthy;
                        s.quarantine_until = None;
                        restored += 1;
                    }
                }
            }
        }
        restored
    }

    /// 池健康检测 (C5 自愈):
    /// (a) 池内不应存在已失效/不健康账户的 active lease (health != Healthy 时 in_flight 必须为 0);
    /// (b) 所有账户 in_flight 不超并发上限 (租约计数与内部状态一致)。
    pub fn is_healthy(&self) -> bool {
        let acc = match self.accounts.read() {
            Ok(a) => a,
            Err(_) => return false,
        };
        acc.values().all(|s| {
            (s.health == AccountHealth::Healthy || s.in_flight == 0)
                && s.in_flight <= s.max_concurrent
        })
    }

    /// 驱逐不健康账户 (health == Unhealthy): 移除账户并随之释放其 lease
    /// (in_flight 槽随账户消失)。返回被驱逐账户名列表; 调用后 is_healthy() 为 true。
    pub fn evict_unhealthy(&mut self) -> Vec<String> {
        let mut acc = self.accounts.write().unwrap_or_else(|e| e.into_inner());
        let evicted: Vec<String> = acc.iter()
            .filter(|(_, s)| s.health == AccountHealth::Unhealthy)
            .map(|(name, _)| name.clone())
            .collect();
        for name in &evicted {
            acc.remove(name);
        }
        evicted
    }

    /// 健康感知 round-robin 选择并获取并发租约。
    ///
    /// 跳过: 检疫中且冷却未到 / Unhealthy / 并发已达上限 (in_flight >= max_concurrent)。
    /// 所有候选不可用 → 返回错误 (NoHealthyAccount), 不 panic。
    pub fn select(&self, provider: &str) -> Result<AccountLease, AccountPoolError> {
        let now = Instant::now();
        let mut acc = self.accounts.write().unwrap_or_else(|e| e.into_inner());
        if acc.is_empty() {
            return Err(AccountPoolError::NoAccounts(provider.to_string()));
        }
        // 冷却到期自动恢复 (P7 reconcile 语义: 较新的健康观察放行)
        for s in acc.values_mut() {
            if s.health == AccountHealth::Quarantined {
                if let Some(until) = s.quarantine_until {
                    if until <= now {
                        s.health = AccountHealth::Healthy;
                        s.quarantine_until = None;
                    }
                }
            }
        }
        let mut eligible: Vec<String> = Vec::new();
        for (name, s) in acc.iter() {
            if s.provider != provider {
                continue;
            }
            if s.health != AccountHealth::Healthy {
                continue;
            }
            if s.in_flight >= s.max_concurrent {
                continue;
            }
            eligible.push(name.clone());
        }
        if eligible.is_empty() {
            return Err(AccountPoolError::NoHealthyAccount(provider.to_string()));
        }
        let idx = self.cursor.fetch_add(1, Ordering::Relaxed) % eligible.len();
        let chosen = eligible[idx].clone();
        if let Some(s) = acc.get_mut(&chosen) {
            s.in_flight += 1;
            s.total_calls += 1;
        }
        drop(acc);
        Ok(AccountLease {
            state: Arc::clone(&self.accounts),
            name: chosen,
        })
    }

    /// 尝试单独获取某账户租约 (显式账户, 不参与 round-robin)。并发饱和 → Saturated 错误。
    pub fn acquire(&self, name: &str) -> Result<AccountLease, AccountPoolError> {
        let mut acc = self.accounts.write().unwrap_or_else(|e| e.into_inner());
        if !acc.contains_key(name) {
            return Err(AccountPoolError::NoAccounts(name.to_string()));
        }
        let state = acc.get_mut(name).expect("account exists (checked above)");
        if state.in_flight >= state.max_concurrent {
            return Err(AccountPoolError::Saturated(name.to_string()));
        }
        state.in_flight += 1;
        state.total_calls += 1;
        drop(acc);
        Ok(AccountLease {
            state: Arc::clone(&self.accounts),
            name: name.to_string(),
        })
    }

    /// 池配置访问。
    pub fn config(&self) -> &AccountPoolConfig {
        &self.config
    }
}

/// BYOK 订阅条目 (dsh-plugin-subscriptions 吸收, R-P79 代码级接线)。
///
/// 用户已有 ChatGPT (Codex)/Claude/Grok (X Premium) 等订阅可直接作为 LLM provider 复用,
/// 无需新建账户。每条订阅映射为一个可选择的账户, 汇入 AccountPool 健康/检疫/并发语义。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ByokSubscription {
    /// 订阅名称, 如 "codex" / "claude-pro" / "grok-premium"。
    pub name: String,
    /// 订阅所属服务 (映射到 provider), 如 "openai" / "anthropic" / "xai"。
    pub provider: String,
    /// 复用的接入方式: "harness_subscription" (dsh 式: 借既有 CLI 订阅) 等。
    pub kind: ByokKind,
    /// 订阅配额上限 (并发)。
    pub max_concurrent: usize,
}

/// BYOK 订阅接入方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ByokKind {
    /// 借既有 harness/CLI 订阅 (Codex/Claude Code/Grok), 走本地凭证直连。
    HarnessSubscription,
    /// 自带 API 密钥, 但走订阅计费档位 (复用订阅配额而非独立计费)。
    KeyOnSubscriptionPlan,
}

impl ByokSubscription {
    /// 在给定 AccountPool 注册该订阅为可用账户 (R-P42: 汇入既有池, 不建平行系统)。
    pub fn register_into(&self, pool: &AccountPool) {
        pool.register(&self.provider, &self.name, self.max_concurrent);
    }
}

/// BYOK 订阅池 — 管理用户已有订阅的注册/查询/健康聚合。
#[derive(Debug, Default)]
pub struct ByokPool {
    subs: Arc<RwLock<HashMap<String, ByokSubscription>>>,
}

impl ByokPool {
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册一条订阅 (覆盖同名)。
    pub fn register(&self, sub: ByokSubscription) {
        if let Ok(mut m) = self.subs.write() {
            m.insert(sub.name.clone(), sub);
        }
    }

    /// 移除订阅。
    pub fn unregister(&self, name: &str) -> bool {
        self.subs
            .write()
            .map(|mut m| m.remove(name).is_some())
            .unwrap_or(false)
    }

    pub fn get(&self, name: &str) -> Option<ByokSubscription> {
        self.subs
            .read()
            .ok()
            .and_then(|m| m.get(name).cloned())
    }

    pub fn all(&self) -> Vec<ByokSubscription> {
        self.subs
            .read()
            .map(|m| m.values().cloned().collect())
            .unwrap_or_default()
    }

    pub fn len(&self) -> usize {
        self.subs
            .read()
            .map(|m| m.len())
            .unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 全部订阅灌入 AccountPool 作为可用账户。
    pub fn register_all_into(&self, pool: &AccountPool) -> usize {
        let subs = self.all();
        let mut n = 0;
        for sub in &subs {
            sub.register_into(pool);
            n += 1;
        }
        n
    }

    /// 按 provider 过滤订阅。
    pub fn for_provider(&self, provider: &str) -> Vec<ByokSubscription> {
        self.all()
            .into_iter()
            .filter(|s| s.provider == provider)
            .collect()
    }
}

impl Default for AccountPool {
    fn default() -> Self {
        Self::new(AccountPoolConfig::default())
    }
}

/// C5 自愈检测件 (IO, account_pool): 构造含不健康账户的池,
/// evict_unhealthy 驱逐后断言 is_healthy。
pub struct AccountPoolHealer;

impl crate::core::nt_core_self_test::SelfTest for AccountPoolHealer {
    fn name(&self) -> &str {
        "nt_io_provider::account_pool_healer"
    }

    fn self_test(&self) -> Result<(), Vec<String>> {
        let mut failures = Vec::new();

        let healthy = AccountPool::new(AccountPoolConfig::default());
        healthy.register_default("openai", "acc-a");
        healthy.register_default("openai", "acc-b");
        if !healthy.is_healthy() {
            failures.push("healthy pool reported unhealthy".into());
        }

        let mut pool = AccountPool::new(AccountPoolConfig::default());
        pool.register_default("openai", "acc-a");
        pool.register_default("openai", "acc-b");
        let l1 = match pool.acquire("acc-a") {
            Ok(l) => l,
            Err(e) => {
                failures.push(format!("acquire acc-a failed: {}", e));
                return Err(failures);
            }
        };
        let l2 = match pool.acquire("acc-b") {
            Ok(l) => l,
            Err(e) => {
                failures.push(format!("acquire acc-b failed: {}", e));
                return Err(failures);
            }
        };
        for _ in 0..3 {
            pool.record_failure("acc-b");
        }
        if pool.health_of("acc-b") != Some(AccountHealth::Unhealthy) {
            failures.push("acc-b should be Unhealthy after 3 failures".into());
        }
        if pool.is_healthy() {
            failures.push("unhealthy account with active lease not detected".into());
        }
        let evicted = pool.evict_unhealthy();
        if evicted.is_empty() {
            failures.push("evict_unhealthy removed nothing".into());
        }
        if !pool.is_healthy() {
            failures.push("pool still unhealthy after eviction".into());
        }
        drop(l1);
        drop(l2);

        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    fn test_config() -> AccountPoolConfig {
        AccountPoolConfig {
            quarantine_cooldown: Duration::from_millis(50),
            default_max_concurrent: 2,
            unhealthy_failure_threshold: 3,
        }
    }

    // (a) round-robin 在健康账户间轮转
    #[test]
    fn round_robin_rotates_across_healthy_accounts() {
        let pool = AccountPool::new(test_config());
        pool.register_default("openai", "acc-a");
        pool.register_default("openai", "acc-b");
        pool.register_default("openai", "acc-c");

        let mut seen = std::collections::HashSet::new();
        for _ in 0..3 {
            let lease = pool.select("openai").expect("healthy account available");
            seen.insert(lease.account_name().to_string());
        }
        assert_eq!(seen.len(), 3, "all three accounts should be visited by round-robin");
    }

    // (b) rate-limit 检疫 → 冷却到期后恢复
    #[test]
    fn rate_limited_account_quarantined_then_restored() {
        let pool = AccountPool::new(test_config());
        pool.register_default("openai", "acc-a");
        pool.register_default("openai", "acc-b");

        // 检疫 acc-a → 只剩 acc-b 可被选择
        pool.quarantine("acc-a");
        assert_eq!(pool.health_of("acc-a"), Some(AccountHealth::Quarantined));
        assert_eq!(pool.quarantined_count("openai"), 1);

        let lease = pool.select("openai").expect("acc-b still available");
        assert_eq!(lease.account_name(), "acc-b");

        // 冷却 (50ms) 后自动恢复 (reconcile / select 内 auto-release)
        thread::sleep(Duration::from_millis(80));
        let restored = pool.reconcile();
        assert_eq!(restored, 1, "acc-a should auto-restore after cooldown");
        assert_eq!(pool.health_of("acc-a"), Some(AccountHealth::Healthy));
        assert_eq!(pool.quarantined_count("openai"), 0);
    }

    // (c) 并发租约封顶单账户 in-flight
    #[test]
    fn concurrency_lease_caps_in_flight_per_account() {
        let pool = AccountPool::new(test_config()); // max_concurrent = 2
        pool.register_default("openai", "acc-a");
        pool.register_default("openai", "acc-b");

        let l1 = pool.select("openai").expect("lease 1");
        let l2 = pool.select("openai").expect("lease 2");
        assert_eq!(pool.in_flight_of("acc-a"), Some(1));
        assert_eq!(pool.in_flight_of("acc-b"), Some(1));

        // 第三个租约: acc-a/b 均已 in_flight=1 < 2, 仍可再各租一个
        let l3 = pool.select("openai").expect("lease 3 (both under cap)");
        assert_eq!(pool.in_flight_of(l3.account_name()), Some(2));

        // 显式 acquire 已满账户 → Saturated
        let err = pool.acquire(l3.account_name()).expect_err("cap reached");
        assert!(matches!(err, AccountPoolError::Saturated(_)));

        drop(l1);
        drop(l2);
        drop(l3);
        assert_eq!(pool.in_flight_of("acc-a"), Some(0));
    }

    // (d) 全部检疫 → 返回错误而非 panic
    #[test]
    fn all_quarantined_returns_error_not_panic() {
        let pool = AccountPool::new(test_config());
        pool.register_default("openai", "acc-a");
        pool.register_default("openai", "acc-b");
        pool.quarantine("acc-a");
        pool.quarantine("acc-b");
        assert_eq!(pool.quarantined_count("openai"), 2);

        let err = pool.select("openai").expect_err("all quarantined");
        assert!(matches!(err, AccountPoolError::NoHealthyAccount(_)));
    }

    // 辅助: 连续失败超阈 → Unhealthy (不再被选择)
    #[test]
    fn repeated_failures_mark_unhealthy() {
        let pool = AccountPool::new(test_config());
        pool.register_default("openai", "acc-a");
        pool.record_failure("acc-a");
        pool.record_failure("acc-a");
        pool.record_failure("acc-a");
        assert_eq!(pool.health_of("acc-a"), Some(AccountHealth::Unhealthy));
        assert_eq!(pool.healthy_count("openai"), 0);
        let err = pool.select("openai").expect_err("unhealthy excluded");
        assert!(matches!(err, AccountPoolError::NoHealthyAccount(_)));
    }

    // (e) C5 不变量: 健康池 (含活跃租约) 始终健康
    #[test]
    fn healthy_pool_reports_healthy() {
        let pool = AccountPool::new(test_config());
        pool.register_default("openai", "acc-a");
        pool.register_default("openai", "acc-b");
        assert!(pool.is_healthy(), "fresh pool must be healthy");

        let _l1 = pool.select("openai").expect("lease 1");
        let _l2 = pool.select("openai").expect("lease 2");
        assert!(pool.is_healthy(), "healthy accounts with leases stay healthy");
    }

    // (f) C5 自愈: 不健康账户 (持有 lease) 被驱逐并恢复健康
    #[test]
    fn unhealthy_account_is_evicted() {
        let mut pool = AccountPool::new(test_config());
        pool.register_default("openai", "acc-a");
        pool.register_default("openai", "acc-b");
        let l1 = pool.acquire("acc-a").expect("lease acc-a");
        let l2 = pool.acquire("acc-b").expect("lease acc-b");

        for _ in 0..3 {
            pool.record_failure("acc-b");
        }
        assert_eq!(pool.health_of("acc-b"), Some(AccountHealth::Unhealthy));
        assert_eq!(pool.in_flight_of("acc-b"), Some(1), "lease still held");
        assert!(!pool.is_healthy(), "unhealthy account with active lease detected");

        let evicted = pool.evict_unhealthy();
        assert_eq!(evicted, vec!["acc-b".to_string()]);
        assert!(!pool.contains("acc-b"), "evicted account must be removed");
        assert!(pool.is_healthy(), "pool healthy after eviction");

        drop(l1);
        drop(l2);
    }

    // BYOK 订阅池 (dsh-plugin-subscriptions 吸收): 注册 → 汇入 AccountPool → 可选租约
    #[test]
    fn byok_subscription_registers_into_account_pool() {
        let pool = AccountPool::new(test_config());
        let byok = ByokPool::new();

        byok.register(ByokSubscription {
            name: "codex".to_string(),
            provider: "openai".to_string(),
            kind: ByokKind::HarnessSubscription,
            max_concurrent: 2,
        });
        byok.register(ByokSubscription {
            name: "grok-premium".to_string(),
            provider: "xai".to_string(),
            kind: ByokKind::KeyOnSubscriptionPlan,
            max_concurrent: 3,
        });

        assert_eq!(byok.len(), 2);
        assert_eq!(byok.for_provider("xai").len(), 1);

        let registered = byok.register_all_into(&pool);
        assert_eq!(registered, 2);
        assert_eq!(pool.total_accounts(), 2);

        let lease = pool.select("openai").expect("codex subscription usable");
        assert_eq!(lease.account_name(), "codex");
        assert_eq!(pool.healthy_count("xai"), 1);
    }

    #[test]
    fn byok_subscription_unregister_and_query() {
        let byok = ByokPool::new();
        byok.register(ByokSubscription {
            name: "claude-pro".to_string(),
            provider: "anthropic".to_string(),
            kind: ByokKind::HarnessSubscription,
            max_concurrent: 1,
        });
        assert!(byok.get("claude-pro").is_some());
        assert!(byok.unregister("claude-pro"));
        assert!(byok.get("claude-pro").is_none());
        assert!(byok.is_empty());
    }
}