//! policy — 从 `nt_io_browser_engine.rs` 拆分 (行为零变更).
//! 原文逐行搬运, 仅补可见性/导入。


use super::types::BrowserAction;

/// SSRF 拒绝判定（纯函数）：回环/私网/元数据/空主机一律拒绝
pub(crate) fn ssrf_refused(host: &str) -> bool {
    let h = host.trim().trim_matches(['[', ']']).to_lowercase();
    let h = h.strip_suffix('.').unwrap_or(&h);
    if h.is_empty() || h == "localhost" || h == "0.0.0.0" || h == "::1" {
        return true;
    }
    if h.ends_with(".localhost") || h.ends_with(".internal") || h.ends_with(".local") {
        return true;
    }
    // IPv4 私网段
    let parts: Vec<&str> = h.split('.').collect();
    if parts.len() == 4 && parts.iter().all(|p| p.parse::<u8>().is_ok()) {
        let oct: Vec<u8> = parts.iter().map(|p| p.parse().unwrap_or(0)).collect();
        if oct[0] == 127 || oct[0] == 10 {
            return true;
        }
        if oct[0] == 172 && (16..=31).contains(&oct[1]) {
            return true;
        }
        if oct[0] == 192 && oct[1] == 168 {
            return true;
        }
        if oct[0] == 169 && oct[1] == 254 {
            return true;
        }
        if oct == [0, 0, 0, 0] {
            return true;
        }
    }
    // IPv6 回环/未指定/链路本地/唯一本地
    if h == "::1" || h == "::" {
        return true;
    }
    let hl = h.to_lowercase();
    if hl.starts_with("fe80:") || hl.starts_with("fc") || hl.starts_with("fd") {
        return true;
    }
    false
}

/// allowlist 判定（精确或后缀匹配，大小写不敏感）
pub(crate) fn domain_allowed(host: &str, allowlist: &[String]) -> bool {
    let h = host.trim().trim_end_matches('.').to_lowercase();
    allowlist.iter().any(|entry| {
        let e = entry.trim().trim_end_matches('.').to_lowercase();
        !e.is_empty() && (h == e || h.ends_with(&format!(".{e}")))
    })
}

/// Retry-After 解析（仅秒数；HTTP 日期不算，默认 60s；上限 300s）
pub(crate) fn parse_retry_after_secs(headers: &reqwest::header::HeaderMap) -> u64 {
    headers
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.trim().parse::<u64>().ok())
        .unwrap_or(60)
        .min(300)
}

/// 审计事件（无正文：只留路由类元数据，secrets 永不进审计）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditEvent {
    pub ts_ms: u64,
    pub session_id: String,
    pub action_kind: String,
    pub url: String,
    pub success: bool,
    pub duration_ms: u64,
    pub error_kind: Option<String>,
}

pub(crate) fn action_kind(action: &BrowserAction) -> &'static str {
    match action {
        BrowserAction::Navigate { .. } => "navigate",
        BrowserAction::Click { .. } => "click",
        BrowserAction::Type { .. } => "type",
        BrowserAction::Screenshot => "screenshot",
        BrowserAction::GetContent => "get_content",
        BrowserAction::Scroll { .. } => "scroll",
        BrowserAction::FillField { .. } => "fill",
        BrowserAction::SelectOption { .. } => "select",
        BrowserAction::Check { .. } => "check",
        BrowserAction::Uncheck { .. } => "uncheck",
        BrowserAction::Press { .. } => "press",
        BrowserAction::GetText { .. } => "gettext",
        BrowserAction::PrintPdf => "pdf",
        BrowserAction::SubmitForm { .. } => "submit",
        BrowserAction::WaitForElement { .. } => "wait",
        BrowserAction::Sleep { .. } => "sleep",
        BrowserAction::ExecuteJs { .. } => "execjs",
        BrowserAction::GoBack => "back",
        BrowserAction::GoForward => "forward",
        BrowserAction::Reload => "reload",
        BrowserAction::CloseTab => "closetab",
        BrowserAction::NewTab { .. } => "newtab",
        BrowserAction::SwitchTab { .. } => "switchtab",
        BrowserAction::SaveState { .. } => "save",
        BrowserAction::LoadState { .. } => "load",
        BrowserAction::Download { .. } => "download",
        BrowserAction::Upload { .. } => "upload",
    }
}

// ============================================================================
// 礼貌爬取（P1）：站点限速 + robots.txt（TTL 缓存，失败放行）
// ============================================================================
