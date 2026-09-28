//! `nt_cmd_llamacpp` — 本地推理（llama.cpp）控制面：探活 / 列模型 / 启 / 停 / 换模型。
//!
//! ## 为什么不自己拼命令行
//!
//! 归档掉的 `src-tauri`（NeoTrix 桌面端）当初在这件事上栽过：它为了不依赖
//! `neotrix-core`（214 个依赖）**自己复制了一份进程启动**（`llamacpp.rs:806`），
//! 那份副本缺 `--jinja` 且 `--ctx-size` 写死 4096 —— 直接后果是 Qwen3.5 系
//! "装完开不了话"（工具调用 XML 与 `<think>` 块以纯文本漏出；thinking 模式在
//! 4K 上下文下已废）。
//!
//! 复制实现是 bug 的来源，不是解法。所以本模块**不持有任何启动逻辑**，只做
//! IPC 转发，全部委托给 `neotrix_neobot::nt_llama` —— 那是 CLI 与本 App 共用的
//! 唯一实现（2026-09-28 由 `neotrix-core` 下沉而来，`neotrix-core` 侧改为
//! re-export，两边不会再分叉）。
//!
//! ## 命令面
//!
//! | 命令 | 语义 |
//! |---|---|
//! | `neobot_llamacpp_health` | 端点是否在服务（不启进程，纯探活） |
//! | `neobot_llamacpp_models` | 扫盘发现的 GGUF + 当前在跑的模型 |
//! | `neobot_llamacpp_start` | 按最优参数启动（可指定模型） |
//! | `neobot_llamacpp_stop` | 停掉本 App 拉起的进程 |
//! | `neobot_llamacpp_swap` | 停 → 换模型 → 重启（原子语义：不会半启动） |

use std::path::PathBuf;

use neotrix_neobot::nt_llama::{
    compute_optimal_config, find_executable, global_manager, llamacpp_base_url, llamacpp_port,
    model_search_dirs, scan_models, GgufModel, HardwareProfile, LlamaProcessManager,
    ReasoningMode,
};
use serde::Serialize;

/// 端点探活结果。
///
/// `model` 用 `Option` 而不是空串：空串既可能是"服务着但没有模型"，也可能是
/// 调用方没填，前端无法区分就会渲染出 "当前模型：无" 这种假信息。
#[derive(Debug, Serialize)]
pub struct NeobotLlamaHealth {
    pub reachable: bool,
    pub base_url: String,
    pub port: u16,
    pub model: Option<String>,
    pub ctx_size: u32,
    pub reasoning: String,
}

#[derive(Debug, Serialize)]
pub struct NeobotLlamaModel {
    pub name: String,
    pub path: String,
    pub size_gb: f64,
    /// `None` = 文件名里解析不出参数量（如 `mmproj-*`），不是 0。
    pub param_b: Option<u32>,
    pub selected: bool,
}

#[derive(Debug, Serialize)]
pub struct NeobotLlamaState {
    pub running: bool,
    pub base_url: String,
    pub port: u16,
    pub ctx_size: u32,
    pub model: Option<String>,
    pub models: Vec<NeobotLlamaModel>,
    pub search_dirs: Vec<String>,
}

fn gguf_to_item(m: &GgufModel, selected: Option<&str>) -> NeobotLlamaModel {
    NeobotLlamaModel {
        name: m.name.clone(),
        path: m.path.to_string_lossy().to_string(),
        size_gb: (m.size_gb * 100.0).round() / 100.0,
        param_b: m.param_count,
        selected: selected.is_some_and(|s| s == m.name),
    }
}

fn reasoning_label(mode: ReasoningMode) -> String {
    match mode {
        ReasoningMode::Auto => "auto".into(),
        ReasoningMode::On => "on".into(),
        ReasoningMode::Off => "off".into(),
    }
}

/// 端点在服务吗？**不启进程** —— 只想知道状态时用这个。
#[tauri::command]
pub async fn neobot_llamacpp_health() -> Result<NeobotLlamaHealth, String> {
    let port = llamacpp_port_or_default();
    let base_url = llama_base_url_or_default();
    let reachable = llama_is_port_in_use(port).await;

    // 只有端点真的在服务时才去问它当前跑的是哪个模型 —— 探活失败时读
    // /v1/models 只会多一次必然失败的连接。
    let model = if reachable { global_manager().current_model().await } else { None };

    let cfg = global_manager().current_config().await;
    Ok(NeobotLlamaHealth {
        reachable,
        base_url,
        port,
        model,
        ctx_size: cfg.ctx_size,
        reasoning: reasoning_label(cfg.reasoning),
    })
}

/// 扫盘发现本地 GGUF，并标出当前在跑的是哪个。
#[tauri::command]
pub async fn neobot_llamacpp_models() -> Result<Vec<NeobotLlamaModel>, String> {
    let port = llamacpp_port_or_default();
    let selected = if llama_is_port_in_use(port).await {
        global_manager().current_model().await
    } else {
        None
    };
    let selected_ref = selected.as_deref();
    Ok(scan_models().iter().map(|m| gguf_to_item(m, selected_ref)).collect())
}

/// 当前完整状态：端点 + 在跑模型 + 盘上模型 + 搜索路径。
///
/// 一次调用给全，避免前端为了画一个面板发三个 IPC。
#[tauri::command]
pub async fn neobot_llamacpp_state() -> Result<NeobotLlamaState, String> {
    let port = llamacpp_port_or_default();
    let base_url = llama_base_url_or_default();
    let running = llama_is_port_in_use(port).await;
    let model = if running { global_manager().current_model().await } else { None };

    let cfg = global_manager().current_config().await;
    let selected_ref = model.as_deref();
    let models: Vec<NeobotLlamaModel> =
        scan_models().iter().map(|m| gguf_to_item(m, selected_ref)).collect();

    Ok(NeobotLlamaState {
        running,
        base_url,
        port,
        ctx_size: cfg.ctx_size,
        model,
        models,
        search_dirs: model_search_dirs()
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect(),
    })
}

/// 启动本地推理。
///
/// `model` 为 `None` 时用 `select_best_model()`（盘上最大的那个）—— 这与
/// `neotrix-core` 侧 `auto_start()` 的默认行为一致，所以两个入口不会启动出
/// 不同的模型。
///
/// 已经是 `Ok(())`（幂等）：端点在跑就直接返回，不重复起进程。
#[tauri::command]
pub async fn neobot_llamacpp_start(model: Option<String>) -> Result<String, String> {
    let port = llamacpp_port_or_default();
    if llama_is_port_in_use(port).await {
        return Ok(format!("已在运行 (port {port})"));
    }
    if find_executable().is_none() {
        return Err("llama-server 不在 PATH 里。装一个: brew install llama.cpp".into());
    }

    if let Some(name) = model {
        let picked = scan_models()
            .into_iter()
            .find(|m| m.name == name)
            .ok_or_else(|| format!("盘上找不到模型 {name}。先调 neobot_llamacpp_models 看可选项"))?;
        let hw = HardwareProfile::detect();
        let cfg = compute_optimal_config(&picked.path, &hw);
        let mgr = global_manager();
        // 换 config 再起：auto_start 内部会自己选盘上最大的模型，直接调它
        // 会忽略用户刚指定的这个。
        mgr.set_config(cfg).await;
    }

    global_manager().auto_start().await?;
    Ok(format!("已启动 (port {port})"))
}

/// 停掉本 App 拉起的进程。
#[tauri::command]
pub async fn neobot_llamacpp_stop() -> Result<String, String> {
    global_manager().stop().await;
    Ok("已停止".into())
}

/// 换模型：停 → 重启。
///
/// **先停再启**是刻意的：同名端口上两个 llama-server 会抢 KV cache，
/// 第二个起来时 prefill 会随机失败（`Failed to parse input at pos N` 那种
/// 难查的现象）。宁可中间有几秒不可用，也不要两个进程并存。
#[tauri::command]
pub async fn neobot_llamacpp_swap(model: String) -> Result<String, String> {
    let target = scan_models()
        .into_iter()
        .find(|m| m.name == model)
        .ok_or_else(|| format!("盘上找不到模型 {model}"))?;
    let path: PathBuf = target.path.clone();

    global_manager().stop().await;
    let hw = HardwareProfile::detect();
    let cfg = compute_optimal_config(&path, &hw);
    global_manager().set_config(cfg).await;
    global_manager().auto_start().await?;
    Ok(format!("已切到 {model}"))
}

// ── 内部小工具 ────────────────────────────────────────────────
//
// 端点常量与 HTTP 探测**全部**走 `nt_llama` 的单一真源。这里只做"拿不到就
// 兜底"的包装，让命令保持统一的 `-> Result<_, String>` 签名。
// 刻意不在 App 层发 HTTP：那会把"怎么问端点"复制一份，而端点的
// id→文件名归一规则只有一处是对的（剥路径 + 剥 .gguf）。

fn llamacpp_port_or_default() -> u16 {
    llamacpp_port()
}

fn llama_base_url_or_default() -> String {
    llamacpp_base_url()
}

async fn llama_is_port_in_use(port: u16) -> bool {
    LlamaProcessManager::is_port_in_use(port).await
}
