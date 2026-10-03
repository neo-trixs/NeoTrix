//! Payload 判别层 — 把「返回 HTML 而非 JSON」和「HTTP 200 + 错误信封」变成类型化错误。
//!
//! 吸收自两个同类项目的可复用资产：
//!
//! 1. **OpenCLI `LoginWallError`**（`src/errors.ts:186` + `src/utils.ts:139-159`，
//!    PR #1668 引入，退出码 77 / `EX_NOPERM`）。
//!    其判别面**纯粹是内容嗅探**，无状态码、无 GraphQL error code：
//!    `Content-Type` 含 `text/html`，**或** body 前导标签匹配
//!    `^<(?:!doctype|html|head|body|title)(?:[\s>/]|$)`（不分大小写）。
//!    动机是压过 `SyntaxError: Unexpected token '<'`——即把登录墙
//!    误报成 JSON 解析失败。
//!
//! 2. **AutoCLI `cascade.rs` 的 `json.code !== 0` 规则**。
//!    HTTP 状态码 200 但 body 是 `{"code":-1,"msg":"请登录"}` 这种错误信封时，
//!    **状态码嗅探在结构上不可能抓到** —— 这是它补上前者盲区的那一层。
//!
//! ## 本仓实测依据（2026-10-03，非推断）
//!
//! `cdn.syndication.twimg.com/tweet-result` 对不存在的推文返回
//! `<!DOCTYPE html>...<title>X / ?</title>`（200/404 + `text/html`），
//! 而 `api.vxtwitter.com/<user>` 对不存在的用户返回
//! `{"error":"User not found."}` + `application/json` + **404**。
//! ⇒ 同一「抓不到东西」的语义，两个上游用了两种完全不同的载体。
//! 没有这一层，extractor 只能把两者都报成误导性的 `Parse` 错误。

use std::fmt;

/// HTML 前导标签白名单 —— 与 OpenCLI 的正则 alternation 逐项对应。
///
/// ⛔ 末尾的 `[\s>/]` 终止符守卫必须保留：OpenCLI 原文注释说明它是为了让
/// `<htmlfoo` **不**匹配。去掉守卫会把自定义元素名误判成登录墙。
const HTML_LEAD_TAGS: [&str; 5] = ["!doctype", "html", "head", "body", "title"];

/// body 预览长度上限 —— 对齐 OpenCLI `utils.ts` 的 `slice(0, 100)`。
const PREVIEW_LEN: usize = 100;

/// 一次 HTTP 响应的判别结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PayloadVerdict {
    /// 正常 JSON 载荷。
    Json(serde_json::Value),
    /// 服务端返回了 HTML：登录墙 / WAF 挑战 / 频控 interstitial / 该资源根本不存在。
    HtmlInsteadOfJson {
        status: u16,
        preview: String,
    },
    /// HTTP 成功，但 body 是业务错误信封（如 `{"code":-1,"msg":"请登录"}`）。
    ErrorEnvelope { code: i64, message: String },
    /// 既不是 HTML 也不是合法 JSON —— 真正的畸形响应。
    Malformed { preview: String },
    /// 空 body。
    Empty,
}

impl fmt::Display for PayloadVerdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(_) => write!(f, "json"),
            Self::HtmlInsteadOfJson { status, preview } => write!(
                f,
                "login wall or HTML interstitial (HTTP {}): {}",
                status, preview
            ),
            Self::ErrorEnvelope { code, message } => {
                write!(f, "error envelope: code={} msg={}", code, message)
            }
            Self::Malformed { preview } => write!(f, "malformed payload: {}", preview),
            Self::Empty => write!(f, "empty payload"),
        }
    }
}

/// Content-Type 是否表明服务端返回了 HTML。
///
/// 对齐 OpenCLI `utils.ts:141`：仅做 `text/html` 子串匹配，不解析完整 MIME 类型，
/// 故 `application/xhtml+xml` **不**命中（与上游行为一致，不擅自扩大判别面）。
fn content_type_is_html(content_type: &str) -> bool {
    content_type.to_ascii_lowercase().contains("text/html")
}

/// body 是否以 HTML 文档/标签开头。
///
/// 等价于 OpenCLI 的 `/^<(?:!doctype|html|head|body|title)(?:[\s>/]|$)/i`，
/// 但**不引入 regex 依赖**（本仓该路径无 regex crate）。
fn body_starts_with_html(body: &str) -> bool {
    let trimmed = body.trim_start();
    let Some(rest) = trimmed.strip_prefix('<') else {
        return false;
    };
    HTML_LEAD_TAGS.iter().any(|tag| {
        if rest.len() < tag.len() || !rest.as_bytes()[..tag.len()].eq_ignore_ascii_case(tag.as_bytes()) {
            return false;
        }
        match rest[tag.len()..].chars().next() {
            // 标签名恰好结束（`$` 分支）
            None => true,
            // `[\s>/]` 终止符 —— 缺了它 `<htmlfoo` 会误命中
            Some(c) => c.is_whitespace() || c == '>' || c == '/',
        }
    })
}

/// 截断预览，用于错误信息。控制字符转空格以免污染终端输出。
fn preview(body: &str) -> String {
    let head: String = body
        .trim()
        .chars()
        .take(PREVIEW_LEN)
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    head
}

/// 从 JSON 中提取业务错误信封。
///
/// 吸收 AutoCLI `cascade.rs:71-78` 的判据：`code` 存在且非 0 即失败。
/// 仅识别**数值** `code`；字符串 `"code":"0"` 不在此规则内（避免误伤
/// 恰好有名为 code 的字符串字段的正常载荷）。
fn error_envelope(v: &serde_json::Value) -> Option<(i64, String)> {
    let code = v.get("code")?;
    let n = code.as_i64()?;
    if n == 0 {
        return None;
    }
    let message = v
        .get("msg")
        .or_else(|| v.get("message"))
        .or_else(|| v.get("error"))
        .and_then(|m| m.as_str())
        .unwrap_or("(no message field)")
        .to_string();
    Some((n, message))
}

/// 判别一次响应的载荷。
///
/// 顺序有意为之，**内容优先于状态码**（对齐 OpenCLI `BROWSER_JSON_SNIFF_FN`
/// 先查 `looksLikeHtml` 再查 `!r.ok` 的次序）：
///
/// 1. HTML 判别先于 JSON 解析 —— 否则登录墙被误报成 `Malformed`；
/// 2. JSON 解析成功后仍要查错误信封 —— 否则 200 + `{"code":-1}` 被当成成功。
///
/// `status` 不参与 1/2 的判定，仅用于错误信息与最终的成功性判断。
pub fn classify(status: u16, content_type: &str, body: &str) -> PayloadVerdict {
    if body.trim().is_empty() {
        return PayloadVerdict::Empty;
    }

    if content_type_is_html(content_type) || body_starts_with_html(body) {
        return PayloadVerdict::HtmlInsteadOfJson {
            status,
            preview: preview(body),
        };
    }

    match serde_json::from_str::<serde_json::Value>(body) {
        Ok(v) => match error_envelope(&v) {
            Some((code, message)) => PayloadVerdict::ErrorEnvelope { code, message },
            None => PayloadVerdict::Json(v),
        },
        Err(_) => PayloadVerdict::Malformed {
            preview: preview(body),
        },
    }
}

/// 便捷判定：本次响应是否应被判为「需要登录/被墙」。
///
/// 供 channel 后端探测复用 —— CLI 后端（bird/opencli）在未登录时同样会
/// 把登录页 HTML 打到 stdout，用状态码是识别不出来的。
pub fn is_login_walled(content_type: &str, body: &str) -> bool {
    matches!(
        classify(0, content_type, body),
        PayloadVerdict::HtmlInsteadOfJson { .. }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── HTML 前导判别（对齐 OpenCLI 正则的逐项行为）──────────────────

    #[test]
    fn detects_doctype_html() {
        assert!(body_starts_with_html(
            "<!DOCTYPE html><html><head><title>X / ?</title>"
        ));
        assert!(body_starts_with_html("  \n <!doctype html>"));
    }

    #[test]
    fn detects_bare_tags_with_terminators() {
        assert!(body_starts_with_html("<html lang=\"en\">"));
        assert!(body_starts_with_html("<body>"));
        assert!(body_starts_with_html("<head/>"));
        assert!(body_starts_with_html("<title>x"));
        // 标签名恰好结束 ⇒ `$` 分支
        assert!(body_starts_with_html("<html"));
    }

    #[test]
    fn tag_name_terminator_guard_prevents_false_positive() {
        // ⛔ 少了终止符守卫，`<htmlfoo` 会被误判成登录墙（OpenCLI 原文注释同此）
        assert!(!body_starts_with_html("<htmlfoo>"));
        assert!(!body_starts_with_html("<bodyguard"));
        assert!(!body_starts_with_html("<titling"));
    }

    #[test]
    fn json_body_is_not_html() {
        assert!(!body_starts_with_html(r#"{"error":"User not found."}"#));
        assert!(!body_starts_with_html("[1,2,3]"));
        assert!(!body_starts_with_html("just setting up my twttr"));
    }

    // ── Content-Type 判别 ──────────────────────────────────────────

    #[test]
    fn content_type_matching_is_substring_based() {
        assert!(content_type_is_html("text/html"));
        assert!(content_type_is_html("TEXT/HTML; charset=utf-8"));
        // 与上游一致：xhtml 不在判别面内
        assert!(!content_type_is_html("application/xhtml+xml"));
        assert!(!content_type_is_html("application/json"));
    }

    // ── classify 全链路 ────────────────────────────────────────────

    #[test]
    fn html_instead_of_json_wins_over_status_ok() {
        // 本仓实测：syndication 对不存在的推文返回 HTML
        let body = "<!DOCTYPE html>\n<html lang=\"en\" class=\"dog\">\n<title>X / ?</title>";
        let v = classify(404, "text/html; charset=utf-8", body);
        assert_eq!(
            v,
            PayloadVerdict::HtmlInsteadOfJson {
                status: 404,
                preview: preview(body),
            }
        );
    }

    #[test]
    fn json_error_envelope_is_caught_despite_200() {
        // 吸收 AutoCLI `json.code !== 0` —— 状态码嗅探结构上抓不到这一层
        let v = classify(200, "application/json", r#"{"code":-1,"msg":"请登录"}"#);
        assert_eq!(
            v,
            PayloadVerdict::ErrorEnvelope {
                code: -1,
                message: "请登录".into()
            }
        );
    }

    #[test]
    fn code_zero_is_success_not_envelope() {
        let v = classify(200, "application/json", r#"{"code":0,"data":[1]}"#);
        assert!(matches!(v, PayloadVerdict::Json(_)));
    }

    #[test]
    fn non_numeric_code_is_not_envelope() {
        // 字符串 code 不在 AutoCLI 判据内，避免误伤正常载荷
        let v = classify(200, "application/json", r#"{"code":"AB-123","name":"x"}"#);
        assert!(matches!(v, PayloadVerdict::Json(_)));
    }

    #[test]
    fn envelope_reads_alternate_message_keys() {
        for (body, want) in [
            (r#"{"code":7,"msg":"m"}"#, "m"),
            (r#"{"code":7,"message":"m2"}"#, "m2"),
            (r#"{"code":7,"error":"m3"}"#, "m3"),
            (r#"{"code":7}"#, "(no message field)"),
        ] {
            match classify(200, "application/json", body) {
                PayloadVerdict::ErrorEnvelope { message, .. } => assert_eq!(message, want),
                other => panic!("expected envelope, got {:?}", other),
            }
        }
    }

    #[test]
    fn real_syndication_tweet_parses_to_json() {
        // 本仓实测真实响应（id=20）的关键字段，确保解析面与上游一致
        let body = r#"{"__typename":"Tweet","favorite_count":309306,"id_str":"20",
            "text":"just setting up my twttr",
            "user":{"id_str":"12","name":"jack","screen_name":"jack","is_blue_verified":true}}"#;
        match classify(200, "application/json", body) {
            PayloadVerdict::Json(v) => {
                assert_eq!(v["id_str"], "20");
                assert_eq!(v["favorite_count"], 309306);
                assert_eq!(v["user"]["screen_name"], "jack");
            }
            other => panic!("expected json, got {:?}", other),
        }
    }

    #[test]
    fn real_vxtwitter_not_found_parses_to_json_not_html() {
        // 本仓实测：api.vxtwitter.com 对不存在用户返回 JSON 404
        match classify(404, "application/json", r#"{"error":"User not found."}"#) {
            PayloadVerdict::Json(v) => assert_eq!(v["error"], "User not found."),
            other => panic!("expected json, got {:?}", other),
        }
    }

    #[test]
    fn empty_and_malformed_are_distinguished() {
        assert_eq!(classify(204, "application/json", "   "), PayloadVerdict::Empty);
        assert!(matches!(
            classify(200, "application/json", "not json at all"),
            PayloadVerdict::Malformed { .. }
        ));
    }

    #[test]
    fn preview_strips_control_chars_and_bounds_length() {
        let body = "a\u{0}b\u{7}c";
        assert_eq!(preview(body), "a b c");
        let long = "x".repeat(500);
        assert_eq!(preview(&long).chars().count(), PREVIEW_LEN);
    }

    #[test]
    fn is_login_walled_helper() {
        assert!(is_login_walled("text/html", "<html><body>login</body></html>"));
        assert!(!is_login_walled("application/json", r#"{"ok":true}"#));
        assert!(!is_login_walled("text/plain", "plain"));
    }
}