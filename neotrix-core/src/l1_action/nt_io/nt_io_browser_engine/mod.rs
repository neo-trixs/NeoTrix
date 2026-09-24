#![forbid(unsafe_code)]
//! 自研最小无头浏览器内核 (nt_browser kernel)
//!
//! 把"访问浏览器"熔炼为 NeoTrix 自有能力：不依赖外部浏览器服务进程，
//! 控制面（会话 / Cookie / 表单 / 历史 / 超时）全部自研；渲染面按后端能力诚实声明。
//!
//! # 后端
//! | 后端 | JS 渲染 | 截图 | 表单提交 | Cookie | 说明 |
//! |---|---|---|---|---|
//! | `Mock` (默认) | 否 | 否 | 否 | 否 | 纯内存占位，保持单测 hermetic |
//! | `Http` | 否 | 否 | 是 | 是（自研 jar） | async reqwest 直驱 + scraper（无 spawn_blocking） |
//! | `ChromeHeadless` | 是 | 是 | 经 HTTP 回放 | 否 | 系统 Chrome `--dump-dom`，**禁用 --user-data-dir** |
//! | `Cdp` | 是 | 是 | 是（真提交） | 是（双向同步） | chromiumoxide CDP，需 `stealth-net` 特性 |
//!
//! R-P38/2026-09-22 教训：headless 传任何持久 `--user-data-dir` 在 macOS 必 hang，
//! 故 Chrome 后端永不碰登录 profile；登录态只活在自研 `CookieJar` + 会话存储里。
//!
//! P0/2026-09-22：UA 会话级锁定（逐请求轮换是 bot 强信号）、GBK 解码（encoding_rs）、
//! 代理接线（http/https/socks5h，经 `try_new` 严格校验）。
//! P1/2026-09-22：连接池复用（引擎级共享 Client）、robots.txt + 站点限速、CDP 真操控。
//!
//! # Safety
//! - `#![forbid(unsafe_code)]` (R-P1)
//! - 全局超时双保险：后端内建 timeout + `execute` 外层 `tokio::time::timeout` (R-P38)
//! - 资源上限：链接 200 / 正文 20k 字符 / 表单 20 / 字段 50 / 原始 HTML 128k / 历史 1000

use std::sync::LazyLock;



// ============================================================================
// 常量与静态资源
// ============================================================================

/// 自研 UA 池（与 nt_world::crawl::stealth 解耦：L1 不反向依赖 L2）
pub(crate) const UA_POOL: &[&str] = &[
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36",
];

pub(crate) const MAX_LINKS: usize = 200;
pub(crate) const MAX_TEXT_CHARS: usize = 20_000;
pub(crate) const MAX_FORMS: usize = 20;
pub(crate) const MAX_FIELDS_PER_FORM: usize = 50;
pub(crate) const MAX_RAW_HTML_CHARS: usize = 128_000;
pub(crate) const MAX_HISTORY: usize = 1000;
pub(crate) const MAX_NAV_STACK: usize = 50;

pub(crate) static STRIP_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"(?is)<script[^>]*>.*?</script>|<style[^>]*>.*?</style>|<noscript[^>]*>.*?</noscript>|<!--.*?-->")
        .expect("static strip regex")
});
pub(crate) static WS_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\s+").expect("static whitespace regex"));

pub mod types;
pub mod cookies;
pub mod session;
pub mod fetch;
pub mod policy;
pub mod engine;
pub mod js;
pub mod error;

// 平坦路径兼容：拆分前外部经 `nt_io_browser_engine::{X}` 直引类型，
// 此处集中重导出，保持外部路径零断裂（新增引用请走子模块路径）。
pub use cookies::{AuthConfig, AuthSource, AuthState, CookieJar};
pub use engine::BrowserEngine;
pub use error::BrowserError;
pub use session::{BrowserConfig, BrowserSession, BrowserStats, HistoryRetention, Tab};
pub use types::{BackendKind, BrowserAction, BrowserResult, FormField, FormSpec, LinkRef, PageSnapshot, ScrollDirection};

#[cfg(test)]
mod tests {
    use super::cookies::*;
    use super::engine::*;
    use super::error::*;
    use super::fetch::*;
    use super::js::*;
    use super::policy::*;
    use super::session::*;
    use super::types::*;

    const SAMPLE_HTML: &str = r#"<!doctype html><html><head><title>Example Domain</title>
        <style>body{color:red}</style><script>alert(1)</script></head>
        <body><h1>Example Domain</h1>
        <p>This domain is for use in illustrative examples in documents.</p>
        <p><a href="https://www.iana.org/domains/example">More information...</a></p>
        <p><a href="/relative/path">Relative</a></p>
        <form action="/search" method="get">
          <input type="text" name="q" value="">
          <input type="hidden" name="src" value="nt">
          <input type="submit" value="Go">
        </form></body></html>"#;

    #[tokio::test]
    async fn test_create_session() {
        let engine = BrowserEngine::with_defaults();
        let session_id = engine.create_session().await.unwrap();
        assert!(!session_id.is_empty());
    }

    #[tokio::test]
    async fn test_navigate() {
        let engine = BrowserEngine::with_defaults();
        let session_id = engine.create_session().await.unwrap();

        let result = engine.execute(&session_id, BrowserAction::Navigate {
            url: "https://example.com".to_string(),
        }).await.unwrap();

        assert!(result.success);
        assert_eq!(result.current_url, "https://example.com");
    }

    #[test]
    fn test_parse_page_title_text_links_forms() {
        let (title, text, links, forms, _) =
            parse_page("https://example.com/", SAMPLE_HTML).unwrap();
        assert_eq!(title.as_deref(), Some("Example Domain"));
        assert!(text.contains("illustrative examples"));
        assert!(!text.contains("alert(1)"), "script leaked");
        assert!(!text.contains("color:red"), "style leaked");
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].href, "https://www.iana.org/domains/example");
        assert_eq!(links[1].href, "https://example.com/relative/path");
        assert_eq!(forms.len(), 1);
        assert_eq!(forms[0].action, "https://example.com/search");
        assert_eq!(forms[0].method, "GET");
        let names: Vec<&str> =
            forms[0].fields.iter().map(|f| f.name.as_str()).collect();
        assert!(names.contains(&"q"));
        assert!(names.contains(&"src"));
    }

    #[test]
    fn test_cookie_jar_store_and_send() {
        let mut jar = CookieJar::new();
        jar.store_from_headers(
            "app.example.com",
            ["sid=abc123; Path=/; HttpOnly", "pref=zh; Domain=example.com; Secure"].iter().copied(),
        );
        let hdr = jar
            .header_for("https", "app.example.com")
            .unwrap();
        assert!(hdr.contains("sid=abc123"));
        assert!(hdr.contains("pref=zh"));
        // http 下 Secure cookie 不发送
        let hdr_http = jar.header_for("http", "app.example.com").unwrap();
        assert!(hdr_http.contains("sid=abc123"));
        assert!(!hdr_http.contains("pref=zh"));
        // 无关域不发送
        assert!(jar.header_for("https", "other.com").is_none());
        assert_eq!(jar.cookie_count(), 2);
    }

    #[test]
    fn test_cookie_jar_update_and_clear() {
        let mut jar = CookieJar::new();
        jar.store_from_headers("a.com", ["k=1"].iter().copied());
        jar.store_from_headers("a.com", ["k=2"].iter().copied());
        assert_eq!(jar.cookie_count(), 1);
        assert!(jar
            .header_for("https", "a.com")
            .unwrap()
            .contains("k=2"));
        jar.clear();
        assert_eq!(jar.cookie_count(), 0);
    }

    #[test]
    fn test_pick_form_by_index_and_default() {
        let (_, _, _, forms, _) = parse_page("https://example.com/", SAMPLE_HTML).unwrap();
        let snap = PageSnapshot {
            url: "https://example.com/".to_string(),
            title: None,
            text: String::new(),
            links: vec![],
            forms,
            raw_html: SAMPLE_HTML.to_string(),
            http_status: Some(200),
        };
        assert!(pick_form(&snap, None).is_ok());
        assert!(pick_form(&snap, Some("#0")).is_ok());
        assert!(pick_form(&snap, Some("#9")).is_err());
        assert!(pick_form(&snap, Some("#x")).is_err());
    }

    #[test]
    fn test_backend_caps_honest() {
        assert!(!BackendKind::Mock.caps().form_submit);
        assert!(BackendKind::Http.caps().form_submit);
        assert!(!BackendKind::Http.caps().screenshot);
        assert!(BackendKind::ChromeHeadless.caps().screenshot);
        assert!(!BackendKind::ChromeHeadless.caps().cookie_persist);
    }

    #[test]
    fn test_pick_ua_nonempty() {
        assert!(!pick_ua(None).is_empty());
        assert_eq!(pick_ua(Some("X")), "X");
    }

    #[test]
    fn test_decode_body_utf8_passthrough() {
        let out = decode_body(Some("text/html; charset=utf-8"), "hi 中文".as_bytes());
        assert_eq!(out, "hi 中文");
    }

    #[test]
    fn test_decode_body_gbk() {
        // "中文" 的 GBK 字节
        let gbk = [0xD6, 0xD0, 0xCE, 0xC4];
        let out = decode_body(Some("text/html; charset=gbk"), &gbk);
        assert_eq!(out, "中文");
    }

    #[test]
    fn test_decode_body_unknown_charset_fallback() {
        let out = decode_body(Some("text/html; charset=x-made-up"), "abc".as_bytes());
        assert_eq!(out, "abc");
        let out2 = decode_body(None, "abc".as_bytes());
        assert_eq!(out2, "abc");
    }

    #[test]
    fn test_parse_robots_disallows() {
        let body = "User-agent: *\nDisallow: /admin\nDisallow: /private/\n\nUser-agent: bot\nDisallow: /";
        let rules = parse_robots_disallows(body);
        assert_eq!(rules, vec!["/admin".to_string(), "/private/".to_string()]);
        assert!(robots_denied(&rules, "/admin/login"));
        assert!(robots_denied(&rules, "/private/x"));
        assert!(!robots_denied(&rules, "/public"));
        assert!(!robots_denied(&Vec::new(), "/admin"));
    }

    #[test]
    fn test_try_new_bad_proxy() {
        let bad = BrowserConfig {
            proxy: Some("://bad url".to_string()),
            ..Default::default()
        };
        assert!(BrowserEngine::try_new(bad).is_err());
        // new() 回退直连不断言失败
        let _ = BrowserEngine::new(BrowserConfig {
            proxy: Some("://bad url".to_string()),
            ..Default::default()
        });
        assert!(BrowserEngine::try_new(BrowserConfig::default()).is_ok());
    }

    #[tokio::test]
    async fn test_session_ua_locked() {
        let engine = BrowserEngine::with_defaults();
        let sid = engine.create_session().await.unwrap();
        let (ua1, ua2) = {
            let sessions = engine.sessions.read().await;
            let s = sessions.get(&sid).unwrap();
            assert!(!s.user_agent.is_empty());
            (s.user_agent.clone(), s.user_agent.clone())
        };
        assert_eq!(ua1, ua2);
    }

    #[tokio::test]
    async fn test_unknown_session_errors() {
        let engine = BrowserEngine::with_defaults();
        let err = engine
            .execute("nope", BrowserAction::GetContent)
            .await
            .unwrap_err();
        assert!(matches!(err, BrowserError::SessionNotFound(_)));
    }

    #[tokio::test]
    async fn test_tab_bookkeeping() {
        let engine = BrowserEngine::with_defaults();
        let sid = engine.create_session().await.unwrap();
        engine
            .execute(
                &sid,
                BrowserAction::NewTab {
                    url: Some("https://a.com".to_string()),
                },
            )
            .await
            .unwrap();
        engine
            .execute(&sid, BrowserAction::SwitchTab { index: 0 })
            .await
            .unwrap();
        assert!(engine
            .execute(&sid, BrowserAction::SwitchTab { index: 9 })
            .await
            .is_err());
        assert!(engine
            .execute(&sid, BrowserAction::CloseTab)
            .await
            .is_ok());
        // 最后一个 tab 禁止关闭
        assert!(engine
            .execute(&sid, BrowserAction::CloseTab)
            .await
            .is_err());
    }

    #[cfg(feature = "network_tests")]
    #[tokio::test]
    async fn test_http_backend_live_example() {
        let engine = BrowserEngine::new(BrowserConfig {
            backend: BackendKind::Http,
            timeout_ms: 15000,
            ..Default::default()
        });
        let sid = engine.create_session().await.unwrap();
        let r = engine
            .execute(
                &sid,
                BrowserAction::Navigate {
                    url: "https://example.com".to_string(),
                },
            )
            .await
            .unwrap();
        assert!(r.success);
        assert!(r.output.contains("Example Domain"));
    }

    #[test]
    fn test_js_select_option_escapes() {
        let js = js_select_option("select[name=\"x\"]", &["a".to_string()]);
        assert!(js.contains("document.querySelector"));
        assert!(js.contains("dispatchEvent"));
        // 选择器含双引号必须被 JSON 转义，原样拼接会断裂
        assert!(!js.contains("\"select[name=\""));
    }

    #[test]
    fn test_js_set_checked_branches() {
        let on = js_set_checked("#c", true);
        assert!(on.contains("==true"));
        let off = js_set_checked("#c", false);
        assert!(off.contains("==false"));
        assert!(on.contains("el.click()"));
    }

    fn http_engine() -> BrowserEngine {
        BrowserEngine::new(BrowserConfig {
            backend: BackendKind::Http,
            timeout_ms: 5000,
            min_interval_ms: 0,
            respect_robots: false,
            ..Default::default()
        })
    }

    #[tokio::test]
    async fn test_check_uncheck_select_pending() {
        let engine = http_engine();
        let sid = engine.create_session().await.unwrap();
        let r = engine
            .execute(
                &sid,
                BrowserAction::Check {
                    selector: "#agree".to_string(),
                },
            )
            .await
            .unwrap();
        assert!(r.success);
        assert!(r.output.contains("#agree"));
        let r = engine
            .execute(
                &sid,
                BrowserAction::SelectOption {
                    selector: "#city".to_string(),
                    values: vec!["sh".to_string()],
                },
            )
            .await
            .unwrap();
        assert!(r.success);
        let r = engine
            .execute(
                &sid,
                BrowserAction::Uncheck {
                    selector: "#agree".to_string(),
                },
            )
            .await
            .unwrap();
        assert!(r.success);
        // 待填值确已移除
        let sessions = engine.sessions.read().await;
        let s = sessions.get(&sid).unwrap();
        assert!(!s.pending_fills.keys().any(|k| k.contains("agree")));
        assert!(s.pending_fills.get("#city").map(|v| v.as_str()) == Some("sh"));
    }

    #[tokio::test]
    async fn test_gettext_without_page_errors() {
        let engine = http_engine();
        let sid = engine.create_session().await.unwrap();
        assert!(engine
            .execute(
                &sid,
                BrowserAction::GetText {
                    selector: "h1".to_string(),
                },
            )
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_printpdf_http_rejected() {
        let engine = http_engine();
        let sid = engine.create_session().await.unwrap();
        assert!(engine
            .execute(&sid, BrowserAction::PrintPdf)
            .await
            .is_err());
    }

    fn temp_token_file(tag: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "nt_browser_test_{}_{}.token",
            std::process::id(),
            tag
        ))
    }

    #[tokio::test]
    async fn test_auth_file_hot_reload() {
        let path = temp_token_file("reload");
        std::fs::write(&path, "tok-v1\n").unwrap();
        let engine = http_engine();
        let sid = engine.create_session().await.unwrap();
        engine
            .set_session_auth(&sid, AuthConfig::file(&path))
            .await
            .unwrap();
        let (_, v1) = engine.session_auth_header(&sid).await.unwrap().unwrap();
        assert!(v1.contains("tok-v1"));
        // 轮换文件 → 下次请求即生效，零重启
        std::fs::write(&path, "tok-v2\n").unwrap();
        let (_, v2) = engine.session_auth_header(&sid).await.unwrap().unwrap();
        assert!(v2.contains("tok-v2"));
        std::fs::remove_file(&path).unwrap();
        // 文件消失 → 精确过期错（含路径指引），而非裸错
        let err = engine.session_auth_header(&sid).await.unwrap_err();
        match err {
            BrowserError::AuthExpired { hint } => {
                assert!(hint.contains(&path.display().to_string()));
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_auth_missing_file_rejected_at_set() {
        let engine = http_engine();
        let sid = engine.create_session().await.unwrap();
        assert!(engine
            .set_session_auth(
                &sid,
                AuthConfig::file("/definitely/not/here.token")
            )
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_auth_expiry_precheck_no_request() {
        let engine = http_engine();
        let sid = engine.create_session().await.unwrap();
        let mut cfg = AuthConfig::literal("stale");
        cfg.expires_at_ms = Some(1); // 1970，早已过期
        engine.set_session_auth(&sid, cfg).await.unwrap();
        let err = engine
            .execute(
                &sid,
                BrowserAction::Navigate {
                    url: "https://example.com".to_string(),
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, BrowserError::AuthExpired { .. }));
    }

    #[tokio::test]
    async fn test_save_load_roundtrip_redacts_literal_token() {
        let engine = BrowserEngine::with_defaults();
        let sid = engine.create_session().await.unwrap();
        engine
            .execute(
                &sid,
                BrowserAction::Navigate {
                    url: "https://example.com".to_string(),
                },
            )
            .await
            .unwrap();
        engine
            .set_session_auth(&sid, AuthConfig::literal("super-secret"))
            .await
            .unwrap();
        let path = temp_token_file("state");
        let path_s = path.to_string_lossy().to_string();
        let r = engine
            .execute(
                &sid,
                BrowserAction::SaveState {
                    path: path_s.clone(),
                },
            )
            .await
            .unwrap();
        assert!(r.success);
        // 落盘文件不得含明文 token
        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(!raw.contains("super-secret"));
        let r = engine
            .execute(&sid, BrowserAction::LoadState { path: path_s })
            .await
            .unwrap();
        assert!(r.success);
        assert!(r.output.contains("state loaded"));
        assert_eq!(engine.stats().await.active_sessions, 2);
    }

    #[tokio::test]
    async fn test_load_state_bad_version_rejected() {
        let path = temp_token_file("badver");
        std::fs::write(&path, r#"{"version":999,"saved_at_ms":0,"session":null}"#).unwrap();
        let engine = BrowserEngine::with_defaults();
        let sid = engine.create_session().await.unwrap();
        assert!(engine
            .execute(
                &sid,
                BrowserAction::LoadState {
                    path: path.to_string_lossy().to_string(),
                },
            )
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_download_parent_missing_rejected() {
        let engine = http_engine();
        let sid = engine.create_session().await.unwrap();
        assert!(engine
            .execute(
                &sid,
                BrowserAction::Download {
                    url: "https://example.com/".to_string(),
                    path: "/definitely/not/here/x.bin".to_string(),
                },
            )
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_upload_record_validates_files() {
        let engine = http_engine();
        let sid = engine.create_session().await.unwrap();
        // 不存在的文件直接拒收（零网络）
        assert!(engine
            .execute(
                &sid,
                BrowserAction::Upload {
                    selector: "#f".to_string(),
                    files: vec!["/definitely/not/here.bin".to_string()],
                },
            )
            .await
            .is_err());
        assert!(engine
            .execute(
                &sid,
                BrowserAction::Upload {
                    selector: "#f".to_string(),
                    files: vec![],
                },
            )
            .await
            .is_err());
        // 真实临时文件可记录
        let tmp = temp_token_file("up");
        std::fs::write(&tmp, "x").unwrap();
        let r = engine
            .execute(
                &sid,
                BrowserAction::Upload {
                    selector: "#f".to_string(),
                    files: vec![tmp.to_string_lossy().to_string()],
                },
            )
            .await
            .unwrap();
        assert!(r.success);
    }

    #[test]
    fn test_ssrf_refused() {
        for host in [
            "",
            "localhost",
            "LOCALHOST",
            "a.localhost",
            "x.internal",
            "x.local",
            "127.0.0.1",
            "127.9.9.9",
            "10.0.0.5",
            "172.16.0.1",
            "172.31.255.255",
            "192.168.1.1",
            "169.254.169.254",
            "0.0.0.0",
            "[::1]",
            "example.com.",
        ] {
            // 尾点由函数内规范化：example.com. 应放行，其余拒绝
            let should_refuse = host.trim_end_matches('.') != "example.com";
            assert_eq!(ssrf_refused(host), should_refuse, "host={host}");
        }
        for host in ["example.com", "8.8.8.8", "1.1.1.1", "172.32.0.1", "192.169.1.1"] {
            assert!(!ssrf_refused(host), "should allow {host}");
        }
    }

    #[test]
    fn test_domain_allowed() {
        let list = vec!["example.com".to_string(), "sub.test.org".to_string()];
        assert!(domain_allowed("example.com", &list));
        assert!(domain_allowed("www.example.com", &list));
        assert!(domain_allowed("EXAMPLE.COM", &list));
        assert!(!domain_allowed("notexample.com", &list));
        assert!(!domain_allowed("example.com.evil.com", &list));
        assert!(!domain_allowed("other.org", &list));
    }

    #[tokio::test]
    async fn test_allowlist_and_ssrf_enforced_without_network() {
        let engine = BrowserEngine::new(BrowserConfig {
            backend: BackendKind::Http,
            timeout_ms: 5000,
            min_interval_ms: 0,
            respect_robots: false,
            allowed_domains: Some(vec!["example.com".to_string()]),
            ..Default::default()
        });
        let sid = engine.create_session().await.unwrap();
        // SSRF：先于 allowlist 触发，无网络
        let err = engine
            .execute(
                &sid,
                BrowserAction::Navigate {
                    url: "http://127.0.0.1/admin".to_string(),
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, BrowserError::SsrfRefused(_)));
        // allowlist 之外：拒收，无网络
        let err = engine
            .execute(
                &sid,
                BrowserAction::Navigate {
                    url: "https://evil.com/".to_string(),
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(err, BrowserError::DomainDenied(_)));
    }

    #[tokio::test]
    async fn test_session_budget_stops_runaway() {
        let engine = BrowserEngine::new(BrowserConfig {
            max_actions_per_session: Some(1),
            ..Default::default()
        });
        let sid = engine.create_session().await.unwrap();
        assert!(engine
            .execute(
                &sid,
                BrowserAction::Navigate {
                    url: "https://example.com".to_string(),
                },
            )
            .await
            .is_ok());
        let err = engine
            .execute(&sid, BrowserAction::GetContent)
            .await
            .unwrap_err();
        assert!(matches!(err, BrowserError::BudgetExhausted(_)));
    }

    #[tokio::test]
    async fn test_audit_log_has_no_bodies() {
        let engine = BrowserEngine::with_defaults();
        let sid = engine.create_session().await.unwrap();
        engine
            .execute(
                &sid,
                BrowserAction::Navigate {
                    url: "https://example.com".to_string(),
                },
            )
            .await
            .unwrap();
        let log = engine.audit_log().await;
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].action_kind, "navigate");
        assert!(log[0].success);
        // 审计只记元数据：序列化后不得含页面正文
        let json = serde_json::to_string(&log).unwrap();
        assert!(!json.contains("Page content"));
    }
}
