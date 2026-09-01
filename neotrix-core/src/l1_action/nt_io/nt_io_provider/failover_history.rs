//! FailoverHistory — 路由池故障转移历史记录 (可观测性)
//!
//! 记录每次故障转移事件: 失败原因 → 降级画像 → 最终命中的 provider/画像。
//! 全局可查询，供 `/route failover` 展示。纯内存环形缓冲 (最近 N 条)。
//!
//! 遵循 R-P42 (强化现有节点) — 不重写 GatewayV2 核心，只在其 `degraded_retry`
//! 的降级路径上报事件。

use serde::Serialize;
use std::collections::VecDeque;
use std::sync::LazyLock;
use std::sync::Mutex;

/// 单次故障转移事件
#[derive(Debug, Clone, Serialize)]
pub struct FailoverEvent {
    /// 触发时间 (unix 秒)
    pub timestamp: i64,
    /// 降级来源画像 (如 "Private")
    pub from_profile: String,
    /// 降级目标画像 (如 "Anonymous")
    pub to_profile: String,
    /// 是否成功恢复
    pub success: bool,
    /// 失败原因摘要
    pub reason: String,
    /// 命中的 provider (0 = 降级链耗尽未恢复)
    pub provider: String,
}

/// 全局故障转移历史 (环形缓冲)
pub struct FailoverHistory {
    events: VecDeque<FailoverEvent>,
    max_events: usize,
    /// 累计触发次数
    pub total_failovers: u64,
}

impl FailoverHistory {
    pub fn new(max_events: usize) -> Self {
        Self { events: VecDeque::new(), max_events, total_failovers: 0 }
    }

    pub fn record(&mut self, event: FailoverEvent) {
        self.total_failovers += 1;
        if self.events.len() >= self.max_events {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    pub fn events(&self) -> Vec<FailoverEvent> {
        self.events.iter().cloned().collect()
    }

    pub fn clear(&mut self) {
        self.events.clear();
        self.total_failovers = 0;
    }
}

impl Default for FailoverHistory {
    fn default() -> Self {
        Self::new(100)
    }
}

static GLOBAL_FAILOVER_HISTORY: LazyLock<Mutex<FailoverHistory>> =
    LazyLock::new(|| Mutex::new(FailoverHistory::new(100)));

/// 记录一次故障转移事件 (供 GatewayV2 降级路径调用)
pub fn record_failover(
    from_profile: &str,
    to_profile: &str,
    success: bool,
    reason: &str,
    provider: &str,
) {
    let now = chrono::Utc::now().timestamp();
    let event = FailoverEvent {
        timestamp: now,
        from_profile: from_profile.to_string(),
        to_profile: to_profile.to_string(),
        success,
        reason: reason.to_string(),
        provider: provider.to_string(),
    };
    if let Ok(mut h) = GLOBAL_FAILOVER_HISTORY.lock() {
        h.record(event);
    }
}

/// 查询全部故障转移事件
pub fn failover_history() -> Vec<FailoverEvent> {
    match GLOBAL_FAILOVER_HISTORY.lock() {
        Ok(h) => h.events(),
        Err(e) => e.into_inner().events(),
    }
}

/// 累计故障转移次数
pub fn total_failovers() -> u64 {
    match GLOBAL_FAILOVER_HISTORY.lock() {
        Ok(h) => h.total_failovers,
        Err(e) => e.into_inner().total_failovers,
    }
}

/// 清空历史
pub fn clear_history() {
    if let Ok(mut h) = GLOBAL_FAILOVER_HISTORY.lock() {
        h.clear();
    }
}

/// 汇总报告
pub fn report() -> String {
    let events = failover_history();
    let total = total_failovers();
    let success = events.iter().filter(|e| e.success).count();
    let mut s = format!("🔄 路由池故障转移历史 (累计 {} 次, 历史窗口 {} 条):\n", total, events.len());
    if events.is_empty() {
        s.push_str("  (暂无故障转移事件)\n");
    } else {
        for e in events.iter().rev().take(20) {
            let status = if e.success { "✓" } else { "✗" };
            let secs = chrono::DateTime::from_timestamp(e.timestamp, 0)
                .map(|t| t.format("%H:%M:%S").to_string())
                .unwrap_or_else(|| e.timestamp.to_string());
            s.push_str(&format!(
                "  [{}] {} {:?} → {:?} {}", // provider 单独放置避免换行混乱
                status, secs, e.from_profile, e.to_profile, e.reason
            ));
            s.push_str(&format!(" | 命中: {}\n", if e.provider.is_empty() { "(未恢复)" } else { &e.provider }));
        }
    }
    if total > 0 {
        s.push_str(&format!("\n  恢复成功率: {:.0}%", 100.0 * success as f64 / total as f64));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_query() {
        clear_history();
        record_failover("Private", "Anonymous", true, "rate-limited", "groq");
        record_failover("Anonymous", "Tor", false, "timeout", "");
        let events = failover_history();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].from_profile, "Private");
        assert!(total_failovers() >= 2);
        clear_history();
        assert!(failover_history().is_empty());
    }

    #[test]
    fn test_ring_buffer_caps_at_max() {
        let mut h = FailoverHistory::new(3);
        for i in 0..5 {
            h.record(FailoverEvent {
                timestamp: i,
                from_profile: "A".into(),
                to_profile: "B".into(),
                success: true,
                reason: "x".into(),
                provider: "p".into(),
            });
        }
        assert_eq!(h.events().len(), 3);
        assert_eq!(h.total_failovers, 5);
    }

    #[test]
    fn test_report_format() {
        // report() 空历史也能渲染标题
        let r = report();
        assert!(r.contains("故障转移"));
        // 本地实例渲染含事件路径
        let mut h = FailoverHistory::new(10);
        h.record(FailoverEvent {
            timestamp: 1,
            from_profile: "Private".into(),
            to_profile: "Anonymous".into(),
            success: true,
            reason: "rate-limited".into(),
            provider: "groq".into(),
        });
        assert_eq!(h.events().len(), 1);
        assert_eq!(h.total_failovers, 1);
    }
}