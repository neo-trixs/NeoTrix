//! `nt_core` — 晶体核心配对（灵魂嵌入机制）。
//!
//! 语义：neobot 默认是纯本地 App（echo / 第三方 LLM API）；
//! `pair` 成功后灵魂嵌入——默认跑轮走晶体核心，`unpair` 即摘除回本地。
//! 配对是运行时健康关系，不是代码依赖：neobot 不依 neotrix-core 编译，
//! 只经 OpenAI 兼容 `/v1` 配对（端点可以是 neotrix 核心服务，也可以是
//! 任何兼容端点；fail-closed：探活失败拒绝配对）。
//!
//! 约定：配对端点同时镜像为 `neotrix` provider 记录（池子可见）；
//! 模型名缺省 `neotrix-crystal`。

use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::nt_error::NtBotError;
use crate::nt_provider::Provider;
use crate::nt_store::{CorePair, NeobotStore};

/// 晶体默认模型名（核心服务的灵魂款）。
pub const CORE_MODEL: &str = "neotrix-crystal";
/// 配对探活超时秒（fail-closed：慢端点不等）。
pub const PAIR_PROBE_SECS: u64 = 5;
/// 跑轮路由探活超时秒（短，避免拖慢首字）。
pub const ROUTE_PROBE_SECS: u64 = 3;
/// 发现超时秒（OpenRouter 全量表 460 条，10s 宽限）。
pub const DISCOVER_SECS: u64 = 10;
/// 池子端点名（配对镜像）。
pub const CORE_PROVIDER: &str = "neotrix";
/// token 环境变量名缺省（永不存 token 值，只存名 + 现读环境）。
pub const DEFAULT_TOKEN_ENV: &str = "CRYSTAL_TOKEN";
/// agent 默认步数（契约：缺省 8，上限 16）。
pub const AGENT_DEFAULT_STEPS: i64 = 8;
pub const AGENT_MAX_STEPS: i64 = 32;

/// token_env 归一化（纯函数）：空/空白回缺省名，否则 trim 原样透传。
pub fn normalize_token_env(raw: &str) -> String {
    let t = raw.trim();
    if t.is_empty() {
        DEFAULT_TOKEN_ENV.to_owned()
    } else {
        t.to_owned()
    }
}

/// agent 单步痕迹行（契约 DTO，一字不差）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TraceRow {
    pub kind: String,
    pub detail: String,
}

/// agent 运行结果（契约 DTO，一字不差；全部 Serialize）。
/// 新增标签三件套（`mode`/`tools`/`usage`）均为 `#[serde(default)]`：
/// 老服务端只回四件套仍可解析，前端按缺省降级（透传/空工具/未计量）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentRunResult {
    pub status: String,
    pub output: String,
    pub trace: Vec<TraceRow>,
    pub model_used: String,
    /// 路由三态（`direct`/`passthrough`/`fallback`；`agent_run` 恒透传）。
    #[serde(default = "default_agent_mode")]
    pub mode: String,
    /// 本轮工具名（去重保序；服务端缺省时由 trace 派生）。
    #[serde(default)]
    pub tools: Vec<String>,
    /// 用量（服务端透传；缺省未计量，前端显 `未计量` 不猜价）。
    #[serde(default)]
    pub usage: Option<crate::nt_types::TokenUsage>,
}

/// `agent_run` 缺省路由（晶体透传；与 `resolve_run_engine` 核心路径同义）。
fn default_agent_mode() -> String {
    "passthrough".to_owned()
}

/// capabilities 摘要（`GET /v1/capabilities` 解析结果；取失败降级为空串/0）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilitiesInfo {
    pub crystal_version: String,
    pub tool_count: usize,
    pub model: String,
}

impl Default for CapabilitiesInfo {
    fn default() -> Self {
        Self {
            crystal_version: String::new(),
            tool_count: 0,
            model: String::new(),
        }
    }
}

/// 契约 `GET /v1/capabilities` 响应纯解析：
/// `{model, crystal_version, tools: [{name,..}], ..}` → 摘要；
/// 缺字段/形状不对即降级（空串/0，不报错挡路）。
pub fn parse_capabilities(value: &serde_json::Value) -> CapabilitiesInfo {
    let model = value
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_owned();
    let crystal_version = value
        .get("crystal_version")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_owned();
    let tool_count = value
        .get("tools")
        .and_then(|v| v.as_array())
        .map(|arr| arr.len())
        .unwrap_or(0);
    CapabilitiesInfo { crystal_version, tool_count, model }
}

/// 契约 `POST /v1/agents/run` 响应纯解析：
/// `{status, output, trace: [{kind, detail}], model_used}`。
/// 标签回填（只增不改）：`tools` 为空即由 trace kinds 派生，
/// `mode` 为空即透传；老服务端四件套照样过。
pub fn parse_agent_run_result(value: serde_json::Value) -> Result<AgentRunResult, NtBotError> {
    let mut result: AgentRunResult =
        serde_json::from_value(value).map_err(|err| NtBotError::Codec(format!("bad agent result: {err}")))?;
    if result.mode.trim().is_empty() {
        result.mode = default_agent_mode();
    }
    if result.tools.is_empty() && !result.trace.is_empty() {
        let kinds: Vec<String> = result.trace.iter().map(|row| row.kind.clone()).collect();
        result.tools = crate::nt_reply_tag::tools_from_trace(&kinds);
    }
    Ok(result)
}

/// 契约 `POST /v1/admin/reload` 响应纯摘要：
/// `{ok, pool, degraded}` → `reload ok: pool=N degraded=M [...]`。
pub fn summarize_reload(value: &serde_json::Value) -> String {
    let pool = value
        .get("pool")
        .and_then(|v| v.as_i64())
        .or_else(|| {
            value
                .get("pool")
                .and_then(|p| p.get("count"))
                .and_then(|v| v.as_i64())
        })
        .unwrap_or(-1);
    let degraded: Vec<String> = value
        .get("degraded")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|e| {
                    e.as_str().map(str::to_owned).or_else(|| {
                        e.get("name").and_then(|n| n.as_str()).map(str::to_owned)
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    if pool >= 0 {
        if degraded.is_empty() {
            format!("reload ok: pool={pool} degraded=0")
        } else {
            format!("reload ok: pool={pool} degraded={} [{}]", degraded.len(), degraded.join(","))
        }
    } else {
        format!("reload ok: {value}")
    }
}

fn base_trimmed(base_url: &str) -> String {
    base_url.trim().trim_end_matches('/').to_owned()
}

fn capabilities_url(base_url: &str) -> String {
    format!("{}/capabilities", base_trimmed(base_url))
}

fn reload_url(base_url: &str) -> String {
    format!("{}/admin/reload", base_trimmed(base_url))
}

fn agents_run_url(base_url: &str) -> String {
    format!("{}/agents/run", base_trimmed(base_url))
}

/// token 现读（绝不存值）：空名/空值回空串（legacy 开放直连，不带头）。
pub fn bearer_for(token_env: &str) -> String {
    let name = token_env.trim();
    if name.is_empty() {
        return String::new();
    }
    std::env::var(name).unwrap_or_default().trim().to_owned()
}

fn get_with_auth(url: &str, token: &str, timeout_secs: u64) -> Result<serde_json::Value, NtBotError> {
    let timeout = std::time::Duration::from_secs(timeout_secs.clamp(1, 15));
    let mut request = ureq::get(url).timeout(timeout);
    if !token.trim().is_empty() {
        request = request.set("Authorization", &format!("Bearer {}", token.trim()));
    }
    request
        .call()
        .map_err(|err| NtBotError::Engine {
            engine: "crystal".to_owned(),
            reason: format!("{err}"),
        })?
        .into_json::<serde_json::Value>()
        .map_err(|err| NtBotError::Engine {
            engine: "crystal".to_owned(),
            reason: format!("bad json: {err}"),
        })
}

fn post_with_auth(
    url: &str,
    token: &str,
    body: &serde_json::Value,
    timeout_secs: u64,
) -> Result<serde_json::Value, NtBotError> {
    let timeout = std::time::Duration::from_secs(timeout_secs.clamp(1, 120));
    let mut request = ureq::post(url).timeout(timeout);
    if !token.trim().is_empty() {
        request = request.set("Authorization", &format!("Bearer {}", token.trim()));
    }
    request
        .send_json(body)
        .map_err(|err| NtBotError::Engine {
            engine: "crystal".to_owned(),
            reason: format!("{err}"),
        })?
        .into_json::<serde_json::Value>()
        .map_err(|err| NtBotError::Engine {
            engine: "crystal".to_owned(),
            reason: format!("bad json: {err}"),
        })
}

/// 拉 capabilities 原始 JSON（前端特性展示用；失败由调用方降级）。
pub fn fetch_capabilities_raw(base_url: &str, token_env: &str) -> Result<serde_json::Value, NtBotError> {
    let token = bearer_for(token_env);
    get_with_auth(&capabilities_url(base_url), &token, ROUTE_PROBE_SECS)
}

/// 拉 capabilities（需配对行 token_env 现读；失败由调用方降级，不在此吞错）。
pub fn fetch_capabilities(base_url: &str, token_env: &str) -> Result<CapabilitiesInfo, NtBotError> {
    let token = bearer_for(token_env);
    let value = get_with_auth(&capabilities_url(base_url), &token, ROUTE_PROBE_SECS)?;
    Ok(parse_capabilities(&value))
}

/// capabilities 降级版（取失败则空串/0，不挡 status；纯网络失败路径）。
pub fn capabilities_or_default(base_url: &str, token_env: &str) -> CapabilitiesInfo {
    fetch_capabilities(base_url, token_env).unwrap_or_default()
}

/// 热重载（`POST /v1/admin/reload`，token_env 现读；返回服务端 counts 摘要）。
pub fn core_reload(store: &NeobotStore) -> Result<String, NtBotError> {
    let pair = store
        .get_core_pair()?
        .ok_or_else(|| NtBotError::Invalid("core unpaired (pair first)".to_owned()))?;
    if pair.via == "cli" {
        return Err(NtBotError::Invalid("core reload needs http pair (cli has no server)".to_owned()));
    }
    let token = bearer_for(&pair.token_env);
    let value = post_with_auth(&reload_url(&pair.base_url), &token, &serde_json::json!({}), 15)?;
    Ok(summarize_reload(&value))
}

/// 服务端 agent 跑一轮（`POST /v1/agents/run`，token_env 现读）。
/// `goal` 非空；`context` 为空即不发；`max_steps` 缺省 8（上限 32，与服务端同钳）。
pub fn agent_run(
    store: &NeobotStore,
    goal: &str,
    context: Option<&str>,
) -> Result<AgentRunResult, NtBotError> {
    agent_run_with_steps(store, goal, context, AGENT_DEFAULT_STEPS)
}

/// 步数可配版（长任务调大；钳 1-32，超出回落默认）。
pub fn agent_run_with_steps(
    store: &NeobotStore,
    goal: &str,
    context: Option<&str>,
    max_steps: i64,
) -> Result<AgentRunResult, NtBotError> {
    let trimmed = goal.trim();
    if trimmed.is_empty() {
        return Err(NtBotError::Invalid("agent goal is empty".to_owned()));
    }
    let steps = if (1..=AGENT_MAX_STEPS).contains(&max_steps) {
        max_steps
    } else {
        AGENT_DEFAULT_STEPS
    };
    let pair = store
        .get_core_pair()?
        .ok_or_else(|| NtBotError::Invalid("core unpaired (pair first)".to_owned()))?;
    if pair.via == "cli" {
        return Err(NtBotError::Invalid("agent run needs http pair (cli has no server)".to_owned()));
    }
    let token = bearer_for(&pair.token_env);
    let mut body = serde_json::json!({
        "goal": trimmed,
        "max_steps": steps,
    });
    if let Some(ctx) = context {
        let c = ctx.trim();
        if !c.is_empty() {
            if let Some(map) = body.as_object_mut() {
                map.insert("context".to_owned(), serde_json::Value::String(c.to_owned()));
            }
        }
    }
    let value = post_with_auth(&agents_run_url(&pair.base_url), &token, &body, 120)?;
    parse_agent_run_result(value)
}

/// 免费条目（neotrix 免费目录机制移植：OpenRouter 在线过滤 + Groq 表 + Zen CLI；
/// 免费≠免 key：HTTP 类仍需对应 key_env，CLI 类走本机 opencode 登录）。
#[derive(Debug, Clone)]
pub struct FreeEntry {
    pub provider: &'static str,
    pub model_id: String,
    pub display: String,
    pub base_url: &'static str,
    pub key_env: &'static str,
    pub via: FreeVia,
}

/// 免费通道：HTTP（OpenAI 兼容直连，需 key）或本机 opencode CLI（免 key，需登录）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FreeVia {
    Http,
    OpencodeCli,
}

/// Groq 免费表（快照；pair 探活会剔除失效项）。
fn groq_table() -> Vec<FreeEntry> {
    [
        ("llama-3.3-70b-versatile", "Llama 3.3 70B (Groq)"),
        ("gemma2-9b-it", "Gemma 2 9B IT (Groq)"),
        ("llama-4-scout-17b-16e-instruct", "Llama 4 Scout 17B (Groq)"),
        ("deepseek-r1-distill-llama-70b", "DeepSeek R1 Distill 70B (Groq)"),
        ("mixtral-8x7b-32768", "Mixtral 8x7B 32K (Groq)"),
    ]
    .into_iter()
    .map(|(model_id, display)| FreeEntry {
        provider: "groq",
        model_id: model_id.to_owned(),
        display: display.to_owned(),
        base_url: "https://api.groq.com/openai/v1",
        key_env: "GROQ_API_KEY",
        via: FreeVia::Http,
    })
    .collect()
}

/// 在线发现（OpenRouter 全量表过滤 pricing.prompt=="0" + Groq 表 + 本机 Zen CLI）。
pub fn discover_free() -> Vec<FreeEntry> {
    let mut out = groq_table();
    out.extend(discover_zen_free());
    let found = discover_openrouter_free();
    // 在线结果优先（新），去重后接 Groq/Zen 表。
    let mut seen: std::collections::HashSet<String> =
        found.iter().map(|e| e.model_id.clone()).collect();
    out.retain(|e| seen.insert(e.model_id.clone()));
    out.splice(0..0, found);
    if out.len() > 40 {
        out.truncate(40);
    }
    out
}

/// 本机 Zen 免费表（`opencode models` 输出过滤 `-free`/`big-pickle`；
/// 未安装/未登录回空；CLI 通道免 key）。
fn discover_zen_free() -> Vec<FreeEntry> {
    crate::nt_engine::OpencodeEngine::zen_models()
        .into_iter()
        .filter(|(_, model)| model.ends_with("-free") || model == "big-pickle")
        .map(|(_, model)| FreeEntry {
            provider: "opencode-zen",
            model_id: format!("opencode/{model}"),
            display: format!("{model} (OpenCode, no key)"),
            base_url: "opencode-cli",
            key_env: "",
            via: FreeVia::OpencodeCli,
        })
        .collect()
}

fn discover_openrouter_free() -> Vec<FreeEntry> {
    let body: serde_json::Value = match ureq::get("https://openrouter.ai/api/v1/models")
        .timeout(std::time::Duration::from_secs(DISCOVER_SECS))
        .call()
    {
        Ok(resp) => match resp.into_json() {
            Ok(v) => v,
            Err(_) => return Vec::new(),
        },
        Err(_) => return Vec::new(),
    };
    body.get("data")
        .and_then(|d| d.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    let id = m.get("id")?.as_str()?;
                    let free = m
                        .get("pricing")
                        .and_then(|p| p.get("prompt"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.trim() == "0")
                        .unwrap_or(false);
                    if !free {
                        return None;
                    }
                    Some(FreeEntry {
                        provider: "openrouter",
                        model_id: id.to_owned(),
                        display: m
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or(id)
                            .to_owned(),
                        base_url: "https://openrouter.ai/api/v1",
                        key_env: "OPENROUTER_API_KEY",
                        via: FreeVia::Http,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn probe(
    base_url: &str,
    model: &str,
    key_env: &str,
    timeout_secs: u64,
) -> Result<(usize, i64), NtBotError> {
    let provider = Provider {
        name: CORE_PROVIDER.to_owned(),
        base_url: base_url.to_owned(),
        key_env: key_env.to_owned(),
        model: model.to_owned(),
        enabled: true,
    };
    provider.validate()?;
    let started = Instant::now();
    let models = provider.list_models(timeout_secs)?;
    Ok((models.len(), started.elapsed().as_millis().min(i64::MAX as u128) as i64))
}

/// 配对（探活成功才写行 + 镜像 provider；死端点直接拒绝）。
/// `token_env` 只存变量名（缺省 `CRYSTAL_TOKEN`），probe/provider 全走该名现读环境。
pub fn pair_core(
    store: &NeobotStore,
    base_url: &str,
    model: &str,
    token_env: &str,
) -> Result<(CorePair, usize), NtBotError> {
    let base = base_url.trim().trim_end_matches('/').to_owned();
    let name = {
        let m = model.trim();
        if m.is_empty() {
            CORE_MODEL.to_owned()
        } else {
            m.to_owned()
        }
    };
    let token_name = normalize_token_env(token_env);
    let (count, latency) = probe(&base, &name, &token_name, PAIR_PROBE_SECS).map_err(|e| {
        NtBotError::Invalid(format!("core unreachable, pair refused: {e}"))
    })?;
    let pair = store.set_core_pair(&base, &name, latency, "http", &token_name)?;
    store.upsert_provider(&Provider {
        name: CORE_PROVIDER.to_owned(),
        base_url: base,
        key_env: token_name,
        model: name,
        enabled: true,
    })?;
    Ok((pair, count))
}

/// 配对免费条目。
/// - HTTP 类：key 缺失直接拒绝并指路，不配对半吊子；
/// - CLI 类（Zen）：本机 `opencode models` 即时复核在列才写行（via=cli，
///   base_url 记 `opencode-cli`）；调用方另行把配置引擎切为 Opencode。
pub fn pair_free(store: &NeobotStore, entry: &FreeEntry) -> Result<(CorePair, usize), NtBotError> {
    if entry.via == FreeVia::OpencodeCli {
        let listed = crate::nt_engine::OpencodeEngine::zen_models();
        let ok = listed
            .iter()
            .any(|(p, m)| format!("{p}/{m}") == entry.model_id);
        if !ok {
            return Err(NtBotError::Invalid(format!(
                "zen model '{}' not listed by local `opencode models` (login?)",
                entry.model_id
            )));
        }
        let pair = store.set_core_pair("opencode-cli", &entry.model_id, 0, "cli", DEFAULT_TOKEN_ENV)?;
        return Ok((pair, listed.len()));
    }
    let key = std::env::var(entry.key_env).unwrap_or_default();
    if key.trim().is_empty() {
        return Err(NtBotError::Invalid(format!(
            "free model needs a key: export {}=<your free key> first ({} / {})",
            entry.key_env, entry.provider, entry.display
        )));
    }
    pair_core(store, entry.base_url, &entry.model_id, entry.key_env)
}

/// 摘除（无配对也算成功，幂等）。
pub fn unpair_core(store: &NeobotStore) -> Result<bool, NtBotError> {
    store.clear_core_pair()
}

/// 配对状态（读行 + 活探；行在但探死 = 灵魂离线）。
/// Online 携带版本与工具数（`/capabilities` 取；失败降级空串/0，不挡 status）。
#[derive(Debug)]
pub enum CoreStatus {
    Unpaired,
    Online {
        pair: CorePair,
        models: usize,
        latency_ms: i64,
        crystal_version: String,
        tool_count: usize,
    },
    Offline {
        pair: CorePair,
        reason: String,
    },
}

pub fn core_status(store: &NeobotStore) -> Result<CoreStatus, NtBotError> {
    let Some(pair) = store.get_core_pair()? else {
        return Ok(CoreStatus::Unpaired);
    };
    // CLI 通道：在列即在线（本机登录态即健康；无服务端版本，降级空串/0）。
    if pair.via == "cli" {
        let listed = crate::nt_engine::OpencodeEngine::zen_models();
        let online = listed
            .iter()
            .any(|(p, m)| format!("{p}/{m}") == pair.model);
        if online {
            return Ok(CoreStatus::Online {
                pair,
                models: listed.len(),
                latency_ms: 0,
                crystal_version: String::new(),
                tool_count: 0,
            });
        }
        return Ok(CoreStatus::Offline {
            reason: "zen model not listed by local `opencode models`".to_owned(),
            pair,
        });
    }
    let started = Instant::now();
    let probe_provider = Provider {
        name: CORE_PROVIDER.to_owned(),
        base_url: pair.base_url.clone(),
        key_env: pair.token_env.clone(),
        model: pair.model.clone(),
        enabled: true,
    };
    match probe_provider.list_models(ROUTE_PROBE_SECS) {
        Ok(models) => {
            let latency_ms = started.elapsed().as_millis().min(i64::MAX as u128) as i64;
            let caps = capabilities_or_default(&pair.base_url, &pair.token_env);
            Ok(CoreStatus::Online {
                latency_ms,
                models: models.len(),
                crystal_version: caps.crystal_version,
                tool_count: caps.tool_count,
                pair,
            })
        }
        Err(e) => Ok(CoreStatus::Offline {
            reason: e.to_string(),
            pair,
        }),
    }
}

/// 跑轮引擎（HTTP 配对且活着 → 核心 HTTP 引擎；CLI 配对走配置引擎，此处 None；
/// 否则 None，调用方回落本地）。
/// Provider 的 key_env 取配对行 token_env 名（现读环境，永不存值）。
pub fn core_engine(
    store: &NeobotStore,
) -> Result<Option<crate::nt_http_engine::HttpEngine>, NtBotError> {
    core_engine_with_model(store, None)
}

/// 配对核心引擎（模型可覆盖：对话选中的池子模型名透传给晶体池内解析；
/// 空即配对模型；探活失败回 None，调用方走直连旧链）。
pub fn core_engine_with_model(
    store: &NeobotStore,
    model_override: Option<&str>,
) -> Result<Option<crate::nt_http_engine::HttpEngine>, NtBotError> {
    let Some(pair) = store.get_core_pair()? else {
        return Ok(None);
    };
    if pair.via == "cli" {
        return Ok(None);
    }
    let provider = Provider {
        name: CORE_PROVIDER.to_owned(),
        base_url: pair.base_url,
        key_env: pair.token_env,
        model: pair.model,
        enabled: true,
    };
    match provider.list_models(ROUTE_PROBE_SECS) {
        Ok(_) => Ok(Some(provider.http_engine(model_override)?)),
        Err(_) => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CORE_MODEL, normalize_token_env, pair_core, parse_agent_run_result, parse_capabilities,
        summarize_reload, unpair_core, DEFAULT_TOKEN_ENV,
    };
    use crate::nt_store::NeobotStore;

    #[test]
    fn pair_refuses_dead_endpoint() {
        let store = NeobotStore::open(":memory:").expect("open");
        // 9 号端口必然拒绝：fail-closed，不等超时。
        let err = pair_core(&store, "http://127.0.0.1:9/v1", CORE_MODEL, "").expect_err("must refuse");
        assert!(err.to_string().contains("pair refused"), "{err}");
        assert!(store.get_core_pair().expect("read").is_none());
    }

    #[test]
    fn pair_rejects_bad_url() {
        let store = NeobotStore::open(":memory:").expect("open");
        assert!(pair_core(&store, "ftp://x/v1", "m", "").is_err());
        assert!(!unpair_core(&store).expect("unpair idempotent"));
    }

    #[test]
    fn pair_free_refuses_missing_key() {
        use super::{FreeEntry, FreeVia, pair_free};
        let store = NeobotStore::open(":memory:").expect("open");
        let entry = FreeEntry {
            provider: "openrouter",
            model_id: "x/y:free".to_owned(),
            display: "X".to_owned(),
            base_url: "https://openrouter.ai/api/v1",
            key_env: "NEOBOT_TEST_MISSING_KEY_XYZ",
            via: FreeVia::Http,
        };
        let err = pair_free(&store, &entry).expect_err("must refuse without key");
        assert!(err.to_string().contains("NEOBOT_TEST_MISSING_KEY_XYZ"), "{err}");
    }

    #[test]
    fn groq_table_is_nonempty() {
        assert!(!super::groq_table().is_empty());
    }

    #[test]
    fn token_env_default_and_passthrough() {
        assert_eq!(normalize_token_env(""), DEFAULT_TOKEN_ENV);
        assert_eq!(normalize_token_env("   "), DEFAULT_TOKEN_ENV);
        assert_eq!(normalize_token_env("CRYSTAL_TOKEN"), "CRYSTAL_TOKEN");
        assert_eq!(normalize_token_env("  MY_TOKEN  "), "MY_TOKEN");
        // 内存库级：空名落缺省，具名透传（永不存 token 值，只存名）。
        let store = NeobotStore::open(":memory:").expect("open");
        let pair = store
            .set_core_pair("http://127.0.0.1:3000/v1", "neotrix-crystal", 1, "http", "")
            .expect("write");
        assert_eq!(pair.token_env, DEFAULT_TOKEN_ENV);
        let pair2 = store
            .set_core_pair("http://127.0.0.1:3000/v1", "neotrix-crystal", 1, "http", "MY_ENV")
            .expect("write");
        assert_eq!(pair2.token_env, "MY_ENV");
        let back = store.get_core_pair().expect("read").expect("row");
        assert_eq!(back.token_env, "MY_ENV");
    }

    #[test]
    fn agent_dto_serializes_exact_contract_names() {
        use super::{AgentRunResult, TraceRow};
        let result = AgentRunResult {
            status: "done".to_owned(),
            output: "ok".to_owned(),
            trace: vec![TraceRow {
                kind: "step".to_owned(),
                detail: "did x".to_owned(),
            }],
            model_used: "neotrix-crystal".to_owned(),
            mode: "passthrough".to_owned(),
            tools: vec!["step".to_owned()],
            usage: None,
        };
        let value = serde_json::to_value(&result).expect("serialize");
        assert_eq!(value.get("status").and_then(|v| v.as_str()), Some("done"));
        assert_eq!(value.get("output").and_then(|v| v.as_str()), Some("ok"));
        assert_eq!(value.get("model_used").and_then(|v| v.as_str()), Some("neotrix-crystal"));
        let trace = value.get("trace").and_then(|v| v.as_array()).expect("trace array");
        assert_eq!(trace.len(), 1);
        assert_eq!(trace[0].get("kind").and_then(|v| v.as_str()), Some("step"));
        assert_eq!(trace[0].get("detail").and_then(|v| v.as_str()), Some("did x"));
        // 回解析同构。
        let back = parse_agent_run_result(value).expect("parse");
        assert_eq!(back, result);
    }

    #[test]
    fn capabilities_parse_degrades_without_blocking() {
        // 契约形状 → 版本/工具数。
        let full = serde_json::json!({
            "model": "neotrix-crystal",
            "crystal_version": "0.2.0",
            "tools": [{"name": "bash", "description": "run"}],
            "features": {"agent_run": true},
            "pool": {"count": 1, "degraded": []},
        });
        let info = parse_capabilities(&full);
        assert_eq!(info.crystal_version, "0.2.0");
        assert_eq!(info.tool_count, 1);
        assert_eq!(info.model, "neotrix-crystal");
        // 缺字段/空对象 → 空串/0（status 不挡）。
        let empty = parse_capabilities(&serde_json::json!({}));
        assert_eq!(empty.crystal_version, "");
        assert_eq!(empty.tool_count, 0);
        let weird = parse_capabilities(&serde_json::json!({"tools": "nope"}));
        assert_eq!(weird.tool_count, 0);
        // reload 摘要纯函数：pool 数字 + degraded 列表。
        let summary = summarize_reload(&serde_json::json!({"ok": true, "pool": 3, "degraded": ["x"]}));
        assert!(summary.contains("pool=3"), "{summary}");
        assert!(summary.contains('x'), "{summary}");
    }
}
