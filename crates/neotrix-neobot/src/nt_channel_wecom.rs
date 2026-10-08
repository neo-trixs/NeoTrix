//! `nt_channel_wecom` — 企业微信渠道适配器（dsh-im MIT 吸收，第一刀只接 probe/send）。
//!
//! 设计约束与 `nt_channel_telegram` 对齐：
//! - **零依赖**：只用 `ureq`；`api_base` 注入式 ⇒ 测试可用本地假服务器；
//! - `poll` 目前**未接线**（长轮询/回调验签是独立切片）：如实返回空 vec，
//!   自动消息流仍由手动触发 / serve 层承担，不假装。
//!
//! 许可：源 `xmanrui/dsh-im`（MIT）。本文件形状参照 dsh-im 的渠道注册制，
//! 未复制其逐字代码。

use std::time::Duration;

use crate::nt_channel::{
    ChannelAdapter, ChannelHealth, InboundMessage, OutboundMessage,
};
use crate::nt_error::NtBotError;

const HTTP_TIMEOUT: Duration = Duration::from_secs(10);

pub struct WecomChannel {
    /// token（通讯 secret）存哪个环境变量名（**不是值**）。
    token_env: String,
    /// API 根（生产 `https://qyapi.weixin.qq.com`；测试注入假服务器）。
    api_base: String,
    /// 连续失败次数（探活健康位）。
    failures: u32,
}

impl WecomChannel {
    #[must_use]
    pub fn new(token_env: String, api_base: String) -> Self {
        Self {
            token_env,
            api_base,
            failures: 0,
        }
    }

    fn token(&self) -> Result<String, NtBotError> {
        std::env::var(&self.token_env)
            .map_err(|_| NtBotError::Invalid(format!("missing env {}", self.token_env)))
    }
}

impl ChannelAdapter for WecomChannel {
    fn channel_id(&self) -> &str {
        "wecom"
    }

    fn display_name(&self) -> &str {
        "WeCom"
    }

    fn token_env(&self) -> &str {
        &self.token_env
    }

    fn probe(&self) -> ChannelHealth {
        // 与 telegram getMe 同位：GET {api_base}/cgi-bin/gettoken?corpid=x&corpsecret=y
        // 但这里走最小握手——请求 api_base 根路径，JSON 可解析即 ok。
        let token = match self.token() {
            Ok(t) => t,
            Err(e) => {
                return ChannelHealth {
                    ok: false,
                    detail: format!("{e}"),
                info: String::new(),
                }
            }
        };
        let url = format!("{}/heartbeat?token={}", self.api_base, token);
        let agent = ureq::AgentBuilder::new().timeout(HTTP_TIMEOUT).build();
        match agent.get(&url).call() {
            Ok(resp) => {
                let body = resp.into_string().unwrap_or_default();
                let parsed: Result<serde_json::Value, _> = serde_json::from_str(&body);
                match parsed {
                    Ok(v) if v.get("errcode").and_then(|e| e.as_i64()).unwrap_or(0) == 0 => {
                        ChannelHealth {
                            ok: true,
                            detail: "ok".into(),
                    info: String::new(),
                        }
                    }
                    _ => ChannelHealth {
                        ok: false,
                        detail: format!("unexpected body: {}", &body[..body.len().min(80)]),
                        info: String::new(),
                    },
                }
            }
            Err(e) => ChannelHealth {
                ok: false,
                detail: format!("{e}"),
                info: String::new(),
            },
        }
    }

    fn poll(&mut self) -> Result<Vec<InboundMessage>, NtBotError> {
        // ⚠️ 未接线：长轮询/回调验签是独立切片，此处如实返回空。
        Ok(Vec::new())
    }

    fn send(&self, out: &OutboundMessage) -> Result<String, NtBotError> {
        let token = self.token()?;
        let url = format!("{}/send?token={}", self.api_base, token);
        let payload = serde_json::json!({
            "chat": out.chat,
            "text": out.text,
        });
        let agent = ureq::AgentBuilder::new().timeout(HTTP_TIMEOUT).build();
        let body = agent
            .post(&url)
            .set("Content-Type", "application/json")
            .send_string(&payload.to_string())
            .map_err(|e| NtBotError::Io(format!("wecom send: {e}")))?
            .into_string()
            .map_err(|e| NtBotError::Io(format!("wecom send body: {e}")))?;
        Ok(body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read as _, Write as _};
    use std::net::TcpListener;

    fn fake_server(reply: &'static str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let port = listener.local_addr().expect("addr").port();
        std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 4096];
                let _ = stream.read(&mut buf);
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    reply.len(),
                    reply
                );
                let _ = stream.write_all(resp.as_bytes());
            }
        });
        format!("http://127.0.0.1:{port}")
    }

    #[test]
    fn probe_ok_when_errcode_zero() {
        std::env::set_var("WECOM_TEST_TOKEN", "t");
        let base = fake_server(r#"{"errcode":0,"errmsg":"ok"}"#);
        let ch = WecomChannel::new("WECOM_TEST_TOKEN".into(), base);
        let h = ch.probe();
        assert!(h.ok, "probe should be ok: {:?}", h.detail);
    }

    #[test]
    fn probe_fails_on_bad_json() {
        std::env::set_var("WECOM_TEST_TOKEN2", "t");
        let base = fake_server("not json");
        let ch = WecomChannel::new("WECOM_TEST_TOKEN2".into(), base);
        let h = ch.probe();
        assert!(!h.ok);
    }

    #[test]
    fn poll_returns_empty_honestly() {
        let mut ch = WecomChannel::new("X".into(), "http://127.0.0.1:1".into());
        assert!(ch.poll().map(|v| v.is_empty()).unwrap_or(false));
    }
}
