//! Browser XHR capture — 浏览器网络捕获（融合 WSD 实战）.
//!
//! 主仓浏览器栈缺的两块：① XHR 捕获（requestWillBeSent 取完整 URL＋
//! Network.getResponseBody 趁热取体——70＋ 私有 API 端点由此发现）；
//! ② 文本/坐标点击 JS 生成（引擎 Click v1 仅支持有 href 的 `<a>`）。
//! 融合方式：纯函数＋JS 片段生成，经既有 `ExecuteJs` 动作执行，
//! 不碰引擎核心。来源：WSD `nt_browser.py`（抽屉 42 XHR 实证）。

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// 捕获到的 XHR
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapturedXhr {
    pub method: String,
    pub url: String,
    /// POST body 摘要（前 500 字符）
    pub post_data: String,
}

/// CDP performance-log 单条 message 解析
#[derive(Debug, Clone, Deserialize)]
struct CdpEnvelope {
    message: CdpMessage,
}

#[derive(Debug, Clone, Deserialize)]
struct CdpMessage {
    method: String,
    params: serde_json::Value,
}

/// 从 performance-log JSON 条目中提取 XHR（fragment 过滤，如 "/rapi/"）。
///
/// 输入：`driver.get_log("performance")` 每条的 `message` 字符串。
/// 注意：`get_log` 会消费缓冲，调用方自行决定排空时机。
pub fn parse_xhr_entries(entries: &[String], fragment: &str) -> Vec<CapturedXhr> {
    let mut out = Vec::new();
    for entry in entries {
        let env: CdpEnvelope = match serde_json::from_str(entry) {
            Ok(e) => e,
            Err(_) => continue,
        };
        if env.message.method != "Network.requestWillBeSent" {
            continue;
        }
        let req = &env.message.params["request"];
        let url = req.get("url").and_then(|v| v.as_str()).unwrap_or("");
        if !url.contains(fragment) {
            continue;
        }
        let method = req
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let post = req.get("postData").and_then(|v| v.as_str()).unwrap_or("");
        out.push(CapturedXhr {
            method,
            url: url.to_string(),
            post_data: post.chars().take(500).collect(),
        });
    }
    out
}

/// getResponseBody 结果解码（base64 透明处理）
pub fn decode_response_body(body: &str, base64_encoded: bool) -> Result<String, String> {
    if !base64_encoded {
        return Ok(body.to_string());
    }
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(body)
        .map_err(|e| format!("base64 decode failed: {e}"))?;
    String::from_utf8(bytes).map_err(|e| format!("utf8 failed: {e}"))
}

/// 点击目标（引擎 Click v1 的补集）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClickTarget {
    /// CSS 选择器组（依次尝试，JS 兜底）
    Css(Vec<String>),
    /// 可见文本精确匹配（限定无子元素节点）
    Text(String),
    /// 视口坐标（虚拟表格等无文本节点场景）
    Coords { x: i32, y: i32 },
}

/// 生成经 `ExecuteJs` 执行的点击 JS（返回是否命中由脚本返回值表达）
pub fn click_js(target: &ClickTarget) -> String {
    match target {
        ClickTarget::Css(selectors) => {
            let list = selectors
                .iter()
                .map(|s| format!("'{s}'"))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "(()=>{{for(const sel of [{list}]){{for(const el of document.querySelectorAll(sel))\
                 {{try{{if(el.offsetParent!==null){{try{{el.click()}}catch(e){{}}}}return sel}}}}catch(e){{}}}}return null}})()"
            )
        }
        ClickTarget::Text(text) => {
            let want = text.replace('\'', "\\'");
            format!(
                "(()=>{{const els=[...document.querySelectorAll('*')].filter(e=>e.children.length===0);\
                 const c=els.find(e=>(e.innerText||'').trim()==='{want}');\
                 if(c){{c.click();return true}}return false}})()"
            )
        }
        ClickTarget::Coords { x, y } => {
            format!(
                "(()=>{{const el=document.elementFromPoint({x},{y});\
                 if(el){{el.click();return true}}return false}})()"
            )
        }
    }
}

/// 登录壳检查（防 SAAS 登录页误判：须见应用壳且不见安全登录按钮）
pub fn login_shell_ok(page_source: &str) -> bool {
    (page_source.contains("工作台") || page_source.contains("客户"))
        && !page_source.contains("安全登录")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(method: &str, url: &str) -> String {
        serde_json::json!({ "message": {
            "method": method,
            "params": { "request": { "method": "GET", "url": url } }
        }})
        .to_string()
    }

    #[test]
    fn parses_matching_xhr_only() {
        let entries = vec![
            entry(
                "Network.requestWillBeSent",
                "https://trade.joinf.com/rapi/d/customers?num=1",
            ),
            entry("Network.requestWillBeSent", "https://example.com/a.png"),
            entry("Network.responseReceived", "https://trade.joinf.com/rapi/x"),
        ];
        let out = parse_xhr_entries(&entries, "/rapi/");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].method, "GET");
        assert!(out[0].url.contains("num=1"));
    }

    #[test]
    fn skips_malformed_entries() {
        let out = parse_xhr_entries(&["not json".to_string()], "/rapi/");
        assert!(out.is_empty());
    }

    #[test]
    fn click_js_embeds_target() {
        let js = click_js(&ClickTarget::Text("跟进".to_string()));
        assert!(js.contains("跟进"));
        let js2 = click_js(&ClickTarget::Coords { x: 350, y: 225 });
        assert!(js2.contains("elementFromPoint(350,225)"));
    }

    #[test]
    fn body_decode_roundtrip() {
        assert_eq!(decode_response_body("hi", false).expect("plain"), "hi");
        assert_eq!(decode_response_body("aGk=", true).expect("b64"), "hi");
        assert!(decode_response_body("!!!", true).is_err());
    }

    #[test]
    fn login_shell_gate() {
        assert!(login_shell_ok("工作台 客户 列表"));
        assert!(!login_shell_ok("工作台 用户登录 安全登录"));
        assert!(!login_shell_ok("空白页"));
    }
}
