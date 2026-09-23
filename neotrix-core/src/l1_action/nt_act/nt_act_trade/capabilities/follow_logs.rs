//! Customer follow logs — 逐客户跟进日志（参数构造＋解析＋限流熔断）.
//!
//! 端点 `GET /d/customer/logs` 实证（2026-09-23，WSD001 管理员通道）：
//! - **参数必须带时分秒**：`startDate=YYYY-MM-DD 00:00:00`、
//!   `endDate=YYYY-MM-DD 23:59:59`，且附 `num`/`size`；裸日期返回
//!   空列表（假成功，曾误判为权限墙）。
//! - 响应 `data` 为裸 list 或 `{values|list}`，形状容错解析。
//! - `X-User` 请求头无关（136154/187820 结果相同），管理员 session
//!   可直查任意客户，operator switch 对本端点非必需。
//! - **IP 级日配额全公司共享**：连续 2 次「系统繁忙」即熔断停机，
//!   冷却重试对日配额无效；断点续传次日再跑。
//!
//! 来源：WSD `nt_customer_follows.py` + `nt_admin.py follows-bulk`。

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// 跟进日志路径（base = "/rapi/d" 或完整 URL 拼接）
pub const FOLLOW_LOGS_PATH: &str = "/d/customer/logs";

/// 默认单页条数
pub const DEFAULT_SIZE: u32 = 50;
/// 默认回看窗口（天）
pub const DEFAULT_WINDOW_DAYS: u32 = 365;
/// 熔断阈值：连续繁忙次数
pub const RATE_LIMIT_BREAKER: u32 = 2;

/// 构造 `/d/customer/logs` 查询参数（有序，便于直接拼 query string）。
///
/// `start_ymd` / `end_ymd` 格式 `YYYY-MM-DD`；本函数补时分秒。
/// 错误日期格式返回空 Vec（调用方应拒绝发请求，而非裸发假参数）。
pub fn follow_log_params(
    customer_id: &str,
    start_ymd: &str,
    end_ymd: &str,
    num: u32,
    size: u32,
) -> Vec<(String, String)> {
    if customer_id.is_empty() || !is_ymd(start_ymd) || !is_ymd(end_ymd) {
        return Vec::new();
    }
    vec![
        ("customerId".into(), customer_id.to_string()),
        ("startDate".into(), format!("{start_ymd} 00:00:00")),
        ("endDate".into(), format!("{end_ymd} 23:59:59")),
        ("num".into(), num.to_string()),
        ("size".into(), size.to_string()),
    ]
}

fn is_ymd(s: &str) -> bool {
    // YYYY-MM-DD，10 字符，第 5/8 位为 '-'
    s.len() == 10
        && s.as_bytes().get(4) == Some(&b'-')
        && s.as_bytes().get(7) == Some(&b'-')
        && s.bytes().filter(|b| b.is_ascii_digit()).count() == 8
}

/// 单条跟进日志（ tolerance：缺字段给默认，不炸）
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct FollowLogEntry {
    pub id: String,
    /// 毫秒时间戳（前端 `time` 字段）
    pub time_ms: u64,
    pub log_type: String,
    pub operator_name: String,
    pub comments: String,
    pub contact_name: String,
    pub business_id: String,
}

/// 从 `/d/customer/logs` 响应 JSON 解析条目列表。
///
/// 形状容错：`data` 为数组，或 `{values: [...]}` / `{list: [...]}`；
/// 顶层直接是数组也接受。非对象条目跳过。
pub fn parse_follow_logs(value: &serde_json::Value) -> Vec<FollowLogEntry> {
    let arr = value
        .get("data")
        .and_then(|d| {
            if d.is_array() {
                d.as_array()
            } else {
                d.get("values")
                    .or_else(|| d.get("list"))
                    .and_then(|v| v.as_array())
            }
        })
        .or_else(|| value.as_array());
    let mut out = Vec::new();
    if let Some(items) = arr {
        for it in items {
            if !it.is_object() {
                continue;
            }
            let s = |k: &str| -> String {
                it.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string()
            };
            let time_ms = it.get("time").and_then(|v| v.as_u64()).unwrap_or(0);
            out.push(FollowLogEntry {
                id: s("id"),
                time_ms,
                log_type: it
                    .get("logType")
                    .and_then(|v| v.as_i64())
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| s("logType")),
                operator_name: s("operatorName"),
                comments: {
                    let c = s("comments");
                    if c.is_empty() {
                        s("logDetail")
                    } else {
                        c
                    }
                },
                contact_name: s("contactName"),
                business_id: s("businessId"),
            });
        }
    }
    out
}

/// 判定响应是否为「系统繁忙」限流（非业务失败）。
pub fn is_rate_limited(value: &serde_json::Value) -> bool {
    value.get("success").and_then(|v| v.as_bool()) == Some(false)
        && value
            .get("errMsg")
            .and_then(|v| v.as_str())
            .map(|m| m.contains('忙'))
            .unwrap_or(false)
}

/// IP 级日配额熔断器：连续 N 次繁忙即停，不烧冷却硬闯。
///
/// 语义：成功清零计数；繁忙累加；达 [`RATE_LIMIT_BREAKER`] 返回
/// `should_stop=true` 且后续保持 true（需 `reset()` 才能继续——
/// 对应「次日再跑」）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateCircuitBreaker {
    consecutive_limits: u32,
    threshold: u32,
    tripped: bool,
}

impl Default for RateCircuitBreaker {
    fn default() -> Self {
        Self::new(RATE_LIMIT_BREAKER)
    }
}

impl RateCircuitBreaker {
    pub fn new(threshold: u32) -> Self {
        Self {
            consecutive_limits: 0,
            threshold: threshold.max(1),
            tripped: false,
        }
    }

    /// 记一次成功请求（清零连续计数；已 tripped 不自动恢复）。
    pub fn record_success(&mut self) {
        if !self.tripped {
            self.consecutive_limits = 0;
        }
    }

    /// 记一次限流；返回是否应立即停机。
    pub fn record_rate_limit(&mut self) -> bool {
        if self.tripped {
            return true;
        }
        self.consecutive_limits += 1;
        if self.consecutive_limits >= self.threshold {
            self.tripped = true;
        }
        self.tripped
    }

    /// 是否已熔断（停机存档，次日 reset）。
    pub fn tripped(&self) -> bool {
        self.tripped
    }

    /// 人工/定时复位（新配额窗口）。
    pub fn reset(&mut self) {
        self.consecutive_limits = 0;
        self.tripped = false;
    }

    pub fn consecutive_limits(&self) -> u32 {
        self.consecutive_limits
    }
}

/// 断点续传计算器：给定全量 id 与已抓 done set，返回 todo。
pub fn resume_todo(all_ids: &[String], done: &std::collections::HashSet<String>) -> Vec<String> {
    all_ids
        .iter()
        .filter(|id| !done.contains(id.as_str()))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn params_include_time_components() {
        let p = follow_log_params("242856420", "2025-09-23", "2026-09-23", 1, 50);
        assert_eq!(p.len(), 5);
        let get = |k: &str| p.iter().find(|(a, _)| a == k).map(|(_, v)| v.as_str());
        assert_eq!(get("customerId"), Some("242856420"));
        assert_eq!(get("startDate"), Some("2025-09-23 00:00:00"));
        assert_eq!(get("endDate"), Some("2026-09-23 23:59:59"));
        assert_eq!(get("num"), Some("1"));
        assert_eq!(get("size"), Some("50"));
    }

    #[test]
    fn bad_dates_yield_empty_params() {
        assert!(follow_log_params("1", "2025-9-3", "2026-09-23", 1, 50).is_empty());
        assert!(follow_log_params("", "2025-09-23", "2026-09-23", 1, 50).is_empty());
        assert!(follow_log_params("1", "2025-09-23", "not-a-date", 1, 50).is_empty());
    }

    #[test]
    fn parse_bare_list_and_wrapped_shapes() {
        let bare = serde_json::json!([
            {"id": "9", "time": 1790062437568u64, "logType": 1,
             "operatorName": "白亚婷", "comments": "已发PI", "contactName": "Tom"}
        ]);
        let e = parse_follow_logs(&bare);
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].time_ms, 1790062437568);
        assert_eq!(e[0].operator_name, "白亚婷");
        assert_eq!(e[0].comments, "已发PI");

        let wrapped = serde_json::json!({"data": {"values": [
            {"id": "1", "logDetail": "fallback comment"}
        ]}});
        let e2 = parse_follow_logs(&wrapped);
        assert_eq!(e2.len(), 1);
        assert_eq!(e2[0].comments, "fallback comment");

        let empty = serde_json::json!({"data": [], "success": true});
        assert!(parse_follow_logs(&empty).is_empty());
    }

    #[test]
    fn rate_limited_detection() {
        let busy = serde_json::json!({"success": false, "errMsg": "系统繁忙，请稍后再试！"});
        assert!(is_rate_limited(&busy));
        let auth = serde_json::json!({"success": false, "errMsg": "401 authorize"});
        assert!(!is_rate_limited(&auth));
        let ok = serde_json::json!({"success": true, "data": []});
        assert!(!is_rate_limited(&ok));
    }

    #[test]
    fn breaker_trips_on_two_consecutive_limits() {
        let mut b = RateCircuitBreaker::default();
        assert!(!b.record_rate_limit()); // 1/2
        assert!(b.record_rate_limit()); // 2/2 → tripped
        assert!(b.tripped());
        assert!(b.record_rate_limit()); // 保持 tripped
        b.record_success(); // tripped 后 success 不解熔
        assert!(b.tripped());
        b.reset();
        assert!(!b.tripped());
        assert!(!b.record_rate_limit()); // 复位后从 1 重新计
    }

    #[test]
    fn breaker_success_clears_streak() {
        let mut b = RateCircuitBreaker::new(2);
        assert!(!b.record_rate_limit());
        b.record_success();
        assert!(!b.record_rate_limit()); // streak 已清，重新 1/2
        assert_eq!(b.consecutive_limits(), 1);
    }

    #[test]
    fn resume_skips_done_ids() {
        let all: Vec<String> = ["a", "b", "c"].iter().map(|s| s.to_string()).collect();
        let mut done = HashSet::new();
        done.insert("b".to_string());
        let todo = resume_todo(&all, &done);
        assert_eq!(todo, vec!["a".to_string(), "c".to_string()]);
    }
}
