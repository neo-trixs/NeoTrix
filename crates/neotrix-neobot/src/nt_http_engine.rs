//! `nt_http_engine` — OpenAI 兼容 HTTP 引擎.
//!
//! 让 neobot 直连本地模型（Ollama `http://127.0.0.1:11434/v1`，
//! LM Studio `http://127.0.0.1:1234/v1`，vLLM，或
//! `OPENAI_BASE_URL` 指向的自建网关）。
//! 同步阻塞实现（`ureq`），与 `EngineAdapter` 同步 trait 对齐，无新增异步依赖.
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
    /// 跨会话个人记忆（`MEMORY.md` 全文；None/空则不注入）。
    memory_context: Option<String>,
}

impl std::fmt::Debug for HttpEngine {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("HttpEngine")
            .field("base_url", &self.config.base_url)
            .field("model", &self.config.model)
            .field("offer_computer", &self.offer_computer)
            .field("memory_context", &self.memory_context.as_ref().map(|_| "[set]"))
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
            memory_context: None,
        })
    }

    /// `NEOBOT_OFFER_COMPUTER=1` 时向模型开放 `computer_act` schema.
    pub fn with_computer(mut self, offer: bool) -> Self {
        self.offer_computer = offer;
        self
    }

    /// 注入跨会话记忆（`MEMORY.md` 全文；调用方从配置数据目录读）。
    pub fn with_memory_context(mut self, memory: Option<String>) -> Self {
        self.memory_context = memory.filter(|m| !m.trim().is_empty());
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
            memory_context: None,
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
            memory_context: None,
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
        let mut system = SYSTEM_PROMPT.to_owned();
        if self.vision_capable() {
            system.push_str(VISION_PROMPT);
        }
        // Qwen-MM-Plugins 会话工具的挂载判据与 `tool_schemas` 同一函数
        // （`resolve_core_launch().is_ok()`）——两处必须同时成立，否则会出现
        // 「提示词教它用、schema 里却没有」或反向的空转。
        if qwen_mm_mounted() {
            system.push_str(QWEN_MM_PROMPT);
        }
        // PDF 定位工具是**常挂载**的（编译进二进制），所以提示词也无条件跟着上 ——
        // 与 Qwen 那组「挂载判据必须两处同源」是同一条纪律的另一面。
        system.push_str(PDF_GROUND_PROMPT);
        if let Some(memory) = self.memory_context.as_deref() {
            system.push_str("\n\n");
            system.push_str(memory);
        }
        messages.push(serde_json::json!({
            "role": "system",
            "content": system,
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
        let tools = tool_schemas(self.offer_computer, self.vision_capable());
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
        TranscriptRole::Tool => {
            let tool_call_id = item.tool_call_id.as_deref().unwrap_or("");
            match item.image.as_ref() {
                // 无图 = 今天的样子（content 是字符串）。不为了「统一」把所有
                // tool 行都改成数组形状：老版本 Ollama 之类的本地端点对
                // tool 消息的数组 content 挑刺，没必要给无关轮次惹这个麻烦。
                None => serde_json::json!({
                    "role": "tool",
                    "tool_call_id": tool_call_id,
                    "content": item.content,
                }),
                // 有图：content 升级成数组 —— 文本一段 + 真图像部件一段。
                // 图像**必须**以 `image_url` 部件抵达模型：base64 若落回字符串
                // 通道，模型收到的是一堵字符墙，它会开始「描述」没看见的东西。
                Some(image) => serde_json::json!({
                    "role": "tool",
                    "tool_call_id": tool_call_id,
                    "content": [
                        {"type": "text", "text": item.content},
                        {"type": "image_url", "image_url": {"url": image.data_url()}},
                    ],
                }),
            }
        }
    }
}

/// 系统提示（CLI 即协议：模型唯一的世界动作入口是
/// `bash` 里的 `neobot` CLI 子命令 + 3 个文件工具；`set_turn_status`
/// 声明终态，结束时必调）。
const SYSTEM_PROMPT: &str = "你是 neobot 本地助手，跑在用户自己的机器上。\n\n世界动作只能经 bash 调 `neobot` CLI（本地可执行，无密钥）：\n- `neobot task list` 看任务；`neobot task claim <id> --actor <你名字>` 认领；`neobot task release` 交回；`neobot task cancel/retry` 取消/重跑\n- `neobot audit list` 看审计（只读）；`neobot models` 看本地模型池\n- `neobot routine list` 看定时例行；`neobot skill list` 看已装技能\n- `neobot ledger` 看成本账；`neobot doctor` 自检\n\n联网能力（客户端直调）：`web_search` 查资料（Bing→Wikipedia 回退，证据行自带出处）；`web_fetch` 抓页面正文（只收 http/https）。时效问题先搜再答，不凭记忆编。\n\n文件读写在 workspace 内：read_file / write_file / edit_file（edit 必须精确一次匹配）。\n技能只是指令参考，不扩展能力：你能调的只有网关后的工具。\nset_turn_status 声明本轮状态：done（办完）/ continue（还有活）/ needs_clarification（要问一句）/ blocked（明确失败）/ waiting（已行动、等外部）。纯回复也要先调 set_turn_status 再结束。\n拿不准就问（needs_clarification），不要瞎猜执行。\n\n回答格式契约（降信息密度）：\n- 结论先行（一句话先给答案），再给依据/步骤；不复述用户问题。\n- 超过 5 行用分节（结论/依据/下一步），列表优先，段落不超过 3 行。\n- 联网结论必须带出处（[标题 — url]）；不确定的标“不确定”，不编。\n- 拒绝废话开场（不说“好的”“当然”）；无可答时直接说缺什么。\n- 语气像人：有温度、直接，短句为主；坏消息先给结论再给原因；办成了可以一句轻快确认，不许表情包刷屏、不许过度寒暄。";

/// 视觉能力附加条款 —— 只在 `vision_capable()` 为真时接在系统提示后面。
///
/// 为什么要单独一段：模型看见一张**真的**进了上下文的图片时，最该做的是描述它；
/// 而工具失败/没挂载时最该做的是**说没看见**。这两种情形必须用不同的话术
/// 区分开，否则模型会把「我没拿到图」也讲成一段像模像样的画面描述。
const VISION_PROMPT: &str = "\n\n视觉：你有 `read_image` 工具。工具结果里带图片时，你会收到**真的**图像部件——直接描述你看见的画面。\
工具报错、或结果里只有路径而没有图像部件时，说明这张图没到你手上：直说「我看不到这张图」，\
不要根据文件名、路径或上下文猜画面内容。\n";

/// 多模态工具挂载后的附加条款 —— 只在 Qwen-MM 会话工具**已挂载**时接在
/// 系统提示后面（挂载判据 = 探测到服务器，见 `tool_schemas`）。
///
/// 存在的理由：这些工具的能力来自外部 Python 进程，**可能缺依赖**（ffmpeg、
/// LibreOffice…）。没挂载时模型压根看不见它们（不会浪费轮次）；挂载了但某个
/// 工具运行期报缺依赖时，模型必须**转述那句错**而不是编一个结果 —— 与
/// `VISION_PROMPT` 同一纪律：没看到就说没看到。
const QWEN_MM_PROMPT: &str = "\n\n多模态文件工具（Qwen-MM-Plugins 会话）：\
`qwen_media_info` / `qwen_read_video` / `qwen_visualize` 读媒体与文档，`qwen_save_view` 把页/帧落盘。\
路径一律给 workspace 内的相对路径（绝对路径与 `..` 会被网关拒）。\
看视频/长文档：先 `qwen_media_info`，再小步取帧或渲染页面，别一次要几十页。\
任一工具报错（含「缺 ffmpeg / 缺 LibreOffice」）时，把那句错如实转述给用户，别假装成功、别用文字描述替代你没看到的画面。\n";

/// PDF 文字定位（本地、零外部依赖，故**常挂载**）。
///
/// 存在的理由与 Qwen 那组相反：它不需要探测任何东西（`lopdf` 编译进二进制），
/// 所以「按能力挂载」的成本为零、收益为零。两条纪律的共同点是**提示词与
/// schema 必须同时到位**，否则模型会被教一个调不到的工具。
const PDF_GROUND_PROMPT: &str = "\n\nPDF 定位：`pdf_ground_text` 在 PDF 文字层里按词定位，返回页码 + 0-1000 归一化框（y 已翻成图像坐标系）。\
要指「这句话在哪」或要裁某段时先调它，别自己编坐标。\
它只认 PDF 文字层：扫描件/文字转轮廓的 PDF 会明说没有文字层 —— 那时**如实说定位不到**，\
改用 `qwen_visualize` 看渲染页，别拿估计的框当定位结果交差。\n";

/// Qwen-MM-Plugins 会话工具是否挂载（唯一判据，`tool_schemas` 与
/// `chat_body` 共用——提示词与 schema 必须同步，否则模型会被教一个它调不到的
/// 工具，或拿到一个没人教它的工具）。
///
/// 探测是**零 spawn** 的 PATH/目录检查（`nt_qwen_mm::resolve_core_launch`），
/// 所以每请求做一次也就几次 syscall；探测失败 ⇒ 不挂载（fail-closed：
/// 不支持 ≠ 已列出）。
fn qwen_mm_mounted() -> bool {
    crate::nt_qwen_mm::resolve_core_launch().is_ok()
}

/// OpenAI function schemas — 与本地网关工具 1:1 (`computer_act` 仅 opt-in).
///
/// `read_image` 同样**按能力挂载**：引擎/模型看不见图时就不摆上桌。挂了却在
/// 执行期才失败，模型会白白浪费一轮去发现「这台机器没有眼睛」。
fn tool_schemas(offer_computer: bool, offer_vision: bool) -> Vec<serde_json::Value> {
    let offer_qwen_mm = qwen_mm_mounted();
    let mut tools = vec![
        serde_json::json!({"type": "function", "function": {
            "name": "bash",
            "description": "在 workspace 内执行 shell(唯一世界动作入口；`neobot task/audit/routine/skill/ledger` 等子命令走这里)",
            "parameters": {"type": "object", "properties": {
                "command": {"type": "string", "description": "shell 命令"},
            }, "required": ["command"]},
        }}),
        serde_json::json!({"type": "function", "function": {
            "name": "set_turn_status",
            "description": "终态协议信号, 防沉默即完成误判（status 必填；reason 写清为什么停）",
            "parameters": {"type": "object", "properties": {
                "status": {"type": "string", "enum": ["done", "continue", "needs_clarification", "blocked", "waiting"]},
                "reason": {"type": "string"},
                "next_step": {"type": "string"},
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
    // 联网读写（对话即 crystal 全能力外表：chat 路径直调，不经服务端中转）。
    tools.push(serde_json::json!({"type": "function", "function": {
            "name": "web_search",
            "description": "联网搜索（Bing → Wikipedia 回退；证据行返回；count 缺省 5 上限 10）。新闻类直接传中文热点词（如“今日要闻”），勿改写，改写会丢热点直连。",
            "parameters": {"type": "object", "properties": {
                "query": {"type": "string", "description": "搜索关键词（新闻类保留热点词）"},
                "count": {"type": "integer", "description": "条数 1-10"},
            }, "required": ["query"]},
    }}));
    tools.push(serde_json::json!({"type": "function", "function": {
        "name": "web_fetch",
        "description": "抓取网页正文（只允许 http/https；4000 字截断）",
        "parameters": {"type": "object", "properties": {
            "url": {"type": "string", "description": "http(s) 地址"},
        }, "required": ["url"]},
    }}));
    if offer_vision {
        tools.push(serde_json::json!({"type": "function", "function": {
            "name": "read_image",
            "description": "真正**看**一张 workspace 内的图片（png/jpeg/gif/webp，按魔数判型，4MiB 上限）。图片会作为多模态部件随下一轮请求抵达你，你能直接描述画面。文本文件用 read_file，别用这个。",
            "parameters": {"type": "object", "properties": {
                "path": {"type": "string", "description": "workspace 内相对路径（如 attachments/shot.png）"},
            }, "required": ["path"]},
        }}));
    }
    // Qwen-MM-Plugins 会话工具（模型自主调；**按能力挂载**——探测不到服务器就
    // 不摆上桌，理由同 `read_image`：挂了却在执行期才失败，模型会白白浪费
    // 一轮去发现「这台机器没装」）。
    //
    // 全部 `required` 只留路径类参数：`budget`/`fps`/`pages` 交给上游默认值
    // （core 的默认值是调过的：budget=normal、fps=0 自动按片长选）。
    if offer_qwen_mm {
        tools.push(serde_json::json!({"type": "function", "function": {
            "name": "qwen_media_info",
            "description": "先查媒体/文档元数据（ffprobe 读头，不解码整片，秒回）。视频/音频**先调这个**再看别处：拿 codec/fps/旋转/VFR 标志，否则时间戳算错。NIfTI 会给 shape/dtype/spacing/orientation。",
            "parameters": {"type": "object", "properties": {
                "path": {"type": "string", "description": "workspace 内相对路径"},
                "raw": {"type": "boolean", "description": "返回原始 JSON（默认 false=人读摘要）"},
            }, "required": ["path"]},
        }}));
        tools.push(serde_json::json!({"type": "function", "function": {
            "name": "qwen_read_video",
            "description": "抽视频帧（fps=0 按片长自动选采样率；预算 small/normal/large 控分辨率）。抽出的帧会作为图像部件随下一轮抵达你。**先调 qwen_media_info** 再调它。",
            "parameters": {"type": "object", "properties": {
                "video_path": {"type": "string", "description": "workspace 内相对路径"},
                "fps": {"type": "number", "description": "采样率；0=自动（默认）"},
                "budget": {"type": "string", "enum": ["small", "normal", "large"], "description": "分辨率预算（默认 normal）"},
                "start_time": {"type": "number", "description": "窗口起（秒）"},
                "end_time": {"type": "number", "description": "窗口止（秒）"},
            }, "required": ["video_path"]},
        }}));
        tools.push(serde_json::json!({"type": "function", "function": {
            "name": "qwen_visualize",
            "description": "把任意文件渲染成图给你看：PDF/SVG 页、CSV/XLSX 表、代码高亮、DrawIO、字幕、NIfTI 体（本地只读，非诊断用途）、GIS、3D、notebook、LaTeX。**看文档/数据/代码用这个**，别只 read_file 读文本。",
            "parameters": {"type": "object", "properties": {
                "file_path": {"type": "string", "description": "workspace 内相对路径"},
                "pages": {"type": "string", "description": "页码范围，如 \"1-5\""},
                "budget": {"type": "string", "enum": ["small", "normal", "large"], "description": "分辨率预算（默认 large）"},
                "max_pages": {"type": "integer", "description": "页数上限（默认 20）"},
            }, "required": ["file_path"]},
        }}));
        tools.push(serde_json::json!({"type": "function", "function": {
            "name": "qwen_save_view",
            "description": "把文档页/视频帧**落盘**成 workspace 内 .neotrix-mm/ 下的图片文件（不直接回显图像）。要随后放大、标注或细看某几页/某几帧时用这个，再用 read_image 读产出的文件。",
            "parameters": {"type": "object", "properties": {
                "file_path": {"type": "string", "description": "workspace 内相对路径（源）"},
                "pages": {"type": "string", "description": "页码范围，如 \"1,4,7\""},
                "times": {"type": "array", "items": {"type": "number"}, "description": "视频时间点（秒）"},
            }, "required": ["file_path"]},
        }}));
    }
    // PDF 文字定位：**常挂载**（编译进二进制，不 spawn、不探测外部依赖），
    // 与 Qwen 那组按能力挂载的相反 —— 它唯一的外部条件是「文件是 PDF」。
    tools.push(serde_json::json!({"type": "function", "function": {
        "name": "pdf_ground_text",
        "description": "在 **PDF** 里按词/短语定位，返回页码 + 0-1000 归一化坐标框（y 已翻成图像坐标系，可直接换算像素裁剪）。想「指出这句话在第几页哪个位置」「把含某个词的区域裁出来」先用它，别自己猜坐标。**只对有文字层的 PDF 有效**：扫描件/文字转轮廓的 PDF 会明确说没有文字层，那种情况改用 qwen_visualize 看图。",
        "parameters": {"type": "object", "properties": {
            "path": {"type": "string", "description": "workspace 内的 .pdf 相对路径"},
            "query": {"type": "string", "description": "要找的词或短语（大小写不敏感，跨 Tj 拆字也能匹配）"},
            "max_pages": {"type": "integer", "description": "扫描页数上限（默认 40）"},
        }, "required": ["path", "query"]},
    }}));
    // 侧边栏导航：模型**提议**界面打开什么，Rust 侧只成文不执行。
    tools.push(serde_json::json!({"type": "function", "function": {
        "name": "sidebar_open",
        "description": "让界面在侧边栏打开文件/页签（只是提议，界面可拒；workspace 外的路径会被网关拒）",
        "parameters": {"type": "object", "properties": {
            "topic": {"type": "string", "enum": ["files", "changes", "tasks", "chat"], "description": "要打开的页签"},
            "path": {"type": "string", "description": "workspace 内相对路径（topic=files 时必填）"},
        }, "required": ["topic"]},
    }}));
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
        side_effects: Vec::new(),
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

/// `NEOBOT_VISION` 的显式裁决（`1`=开 / `0`=关）；没设或不是这两个值 → `None`。
///
/// 存在的意义是给**认不出来**的模型一个出口：本机自训的 VL 模型名字千奇百怪，
/// 与其让启发式猜，不如让运维说一句实话。
fn vision_enabled_by_env() -> Option<bool> {
    match std::env::var("NEOBOT_VISION").ok()?.trim() {
        "1" => Some(true),
        "0" => Some(false),
        _ => None,
    }
}

/// 模型名 → 是否**大概率**支持视觉输入。
///
/// **默认拒绝**：认不出的名字一律当「不支持」。因为猜错的代价不对称 ——
/// - 误判「支持」：请求带上 `image_url` 部件，非视觉端点要么 400（浪费一轮、
///   错误信息晦涩），要么更糟：某些网关**静默丢弃**部件，模型于是开始描述一张
///   它从未看见的图。这是编造，比明说看不见坏得多。
/// - 误判「不支持」：用户只多读一句「这个引擎看不到图」，配上 `NEOBOT_VISION=1`
///   即可。看得见的错比看不见的错便宜。
///
/// 只在 `provider/model` 的**裸模型段**上匹配，避免 `openai/gpt-4o` 里的
/// provider 前缀误伤（或误救）判定。
pub fn model_likely_vision(model: &str) -> bool {
    let bare = model.rsplit('/').next().unwrap_or(model).to_ascii_lowercase();
    const NEEDLES: &[&str] = &[
        // 通用视觉后缀 / 家族名（本地 Ollama、vLLM、自训模型都吃这一套）。
        //
        // `vl` 取**裸串**是刻意的：Ollama 把它当 tag 拼名字（`qwen2.5vl`、
        // `minicpm-v` 都不带分隔符），按 `-vl` 匹配会漏掉最常见的那批。误判为
        // 能看的代价是一次诚实的 400 + 一轮浪费；漏判的代价是功能整个用不了。
        "vl", "vision", "llava", "moondream", "pixtral", "internvl", "idefics", "omni",
        "minicpm",
        // 已知多模态家族。
        "gpt-4o", "gpt-4.1", "gpt-4.5", "gpt-4-turbo", "gpt-4-vision", "gpt-5",
        "claude-3", "claude-4", "claude-sonnet", "claude-opus", "claude-haiku", "claude",
        "gemini", "llama-4", "llama4", "gemma-3", "phi-3.5-vision", "phi-4-multimodal",
    ];
    NEEDLES.iter().any(|needle| bare.contains(needle))
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

    fn vision_capable(&self) -> bool {
        vision_enabled_by_env().unwrap_or_else(|| model_likely_vision(&self.config.model))
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
                image: None,
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
            apply_sse_chunk(&mut content, &mut partials, on_delta, &chunk);
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

/// SSE 单 chunk 累积（契约事件形状）：
/// - `choices[0].delta.content` 字符串增量（回调 + 拼接）；
/// - `choices[0].delta.tool_calls` 数组按 `index` 累积
///  （`id` / `function.name` / `function.arguments` 分片拼接，随 `[DONE]` 与
///   `EngineTurn.tool_calls` 合流由调用方完成）；
/// - 兼容 `choices[0].message.tool_calls`（`finish_reason=tool_calls` 独立事件
///   把完整 tool_calls 放在 message 而非 delta 时）；
/// - `arguments` 为字符串分片直接拼接，非字符串（对象/数字）则序列化后拼接，
///   空/非法 chunk 静默跳过；非流式行为不受影响。
fn apply_sse_chunk(
    content: &mut String,
    partials: &mut Vec<StreamToolCall>,
    on_delta: &mut dyn FnMut(&str),
    chunk: &serde_json::Value,
) {
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
        accumulate_tool_calls(partials, calls, None);
    }
    if let Some(calls) = chunk
        .pointer("/choices/0/message/tool_calls")
        .and_then(|value| value.as_array())
    {
        accumulate_tool_calls(partials, calls, None);
    }
}

/// tool_calls 数组累积（`index` 缺省按到达序；`id/name` 非空覆盖，
/// `arguments` 分片拼接；调用方保证入参为数组元素切片）。
fn accumulate_tool_calls(
    partials: &mut Vec<StreamToolCall>,
    calls: &[serde_json::Value],
    base_index: Option<usize>,
) {
    for (offset, call) in calls.iter().enumerate() {
        let index = call
            .get("index")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .or(base_index.map(|b| b + offset))
            .unwrap_or(offset);
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
        if let Some(args_value) = call.pointer("/function/arguments") {
            if let Some(fragment) = args_value.as_str() {
                slot.arguments.push_str(fragment);
            } else if !args_value.is_null() {
                slot.arguments.push_str(&args_value.to_string());
            }
        }
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
    /// 返回 `(base_url, replay_flag)`。`replay_flag` 置位后，下一次请求会把
    /// 应答游标拨回序列开头（供整轮重试时重放同一脚本）。
    fn fake_server_seq(replies: Vec<String>) -> (String, std::sync::Arc<std::sync::atomic::AtomicBool>) {
        use std::io::{Read as _, Write as _};
        use std::sync::{Arc, Mutex};
        // 队列打空后循环重放：调用方失败重试时永远能拿到完整序列，
        // 否则一次瞬时抖动烧掉队列后后续全是 `{}`，并行必现 flake。
        let queue = Arc::new(Mutex::new((replies, 0usize)));
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr").to_string();
        // 「重放」开关：调用方在**重试整轮之前**置位，服务端据此把游标拨回 0。
        //
        // 没有它的时候，一次失败的重试会让下一轮从序列中段开始（拿到「没有
        // tool_calls 的那条」），于是这轮直接 Done、断言里的 bash 审计永远
        // 找不到 —— 失败点离真正的原因十万八千里。实测并行下 8 次跑挂 3 次。
        let replay = Arc::new(std::sync::atomic::AtomicBool::new(true));
        // 线程拿走一份；调用方留一份用来在重试前置位。
        let replay_flag = Arc::clone(&replay);
        std::thread::spawn(move || {
            for _ in 0..64 {
                if replay_flag.swap(false, std::sync::atomic::Ordering::SeqCst) {
                    if let Ok(mut q) = queue.lock() {
                        q.1 = 0;
                    }
                }
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
                    .and_then(|mut q| {
                        if q.0.is_empty() {
                            None
                        } else {
                            let i = q.1 % q.0.len();
                            q.1 += 1;
                            Some(q.0[i].clone())
                        }
                    })
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
        (format!("http://{addr}/v1"), replay)
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
        let (base, replay) = fake_server_seq(vec![first, second]);
        let engine = HttpEngine::new(test_config(&base), String::new()).expect("engine");
        let dir = crate::nt_testutil::temp_dir("toolloop-test");
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
            extra_deny: Vec::new(),
            write_budget: crate::nt_config::default_write_budget(),
        };
        config.validate().expect("validate");
        let store = crate::nt_store::NeobotStore::open(":memory:").expect("open");
        let mut status = None;
        for _ in 0..40 {
            // 每次尝试前让服务端重放脚本，否则重试会落在序列中段
            // （拿到「无 tool_calls」那条），这轮直接 Done，下面的 bash
            // 审计断言就永远对不上 —— 报错还指错了地方。
            replay.store(true, std::sync::atomic::Ordering::SeqCst);
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

    fn tool_named<'a>(tools: &'a [serde_json::Value], name: &str) -> Option<&'a serde_json::Value> {
        tools
            .iter()
            .find(|tool| tool.pointer("/function/name").and_then(|v| v.as_str()) == Some(name))
    }

    #[test]
    fn qwen_mm_mounting_is_honest_both_ways() {
        // 挂载判据与 schema 挂载**同一函数**：探测到服务器 ⟺ 4 个工具在列。
        // 无论本机装没装，这个双向蕴含都成立（装了：两边真；没装：两边假），
        // 所以测试是确定性的 —— 不会出现在 CI 没装、本机装了就红的那种 flake。
        use super::{qwen_mm_mounted, tool_schemas};
        let mounted = qwen_mm_mounted();
        let schemas = tool_schemas(false, false);
        for name in ["qwen_media_info", "qwen_read_video", "qwen_visualize", "qwen_save_view"] {
            assert_eq!(
                tool_named(&schemas, name).is_some(),
                mounted,
                "schema presence of '{name}' must track mount predicate"
            );
        }
        if mounted {
            // 挂载时：media_info 只读 1 个 required（path），别的不许偷偷加必填。
            let mi = tool_named(&schemas, "qwen_media_info").expect("mounted");
            let required = mi
                .pointer("/function/parameters/required")
                .and_then(serde_json::Value::as_array)
                .map(Vec::len);
            assert_eq!(required, Some(1));
        }
    }

    #[test]
    fn pdf_ground_text_is_always_advertised_and_never_claims_ocr() {
        use super::tool_schemas;
        // 与 Qwen 那组**相反**：它编译进二进制，没有「装没装」这回事 ⇒ 恒挂载，
        // 两种视能下都必须在列。哪天若改成依赖外部二进制，此断言必须改成
        // `qwen_mm_mounting_is_honest_both_ways` 那样的双向蕴含。
        for (computer, vision) in [(false, false), (false, true), (true, true)] {
            let schemas = tool_schemas(computer, vision);
            let schema = tool_named(&schemas, "pdf_ground_text")
                .unwrap_or_else(|| panic!("must be advertised (computer={computer})"));
            // 两个必填：不给 query 就不知道要定位什么词。
            let required = schema
                .pointer("/function/parameters/required")
                .and_then(serde_json::Value::as_array)
                .expect("required list");
            assert_eq!(required.len(), 2, "required={required:?}");
            // 描述里那两句诚实边界不许被后人「精简」掉：它只对有文字层的 PDF
            // 有效，且**不是**通用 OCR。描述是模型唯一的能力说明书。
            let desc = schema
                .pointer("/function/description")
                .and_then(|v| v.as_str())
                .expect("description");
            assert!(desc.contains("扫描件"), "诚实边界被删：{desc}");
            assert!(!desc.contains("OCR"), "别把它说成通用 OCR：{desc}");
        }
    }

    #[test]
    fn read_image_is_advertised_only_when_the_model_can_see() {
        use super::tool_schemas;
        let blind = tool_schemas(false, false);
        // 看不见就别摆上桌：挂了却在执行期才失败，模型会白费一轮才发现没眼睛。
        assert!(tool_named(&blind, "read_image").is_none());
        let seeing = tool_schemas(false, true);
        let schema = tool_named(&seeing, "read_image").expect("read_image schema");
        assert_eq!(
            schema
                .pointer("/function/parameters/required/0")
                .and_then(|v| v.as_str()),
            Some("path")
        );
        assert_eq!(
            schema
                .pointer("/function/parameters/properties/path/type")
                .and_then(|v| v.as_str()),
            Some("string")
        );
        // computer_act 的 opt-in 语义没被这次改动带歪。
        assert!(tool_named(&tool_schemas(true, true), "computer_act").is_some());
        assert!(tool_named(&blind, "read_file").is_some());
    }

    #[test]
    fn tool_row_becomes_a_multimodal_content_array() {
        use super::transcript_message;
        use crate::nt_types::{ImagePart, TranscriptItem, TranscriptRole};
        // 无图：content 保持字符串（老端点对 tool 行的数组 content 挑刺，
        // 没必要给无关轮次惹这个麻烦）。
        let plain = transcript_message(&TranscriptItem {
            role: TranscriptRole::Tool,
            content: "hello".to_owned(),
            tool_calls: Vec::new(),
            tool_call_id: Some("c1".to_owned()),
            image: None,
        });
        assert_eq!(plain.pointer("/content").and_then(|v| v.as_str()), Some("hello"));
        // 有图：content 升级成数组，图像是真 `image_url` 部件。
        let with_image = transcript_message(&TranscriptItem {
            role: TranscriptRole::Tool,
            content: "attached 1 image".to_owned(),
            tool_calls: Vec::new(),
            tool_call_id: Some("c2".to_owned()),
            image: Some(ImagePart {
                media_type: "image/png".to_owned(),
                base64: "iVBORw0KGgo=".to_owned(),
            }),
        });
        assert_eq!(with_image.pointer("/role").and_then(|v| v.as_str()), Some("tool"));
        assert_eq!(
            with_image.pointer("/tool_call_id").and_then(|v| v.as_str()),
            Some("c2")
        );
        let parts = with_image
            .get("content")
            .and_then(|v| v.as_array())
            .expect("content must be an array when an image rides along");
        assert_eq!(parts.len(), 2);
        assert_eq!(parts.first().and_then(|p| p.pointer("/type")).and_then(|v| v.as_str()), Some("text"));
        assert_eq!(
            parts.last().and_then(|p| p.pointer("/type")).and_then(|v| v.as_str()),
            Some("image_url")
        );
        assert_eq!(
            parts.last().and_then(|p| p.pointer("/image_url/url")).and_then(|v| v.as_str()),
            Some("data:image/png;base64,iVBORw0KGgo=")
        );
    }

    #[test]
    fn vision_heuristic_is_fail_closed_and_narrow() {
        use super::model_likely_vision;
        for yes in [
            "qwen2.5vl", "llama3.2-vision", "llava:13b", "minicpm-v", "gpt-4o",
            "openai/gpt-4o-mini", "claude-sonnet-4-5", "gemini-2.5-pro", "gemma-3-12b",
            "moondream", "internvl2", "pixtral-12b",
        ] {
            assert!(model_likely_vision(yes), "{yes} should look multimodal");
        }
        // 认不出即当「不支持」：误判为支持的代价是模型开始编造画面。
        for no in ["llama3.2", "qwen2.5", "mistral-nemo", "deepseek-r1", "", "gpt-3.5-turbo"] {
            assert!(!model_likely_vision(no), "{no} must not be assumed multimodal");
        }
    }

    /// `NEOBOT_VISION` 是**进程级**变量，两个用它/依赖它缺席的用例并行跑就会
    /// 互相踩（一个设成 `0`，另一个正好在断言「启发式说是能看的」）。
    /// 这把锁把「动这个变量」和「读这个变量」串成一条线。
    static VISION_ENV: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn neoBOT_vision_env_overrides_the_heuristic_both_ways() {
        use crate::nt_engine::EngineAdapter;
        let _guard = VISION_ENV.lock();
        let base = test_config("http://127.0.0.1:1/v1");
        // 认不出的模型 + NEOBOT_VISION=1 → 开（给自训 VL 模型一个出口）。
        std::env::set_var("NEOBOT_VISION", "1");
        let forced_on = HttpEngine::new(base.clone(), String::new()).expect("engine");
        assert!(forced_on.vision_capable());
        // 明说不许：即便模型名看着能看。
        std::env::set_var("NEOBOT_VISION", "0");
        let forced_off = HttpEngine::new(
            HttpEngineConfig {
                model: "gpt-4o".to_owned(),
                ..base.clone()
            },
            String::new(),
        )
        .expect("engine");
        assert!(!forced_off.vision_capable());
        // 乱值不算裁决（回落到启发式，不是回落到 true）。
        std::env::set_var("NEOBOT_VISION", "maybe");
        let ignored = HttpEngine::new(base.clone(), String::new()).expect("engine");
        assert!(!ignored.vision_capable());
        std::env::remove_var("NEOBOT_VISION");
        // 撤掉环境变量后回到启发式。
        let heuristic = HttpEngine::new(base, String::new()).expect("engine");
        assert!(!heuristic.vision_capable());
    }

    #[test]
    fn image_part_reaches_the_wire_as_a_real_content_part() {
        // 端到端那一刀：把带图的 tool 行推进 `chat_body`，看它是不是真的
        // 变成了 `image_url` 部件。前面那些断言都是在验零件，这是验总装。
        use super::tool_schemas;
        use crate::nt_engine::EngineAdapter;
        use crate::nt_types::{ImagePart, TranscriptItem, TranscriptRole};
        {
            let _guard = VISION_ENV.lock();
            // 自己先把变量清干净：残留的 `0` 会让本用例凭空失败。
            std::env::remove_var("NEOBOT_VISION");
        }
        let base = fake_server(|request_line, _| {
            assert!(request_line.contains("POST /v1/chat/completions"));
            serde_json::json!({"choices": [{"message": {"content": "ok"}}]}).to_string()
        });
        let engine = HttpEngine::new(
            HttpEngineConfig {
                base_url: base,
                model: "qwen2.5vl".to_owned(),
                timeout_secs: 5,
            },
            String::new(),
        )
        .expect("engine");
        assert!(engine.vision_capable());
        let history = vec![TranscriptItem {
            role: TranscriptRole::Tool,
            content: "attached 1 image as a multimodal part: shot.png".to_owned(),
            tool_calls: Vec::new(),
            tool_call_id: Some("c1".to_owned()),
            image: Some(ImagePart {
                media_type: "image/png".to_owned(),
                base64: "iVBORw0KGgoAAAANSUhEUg".to_owned(),
            }),
        }];
        let mut ok = false;
        for _ in 0..40 {
            if engine.run_turn_with_history("describe it", &history).is_ok() {
                ok = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        assert!(ok);
        // 复用同一个 handler 抓请求体做不到（FnOnce 已跑）——重建一次，
        // 这次把 body 存出来给断言看。
        let captured: std::sync::Arc<std::sync::Mutex<Option<String>>> =
            std::sync::Arc::new(std::sync::Mutex::new(None));
        let sink = std::sync::Arc::clone(&captured);
        let base2 = fake_server(move |_request_line, body| {
            if let Ok(mut slot) = sink.lock() {
                *slot = Some(body.to_owned());
            }
            serde_json::json!({"choices": [{"message": {"content": "ok"}}]}).to_string()
        });
        let engine2 = HttpEngine::new(
            HttpEngineConfig {
                base_url: base2,
                model: "qwen2.5vl".to_owned(),
                timeout_secs: 5,
            },
            String::new(),
        )
        .expect("engine");
        let mut sent = false;
        for _ in 0..40 {
            if engine2.run_turn_with_history("describe it", &history).is_ok() {
                sent = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        assert!(sent);
        let body = captured
            .lock()
            .ok()
            .and_then(|slot| slot.clone())
            .expect("request body captured");
        let parsed: serde_json::Value = serde_json::from_str(&body).expect("valid json request");
        assert_eq!(
            parsed.pointer("/messages/1/content/1/type").and_then(|v| v.as_str()),
            Some("image_url"),
            "the image must be a real content part: {body}"
        );
        assert_eq!(
            parsed
                .pointer("/messages/1/content/1/image_url/url")
                .and_then(|v| v.as_str()),
            Some("data:image/png;base64,iVBORw0KGgoAAAANSUhEUg")
        );
        // schema 里也得有 read_image（否则模型压根不会去调）。
        let tools: Vec<serde_json::Value> =
            serde_json::from_value(parsed.get("tools").cloned().unwrap_or(serde_json::json!([])))
                .expect("tools array");
        assert!(tool_named(&tools, "read_image").is_some());
        assert!(tool_named(&tool_schemas(false, false), "read_image").is_none());
    }

    #[test]
    fn sse_chunk_accumulates_contract_tool_calls_delta() {
        use super::{StreamToolCall, apply_sse_chunk};
        let mut content = String::new();
        let mut partials: Vec<StreamToolCall> = Vec::new();
        let mut deltas = String::new();
        let mut emit = |s: &str| deltas.push_str(s);
        // 契约形状：delta.content + delta.tool_calls 分片（index 对齐）。
        let c1 = serde_json::json!({
            "choices": [{"delta": {"content": "hi"}, "index": 0}],
            "object": "chat.completion.chunk",
        });
        apply_sse_chunk(&mut content, &mut partials, &mut emit, &c1);
        let c2 = serde_json::json!({
            "choices": [{"delta": {"tool_calls": [
                {"index": 0, "id": "c1", "function": {"name": "bash", "arguments": "{\"comma"}}
            ]}, "index": 0}],
            "object": "chat.completion.chunk",
        });
        apply_sse_chunk(&mut content, &mut partials, &mut emit, &c2);
        let c3 = serde_json::json!({
            "choices": [{"delta": {"tool_calls": [
                {"index": 0, "function": {"arguments": "nd\":\"echo hi\"}"}}
            ]}, "index": 0}],
            "object": "chat.completion.chunk",
        });
        apply_sse_chunk(&mut content, &mut partials, &mut emit, &c3);
        // finish_reason=tool_calls 独立事件：message.tool_calls 合流。
        let c4 = serde_json::json!({
            "choices": [{"message": {"tool_calls": [
                {"index": 1, "id": "c2", "function": {"name": "read_file", "arguments": "{\"path\":\"a\"}"}}
            ]}, "finish_reason": "tool_calls"}],
        });
        apply_sse_chunk(&mut content, &mut partials, &mut emit, &c4);
        assert_eq!(content, "hi");
        assert_eq!(deltas, "hi");
        assert_eq!(partials.len(), 2);
        assert_eq!(partials[0].id, "c1");
        assert_eq!(partials[0].name, "bash");
        assert_eq!(partials[0].arguments, "{\"command\":\"echo hi\"}");
        assert_eq!(partials[1].name, "read_file");
    }
}
