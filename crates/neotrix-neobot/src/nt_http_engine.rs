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
use crate::nt_types::{ToolCall, ToolName, TokenUsage, TranscriptItem, TranscriptRole, TurnStatus};

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
    /// 是否向模型开放 `computer_act` (执行仍受网关 allowlist 门控).
    offer_computer: bool,
}

impl std::fmt::Debug for HttpEngine {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("HttpEngine")
            .field("base_url", &self.config.base_url)
            .field("model", &self.config.model)
            .field("offer_computer", &self.offer_computer)
            .field("api_key", &"[redacted]")
            .finish()
    }
}

impl HttpEngine {
    pub fn new(config: HttpEngineConfig, api_key: String) -> Result<Self, NtBotError> {
        config.validate()?;
        Ok(Self {
            config,
            api_key,
            offer_computer: false,
        })
    }

    /// `NEOBOT_OFFER_COMPUTER=1` 时向模型开放 `computer_act` schema.
    pub fn with_computer(mut self, offer: bool) -> Self {
        self.offer_computer = offer;
        self
    }

    pub fn from_env() -> Result<Self, NtBotError> {
        let (config, api_key) = HttpEngineConfig::from_env()?;
        let offer = std::env::var("NEOBOT_OFFER_COMPUTER")
            .map(|raw| raw.trim() == "1")
            .unwrap_or(false);
        Ok(Self {
            config,
            api_key,
            offer_computer: offer,
        })
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
            offer_computer: false,
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

    fn chat_body(&self, prompt: &str, history: &[TranscriptItem], stream: bool) -> serde_json::Value {
        let mut messages = Vec::with_capacity(history.len() + 2);
        messages.push(serde_json::json!({
            "role": "system",
            "content": "你是 neobot 本地助手。需要行动时调用工具, 否则直接回复。结束时调用 set_turn_status。",
        }));
        for item in history {
            messages.push(transcript_message(item));
        }
        messages.push(serde_json::json!({"role": "user", "content": prompt}));
        let mut body = serde_json::json!({
            "model": self.config.model,
            "messages": messages,
            "temperature": 0.2,
            "stream": stream,
        });
        let tools = tool_schemas(self.offer_computer);
        if let Some(map) = body.as_object_mut() {
            map.insert("tools".to_owned(), serde_json::Value::Array(tools));
        }
        body
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

fn transcript_message(item: &TranscriptItem) -> serde_json::Value {
    match item.role {
        TranscriptRole::User => serde_json::json!({"role": "user", "content": item.content}),
        TranscriptRole::Assistant => {
            let calls: Vec<serde_json::Value> = item
                .tool_calls
                .iter()
                .map(|call| {
                    serde_json::json!({
                        "id": call.id,
                        "type": "function",
                        "function": {
                            "name": call.name.as_str(),
                            "arguments": call.args.to_string(),
                        },
                    })
                })
                .collect();
            if calls.is_empty() {
                serde_json::json!({"role": "assistant", "content": item.content})
            } else {
                serde_json::json!({
                    "role": "assistant",
                    "content": item.content,
                    "tool_calls": calls,
                })
            }
        }
        TranscriptRole::Tool => serde_json::json!({
            "role": "tool",
            "tool_call_id": item.tool_call_id.as_deref().unwrap_or(""),
            "content": item.content,
        }),
    }
}

/// OpenAI function schemas — 与本地网关工具 1:1 (`computer_act` 仅 opt-in).
fn tool_schemas(offer_computer: bool) -> Vec<serde_json::Value> {
    let mut tools = vec![
        serde_json::json!({"type": "function", "function": {
            "name": "bash",
            "description": "在 workspace 内执行 shell(唯一世界动作入口)",
            "parameters": {"type": "object", "properties": {
                "command": {"type": "string", "description": "shell 命令"},
            }, "required": ["command"]},
        }}),
        serde_json::json!({"type": "function", "function": {
            "name": "set_turn_status",
            "description": "终态协议信号, 防沉默即完成误判",
            "parameters": {"type": "object", "properties": {
                "status": {"type": "string", "enum": ["done", "continue", "needs_clarification", "blocked", "waiting"]},
                "reason": {"type": "string"},
            }, "required": ["status"]},
        }}),
        serde_json::json!({"type": "function", "function": {
            "name": "read_file",
            "description": "读 workspace 内文件(512KiB 上限)",
            "parameters": {"type": "object", "properties": {
                "path": {"type": "string"},
            }, "required": ["path"]},
        }}),
        serde_json::json!({"type": "function", "function": {
            "name": "write_file",
            "description": "写 workspace 内文件(2MiB 上限)",
            "parameters": {"type": "object", "properties": {
                "path": {"type": "string"},
                "content": {"type": "string"},
            }, "required": ["path", "content"]},
        }}),
        serde_json::json!({"type": "function", "function": {
            "name": "edit_file",
            "description": "精确一次匹配编辑(多/零匹配拒绝)",
            "parameters": {"type": "object", "properties": {
                "path": {"type": "string"},
                "old": {"type": "string"},
                "new": {"type": "string"},
            }, "required": ["path", "old", "new"]},
        }}),
    ];
    if offer_computer {
        tools.push(serde_json::json!({"type": "function", "function": {
            "name": "computer_act",
            "description": "受控 computer 动作(网关 allowlist 门控, 无后端时诚实失败)",
            "parameters": {"type": "object", "properties": {
                "action": {"type": "string", "enum": ["navigate", "click", "type", "key", "scroll", "screenshot", "read_file", "write_file", "list_files"]},
                "target": {"type": "string"},
                "text": {"type": "string"},
            }, "required": ["action"]},
        }}));
    }
    tools
}

/// 非流式 chat → EngineTurn. 有 tool_calls 即 Continue (执行后再回填).
fn parse_chat_turn(model: &str, value: serde_json::Value) -> Result<EngineTurn, NtBotError> {
    let chat: ChatResponse = serde_json::from_value(value).map_err(|err| NtBotError::Engine {
        engine: engine_id_of(model),
        reason: format!("unexpected chat schema: {err}"),
    })?;
    let Some(choice) = chat.choices.first() else {
        return Err(NtBotError::Engine {
            engine: engine_id_of(model),
            reason: "empty choices".to_owned(),
        });
    };
    let tool_calls = choice
        .message
        .tool_calls
        .iter()
        .map(|raw| decode_tool_call(&raw.id, &raw.function.name, &raw.function.arguments))
        .collect::<Vec<_>>();
    let usage = chat.usage.unwrap_or_default();
    let mut content = choice.message.content.as_deref().unwrap_or("").to_owned();
    if choice.finish_reason.as_deref() == Some("length") {
        content = format!("[truncated by max_tokens] {content}");
    }
    Ok(finish_turn(
        &content,
        tool_calls,
        Some(TokenUsage {
            prompt_tokens: usage.prompt_tokens.max(0),
            completion_tokens: usage.completion_tokens.max(0),
            cost_usd: 0.0,
        }),
    ))
}

fn decode_tool_call(id: &str, name: &str, arguments: &str) -> ToolCall {
    let args = serde_json::from_str(arguments).unwrap_or(serde_json::Value::Null);
    ToolCall {
        id: if id.is_empty() {
            format!("call-{name}")
        } else {
            id.to_owned()
        },
        name: ToolName::parse(name),
        args,
    }
}

fn finish_turn(content: &str, tool_calls: Vec<ToolCall>, usage: Option<TokenUsage>) -> EngineTurn {
    let text = if content.trim().is_empty() && tool_calls.is_empty() {
        "(empty reply)".to_owned()
    } else {
        content.to_owned()
    };
    let status = if tool_calls.is_empty() {
        TurnStatus::Done
    } else {
        TurnStatus::Continue
    };
    EngineTurn {
        assistant_text: text,
        status,
        tool_calls,
        usage,
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
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct ChatMessage {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    tool_calls: Vec<RawToolCall>,
}

#[derive(Debug, Default, Deserialize)]
struct RawToolCall {
    #[serde(default)]
    id: String,
    #[serde(default)]
    function: RawFunction,
}

#[derive(Debug, Default, Deserialize)]
struct RawFunction {
    #[serde(default)]
    name: String,
    #[serde(default)]
    arguments: String,
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
        let history: Vec<TranscriptItem> = inbox
            .iter()
            .map(|content| TranscriptItem {
                role: TranscriptRole::User,
                content: content.clone(),
                tool_calls: Vec::new(),
                tool_call_id: None,
            })
            .collect();
        self.run_turn_with_history(prompt, &history)
    }

    fn run_turn_with_history(
        &self,
        prompt: &str,
        history: &[TranscriptItem],
    ) -> Result<EngineTurn, NtBotError> {
        let body = self.chat_body(prompt, history, false);
        let value = self.post_json(&self.chat_url(), &body)?;
        parse_chat_turn(&self.config.model, value)
    }

    fn run_turn_stream(
        &self,
        prompt: &str,
        history: &[TranscriptItem],
        on_delta: &mut dyn FnMut(&str),
    ) -> Result<EngineTurn, NtBotError> {
        use std::io::BufRead as _;
        let body = self.chat_body(prompt, history, true);
        let timeout = Duration::from_secs(self.config.timeout_secs);
        let mut request = ureq::post(&self.chat_url()).timeout(timeout);
        if !self.api_key.trim().is_empty() {
            request = request.set("Authorization", &format!("Bearer {}", self.api_key.trim()));
        }
        let response = request
            .send_json(body)
            .map_err(|err| http_err(&self.config.model, err))?;
        let reader = response.into_reader();
        let mut content = String::new();
        let mut partials: Vec<StreamToolCall> = Vec::new();
        let mut usage = ChatUsage::default();
        for line in std::io::BufReader::new(reader).lines() {
            let line = line.map_err(|err| NtBotError::Engine {
                engine: engine_id_of(&self.config.model),
                reason: format!("stream read: {err}"),
            })?;
            let data = line.strip_prefix("data:").map(str::trim).unwrap_or("");
            if data.is_empty() || data == "[DONE]" {
                continue;
            }
            let chunk: serde_json::Value = match serde_json::from_str(data) {
                Ok(chunk) => chunk,
                Err(_) => continue,
            };
            if let Some(text) = chunk
                .pointer("/choices/0/delta/content")
                .and_then(|value| value.as_str())
            {
                content.push_str(text);
                on_delta(text);
            }
            if let Some(calls) = chunk
                .pointer("/choices/0/delta/tool_calls")
                .and_then(|value| value.as_array())
            {
                for call in calls {
                    let index = call.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                    while partials.len() <= index {
                        partials.push(StreamToolCall::default());
                    }
                    let Some(slot) = partials.get_mut(index) else {
                        continue;
                    };
                    if let Some(id) = call.get("id").and_then(|v| v.as_str()) {
                        if !id.is_empty() {
                            slot.id = id.to_owned();
                        }
                    }
                    if let Some(name) = call.pointer("/function/name").and_then(|v| v.as_str()) {
                        if !name.is_empty() {
                            slot.name = name.to_owned();
                        }
                    }
                    if let Some(args) = call.pointer("/function/arguments").and_then(|v| v.as_str()) {
                        slot.arguments.push_str(args);
                    }
                }
            }
            // neotrix 服务端也可能在末 chunk 带 usage.
            if let Some(value) = chunk.get("usage") {
                if let Ok(parsed) = serde_json::from_value::<ChatUsage>(value.clone()) {
                    usage = parsed;
                }
            }
        }
        let tool_calls = partials
            .into_iter()
            .filter(|slot| !slot.name.is_empty())
            .map(|slot| decode_tool_call(&slot.id, &slot.name, &slot.arguments))
            .collect::<Vec<_>>();
        Ok(finish_turn(
            &content,
            tool_calls,
            Some(TokenUsage {
                prompt_tokens: usage.prompt_tokens.max(0),
                completion_tokens: usage.completion_tokens.max(0),
                cost_usd: 0.0,
            }),
        ))
    }
}

#[derive(Debug, Default)]
struct StreamToolCall {
    id: String,
    name: String,
    arguments: String,
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
    fn stream_accumulates_deltas_and_tool_calls() {
        let sse = "data: {\"choices\":[{\"delta\":{\"content\":\"hel\"}}]}\n\n\
                   data: {\"choices\":[{\"delta\":{\"content\":\"lo\"}}]}\n\n\
                   data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"c1\",\"function\":{\"name\":\"bash\",\"arguments\":\"{\\\"comma\"}}]}}]}\n\n\
                   data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"function\":{\"arguments\":\"nd\\\":\\\"echo hi\\\"}\"}}]}}]}\n\n\
                   data: [DONE]\n\n";
        let base = fake_raw_server(sse);
        let engine = HttpEngine::new(test_config(&base), String::new()).expect("engine");
        let mut deltas = String::new();
        let mut turn = None;
        for _ in 0..40 {
            let mut capture = String::new();
            match engine.run_turn_stream("hi", &[], &mut |delta| capture.push_str(delta)) {
                Ok(done) => {
                    deltas = capture;
                    turn = Some(done);
                    break;
                }
                Err(_) => std::thread::sleep(std::time::Duration::from_millis(50)),
            }
        }
        let turn = turn.expect("turn");
        assert_eq!(deltas, "hello");
        assert_eq!(turn.assistant_text, "hello");
        assert_eq!(turn.tool_calls.len(), 1);
        assert_eq!(turn.tool_calls[0].id, "c1");
        assert_eq!(
            turn.tool_calls[0].args.get("command").and_then(|v| v.as_str()),
            Some("echo hi")
        );
    }

    /// 原始字节应答的 fake 服务器 (SSE 用).
    fn fake_raw_server(reply: &str) -> String {
        use std::io::{Read as _, Write as _};
        let reply = reply.to_owned();
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
            let content_len = raw
                .lines()
                .take_while(|line| !line.trim().is_empty())
                .filter_map(|line| line.strip_prefix("Content-Length:"))
                .filter_map(|rest| rest.trim().parse::<usize>().ok())
                .next()
                .unwrap_or(0);
            let mut owned = raw
                .find("\r\n\r\n")
                .and_then(|idx| raw.get(idx + 4..))
                .unwrap_or("")
                .to_owned();
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
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                reply.len(),
                reply
            );
            let _ = stream.write_all(response.as_bytes());
        });
        format!("http://{addr}/v1")
    }

    /// 按序应答的 fake 服务器 (多跳回路用, 用完即停).
    fn fake_server_seq(replies: Vec<String>) -> String {
        use std::io::{Read as _, Write as _};
        use std::sync::{Arc, Mutex};
        let queue = Arc::new(Mutex::new(replies));
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr").to_string();
        std::thread::spawn(move || {
            for _ in 0..16 {
                let Ok((mut stream, _)) = listener.accept() else {
                    return;
                };
                let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(5)));
                let mut buf = vec![0u8; 65536];
                let Ok(n) = stream.read(&mut buf) else {
                    continue;
                };
                if n == 0 {
                    continue;
                }
                let reply = queue
                    .lock()
                    .ok()
                    .and_then(|mut q| if q.is_empty() { None } else { Some(q.remove(0)) })
                    .unwrap_or_else(|| "{}".to_owned());
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    reply.len(),
                    reply
                );
                if stream.write_all(response.as_bytes()).is_err() {
                    return;
                }
            }
        });
        format!("http://{addr}/v1")
    }

    #[test]
    fn tool_loop_runs_bash_then_done() {
        let first = serde_json::json!({
            "choices": [{"message": {
                "content": "",
                "tool_calls": [{"id": "c1", "type": "function",
                    "function": {"name": "bash", "arguments": "{\"command\":\"echo loop-hi\"}"}}],
            }, "finish_reason": "tool_calls"}],
            "usage": {"prompt_tokens": 20, "completion_tokens": 5},
        })
        .to_string();
        let second = serde_json::json!({
            "choices": [{"message": {"content": "saw loop-hi"}, "finish_reason": "stop"}],
            "usage": {"prompt_tokens": 30, "completion_tokens": 3},
        })
        .to_string();
        let base = fake_server_seq(vec![first, second]);
        let engine = HttpEngine::new(test_config(&base), String::new()).expect("engine");
        let dir = std::env::temp_dir().join("neobot-toolloop-test");
        let _ = std::fs::remove_dir_all(&dir);
        let config = crate::nt_config::NeobotConfig {
            data_dir: dir.clone(),
            workspace_dir: dir.join("workspace"),
            policy_mode: crate::nt_config::PolicyMode::Enforce,
            human_has_control: false,
            max_steps: 4,
            engine: crate::nt_config::EngineKind::Echo,
            computer_allow: Vec::new(),
            computer_hosts: Vec::new(),
        };
        config.validate().expect("validate");
        let store = crate::nt_store::NeobotStore::open(":memory:").expect("open");
        let mut status = None;
        for _ in 0..40 {
            match crate::run_local_turn(&store, &config, &engine, "loop", "go") {
                Ok(done) => {
                    status = Some(done);
                    break;
                }
                Err(_) => std::thread::sleep(std::time::Duration::from_millis(50)),
            }
        }
        assert_eq!(status, Some(crate::nt_types::TurnStatus::Done));
        // bash 真执行且审计放行, 任务终态 Done.
        let audits = store.list_audit(20).expect("audits");
        assert!(audits.iter().any(|event| event.tool == "bash"
            && matches!(event.decision, crate::nt_audit::AuditDecision::Allow)));
        let tasks = store.list_tasks(10).expect("list");
        assert!(tasks
            .iter()
            .any(|task| task.status == crate::nt_types::TaskStatus::Done));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn config_validation() {
        let bad = HttpEngineConfig {
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
