//! `nt_http_engine` — OpenAI 兼容 HTTP 引擎.
//!
//! 让 neobot 直连本地模型 (Ollama `http://127.0.0.1:11434/v1`,
//! LM Studio `http://127.0.0.1:1234/v1`, vLLM, 或
//! `OPENAI_BASE_URL` 指向的自建网关 — openbot 预留的同一道缝).
//! 同步阻塞实现 (`ureq`), 与 `EngineAdapter` 同步 trait 对齐, 无新增异步依赖.
//!
//! 安全: `api_key` 只活在内存 + 环境变量, 永不落 `config.json`
//! (`EngineKind::Http` 只存 `base_url`/`model`), 错误串永不携带 key
//! (服务端错误正文先过 `redact_detail` 再截断).

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::nt_audit::redact_detail;
use crate::nt_engine::{EngineAdapter, EngineTurn};
use crate::nt_error::NtBotError;
use crate::nt_types::{TokenUsage, TurnStatus};

/// 默认本地端点 (Ollama).
pub const DEFAULT_BASE_URL: &str = "http://127.0.0.1:11434/v1";
/// 默认超时秒.
pub const DEFAULT_TIMEOUT_SECS: u64 = 120;

/// HTTP 引擎配置 (`NEOBOT_BASE_URL` / `NEOBOT_API_KEY` / `NEOBOT_MODEL`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpEngineConfig {
    pub base_url: String,
    pub model: String,
    pub timeout_secs: u64,
}

impl HttpEngineConfig {
    /// 只解析端点 (base/key/timeout), 不要求模型 — `neobot models` 用.
    pub fn endpoint_from_env() -> Result<(String, String, u64), NtBotError> {
        let base_url = std::env::var("NEOBOT_BASE_URL")
            .unwrap_or_else(|_| DEFAULT_BASE_URL.to_owned());
        let api_key = std::env::var("NEOBOT_API_KEY").unwrap_or_default();
        let timeout_secs = std::env::var("NEOBOT_TIMEOUT_SECS")
            .ok()
            .and_then(|raw| raw.trim().parse::<u64>().ok())
            .filter(|secs| *secs > 0 && *secs <= 600)
            .unwrap_or(DEFAULT_TIMEOUT_SECS);
        let base_url = base_url.trim().trim_end_matches('/').to_owned();
        if !(base_url.starts_with("http://") || base_url.starts_with("https://")) {
            return Err(NtBotError::Invalid(format!(
                "base_url must start with http(s)://, got '{base_url}'"
            )));
        }
        Ok((base_url, api_key, timeout_secs))
    }

    pub fn from_env() -> Result<(Self, String), NtBotError> {
        let (base_url, api_key, timeout_secs) = Self::endpoint_from_env()?;
        let model = std::env::var("NEOBOT_MODEL").unwrap_or_default();
        let config = Self {
            base_url,
            model: model.trim().to_owned(),
            timeout_secs,
        };
        config.validate()?;
        Ok((config, api_key))
    }

    pub fn validate(&self) -> Result<(), NtBotError> {
        if !(self.base_url.starts_with("http://") || self.base_url.starts_with("https://")) {
            return Err(NtBotError::Invalid(format!(
                "base_url must start with http(s)://, got '{}'",
                self.base_url
            )));
        }
        if self.model.trim().is_empty() {
            return Err(NtBotError::Invalid(
                "NEOBOT_MODEL is required for http engine".to_owned(),
            ));
        }
        Ok(())
    }
}

/// OpenAI 兼容引擎 (key 内存持有, `Debug` 脱敏).
pub struct HttpEngine {
    config: HttpEngineConfig,
    api_key: String,
}

impl std::fmt::Debug for HttpEngine {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("HttpEngine")
            .field("base_url", &self.config.base_url)
            .field("model", &self.config.model)
            .field("api_key", &"[redacted]")
            .finish()
    }
}

impl HttpEngine {
    pub fn new(config: HttpEngineConfig, api_key: String) -> Result<Self, NtBotError> {
        config.validate()?;
        Ok(Self { config, api_key })
    }

    pub fn from_env() -> Result<Self, NtBotError> {
        let (config, api_key) = HttpEngineConfig::from_env()?;
        Self::new(config, api_key)
    }

    /// 仅列表模式 (不校验模型名) — `neobot models` 用.
    pub fn for_listing() -> Result<Self, NtBotError> {
        let (base_url, api_key, timeout_secs) = HttpEngineConfig::endpoint_from_env()?;
        Ok(Self::for_listing_with_base_and_key(&base_url, &api_key, timeout_secs))
    }

    fn for_listing_with_base_and_key(base_url: &str, api_key: &str, timeout_secs: u64) -> Self {
        Self {
            config: HttpEngineConfig {
                base_url: base_url.to_owned(),
                model: String::new(),
                timeout_secs,
            },
            api_key: api_key.to_owned(),
        }
    }

    #[cfg(test)]
    fn for_listing_with_base(base_url: &str) -> Self {
        Self::for_listing_with_base_and_key(base_url, "", 5)
    }

    fn chat_url(&self) -> String {
        format!("{}/chat/completions", self.config.base_url)
    }

    fn models_url(&self) -> String {
        format!("{}/models", self.config.base_url)
    }

    fn post_json(
        &self,
        url: &str,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, NtBotError> {
        let timeout = Duration::from_secs(self.config.timeout_secs);
        let mut request = ureq::post(url).timeout(timeout);
        if !self.api_key.trim().is_empty() {
            request = request.set("Authorization", &format!("Bearer {}", self.api_key.trim()));
        }
        let response = request.send_json(body).map_err(|err| http_err(&self.config.model, err))?;
        response.into_json::<serde_json::Value>().map_err(|err| NtBotError::Engine {
            engine: engine_id_of(&self.config.model),
            reason: format!("bad json response: {err}"),
        })
    }
    /// 列出服务端模型 (`GET /v1/models`), 返回 `(id, owned_by)`.
    /// `owned_by` 为空时回落为 id 的 `/` 前缀 (与服务端 `openai_list_models` 一致).
    pub fn list_models(&self) -> Result<Vec<(String, String)>, NtBotError> {
        let timeout = Duration::from_secs(self.config.timeout_secs.min(15));
        let mut request = ureq::get(&self.models_url()).timeout(timeout);
        if !self.api_key.trim().is_empty() {
            request = request.set("Authorization", &format!("Bearer {}", self.api_key.trim()));
        }
        let value = request
            .call()
            .map_err(|err| http_err(&self.config.model, err))?
            .into_json::<serde_json::Value>()
            .map_err(|err| NtBotError::Engine {
                engine: engine_id_of(&self.config.model),
                reason: format!("bad /models json: {err}"),
            })?;
        let mut out = Vec::new();
        if let Some(data) = value.get("data").and_then(|data| data.as_array()) {
            for entry in data {
                let Some(id) = entry.get("id").and_then(|id| id.as_str()) else {
                    continue;
                };
                let owner = entry
                    .get("owned_by")
                    .and_then(|owner| owner.as_str())
                    .map(str::to_owned)
                    .unwrap_or_else(|| {
                        id.split_once('/')
                            .map(|(prefix, _)| prefix.to_owned())
                            .unwrap_or_else(|| "neotrix".to_owned())
                    });
                out.push((id.to_owned(), owner));
            }
        }
        Ok(out)
    }
}

fn http_err(model: &str, err: ureq::Error) -> NtBotError {
    let reason = match err {
        ureq::Error::Status(code, response) => {
            let body = response.into_string().unwrap_or_default();
            let clean = redact_detail(&body);
            let mut cut = 300;
            while cut > 0 && !clean.is_char_boundary(cut) {
                cut -= 1;
            }
            let snippet = clean.get(..cut.min(clean.len())).unwrap_or("");
            format!("server status {code}: {snippet}")
        }
        ureq::Error::Transport(transport) => format!("transport: {transport}"),
    };
    NtBotError::Engine {
        engine: engine_id_of(model),
        reason,
    }
}

fn engine_id_of(model: &str) -> String {
    format!("http:{model}")
}

/// 池子 id 匹配 — 与服务端 (`openai_chat_completions`) 同语义:
/// 全等, 或裸名命中 `provider/model` 后缀, 或 `provider/` 前缀命中.
fn pool_match(entry_id: &str, wanted: &str) -> bool {
    if entry_id == wanted {
        return true;
    }
    if let Some(suffix) = entry_id.split_once('/').map(|(_, model)| model) {
        if suffix == wanted {
            return true;
        }
    }
    if let Some(prefix) = wanted.strip_suffix('/') {
        if let Some((entry_prefix, _)) = entry_id.split_once('/') {
            if entry_prefix == prefix {
                return true;
            }
        }
    }
    false
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    #[serde(default)]
    choices: Vec<ChatChoice>,
    #[serde(default)]
    usage: Option<ChatUsage>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    #[serde(default)]
    message: ChatMessage,
}

#[derive(Debug, Default, Deserialize)]
struct ChatMessage {
    #[serde(default)]
    content: String,
}

#[derive(Debug, Default, Deserialize)]
struct ChatUsage {
    #[serde(default)]
    prompt_tokens: i64,
    #[serde(default)]
    completion_tokens: i64,
}

#[derive(Debug, Deserialize)]
struct ModelsResponse {
    #[serde(default)]
    data: Vec<ModelEntry>,
}

#[derive(Debug, Deserialize)]
struct ModelEntry {
    #[serde(default)]
    id: String,
}

impl EngineAdapter for HttpEngine {
    fn engine_id(&self) -> &str {
        // `engine_id` 返回静态语义名; 模型名走 `model_name` (ledger 用).
        "http"
    }

    fn model_name(&self) -> &str {
        &self.config.model
    }

    fn probe(&self) -> Result<String, NtBotError> {
        let timeout = Duration::from_secs(self.config.timeout_secs.min(15));
        let mut request = ureq::get(&self.models_url()).timeout(timeout);
        if !self.api_key.trim().is_empty() {
            request = request.set("Authorization", &format!("Bearer {}", self.api_key.trim()));
        }
        let value = request
            .call()
            .map_err(|err| http_err(&self.config.model, err))?
            .into_json::<ModelsResponse>()
            .map_err(|err| NtBotError::Engine {
                engine: engine_id_of(&self.config.model),
                reason: format!("bad /models json: {err}"),
            })?;
        if !value.data.iter().any(|entry| pool_match(&entry.id, &self.config.model)) {
            return Err(NtBotError::Engine {
                engine: engine_id_of(&self.config.model),
                reason: format!(
                    "model '{}' not listed by server ({} models; try `neobot models`)",
                    self.config.model,
                    value.data.len()
                ),
            });
        }
        Ok(format!(
            "http ready ({} {})",
            self.config.base_url, self.config.model
        ))
    }

    fn run_turn(&self, prompt: &str, inbox: &[String]) -> Result<EngineTurn, NtBotError> {
        let mut messages = Vec::with_capacity(inbox.len() + 1);
        for item in inbox {
            messages.push(serde_json::json!({"role": "user", "content": item}));
        }
        messages.push(serde_json::json!({"role": "user", "content": prompt}));
        let body = serde_json::json!({
            "model": self.config.model,
            "messages": messages,
            "temperature": 0.2,
            "stream": false,
        });
        let value = self.post_json(&self.chat_url(), &body)?;
        let chat: ChatResponse = serde_json::from_value(value).map_err(|err| NtBotError::Engine {
            engine: engine_id_of(&self.config.model),
            reason: format!("unexpected chat schema: {err}"),
        })?;
        let Some(choice) = chat.choices.first() else {
            return Err(NtBotError::Engine {
                engine: engine_id_of(&self.config.model),
                reason: "empty choices".to_owned(),
            });
        };
        let usage = chat.usage.unwrap_or_default();
        Ok(EngineTurn {
            assistant_text: if choice.message.content.trim().is_empty() {
                "(empty reply)".to_owned()
            } else {
                choice.message.content.clone()
            },
            status: TurnStatus::Done,
            tool_calls: Vec::new(),
            usage: Some(TokenUsage {
                prompt_tokens: usage.prompt_tokens.max(0),
                completion_tokens: usage.completion_tokens.max(0),
                cost_usd: 0.0,
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{HttpEngine, HttpEngineConfig};
    use crate::nt_engine::EngineAdapter;

    fn test_config(base_url: &str) -> HttpEngineConfig {
        HttpEngineConfig {
            base_url: base_url.to_owned(),
            model: "test-model".to_owned(),
            timeout_secs: 5,
        }
    }

    /// 起一次性 fake OpenAI 服务器, 按 handler 应答后退出.
    fn fake_server(handler: impl FnOnce(&str, &str) -> String + Send + 'static) -> String {
        use std::io::{Read as _, Write as _};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr").to_string();
        std::thread::spawn(move || {
            let Ok((mut stream, _)) = listener.accept() else {
                return;
            };
            let mut buf = vec![0u8; 65536];
            let Ok(n) = stream.read(&mut buf) else {
                return;
            };
            let raw = String::from_utf8_lossy(&buf[..n]).into_owned();
            let mut lines = raw.lines();
            let request_line = lines.next().unwrap_or("").to_owned();
            let mut content_len = 0usize;
            for line in lines.by_ref() {
                if line.trim().is_empty() {
                    break;
                }
                if let Some(rest) = line.strip_prefix("Content-Length:") {
                    content_len = rest.trim().parse().unwrap_or(0);
                }
            }
            let body = raw
                .find("\r\n\r\n")
                .and_then(|idx| raw.get(idx + 4..))
                .unwrap_or("");
            let mut owned = body.to_owned();
            while owned.len() < content_len {
                let mut extra = vec![0u8; 4096];
                let Ok(m) = stream.read(&mut extra) else {
                    break;
                };
                if m == 0 {
                    break;
                }
                owned.push_str(&String::from_utf8_lossy(&extra[..m]));
            }
            let reply = handler(&request_line, &owned);
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                reply.len(),
                reply
            );
            let _ = stream.write_all(response.as_bytes());
        });
        format!("http://{addr}/v1")
    }

    #[test]
    fn chat_roundtrip_parses_content_and_usage() {
        let base = fake_server(|request_line, _body| {
            assert!(request_line.contains("POST /v1/chat/completions"));
            serde_json::json!({
                "choices": [{"message": {"content": "hello from stub"}}],
                "usage": {"prompt_tokens": 10, "completion_tokens": 4},
            })
            .to_string()
        });
        // stub 线程起速竞态: 最多等 2s 直到端口可连.
        let engine = HttpEngine::new(test_config(&base), String::new()).expect("engine");
        let mut turn = None;
        for _ in 0..40 {
            match engine.run_turn("hi", &[]) {
                Ok(done) => {
                    turn = Some(done);
                    break;
                }
                Err(_) => std::thread::sleep(std::time::Duration::from_millis(50)),
            }
        }
        let turn = turn.expect("turn");
        assert_eq!(turn.assistant_text, "hello from stub");
        let usage = turn.usage.expect("usage");
        assert_eq!((usage.prompt_tokens, usage.completion_tokens), (10, 4));
    }

    #[test]
    fn probe_requires_listed_model() {
        let base = fake_server(|request_line, _| {
            assert!(request_line.contains("GET /v1/models"));
            serde_json::json!({"data": [{"id": "test-model"}]}).to_string()
        });
        let engine = HttpEngine::new(test_config(&base), String::new()).expect("engine");
        let mut ok = false;
        for _ in 0..40 {
            if engine.probe().is_ok() {
                ok = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        assert!(ok);

        let base2 = fake_server(|_, _| serde_json::json!({"data": [{"id": "other"}]}).to_string());
        let missing = HttpEngine::new(
            HttpEngineConfig {
                base_url: base2,
                model: "nope".to_owned(),
                timeout_secs: 5,
            },
            String::new(),
        )
        .expect("engine");
        let mut denied = false;
        for _ in 0..40 {
            match missing.probe() {
                Err(err) if err.to_string().contains("not listed") => {
                    denied = true;
                    break;
                }
                _ => std::thread::sleep(std::time::Duration::from_millis(50)),
            }
        }
        assert!(denied);
    }

    #[test]
    fn pool_match_covers_pool_ids() {
        assert!(super::pool_match("ollama/qwen2.5", "ollama/qwen2.5"));
        assert!(super::pool_match("ollama/qwen2.5", "qwen2.5"));
        assert!(super::pool_match("llm7/codestral-latest", "llm7/"));
        assert!(!super::pool_match("ollama/qwen2.5", "llama3.2"));
        assert!(!super::pool_match("gpt-4o", "gpt-4o-mini"));
    }

    #[test]
    fn list_models_parses_pool_shape() {
        let base = fake_server(|request_line, _| {
            assert!(request_line.contains("GET /v1/models"));
            serde_json::json!({"object": "list", "data": [
                {"id": "ollama/qwen2.5", "object": "model", "owned_by": "ollama"},
                {"id": "bare-model", "object": "model"},
            ]})
            .to_string()
        });
        let engine = HttpEngine::for_listing_with_base(&base);
        let mut models = None;
        for _ in 0..40 {
            if let Ok(list) = engine.list_models() {
                models = Some(list);
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        let models = models.expect("models");
        assert_eq!(models.len(), 2);
        assert!(models.contains(&("ollama/qwen2.5".to_owned(), "ollama".to_owned())));
        // 无 owned_by 回落为前缀.
        assert!(models.contains(&("bare-model".to_owned(), "neotrix".to_owned())));
    }

    #[test]
    fn config_validation() {        let bad = HttpEngineConfig {
            base_url: "ftp://x".to_owned(),
            model: "m".to_owned(),
            timeout_secs: 5,
        };
        assert!(bad.validate().is_err());
        let no_model = HttpEngineConfig {
            base_url: "http://127.0.0.1:11434/v1".to_owned(),
            model: String::new(),
            timeout_secs: 5,
        };
        assert!(no_model.validate().is_err());
    }
}
