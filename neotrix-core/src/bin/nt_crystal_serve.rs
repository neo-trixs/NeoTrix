//! `nt_crystal_serve` — 晶体核心服务（二进制）。
//!
//! 对外只暴露**一个**模型 `neotrix-crystal`（OpenAI 兼容 `/v1`），
//! 内部在池子里自主选择可用模型（健康感知顺位尝试：连续失败降级 +
//! 冷却后重试 + 全灭时兜底全试）。
//!
//! 为何不用 `FallbackRouter::complete` 直接调度：它向链上所有候选
//! 转发同一份 `request`（含同一个 `request.model`），跨模型 id 的
//! fallback 会把错误模型名发给上游；晶体层改为按候选逐个改写
//! `request.model` 后直调 adapter，既保留自主选择语义，又保证
//! 每次外发的模型名正确（诚实注记，勿删）。
//!
//! 上游来源（启动时组装）：
//! 1. 本机 `opencode models` 即时表（Zen 匿名直连，免 key；缺 CLI 则跳过）；
//! 2. `CRYSTAL_ZEN_MODELS` 显式覆盖（逗号分隔 `opencode/<id>`，为空则不用）；
//! 3. `CRYSTAL_UPSTREAMS` 显式端点（`base|key|model` 逗号分隔，如 OpenRouter/Groq）。
//! 全空 → 拒绝启动（无灵魂不演戏，fail-closed）。
//!
//! 路由（执行契约见 `docs/plans/2026-09-25-neobot-sole-entry-audit.md` §4）：
//! `GET /v1/models`（单模型，需认证）/ `POST /v1/chat/completions`
//! （需认证；`stream:true` 走 SSE `data:` 行逐块流式，工具调用以独立
//! `tool_calls` 事件发出，最后 `data: [DONE]`，neobot 可解析）/
//! `GET /v1/capabilities`（需认证：模型 + 工具表 + 特性开关 + 版本 + 池状态）/
//! `POST /v1/agents/run`（需认证：服务端 agent 循环，neotrix 原生手）/
//! `POST /v1/admin/reload`（需认证：重读配置重建池）/
//! `GET /healthz`（开放：存活 + 版本 + 池状态）。只绑 127.0.0.1（本地优先，不对外）。
//!
//! 认证：`crystal.toml` 顶层 `token = "..."`（空 = legacy 开放 + 启动 warn；
//! 文件为空时可用 `CRYSTAL_TOKEN` 环境变量兜底）；置了就强制校验所有
//! `/v1/*` + `/admin/*` 的 `Authorization: Bearer <token>`，`/healthz` 恒开放。

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::{Component, PathBuf};
use std::process::ExitCode;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

use neotrix::l1_action::nt_io::universal_model::openai_adapter::OpenAIUniversal;
use neotrix::l1_action::nt_io::universal_model::traits::UniversalModel;
use neotrix_types::llm_types::{LlmRequest, Message, Role};

use neotrix_gateway::skill_registry::{SkillQuery, SkillRegistry};
use neotrix_multi_agent::coordinator::{DecompositionStrategy, MultiAgentCoordinator};

use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;

const CRYSTAL_MODEL: &str = "neotrix-crystal";
/// 晶体服务版本（执行契约 v1；`/healthz` + `/capabilities` 自述）。
const CRYSTAL_VERSION: &str = "0.2.0";
const ZEN_BASE: &str = "https://opencode.ai/zen/v1";
/// CLI 缺席时的保底 Zen 表（以实测可用为准）。
const ZEN_FALLBACK: &[&str] = &["opencode/space-bunny-free", "opencode/big-pickle"];

fn env_or(key: &str, default: &str) -> String {
    match std::env::var(key) {
        Ok(v) if !v.trim().is_empty() => v.trim().to_owned(),
        _ => default.to_owned(),
    }
}

/// 本机 `opencode models` 即时表（`opencode/<id>` 行；超时/缺席回空）。
fn zen_from_cli() -> Vec<String> {
    let output = std::process::Command::new("opencode")
        .arg("models")
        .output();
    let Ok(output) = output else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .filter(|l| {
            l.split_once('/')
                .map(|(p, m)| !p.trim().is_empty() && !m.trim().is_empty())
                .unwrap_or(false)
        })
        .map(str::to_owned)
        .collect()
}

/// 池成员（id 即外发模型名；adapter 不可 Clone，外包 Arc）。
struct PoolModel {
    id: String,
    adapter: Arc<OpenAIUniversal>,
}

/// 晶体可变内层（`RwLock` 守护：读多写少；`reload` 写锁换池，其余读锁快照）。
struct CrystalInner {
    models: Vec<PoolModel>,
    /// 为空 = legacy 开放（启动 warn）；非空 = 强制 Bearer 校验。
    token: String,
    discover_limit: usize,
    file_upstreams: Vec<UpstreamEntry>,
}

/// 晶体状态：RwLock 内层（池 + token + 配置）+ 健康账本（连续失败计数 + 末次失败时刻）。
struct CrystalState {
    inner: RwLock<CrystalInner>,
    health: Mutex<HashMap<String, (u32, Instant)>>,
    /// CDP 浏览器引擎（懒启动；BrowserEngine 非 Sync → std Mutex 包住，
    /// blocking 线程内独占使用，绝不在 worker 直接 .await）。
    cdp: std::sync::Arc<
        std::sync::Mutex<
            Option<neotrix::l1_action::nt_io::nt_io_browser_engine::BrowserEngine>,
        >,
    >,
}

fn new_cdp_engine() -> neotrix::l1_action::nt_io::nt_io_browser_engine::BrowserEngine {
    use neotrix::l1_action::nt_io::nt_io_browser_engine::{BrowserConfig, BrowserEngine};
    use neotrix::l1_action::nt_io::nt_io_browser_engine::types::BackendKind;
    BrowserEngine::new(BrowserConfig {
        backend: BackendKind::Cdp,
        ..Default::default()
    })
}

/// 连续失败达此数即降级（冷却期内排后面）。
const DEGRADE_AFTER: u32 = 2;
/// 降级冷却秒（到期重给机会）。
const COOLDOWN_SECS: u64 = 300;

impl CrystalState {
    /// 当前 token 快照（空 = legacy 开放）。
    fn current_token(&self) -> String {
        self.inner
            .read()
            .map(|g| g.token.clone())
            .unwrap_or_default()
    }

    /// 池快照（id + adapter Arc 克隆；读锁内复制，锁外再做网络 await，不阻塞 reload）。
    fn snapshot_pool(&self) -> Vec<(String, Arc<OpenAIUniversal>)> {
        self.inner
            .read()
            .map(|g| {
                g.models
                    .iter()
                    .map(|m| (m.id.clone(), Arc::clone(&m.adapter)))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 配置快照（reload 在锁外重建池用）。
    fn config_view(&self) -> (usize, Vec<UpstreamEntry>) {
        self.inner
            .read()
            .map(|g| (g.discover_limit, g.file_upstreams.clone()))
            .unwrap_or((5, Vec::new()))
    }

    /// 降级名单（健康账本口径，各端点统一）。
    fn degraded_ids(&self) -> Vec<String> {
        self.health
            .lock()
            .map(|h| h.clone())
            .unwrap_or_default()
            .iter()
            .filter(|(_, (fails, _))| *fails >= DEGRADE_AFTER)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// 池 id + 降级名单（`/healthz`、`/capabilities`、`reload` 回应用）。
    fn pool_view(&self) -> (Vec<String>, Vec<String>) {
        let ids: Vec<String> = self
            .inner
            .read()
            .map(|g| g.models.iter().map(|m| m.id.clone()).collect())
            .unwrap_or_default();
        let degraded = self.degraded_ids();
        (ids, degraded)
    }

    /// 尝试顺序：健康在前，冷却中降级的沉底；全灭时原序全试（不静默失败）。
    fn order(&self) -> Vec<usize> {
        let ids: Vec<String> = self
            .inner
            .read()
            .map(|g| g.models.iter().map(|m| m.id.clone()).collect())
            .unwrap_or_default();
        let health = self.health.lock().map(|h| h.clone()).unwrap_or_default();
        let now = Instant::now();
        let mut ok = Vec::new();
        let mut cool = Vec::new();
        for (i, id) in ids.iter().enumerate() {
            match health.get(id) {
                Some((fails, at)) if *fails >= DEGRADE_AFTER
                    && now.duration_since(*at) < Duration::from_secs(COOLDOWN_SECS) =>
                {
                    cool.push(i)
                }
                _ => ok.push(i),
            }
        }
        ok.extend(cool);
        ok
    }

    fn record(&self, id: &str, success: bool) {
        if let Ok(mut health) = self.health.lock() {
            if success {
                health.remove(id);
            } else {
                let entry = health.entry(id.to_owned()).or_insert((0, Instant::now()));
                entry.0 += 1;
                entry.1 = Instant::now();
            }
        }
    }

    /// 换池（reload 写锁路径）：替换模型表 + 裁剪已摘除成员的健康记录。
    fn swap_pool(&self, pool: Vec<PoolModel>) -> (Vec<String>, Vec<String>) {
        let ids: Vec<String> = pool.iter().map(|m| m.id.clone()).collect();
        if let Ok(mut inner) = self.inner.write() {
            inner.models = pool;
        }
        if let Ok(mut health) = self.health.lock() {
            health.retain(|k, _| ids.iter().any(|id| id == k));
        }
        let degraded = self.degraded_ids();
        (ids, degraded)
    }
}

/// Bearer 校验：token 为空即 legacy 开放（true）；否则必须
/// `Authorization: Bearer <token>` 精确匹配。
fn auth_ok(state: &Arc<CrystalState>, headers: &HeaderMap) -> bool {
    let token = state.current_token();
    if token.trim().is_empty() {
        return true;
    }
    let Some(value) = headers.get(axum::http::header::AUTHORIZATION) else {
        return false;
    };
    let Ok(text) = value.to_str() else {
        return false;
    };
    let Some(bearer) = text.strip_prefix("Bearer ") else {
        return false;
    };
    bearer == token
}

fn unauthorized() -> axum::response::Response {
    (
        StatusCode::UNAUTHORIZED,
        axum::Json(serde_json::json!({ "error": "unauthorized" })),
    )
        .into_response()
}

/// 晶体配置文件（项目主代码配置：`neotrix-core/config/crystal.toml`；
/// 路径可用 `CRYSTAL_CONFIG` 覆盖；缺失即跳过，不挡启动）。
#[derive(Debug, serde::Deserialize)]
struct CrystalFile {
    port: Option<u16>,
    discover_limit: Option<usize>,
    /// 认证 token（执行契约 v1）：空/缺省 = legacy 开放 + 启动 warn；
    /// 置了就强制校验所有 `/v1/*` + `/admin/*` 的 `Authorization: Bearer`。
    #[serde(default)]
    token: String,
    #[serde(default)]
    upstream: Vec<UpstreamEntry>,
}

#[derive(Debug, serde::Deserialize, Clone)]
struct UpstreamEntry {
    #[serde(default)]
    name: String,
    base_url: String,
    #[serde(default)]
    api_key: String,
    #[serde(default)]
    models: Vec<String>,
}

fn config_path() -> Option<std::path::PathBuf> {
    if let Ok(p) = std::env::var("CRYSTAL_CONFIG") {
        let trimmed = p.trim();
        if !trimmed.is_empty() {
            return Some(std::path::PathBuf::from(trimmed));
        }
    }
    let local = std::path::PathBuf::from("neotrix-core/config/crystal.toml");
    if local.is_file() {
        return Some(local);
    }
    None
}

/// 文件配置快照（`None` = 无文件/空文件/解析失败，调用方沿用旧快照，
/// 绝不在解析失败时静默丢 token——fail-safe）。
struct FileConfig {
    port: Option<u16>,
    discover_limit: usize,
    upstreams: Vec<UpstreamEntry>,
    token: String,
}

fn load_file() -> Option<FileConfig> {
    let path = config_path()?;
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    if text.trim().is_empty() {
        return None;
    }
    match toml::from_str::<CrystalFile>(&text) {
        Ok(cfg) => Some(FileConfig {
            port: cfg.port,
            discover_limit: cfg.discover_limit.unwrap_or(5).clamp(1, 20),
            upstreams: cfg.upstream,
            token: cfg.token.trim().to_owned(),
        }),
        Err(e) => {
            eprintln!("nt_crystal_serve: config parse failed ({}), ignoring file", e);
            None
        }
    }
}

/// 探活第三方端点（key 只进 header，不打日志）。
/// 代理与 NO_PROXY（含默认回环）同 fetch.rs 律：配代理后 127.0.0.1 仍直连。
fn proxy_from_env_local() -> Option<reqwest::Url> {
    ["HTTPS_PROXY", "https_proxy", "HTTP_PROXY", "http_proxy", "ALL_PROXY", "all_proxy"]
        .iter()
        .find_map(|k| std::env::var(k).ok())
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty())
        .and_then(|s| reqwest::Url::parse(&s).ok())
}

fn no_proxy_hit_local(host: &str) -> bool {
    let h = host.trim().trim_end_matches('.').to_lowercase();
    if h.is_empty() {
        return false;
    }
    if h == "localhost" || h == "127.0.0.1" || h == "::1" {
        return true;
    }
    ["NO_PROXY", "no_proxy"]
        .iter()
        .find_map(|k| std::env::var(k).ok())
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .any(|rule| {
            let r = rule.trim_end_matches('.').to_lowercase();
            if r == "*" {
                true
            } else if let Some(suffix) = r.strip_prefix('.') {
                h == suffix || h.ends_with(&format!(".{suffix}"))
            } else {
                h == r
            }
        })
}

fn discover_upstream(base: &str, key: &str) -> Vec<String> {
    let mut builder = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10));
    if let Some(proxy) = proxy_from_env_local() {
        builder = builder.proxy(reqwest::Proxy::custom(move |url| {
            if no_proxy_hit_local(url.host_str().unwrap_or("")) {
                None
            } else {
                Some(proxy.clone())
            }
        }));
    }
    let Ok(client) = builder.build() else {
        return Vec::new();
    };
    let url = format!("{}/models", base.trim_end_matches('/'));
    let resp = client.get(&url).bearer_auth(key).send();
    let Ok(resp) = resp else {
        return Vec::new();
    };
    if !resp.status().is_success() {
        return Vec::new();
    }
    resp.json::<serde_json::Value>()
        .ok()
        .and_then(|v| v.get("data").and_then(|d| d.as_array()).cloned())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m.get("id").and_then(|v| v.as_str()).map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}
fn build_pool(file_upstreams: &[UpstreamEntry], discover_limit: usize) -> Result<Vec<PoolModel>, String> {
    let mut pool: Vec<PoolModel> = Vec::new();
    // 1) 显式 Zen 表优先；2) 本机 CLI 即时表；3) 保底表。
    let explicit: Vec<String> = env_or("CRYSTAL_ZEN_MODELS", "")
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect();
    let mut zen: Vec<String> = if explicit.is_empty() {
        zen_from_cli()
    } else {
        explicit
    };
    if zen.is_empty() {
        zen = ZEN_FALLBACK.iter().map(|s| (*s).to_owned()).collect();
    }
    for id in &zen {
        // Zen HTTP 口要裸模型名（`opencode/` 前缀只在 CLI 侧有效，直发会被拒）。
        let wire = id.strip_prefix("opencode/").unwrap_or(id);
        let model = OpenAIUniversal::new(String::new(), wire)
            .with_base_url(ZEN_BASE)
            .with_zen_anonymous(true);
        pool.push(PoolModel { id: wire.to_owned(), adapter: Arc::new(model) });
    }
    // 4) 显式上游（base|key|model 逗号分隔）。
    for item in env_or("CRYSTAL_UPSTREAMS", "").split(',') {
        let parts: Vec<&str> = item.split('|').map(str::trim).collect();
        if parts.len() != 3 || parts.iter().any(|p| p.is_empty()) {
            continue;
        }
        let model = OpenAIUniversal::new(parts[1].to_owned(), parts[2])
            .with_base_url(parts[0]);
        pool.push(PoolModel { id: parts[2].to_owned(), adapter: Arc::new(model) });
    }
    // 5) 配置文件上游（第三方模型库）：逐个探活，只链活的。
    for entry in file_upstreams {
        let name = if entry.name.trim().is_empty() { entry.base_url.clone() } else { entry.name.clone() };
        let base = entry.base_url.trim().trim_end_matches('/').to_owned();
        if base.is_empty() {
            eprintln!("nt_crystal_serve: upstream '{name}' skipped (empty base_url)");
            continue;
        }
        let ids: Vec<String> = if entry.models.is_empty() {
            let found = discover_upstream(&base, &entry.api_key);
            if found.is_empty() {
                eprintln!("nt_crystal_serve: upstream '{name}' dead or key rejected, skipped");
                continue;
            }
            found.into_iter().take(discover_limit).collect()
        } else {
            entry.models.clone()
        };
        // 显式 models 不再探活（以配置为准，跑轮失败由健康账本降级）。
        for id in &ids {
            let model = OpenAIUniversal::new(entry.api_key.clone(), id).with_base_url(&base);
            pool.push(PoolModel { id: id.clone(), adapter: Arc::new(model) });
        }
        eprintln!("nt_crystal_serve: upstream '{name}' ok ({} models)", ids.len());
    }
    if pool.is_empty() {
        return Err("no upstream models (opencode CLI missing and CRYSTAL_UPSTREAMS empty)".to_owned());
    }
    Ok(pool)
}

fn role_of(role: &str) -> Role {
    match role.trim().to_lowercase().as_str() {
        "system" => Role::System,
        "assistant" => Role::Assistant,
        "tool" => Role::Tool,
        _ => Role::User,
    }
}

async fn handle_models(
    axum::extract::State(state): axum::extract::State<Arc<CrystalState>>,
    headers: HeaderMap,
) -> axum::response::Response {
    if !auth_ok(&state, &headers) {
        return unauthorized();
    }
    axum::Json(serde_json::json!({
        "object": "list",
        "data": [{ "id": CRYSTAL_MODEL, "object": "model", "owned_by": "neotrix" }],
    }))
    .into_response()
}

async fn handle_health(
    axum::extract::State(state): axum::extract::State<Arc<CrystalState>>,
) -> axum::Json<serde_json::Value> {
    // /healthz 恒开放（探活与版本自述不设门）。
    let (ids, degraded) = state.pool_view();
    axum::Json(serde_json::json!({
        "ok": true,
        "model": CRYSTAL_MODEL,
        "crystal_version": CRYSTAL_VERSION,
        "pool": ids,
        "degraded": degraded,
    }))
}

/// 服务端 agent 循环实际可用的 neotrix 原生手（`/capabilities` 广播表与
/// `run_agent_loop` 的执行能力必须一致，增工具先增实现再增表）。
fn agent_tool_table() -> serde_json::Value {
    serde_json::json!([
        { "name": "skills_search", "description": "neotrix 原生 skill_registry 检索（scan + search），只读" },
        { "name": "memory_recall", "description": "neotrix 侧记忆体读贯通；未接线时如实降级，用客户端 memory" },
        { "name": "coordinator_plan", "description": "multi-agent coordinator 只读规划（create_plan，不执行）" },
        { "name": "workspace_read", "description": "workspace 内文件读取（禁..与 workspace 外绝对路径）" },
        { "name": "workspace_write", "description": "workspace 内文件写入（禁..与 workspace 外绝对路径）" },
        { "name": "workspace_bash", "description": "workspace 内 shell 执行（cwd 锁定，禁..，workspace 外绝对路径拒绝）" },
        { "name": "web_search", "description": "联网搜索（DDG/Wikipedia 回退，证据标注；count 缺省 5 上限 10）" },
        { "name": "web_fetch", "description": "抓取网页正文（Http 后端；只允许 http/https，4000 字截断）" },
        { "name": "web_act", "description": "真机操控（headless Chrome CDP 单会话单次语义：navigate/click/type/text/shot；selector 为 CSS；shot 回路径）" },
    ])
}

/// neotrix 侧记忆体是否可读（经验库目录存在且有审计/KB 文件即有）。
fn neotrix_memory_available() -> bool {
    memory_sources().0
}

/// 查询分词：ASCII 词（≥2）+ CJK bigram（中英混排整词子串命中率太低）。
fn recall_keywords(query: &str) -> Vec<String> {
    /// 2026-09-29：本地副本已删，改用唯一事实源。
    /// 本处喂给 **bigram** ⇒ 必须用窄口径 `is_cjk_han`（宽口径会把标点
    /// 塞进 bigram 产垃圾词元）。⚠️ 原口径含 ExtA，统一后 ExtA 走 ASCII 分支
    /// —— 该区是罕用生僻字，对召回影响可忽略。
    fn is_cjk(c: char) -> bool {
        neotrix_types::core::nt_cjk::is_cjk_han(c)
    }
    fn flush_ascii(buf: &mut String, out: &mut Vec<String>) {
        if buf.chars().count() >= 2 {
            out.push(buf.clone());
        }
        buf.clear();
    }
    fn flush_cjk(buf: &mut Vec<char>, out: &mut Vec<String>) {
        for w in buf.windows(2) {
            out.push(w.iter().collect());
        }
        buf.clear();
    }
    let mut out = Vec::new();
    let mut ascii = String::new();
    let mut cjk: Vec<char> = Vec::new();
    for c in query.chars() {
        if c.is_ascii_alphanumeric() {
            if !cjk.is_empty() {
                flush_cjk(&mut cjk, &mut out);
            }
            ascii.push(c);
        } else if is_cjk(c) {
            flush_ascii(&mut ascii, &mut out);
            cjk.push(c);
        } else {
            flush_ascii(&mut ascii, &mut out);
            flush_cjk(&mut cjk, &mut out);
        }
    }
    flush_ascii(&mut ascii, &mut out);
    flush_cjk(&mut cjk, &mut out);
    out
}
fn try_neotrix_memory_recall(query: &str) -> Option<String> {
    let (ok, files, kb) = memory_sources_full();
    if !ok {
        return None;
    }
    // 经验 recall：在 neotrix 自身经验库里按分词命中行，回最新 5 条
    // （每条截 200 字，总上限 1500 字）。无命中回 None（调用方用客户端 memory）。
    // 注意：命中内容会进上游 prompt（与 workspace 内容同信任边界：用户自己的数据）。
    let keywords: Vec<String> = recall_keywords(query);
    if keywords.is_empty() {
        return None;
    }
    let mut hits: Vec<String> = Vec::new();
    let mut push_line = |line: &str, tag: &str| {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.len() < 8 {
            return;
        }
        let score: usize = keywords.iter().filter(|k| trimmed.contains(k.as_str())).count();
        if score == 0 {
            return;
        }
        let snippet: String = trimmed.chars().take(200).collect();
        hits.push(format!("[{tag}:{score}] {snippet}"));
    };
    if let Some(kb_text) = kb {
        for line in kb_text.lines() {
            push_line(line, "kb");
        }
    }
    for path in files.into_iter().take(5) {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let lines: Vec<&str> = text.lines().collect();
        let start = lines.len().saturating_sub(200);
        let tag = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("audit")
            .to_owned();
        for line in &lines[start..] {
            push_line(line, &tag);
        }
    }
    if hits.is_empty() {
        return None;
    }
    // 新的在后就是更新的（审计尾部 + 文件按 mtime 倒序），取末尾 5 条。
    let tail: Vec<String> = hits.into_iter().rev().take(5).collect();
    let mut out = tail.into_iter().rev().collect::<Vec<_>>().join("\n");
    if out.chars().count() > 1500 {
        out = out.chars().take(1500).collect();
    }
    Some(out)
}

/// 经验库源（目录 + 最新审计文件倒序 + KB 文本）。
fn memory_sources_full() -> (bool, Vec<std::path::PathBuf>, Option<String>) {
    let (ok, files) = memory_sources();
    if !ok {
        return (false, Vec::new(), None);
    }
    let home = std::env::var("HOME").unwrap_or_default();
    let kb = std::fs::read_to_string(format!("{home}/.neotrix/KB_BRAIN_TASKS.md")).ok();
    (true, files, kb)
}

fn memory_sources() -> (bool, Vec<std::path::PathBuf>) {
    let home = std::env::var("HOME").unwrap_or_default();
    if home.is_empty() {
        return (false, Vec::new());
    }
    let dir = std::path::PathBuf::from(format!("{home}/.neotrix"));
    let Ok(rd) = std::fs::read_dir(&dir) else {
        return (false, Vec::new());
    };
    let mut files: Vec<(std::time::SystemTime, std::path::PathBuf)> = Vec::new();
    for entry in rd.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !name.starts_with("audit_") || !name.ends_with(".jsonl") {
            continue;
        }
        let mtime = entry.metadata().and_then(|m| m.modified()).unwrap_or(std::time::UNIX_EPOCH);
        files.push((mtime, path));
    }
    if files.is_empty() {
        return (false, Vec::new());
    }
    files.sort_by(|a, b| b.0.cmp(&a.0));
    (true, files.into_iter().map(|(_, p)| p).collect())
}

async fn handle_capabilities(
    axum::extract::State(state): axum::extract::State<Arc<CrystalState>>,
    headers: HeaderMap,
) -> axum::response::Response {
    if !auth_ok(&state, &headers) {
        return unauthorized();
    }
    let (ids, degraded) = state.pool_view();
    let memory_read = neotrix_memory_available();
    axum::Json(serde_json::json!({
        "model": CRYSTAL_MODEL,
        "crystal_version": CRYSTAL_VERSION,
        "tools": agent_tool_table(),
        "features": {
            "agent_run": true,
            "memory_read": memory_read,
            "hot_reload": true,
            "streaming_tool_calls": true,
        },
        "pool": { "count": ids.len(), "degraded": degraded },
    }))
    .into_response()
}

async fn handle_reload(
    axum::extract::State(state): axum::extract::State<Arc<CrystalState>>,
    headers: HeaderMap,
) -> axum::response::Response {
    if !auth_ok(&state, &headers) {
        return (
            StatusCode::UNAUTHORIZED,
            axum::Json(serde_json::json!({ "ok": false, "error": "unauthorized" })),
        )
            .into_response();
    }
    // 锁外重建（探活是阻塞 IO，丢进 spawn_blocking，不卡住 async 运行时）。
    // reload 先重读配置文件（热刷新语义）；文件缺失/解析失败则沿用启动期快照，
    // 绝不静默丢 token（fail-safe）。
    let (discover_limit, file_upstreams) = state.config_view();
    let (limit, upstreams) = match load_file() {
        Some(fresh) => {
            if let Ok(mut inner) = state.inner.write() {
                inner.token = fresh.token.clone();
                inner.discover_limit = fresh.discover_limit;
            }
            (fresh.discover_limit, fresh.upstreams)
        }
        None => (discover_limit, file_upstreams),
    };
    let built = tokio::task::spawn_blocking(move || build_pool(&upstreams, limit)).await;
    let pool = match built {
        Ok(Ok(p)) => p,
        Ok(Err(e)) => {
            return (
                StatusCode::BAD_GATEWAY,
                axum::Json(serde_json::json!({ "ok": false, "error": e })),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "ok": false, "error": format!("reload task: {e:?}") })),
            )
                .into_response();
        }
    };
    let (ids, degraded) = state.swap_pool(pool);
    axum::Json(serde_json::json!({ "ok": true, "pool": ids, "degraded": degraded })).into_response()
}

// ── agent 循环（POST /v1/agents/run）─────────────────────────────

/// trace 条目（`{kind, detail}`；失败也只进 trace，不炸）。
fn trace_push(trace: &mut Vec<serde_json::Value>, kind: &str, detail: String) {
    trace.push(serde_json::json!({ "kind": kind, "detail": detail }));
}

/// 按字符截断（防超长输出撑爆响应；中文安全）。
fn snip(s: &str, n: usize) -> String {
    let out: String = s.chars().take(n).collect();
    if s.chars().count() > n {
        format!("{out}…")
    } else {
        out
    }
}

#[derive(Debug, Clone)]
enum AgentOp {
    Read(String),
    Write(String, String),
    Bash(String),
    List(String),
    WebSearch(String, usize),
    WebFetch(String),
    WebAct {
        url: String,
        action: String,
        selector: String,
        text: String,
    },
}

/// `context` 里的显式工具指令（尽力解析；非 JSON / 无 ops 即空）。
/// 接受 `{ "ops": [{ "op": "read|write|bash|web_search|web_fetch", ... }] }`
/// 或顶层数组同构。`exec`/`run` 视为 bash 别名；`search`/`fetch`/`browse` 视为 web 别名。
fn parse_context_ops(context: &str) -> Vec<AgentOp> {
    let value: serde_json::Value =
        serde_json::from_str(context).unwrap_or(serde_json::Value::Null);
    let items: Vec<&serde_json::Value> = match &value {
        serde_json::Value::Array(arr) => arr.iter().collect(),
        serde_json::Value::Object(map) => map
            .get("ops")
            .and_then(|o| o.as_array())
            .map(|a| a.iter().collect())
            .unwrap_or_default(),
        _ => Vec::new(),
    };
    let mut ops = Vec::new();
    for item in items {
        let op = item
            .get("op")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_lowercase();
        match op.as_str() {
            "read" => {
                if let Some(p) = item.get("path").and_then(|v| v.as_str()) {
                    if !p.trim().is_empty() {
                        ops.push(AgentOp::Read(p.to_owned()));
                    }
                }
            }
            "write" => {
                let path = item.get("path").and_then(|v| v.as_str()).unwrap_or("");
                let content = item
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if !path.trim().is_empty() {
                    ops.push(AgentOp::Write(path.to_owned(), content.to_owned()));
                }
            }
            "bash" | "exec" | "run" => {
                let cmd = item
                    .get("cmd")
                    .and_then(|v| v.as_str())
                    .or_else(|| item.get("command").and_then(|v| v.as_str()))
                    .unwrap_or("");
                if !cmd.trim().is_empty() {
                    ops.push(AgentOp::Bash(cmd.to_owned()));
                }
            }
            "ls" | "list" => {
                let path = item.get("path").and_then(|v| v.as_str()).unwrap_or("");
                ops.push(AgentOp::List(path.to_owned()));
            }
            "web_search" | "search" => {
                let query = item
                    .get("query")
                    .and_then(|v| v.as_str())
                    .or_else(|| item.get("q").and_then(|v| v.as_str()))
                    .or_else(|| item.get("text").and_then(|v| v.as_str()))
                    .unwrap_or("");
                let count = item
                    .get("count")
                    .and_then(|v| v.as_u64())
                    .map(|n| n.clamp(1, 10) as usize)
                    .unwrap_or(5);
                if !query.trim().is_empty() {
                    ops.push(AgentOp::WebSearch(query.to_owned(), count));
                }
            }
            "web_fetch" | "fetch" | "browse" => {
                let url = item.get("url").and_then(|v| v.as_str()).unwrap_or("");
                if !url.trim().is_empty() {
                    ops.push(AgentOp::WebFetch(url.to_owned()));
                }
            }
            "web_act" | "act" | "click" | "type" => {
                let explicit = item.get("op").and_then(|v| v.as_str()).unwrap_or("");
                let action = if explicit.eq_ignore_ascii_case("click") {
                    "click".to_owned()
                } else if explicit.eq_ignore_ascii_case("type") {
                    "type".to_owned()
                } else {
                    item.get("action")
                        .and_then(|v| v.as_str())
                        .unwrap_or("text")
                        .trim()
                        .to_lowercase()
                };
                ops.push(AgentOp::WebAct {
                    url: item.get("url").and_then(|v| v.as_str()).unwrap_or("").to_owned(),
                    action,
                    selector: item.get("selector").and_then(|v| v.as_str()).unwrap_or("").to_owned(),
                    text: item
                        .get("text")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_owned(),
                });
            }
            _ => {}
        }
    }
    ops
}

/// workspace 落点：空 = 服务端 cwd；否则禁 `..`，相对路径挂到 cwd 下；
/// 存在则规范化，不存在则创建（调用方声明的工作区，允许创建；后续操作仍受 jail 约束）。
fn resolve_workspace(req: &str) -> Result<PathBuf, String> {
    let trimmed = req.trim();
    if trimmed.is_empty() {
        return std::env::current_dir()
            .map_err(|e| format!("workspace: current_dir: {e}"))?
            .canonicalize()
            .map_err(|e| format!("workspace: canonicalize cwd: {e}"));
    }
    let raw = PathBuf::from(trimmed);
    if raw
        .components()
        .any(|c| matches!(c, Component::ParentDir))
    {
        return Err("workspace refused: '..' not allowed".to_owned());
    }
    let abs = if raw.is_absolute() {
        raw
    } else {
        std::env::current_dir()
            .map_err(|e| format!("workspace: current_dir: {e}"))?
            .join(raw)
    };
    if abs.is_dir() {
        return abs
            .canonicalize()
            .map_err(|e| format!("workspace: canonicalize: {e}"));
    }
    std::fs::create_dir_all(&abs).map_err(|e| format!("workspace: create: {e}"))?;
    abs.canonicalize()
        .map_err(|e| format!("workspace: canonicalize: {e}"))
}

/// 越狱拒绝uls：用户路径拼进 workspace 前先过狱。
/// 拒绝 `..` 与 workspace 外绝对路径；通过即回拼接后路径。
fn jail_join(base: &PathBuf, user: &str) -> Result<PathBuf, String> {
    let trimmed = user.trim();
    if trimmed.is_empty() {
        return Err("empty path".to_owned());
    }
    let raw = PathBuf::from(trimmed);
    if raw
        .components()
        .any(|c| matches!(c, Component::ParentDir))
    {
        return Err(format!("refused: '..' not allowed ({trimmed})"));
    }
    if raw.is_absolute() {
        // 存在则按规范路径判归属，不存在按词法判归属（都不出狱才放行）。
        let canon_base = base
            .canonicalize()
            .unwrap_or_else(|_| base.clone());
        let canon_raw = raw.canonicalize().unwrap_or_else(|_| raw.clone());
        if !canon_raw.starts_with(&canon_base) {
            return Err(format!(
                "refused: absolute path outside workspace ({trimmed})"
            ));
        }
        return Ok(raw);
    }
    Ok(base.join(raw))
}

/// workspace 内文件读（>1MB 拒绝进 trace；内容按字符转码）。
fn read_jailed(base: &PathBuf, user: &str) -> Result<String, String> {
    const READ_MAX_BYTES: u64 = 1_048_576;
    let full = jail_join(base, user)?;
    let meta = std::fs::metadata(&full).map_err(|e| format!("read metadata: {e}"))?;
    if meta.len() > READ_MAX_BYTES {
        return Err(format!("read refused: file too large ({} bytes)", meta.len()));
    }
    let bytes = std::fs::read(&full).map_err(|e| format!("read: {e}"))?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// workspace 内文件写（父目录自动创建，仍在狱内）。
fn write_jailed(base: &PathBuf, user: &str, content: &str) -> Result<usize, String> {
    let full = jail_join(base, user)?;
    if let Some(parent) = full.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| format!("write mkdir: {e}"))?;
        }
    }
    std::fs::write(&full, content).map_err(|e| format!("write: {e}"))?;
    Ok(content.len())
}

/// workspace 内目录列单（最多 100 条，只读）。
fn list_jailed(base: &PathBuf, user: &str) -> Result<Vec<String>, String> {
    let dir = if user.trim().is_empty() {
        base.clone()
    } else {
        jail_join(base, user)?
    };
    let entries =
        std::fs::read_dir(&dir).map_err(|e| format!("ls: {e}"))?;
    let mut names = Vec::new();
    for entry in entries {
        let Ok(entry) = entry else {
            continue;
        };
        names.push(entry.file_name().to_string_lossy().into_owned());
        if names.len() >= 100 {
            break;
        }
    }
    names.sort();
    Ok(names)
}

/// bash 越狱预检：禁 `..`；`/` 开头 token 必须落在 workspace 内
/// （系统 bin 目录 `/bin /sbin /usr/bin /usr/sbin /usr/libexec` 放行执行）。
fn bash_jail_ok(workspace: &str, cmd: &str) -> Result<(), String> {
    const SYSTEM_BIN_DIRS: &[&str] = &[
        "/bin/",
        "/sbin/",
        "/usr/bin/",
        "/usr/sbin/",
        "/usr/libexec/",
    ];
    if cmd.contains("..") {
        return Err("refused: '..' not allowed in bash".to_owned());
    }
    let prefix = if workspace.ends_with('/') {
        workspace.to_owned()
    } else {
        format!("{workspace}/")
    };
    let system_bin = |tok: &str| {
        SYSTEM_BIN_DIRS.iter().any(|d| {
            tok == d.trim_end_matches('/') || tok.starts_with(d)
        })
    };
    for raw in cmd.split(|c: char| {
        c.is_whitespace()
            || matches!(
                c,
                '\'' | '"' | '(' | ')' | ';' | '|' | '&' | '`' | '<' | '>' | '$'
            )
    }) {
        let tok = raw
            .trim()
            .trim_matches(|c| c == '\'' || c == '"')
            .trim_end_matches(|c| c == ',' || c == ':');
        if tok.is_empty() || !tok.starts_with('/') {
            continue;
        }
        if tok == "/" {
            return Err("refused: '/' not allowed in bash".to_owned());
        }
        if tok == workspace || tok.starts_with(&prefix) || system_bin(tok) {
            continue;
        }
        return Err(format!(
            "refused: absolute path outside workspace ({tok})"
        ));
    }
    Ok(())
}

/// workspace 内 shell（cwd 锁定；30s 超时；非零退出也只进 Ok 文本，不炸）。
async fn run_bash(workspace: &PathBuf, cmd: &str) -> Result<String, String> {
    const BASH_TIMEOUT_SECS: u64 = 30;
    let child = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .current_dir(workspace)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("bash spawn: {e}"))?;
    let output = tokio::time::timeout(
        Duration::from_secs(BASH_TIMEOUT_SECS),
        child.wait_with_output(),
    )
    .await
    .map_err(|_| format!("bash timeout ({BASH_TIMEOUT_SECS}s)"))?
    .map_err(|e| format!("bash wait: {e}"))?;
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    if !stderr.trim().is_empty() {
        text.push_str("\n[stderr]\n");
        text.push_str(stderr.trim());
    }
    if output.status.success() {
        Ok(text)
    } else {
        Ok(format!(
            "[exit {}]\n{text}",
            output.status.code().unwrap_or(-1)
        ))
    }
}

/// 真机操控（blocking 外壳）：headless Chrome CDP，单会话单次语义
/// （建会话 → 可选 navigate → 单动作 → 回正文/路径）。
/// action: navigate | click | type | text | shot（缺省 text）。
/// 需要 url（navigate 除外可复用？不复用：每次新会话，单次语义诚实声明）。
fn web_act_blocking(
    cdp: &std::sync::Arc<
        std::sync::Mutex<
            Option<neotrix::l1_action::nt_io::nt_io_browser_engine::BrowserEngine>,
        >,
    >,
    url: &str,
    action: &str,
    selector: &str,
    text: &str,
) -> Result<String, String> {
    use neotrix::l1_action::nt_io::nt_io_browser_engine::types::BrowserAction;
    let target = url.trim().to_owned();
    if !target.is_empty() && !(target.starts_with("http://") || target.starts_with("https://")) {
        return Err(format!("web_act refused (scheme): {target}"));
    }
    let action_name = action.trim().to_lowercase();
    let selector = selector.to_owned();
    let text = text.to_owned();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("runtime: {e}"))?;
    runtime.block_on(async move {
        // 最多两轮：首轮失败若像连接断了（receiver gone/send failed/closed），
        // 重置引擎（下轮重拉 Chrome）再试一次；仍失败如实返回。
        let mut last_error: String;
        for _ in 0..2 {
            // 借用引擎执行（单次加锁，全程同一 blocking 线程；
            // guard 不跨 runtime 边界，不包闭包避免借用逃逸）。
            let attempt: Result<String, String> = async {
                let mut guard = cdp.lock().map_err(|e| format!("cdp lock: {e}"))?;
                if guard.is_none() {
                    *guard = Some(new_cdp_engine());
                }
                let Some(engine) = guard.as_ref() else {
                    return Err("cdp engine missing".to_owned());
                };
                let session = engine
                .create_session()
                .await
                .map_err(|e| format!("session: {e:?}"))?;
            if !target.is_empty() {
                let nav = engine
                    .execute(&session, BrowserAction::Navigate { url: target.clone() })
                    .await
                    .map_err(|e| format!("navigate: {e:?}"))?;
                if !nav.success {
                    return Err(nav.error.unwrap_or_else(|| "navigate failed".to_owned()));
                }
            }
            let act = match action_name.as_str() {
                "navigate" => {
                    if target.is_empty() {
                        return Err("navigate needs url".to_owned());
                    }
                    // 已在上面执行，直接取正文。
                    BrowserAction::GetContent
                }
                "click" => {
                    if selector.trim().is_empty() {
                        return Err("click needs selector".to_owned());
                    }
                    BrowserAction::Click { selector: selector.clone() }
                }
                "type" => {
                    if selector.trim().is_empty() {
                        return Err("type needs selector".to_owned());
                    }
                    BrowserAction::Type { selector: selector.clone(), text: text.clone() }
                }
                "shot" => BrowserAction::Screenshot,
                _ => {
                    if !selector.trim().is_empty() {
                        BrowserAction::GetText { selector: selector.clone() }
                    } else {
                        BrowserAction::GetContent
                    }
                }
            };
            let result = match engine.execute(&session, act).await {
                Ok(r) => r,
                // click 的特殊竞态：neotrix 的 Click 内置 post-click GetContent，
                // 页面正在跳转时会报 Cannot find context——此时点击本身已生效，
                // 转部分成功（指引用 text op 重读新页），不算失败。
                Err(e)
                    if action_name.as_str() == "click"
                        && e.to_string().contains("Cannot find context") =>
                {
                    return Ok(
                        "clicked (page navigating; re-read with a text op)".to_owned(),
                    );
                }
                Err(e) => return Err(format!("act: {e:?}")),
            };
            if !result.success {
                return Err(result.error.unwrap_or_else(|| "act failed".to_owned()));
            }
            if !result.success {
                return Err(result.error.unwrap_or_else(|| "act failed".to_owned()));
            }
            Ok(result.output)
        }
        .await;
        match attempt {
            Ok(out) => return Ok(out),
            Err(e) => {
                let dead = e.contains("receiver is gone")
                    || e.contains("send failed")
                    || e.contains("closed")
                    || e.contains("channel closed");
                last_error = e;
                if dead {
                    // 连接断了：先清掉占着 profile 锁的僵尸 Chrome（chromiumoxide-runner
                    // 专属 profile，不碰用户正常 Chrome），再重置引擎，下轮重拉。
                    // best-effort：杀不掉也不挡，下轮照样试。
                    let _killed: Result<std::process::Output, std::io::Error> =
                        std::process::Command::new("pkill")
                            .args(["-f", "chromiumoxide-runner"])
                            .output();
                    if let Ok(mut guard) = cdp.lock() {
                        *guard = None;
                    }
                    continue;
                }
                return Err(last_error);
            }
        }
        }
    Err("web_act: no attempt ran".to_owned())
    })
}
/// 网页正文抓取（同步外壳，供 spawn_blocking 调用；Http 后端）。
/// BrowserEngine 同上，非 Sync，走 blocking + 独立 runtime。
fn web_fetch_text_blocking(url: &str) -> Result<String, String> {
    use neotrix::l1_action::nt_io::nt_io_browser_engine::{
        BrowserConfig, BrowserEngine,
        types::{BackendKind, BrowserAction},
    };
    let target = url.to_owned();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("runtime: {e}"))?;
    runtime.block_on(async move {
        let engine = BrowserEngine::new(BrowserConfig {
            backend: BackendKind::Http,
            ..Default::default()
        });
        let session = engine
            .create_session()
            .await
            .map_err(|e| format!("session: {e:?}"))?;
        let navigate = tokio::time::timeout(
            std::time::Duration::from_secs(60),
            engine.execute(&session, BrowserAction::Navigate { url: target }),
        )
        .await
        .map_err(|_| "navigate timeout 60s".to_owned())?
        .map_err(|e| format!("navigate: {e:?}"))?;
        if !navigate.success {
            return Err(navigate
                .error
                .unwrap_or_else(|| "navigate failed".to_owned()));
        }
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(60),
            engine.execute(&session, BrowserAction::GetContent),
        )
        .await
        .map_err(|_| "content timeout 60s".to_owned())?
        .map_err(|e| format!("content: {e:?}"))?;
        if !result.success {
            return Err(result.error.unwrap_or_else(|| "fetch failed".to_owned()));
        }
        Ok(result.output)
    })
}

/// 记忆写回：终态摘要追加进 neotrix 审计流（`audit_neobot_crystal.jsonl`），
/// 回灌 recall（recall 按 `audit_*.jsonl` glob 扫描尾部）。best-effort，永不挡返回。
fn write_back_experience(goal: &str, output: &str, model_used: &str, steps: usize) {
    fn snip_local(s: &str, n: usize) -> String {
        let t: String = s.chars().take(n).collect();
        if s.chars().count() > n {
            format!("{t}…")
        } else {
            t
        }
    }
    let home = std::env::var("HOME").unwrap_or_default();
    if home.is_empty() {
        return;
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let line = serde_json::json!({
        "ts": now,
        "source": "neobot-crystal",
        "goal": snip_local(goal.trim(), 300),
        "output": snip_local(output.trim(), 800),
        "model": model_used,
        "steps": steps,
    });
    let path = std::path::PathBuf::from(format!("{home}/.neotrix/audit_neobot_crystal.jsonl"));
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        use std::io::Write as _;
        let _written: Result<(), std::io::Error> =
            writeln!(file, "{}", serde_json::to_string(&line).unwrap_or_default());
    }
}

/// 服务端 agent 循环（确定性 neotrix 原生手序列，不调 LLM）：
/// skills（skill_registry search）→ memory（客户端 memory + neotrix recall 尝试）
/// → coordinator 只读规划 → web_search/web_fetch → workspace 读写/bash
/// （越狱拒绝进 trace）。单步失败只进 trace 不中断；`max_steps` 耗尽即停。
async fn run_agent_loop(
    goal: &str,
    context: &str,
    client_memory: &str,
    max_steps: usize,
    workspace: &PathBuf,
    cdp: &std::sync::Arc<
        std::sync::Mutex<
            Option<neotrix::l1_action::nt_io::nt_io_browser_engine::BrowserEngine>,
        >,
    >,
) -> (Vec<serde_json::Value>, String, usize) {
    let mut trace: Vec<serde_json::Value> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let mut used: usize = 0;

    // 1) skills：neotrix 原生 skill_registry（scan + search，只读）。
    if used < max_steps {
        used += 1;
        let registry = SkillRegistry::with_defaults();
        match registry.scan().await {
            Ok(scanned) => {
                let hits = registry
                    .search(&SkillQuery {
                        text: Some(goal.to_owned()),
                        scope: None,
                        tags: Vec::new(),
                        trigger: None,
                        limit: Some(5),
                    })
                    .await;
                let names: Vec<String> =
                    hits.iter().take(5).map(|s| s.name.clone()).collect();
                let detail = format!(
                    "skill_registry: scanned={scanned} hits={}",
                    if names.is_empty() {
                        "(none)".to_owned()
                    } else {
                        names.join(", ")
                    }
                );
                trace_push(&mut trace, "skill", detail.clone());
                notes.push(detail);
            }
            Err(e) => {
                let detail = format!("skill_registry unavailable: {e:?}");
                trace_push(&mut trace, "warn", detail.clone());
                notes.push(detail);
            }
        }
    }

    // 2) memory：客户端 memory 照单拼入；neotrix 侧 recall 尽力尝试，
    //    接不上就 features.memory_read=false 口径如实广播，绝不硬演。
    if !client_memory.trim().is_empty() {
        trace_push(
            &mut trace,
            "memory",
            format!(
                "client memory ({} chars): {}",
                client_memory.chars().count(),
                snip(client_memory.trim(), 500)
            ),
        );
    }
    match try_neotrix_memory_recall(goal) {
        Some(text) => {
            trace_push(
                &mut trace,
                "memory",
                format!("neotrix recall: {}", snip(&text, 800)),
            );
            notes.push(format!("neotrix recall: {}", snip(&text, 300)));
        }
        None => {
            let detail =
                "neotrix-side memory unavailable (features.memory_read=false); using client memory"
                    .to_owned();
            trace_push(&mut trace, "memory", detail.clone());
            notes.push(detail);
        }
    }

    // 3) coordinator 只读规划（create_plan，不 execute_plan）。
    if used < max_steps {
        used += 1;
        let coordinator = MultiAgentCoordinator::new(2);
        match coordinator
            .create_plan(goal, DecompositionStrategy::ByFeature)
            .await
        {
            Ok(plan) => {
                let descs: Vec<String> = plan
                    .subtasks
                    .iter()
                    .map(|s| s.description.clone())
                    .collect();
                let detail = format!(
                    "plan {}: {}",
                    plan.id,
                    if descs.is_empty() {
                        "(no subtasks)".to_owned()
                    } else {
                        descs.join(" | ")
                    }
                );
                trace_push(&mut trace, "plan", detail.clone());
                notes.push(detail);
            }
            Err(e) => {
                let detail = format!("coordinator unavailable: {e:?}");
                trace_push(&mut trace, "warn", detail.clone());
                notes.push(detail);
            }
        }
    }

    // 4) workspace 操作：context.ops 显式指令优先；无指令时 goal 兜底
    //    仅支持 `read <path>` / `ls [path]`（自然语言绝不自动跑 shell）。
    let mut ops = parse_context_ops(context);
    if ops.is_empty() {
        let g = goal.trim();
        if let Some(rest) = g
            .strip_prefix("read ")
            .or_else(|| g.strip_prefix("READ "))
        {
            if !rest.trim().is_empty() {
                ops.push(AgentOp::Read(rest.trim().to_owned()));
            }
        } else if g == "ls" || g.starts_with("ls ") {
            let p = g.strip_prefix("ls ").map(str::trim).unwrap_or("");
            ops.push(AgentOp::List(p.to_owned()));
        }
    }

    for op in ops {
        if used >= max_steps {
            trace_push(
                &mut trace,
                "warn",
                "max_steps reached, remaining ops skipped".to_owned(),
            );
            notes.push("max_steps reached, remaining ops skipped".to_owned());
            break;
        }
        used += 1;
        match op {
            AgentOp::Read(path) => match read_jailed(workspace, &path) {
                Ok(text) => {
                    let detail = format!(
                        "read {path} ({} chars): {}",
                        text.chars().count(),
                        snip(text.trim(), 2000)
                    );
                    trace_push(&mut trace, "read", detail.clone());
                    notes.push(detail);
                }
                Err(e) => {
                    trace_push(&mut trace, "error", format!("read {path}: {e}"));
                }
            },
            AgentOp::Write(path, content) => match write_jailed(workspace, &path, &content) {
                Ok(n) => {
                    let detail = format!("write {path} ({n} bytes ok)");
                    trace_push(&mut trace, "write", detail.clone());
                    notes.push(detail);
                }
                Err(e) => {
                    trace_push(&mut trace, "error", format!("write {path}: {e}"));
                }
            },
            AgentOp::Bash(cmd) => {
                let ws_text = workspace.to_string_lossy().into_owned();
                match bash_jail_ok(&ws_text, &cmd) {
                    Err(e) => {
                        trace_push(&mut trace, "error", format!("bash refused: {e}"));
                    }
                    Ok(()) => match run_bash(workspace, &cmd).await {
                        Ok(out) => {
                            let detail =
                                format!("bash [{cmd}]: {}", snip(out.trim(), 2000));
                            trace_push(&mut trace, "bash", detail.clone());
                            notes.push(detail);
                        }
                        Err(e) => {
                            trace_push(&mut trace, "error", format!("bash [{cmd}]: {e}"));
                        }
                    },
                }
            }
            AgentOp::List(path) => match list_jailed(workspace, &path) {
                Ok(names) => {
                    let label = if path.trim().is_empty() { "." } else { path.trim() };
                    let detail = format!("ls {label}: {}", snip(&names.join(", "), 2000));
                    trace_push(&mut trace, "read", detail.clone());
                    notes.push(detail);
                }
                Err(e) => {
                    trace_push(&mut trace, "error", format!("ls: {e}"));
                }
            },
            AgentOp::WebSearch(query, count) => {
                // 同步阻塞调用，spawn_blocking 出让 worker（COPY handle_reload 模式）。
                let q = query.clone();
                let n = count;
                match tokio::task::spawn_blocking(move || {
                    neotrix::l2_perception::nt_world::nt_world_search::WebSearchTool::new()
                        .search(&q, n)
                })
                .await
                {
                    Ok(Ok(text)) => {
                        let detail = format!("web_search [{query}]: {}", snip(text.trim(), 2000));
                        trace_push(&mut trace, "web", detail.clone());
                        notes.push(detail);
                    }
                    Ok(Err(e)) => {
                        trace_push(&mut trace, "error", format!("web_search [{query}]: {e}"));
                    }
                    Err(e) => {
                        trace_push(&mut trace, "error", format!("web_search join: {e}"));
                    }
                }
            }
            AgentOp::WebFetch(url) => {
                let target = url.trim().to_owned();
                // fail-closed：只允许 http/https。
                if !(target.starts_with("http://") || target.starts_with("https://")) {
                    trace_push(&mut trace, "error", format!("web_fetch refused (scheme): {target}"));
                } else {
                    // BrowserEngine 非 Sync（内部 Cell）：blocking 线程 + 独立
                    // current_thread runtime 内执行，不污染 worker（仿 run_browse 模式）。
                    match tokio::task::spawn_blocking(move || web_fetch_text_blocking(&target)).await
                    {
                        Ok(Ok(text)) => {
                            let detail =
                                format!("web_fetch [{url}]: {}", snip(text.trim(), 4000));
                            trace_push(&mut trace, "web", detail.clone());
                            notes.push(detail);
                        }
                        Ok(Err(e)) => {
                            trace_push(&mut trace, "error", format!("web_fetch [{url}]: {e}"));
                        }
                        Err(e) => {
                            trace_push(&mut trace, "error", format!("web_fetch join: {e}"));
                        }
                    }
                }
            }
            AgentOp::WebAct { url, action, selector, text } => {
                let u = url.clone();
                let a = action.clone();
                let s = selector.clone();
                let t = text.clone();
                let cdp_handle = cdp.clone();
                let u2 = u.clone();
                let a2 = a.clone();
                match tokio::task::spawn_blocking(move || {
                    web_act_blocking(&cdp_handle, &u, &a, &s, &t)
                })
                .await
                {
                    Ok(Ok(out)) => {
                        let detail = format!(
                            "web_act [{a2} {}]: {}",
                            if u2.trim().is_empty() { "-" } else { u2.trim() },
                            snip(out.trim(), 2000)
                        );
                        trace_push(&mut trace, "web", detail.clone());
                        notes.push(detail);
                    }
                    Ok(Err(e)) => {
                        trace_push(&mut trace, "error", format!("web_act [{a2}]: {e}"));
                    }
                    Err(e) => {
                        trace_push(&mut trace, "error", format!("web_act join: {e}"));
                    }
                }
            }
        }
    }

    let output = format!(
        "goal: {goal}\nworkspace: {}\nsteps: {used}/{max_steps}\n{}",
        workspace.display(),
        if notes.is_empty() {
            "(no actions)".to_owned()
        } else {
            notes.join("\n")
        }
    );
    (trace, output, used)
}

fn agent_error(
    code: u16,
    msg: &str,
) -> axum::response::Response {
    let status = StatusCode::from_u16(code).unwrap_or(StatusCode::BAD_REQUEST);
    (
        status,
        axum::Json(serde_json::json!({
            "status": "error",
            "output": msg,
            "trace": [{ "kind": "error", "detail": msg }],
            "model_used": CRYSTAL_MODEL,
        })),
    )
        .into_response()
}

async fn handle_agent_run(
    axum::extract::State(state): axum::extract::State<Arc<CrystalState>>,
    headers: HeaderMap,
    axum::Json(body): axum::Json<serde_json::Value>,
) -> axum::response::Response {
    if !auth_ok(&state, &headers) {
        return agent_error(401, "unauthorized");
    }
    let goal = body
        .get("goal")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_owned();
    if goal.is_empty() {
        return agent_error(400, "empty goal");
    }
    let max_steps = body
        .get("max_steps")
        .and_then(|v| v.as_u64())
        .map(|n| n.clamp(1, 32) as usize)
        .unwrap_or(8);
    let context = body.get("context").and_then(|v| v.as_str()).unwrap_or("");
    let client_memory = body
        .get("memory")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let ws_param = body
        .get("workspace")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let workspace = match resolve_workspace(ws_param) {
        Ok(p) => p,
        Err(e) => return agent_error(403, &e),
    };
    let (trace, output, used) = run_agent_loop(
        &goal,
        context,
        client_memory,
        max_steps,
        &workspace,
        &state.cdp,
    )
    .await;
    // 记忆写回（终态落审计流，回灌下轮 recall；best-effort）。
    write_back_experience(&goal, &output, CRYSTAL_MODEL, used);
    axum::Json(serde_json::json!({
        "status": "done",
        "output": output,
        "trace": trace,
        "model_used": CRYSTAL_MODEL,
    }))
    .into_response()
}

async fn handle_chat(
    axum::extract::State(state): axum::extract::State<Arc<CrystalState>>,
    headers: HeaderMap,
    axum::Json(body): axum::Json<serde_json::Value>,
) -> axum::response::Response {
    if !auth_ok(&state, &headers) {
        return unauthorized();
    }
    // 工具透传（完整能力调用的一半）：neobot 发来的 function schemas 原样进池，
    // 上游回的 tool_calls 原样返回，neobot 在本地网关下执行后再回环。
    // 历史 tool_calls/tool_call_id 同样回环（neotrix serialize 只认嵌套 function，
    // 此处双写扁平 + 嵌套，防名字丢失）。
    let tools: Vec<neotrix_types::llm_types::Tool> = body
        .get("tools")
        .and_then(|t| t.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|t| {
                    let f = t.get("function")?;
                    Some(neotrix_types::llm_types::Tool {
                        name: f.get("name")?.as_str()?.to_owned(),
                        description: f
                            .get("description")
                            .and_then(|d| d.as_str())
                            .unwrap_or("")
                            .to_owned(),
                        input_schema: f
                            .get("parameters")
                            .cloned()
                            .unwrap_or(serde_json::json!({})),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    let messages: Vec<Message> = body
        .get("messages")
        .and_then(|m| m.as_array())
        .map(|arr| {
            arr.iter()
                .map(|m| {
                    let role = m.get("role").and_then(|r| r.as_str()).unwrap_or("user");
                    let content = m
                        .get("content")
                        .and_then(|c| c.as_str())
                        .unwrap_or("")
                        .to_owned();
                    let mut msg = Message::new(role_of(role), content);
                    if let Some(calls) = m.get("tool_calls").and_then(|c| c.as_array()) {
                        msg.tool_calls = Some(
                            calls
                                .iter()
                                .map(|tc| {
                                    let id = tc
                                        .get("id")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_owned();
                                    let func = tc.get("function");
                                    let name = func
                                        .and_then(|f| f.get("name"))
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_owned();
                                    let arguments = func
                                        .and_then(|f| f.get("arguments"))
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("{}")
                                        .to_owned();
                                    neotrix_types::llm_types::ToolCallInfo {
                                        id,
                                        name: name.clone(),
                                        arguments: arguments.clone(),
                                        function: Some(
                                            neotrix_types::llm_types::ToolCallFunction {
                                                name,
                                                arguments,
                                            },
                                        ),
                                        call_type: Some("function".to_owned()),
                                    }
                                })
                                .collect(),
                        );
                    }
                    if let Some(tcid) = m.get("tool_call_id").and_then(|v| v.as_str()) {
                        msg.tool_call_id = Some(tcid.to_owned());
                    }
                    msg
                })
                .collect()
        })
        .unwrap_or_default();
    if messages.iter().all(|m| m.content.trim().is_empty()) {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": "empty messages" })),
        )
            .into_response();
    }
    // 自主选择：按健康顺序逐个改写 request.model 后直调（保证外发模型名正确）。
    // 读锁内只做快照，锁外 await，不阻塞 reload 写锁。
    let pool = state.snapshot_pool();
    let mut last_error = String::from("pool empty");
    let mut answered = None;
    for idx in state.order() {
        let Some((id, adapter)) = pool.get(idx) else {
            continue;
        };
        let request = LlmRequest {
            model: id.clone(),
            messages: messages.clone(),
            temperature: None,
            max_tokens: 4096,
            tools: tools.clone(),
            image_data: None,
            thinking_budget: None,
            provider_params: std::collections::HashMap::new(),
            constraint_json: None,
            structured_output: None,
            cacheable_prefix_tokens: None,
        };
        match adapter.complete(&request).await {
            Ok(r) => {
                state.record(id, true);
                answered = Some(r);
                break;
            }
            Err(e) => {
                last_error = e.to_string();
                state.record(id, false);
            }
        }
    }
    let response = match answered {
        Some(r) => r,
        None => {
            return (
                axum::http::StatusCode::BAD_GATEWAY,
                axum::Json(serde_json::json!({ "error": format!("crystal pool exhausted: {last_error}") })),
            )
                .into_response();
        }
    };
    // tool_calls 回传（扁平 name/arguments 优先，嵌套缺失回填，保证 neobot 可执行）。
    let out_calls: Vec<serde_json::Value> = response
        .tool_calls
        .as_ref()
        .map(|calls| {
            calls
                .iter()
                .map(|tc| {
                    let (name, arguments) = match &tc.function {
                        Some(f) if !f.name.is_empty() => (f.name.clone(), f.arguments.clone()),
                        _ => (tc.name.clone(), tc.arguments.clone()),
                    };
                    serde_json::json!({
                        "id": tc.id,
                        "type": "function",
                        "function": { "name": name, "arguments": arguments },
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    let finish_reason = if out_calls.is_empty() { "stop" } else { "tool_calls" };
    let stream = body
        .get("stream")
        .and_then(|s| s.as_bool())
        .unwrap_or(false);
    let usage = serde_json::json!({
        "prompt_tokens": response.usage.prompt_tokens,
        "completion_tokens": response.usage.completion_tokens,
        "total_tokens": response.usage.total_tokens,
    });
    if !stream {
        return axum::Json(serde_json::json!({
            "id": format!("crystal-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0)),
            "object": "chat.completion",
            "created": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0),
            "model": CRYSTAL_MODEL,
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": response.content,
                    "tool_calls": out_calls,
                },
                "finish_reason": finish_reason,
            }],
            "usage": usage,
        }))
        .into_response();
    }
    // SSE 真流式：逐块事件经 Body 流式发出（chunked），工具调用以独立
    // `tool_calls` delta 事件发出，最后 `data: [DONE]`。neobot 按 data: 行解析。
    let mut events: Vec<Result<String, std::convert::Infallible>> = Vec::new();
    let chars: Vec<char> = response.content.chars().collect();
    for chunk in chars.chunks(120) {
        let text: String = chunk.iter().collect();
        events.push(Ok(format!(
            "data: {}\n\n",
            serde_json::json!({
                "choices": [{ "delta": { "content": text }, "index": 0 }],
                "object": "chat.completion.chunk",
                "model": CRYSTAL_MODEL,
            })
        )));
    }
    if !out_calls.is_empty() {
        let indexed: Vec<serde_json::Value> = out_calls
            .into_iter()
            .enumerate()
            .map(|(i, mut tc)| {
                tc["index"] = serde_json::json!(i);
                tc
            })
            .collect();
        events.push(Ok(format!(
            "data: {}\n\n",
            serde_json::json!({
                "choices": [{ "delta": { "tool_calls": indexed }, "index": 0 }],
                "object": "chat.completion.chunk",
                "model": CRYSTAL_MODEL,
            })
        )));
        let _ = finish_reason;
    }
    events.push(Ok("data: [DONE]\n\n".to_owned()));
    (
        [
            (
                axum::http::header::CONTENT_TYPE,
                "text/event-stream",
            ),
            (axum::http::header::CACHE_CONTROL, "no-cache"),
        ],
        axum::body::Body::from_stream(futures::stream::iter(events)),
    )
        .into_response()
}

fn real_main() -> Result<(), String> {
    let file_cfg = load_file();
    let (file_port, discover_limit, file_upstreams, file_token) = match &file_cfg {
        Some(c) => (c.port, c.discover_limit, c.upstreams.clone(), c.token.clone()),
        None => (None, 5, Vec::new(), String::new()),
    };
    // token：配置文件优先，空时 `CRYSTAL_TOKEN` 环境变量兜底；仍空 = legacy 开放。
    let token = if file_token.trim().is_empty() {
        env_or("CRYSTAL_TOKEN", "")
    } else {
        file_token
    };
    if token.trim().is_empty() {
        eprintln!(
            "nt_crystal_serve: WARNING no token configured (legacy open mode); \
             set `token` in crystal.toml to require Authorization: Bearer"
        );
    }
    let pool = build_pool(&file_upstreams, discover_limit)?;
    // 端口：环境优先，配置文件次之，缺省 3000。
    let port: u16 = match std::env::var("CRYSTAL_PORT") {
        Ok(v) if !v.trim().is_empty() => v
            .trim()
            .parse()
            .map_err(|_| "CRYSTAL_PORT must be a number".to_owned())?,
        _ => file_port.unwrap_or(3000),
    };
    let state = Arc::new(CrystalState {
        inner: RwLock::new(CrystalInner {
            models: pool,
            token,
            discover_limit,
            file_upstreams,
        }),
        health: Mutex::new(HashMap::new()),
        cdp: std::sync::Arc::new(std::sync::Mutex::new(None)),
    });
    let app = axum::Router::new()
        .route("/v1/models", axum::routing::get(handle_models))
        .route("/v1/chat/completions", axum::routing::post(handle_chat))
        .route("/v1/capabilities", axum::routing::get(handle_capabilities))
        .route("/v1/agents/run", axum::routing::post(handle_agent_run))
        .route("/v1/admin/reload", axum::routing::post(handle_reload))
        .route("/healthz", axum::routing::get(handle_health))
        .with_state(state);
    eprintln!("nt_crystal_serve: single model `{CRYSTAL_MODEL}` v{CRYSTAL_VERSION} on 127.0.0.1:{port}");
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("runtime: {e}"))?;
    runtime.block_on(async move {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| format!("bind {addr}: {e}"))?;
        axum::serve(listener, app)
            .await
            .map_err(|e| format!("serve: {e}"))
    })
}

fn main() -> ExitCode {
    match real_main() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("nt_crystal_serve: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_context_ops, recall_keywords, resolve_workspace, snip, AgentOp};

    #[test]
    fn snip_truncates_by_chars() {
        assert_eq!(snip("abcdef", 10), "abcdef");
        assert_eq!(snip("abcdef", 4), "abcd…");
        assert_eq!(snip("甲乙丙丁", 2), "甲乙…");
    }

    #[test]
    fn recall_keywords_splits_mixed() {
        let got = recall_keywords("GraphRAG记忆修复");
        assert!(got.contains(&"GraphRAG".to_owned()), "{got:?}");
        assert!(got.contains(&"记忆".to_owned()), "{got:?}");
        assert!(got.contains(&"修复".to_owned()), "{got:?}");
        assert!(recall_keywords("，。？！").is_empty());
        assert!(recall_keywords("a").is_empty());
    }

    #[test]
    fn context_ops_parse_all_shapes() {
        let ops = parse_context_ops(
            r#"{"ops": [{"op":"read","path":"a.txt"},{"op":"exec","cmd":"ls"},{"op":"web_search","query":"rust","count":3},{"op":"web_act","url":"https://example.com","action":"click","selector":"a"},{"op":"drop table"}]}"#,
        );
        assert_eq!(ops.len(), 4);
        assert!(matches!(ops[0], AgentOp::Read(_)));
        assert!(matches!(ops[1], AgentOp::Bash(_)));
        assert!(matches!(ops[2], AgentOp::WebSearch(_, 3)));
        assert!(matches!(ops[3], AgentOp::WebAct { .. }));
        assert!(parse_context_ops("not json").is_empty());
    }

    #[test]
    fn workspace_jail_blocks_escape() {
        assert!(resolve_workspace("../../etc").is_err());
        assert!(resolve_workspace("").is_ok());
    }
}
