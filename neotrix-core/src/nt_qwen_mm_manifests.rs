//! Qwen-MM-Plugins 能力 manifest —— 把外部多模态能力注册进 NeoTrix。
//!
//! 吸收源: QwenLM/Qwen-MM-Plugins (Apache-2.0，2026-09-28 main)。
//! 工具名/参数形状逐项核对过其源码（`TOOL` 声明与 Pydantic `Args` 模型）；
//! 描述为转述（非逐字复制），出处见每工具注释。版本钉死
//! `plugin-versions.json`（core/search 均为 `1.1.0`）。
//!
//! 设计（R-P79 同 session 消费者）：
//! - manifest 是**数据**（`McpListedTool`），执行走会话式 stdio
//!  （`register_stdio_session` + `McpSessionTool`，见 `nt_mcp_stdio_session.rs`）。
//! - 启动探测**不 spawn**：只查 PATH 与目录存在性 + 环境变量，离线可用。
//!   探测不到时不注册、返回可操作的 `InstallHint`（fail-closed：不支持≠已列出）。
//! - search 系工具为 credential-gated：无 key 时不注册，只报告缺哪个 key。
//!   key 名只做存在性检查，值永不进日志（见 `search_credential_status`）。
//!
//! 对应 Qwen 侧文件（核对依据，勿删——供下轮升级 manifest 时复核）：
//! - core 7 工具：`src/capabilities/core/qwen_mm_plugins_core/`
//!   `readers/{image,media_info,video}.py` · `visualizers/visualize.py` ·
//!   `producers/{crop,draw_bbox,save_view}.py`
//! - search 3 工具：`src/capabilities/search/qwen_mm_plugins_search/tools/`
//!   `{web_search,web_extractor,image_search}.py`
//! - 启动约定：`src/capabilities/*/.mcp.json`（`uvx --from ...@tag`）
//! - 系统依赖表：`.../core/__init__.py:SYSTEM_DEPS`

use std::path::PathBuf;

use crate::agent::tool::mcp::{McpToolDef, McpTransport, RiskLevel};
use crate::agent::tool::{register_stdio_session_global, McpRegistry};
use crate::nt_mcp_stdio_session::{McpListedTool, DEFAULT_TIMEOUT_MS};

// ---------------------------------------------------------------------------
// 版本与命名（钉死上游 `plugin-versions.json` + `.mcp.json` 约定）
// ---------------------------------------------------------------------------

/// core 能力：插件名 / 版本 / tag。
pub const QWEN_MM_CORE_PLUGIN: &str = "qwen-mm-plugins-core";
pub const QWEN_MM_CORE_VERSION: &str = "1.1.0";

/// search 能力：插件名 / 版本 / tag。
pub const QWEN_MM_SEARCH_PLUGIN: &str = "qwen-mm-plugins-search";
pub const QWEN_MM_SEARCH_VERSION: &str = "1.1.0";

/// 上游 tag 格式：`qwen-mm-plugins-{cap}-v{version}`。
pub fn release_tag(capability: &str, version: &str) -> String {
    format!("qwen-mm-plugins-{capability}-v{version}")
}

/// core 工具默认超时：帧渲染类有界但慢，90s（kill 开关仍在）。
pub const CORE_TIMEOUT_MS: u64 = 90_000;

/// search 工具超时：网络调用服务端自带超时，这里只做上限。
pub const SEARCH_TIMEOUT_MS: u64 = DEFAULT_TIMEOUT_MS;

/// 源码 checkout 定位环境变量（sparse-checkout 只需
/// `src/capabilities/<cap>` + `src/shared` + `src/mcp_framework.py`）。
pub const QWEN_MM_CHECKOUT_ENV: &str = "QWEN_MM_PLUGINS_CHECKOUT";

// ---------------------------------------------------------------------------
// 启动探测
// ---------------------------------------------------------------------------

/// 启动方式（探测结论，按优先级排序）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchVia {
    /// `uvx --from "qwen-mm-plugins[<extra>] @ git+...@<tag>" <entry>`（官方方式）。
    Uvx,
    /// PATH 上的已安装入口（`qwen-mm-plugins-core`）。
    Path,
    /// 源码 checkout：`python3 <pkgdir>`（`__main__.py` 自带 sys.path 注入，
    /// 零 PYTHONPATH 配置）。
    SourceCheckout,
}

/// 探测到的启动方式。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QwenLaunch {
    pub server_name: String,
    pub command: String,
    pub args: Vec<String>,
    pub via: LaunchVia,
}

/// 探测失败：带可操作的安装指引（不注册、不断链）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QwenResolveError {
    pub server: String,
    pub reason: String,
    /// 用户照做即生产（精确命令）。
    pub hint: String,
}

impl std::fmt::Display for QwenResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} unavailable: {}. {}",
            self.server, self.reason, self.hint
        )
    }
}

impl std::error::Error for QwenResolveError {}

/// PATH 查找（只判存在性，不 spawn；可执行位不校验——家用机够用，误报由
/// spawn 失败转为 `Spawn` 错误兜底）。
fn path_lookup(name: &str) -> Option<PathBuf> {
    path_lookup_in(name, None)
}

/// 可注入的环境 —— 生产走真实环境，测试走隔离值，**零全局变更**。
///
/// 背景：直接改 `PATH` 跑测试会和同进程并行测试（12k）竞态
/// （`Command::new("bash")` 靠 PATH 解析）。本结构让探测逻辑可测，
/// 又不碰进程全局状态。
#[derive(Debug, Clone, Default)]
pub struct ResolveEnv {
    /// `None` = 读真实 `PATH`；`Some(dirs)` = 只在这些目录里找。
    pub path_dirs: Option<Vec<PathBuf>>,
    /// `None` = 读真实 `QWEN_MM_PLUGINS_CHECKOUT`；`Some(x)` = 覆盖
    /// （`Some(None)` = 视为未设）。
    pub checkout: Option<Option<PathBuf>>,
}

impl ResolveEnv {
    /// 生产环境（读真实进程环境）。
    pub fn live() -> Self {
        Self::default()
    }
}

fn path_lookup_in(name: &str, dirs: Option<&[PathBuf]>) -> Option<PathBuf> {
    let owned: Vec<PathBuf>;
    let search_dirs: &[PathBuf] = match dirs {
        Some(d) => d,
        None => {
            let path_var = std::env::var_os("PATH")?;
            owned = std::env::split_paths(&path_var).collect();
            &owned
        }
    };
    for dir in search_dirs {
        if dir.as_os_str().is_empty() {
            continue;
        }
        let cand = dir.join(name);
        if cand.is_file() {
            return Some(cand);
        }
    }
    None
}

/// 通用探测：`plugin`（如 `qwen-mm-plugins-core`）+ `extra`（如 `core`）。
fn resolve_launch(
    plugin: &str,
    extra: &str,
    version: &str,
) -> Result<QwenLaunch, QwenResolveError> {
    resolve_launch_with(plugin, extra, version, &ResolveEnv::live())
}

/// 通用探测（环境可注入，供测试隔离——见 `ResolveEnv`）。
fn resolve_launch_with(
    plugin: &str,
    extra: &str,
    version: &str,
    env: &ResolveEnv,
) -> Result<QwenLaunch, QwenResolveError> {
    let lookup = |name: &str| path_lookup_in(name, env.path_dirs.as_deref());
    let checkout_dir: Option<PathBuf> = match &env.checkout {
        None => std::env::var_os(QWEN_MM_CHECKOUT_ENV).map(PathBuf::from),
        Some(o) => o.clone(),
    };
    let capability = plugin.strip_prefix("qwen-mm-plugins-").unwrap_or(plugin);
    let install_hint = format!(
        "install: uvx --from \"qwen-mm-plugins[{extra}] @ git+https://github.com/QwenLM/Qwen-MM-Plugins.git@{} \" {plugin} \
         | or sparse-checkout the repo and set {QWEN_MM_CHECKOUT_ENV}=<dir>",
        release_tag(capability, version),
    );

    // 1) 官方方式：uvx。
    if lookup("uvx").is_some() {
        return Ok(QwenLaunch {
            server_name: plugin.to_string(),
            command: "uvx".to_string(),
            args: vec![
                "--from".to_string(),
                format!(
                    "qwen-mm-plugins[{extra}] @ git+https://github.com/QwenLM/Qwen-MM-Plugins.git@{}",
                    release_tag(capability, version)
                ),
                plugin.to_string(),
            ],
            via: LaunchVia::Uvx,
        });
    }
    // 2) PATH 上已有入口。
    if lookup(plugin).is_some() {
        return Ok(QwenLaunch {
            server_name: plugin.to_string(),
            command: plugin.to_string(),
            args: Vec::new(),
            via: LaunchVia::Path,
        });
    }
    // 3) 源码 checkout（`__main__.py` 自举 sys.path，直接 `python3 <pkgdir>`）。
    if let Some(dir) = checkout_dir {
        let pkg = dir
            .join("src")
            .join("capabilities")
            .join(capability)
            .join(plugin.replace('-', "_"));
        if pkg.join("__main__.py").is_file() {
            let python = lookup("python3")
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "python3".to_string());
            return Ok(QwenLaunch {
                server_name: plugin.to_string(),
                command: python,
                args: vec![pkg.to_string_lossy().to_string()],
                via: LaunchVia::SourceCheckout,
            });
        }
    }
    Err(QwenResolveError {
        server: plugin.to_string(),
        reason: "no uvx, no PATH entry, and no source checkout".to_string(),
        hint: install_hint,
    })
}

/// 探测 core 服务器启动方式。
pub fn resolve_core_launch() -> Result<QwenLaunch, QwenResolveError> {
    resolve_launch(QWEN_MM_CORE_PLUGIN, "core", QWEN_MM_CORE_VERSION)
}

/// 探测 search 服务器启动方式。
pub fn resolve_search_launch() -> Result<QwenLaunch, QwenResolveError> {
    resolve_launch(QWEN_MM_SEARCH_PLUGIN, "search", QWEN_MM_SEARCH_VERSION)
}

// ---------------------------------------------------------------------------
// 系统依赖探测（对 `SYSTEM_DEPS` 表的诚实投影：只报二进制存在性）
// ---------------------------------------------------------------------------

/// 单个系统依赖的探测结论。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemDep {
    pub label: String,
    pub present: bool,
}

/// core 能力需要的外部二进制（`__init__.py:SYSTEM_DEPS` 子集——只含
/// "装即有、无需 pip" 的项；pip 包的存在性不探，避免 spawn python）。
pub fn core_system_dep_status() -> Vec<SystemDep> {
    let probe = |label: &str, bins: &[&str]| SystemDep {
        label: label.to_string(),
        // 任一候选存在即算满足（soffice 是 libreoffice 的别名式存在）。
        present: bins.iter().any(|b| path_lookup(b).is_some()),
    };
    vec![
        probe(
            "read_video / media_info (video & audio)",
            &["ffmpeg", "ffprobe"],
        ),
        probe(
            "visualize: Office / DrawIO (LibreOffice)",
            &["libreoffice", "soffice"],
        ),
        probe("visualize: LaTeX (.tex)", &["pdflatex"]),
        probe("visualize: 3D best-quality render (Blender)", &["blender"]),
    ]
}

/// search 后端 key 存在性（只返回"有/无"，值永不外露）。
pub fn search_credential_status() -> Vec<(String, bool)> {
    [
        "SERPER_API_KEY",
        "TAVILY_API_KEY",
        "EXA_API_KEY",
        "SERPLY_API_KEY",
    ]
    .iter()
    .map(|k| {
        let present = std::env::var_os(k).is_some_and(|v| !v.is_empty());
        (k.to_string(), present)
    })
    .collect()
}

// ---------------------------------------------------------------------------
// Manifest：core 7 工具（转述自源码 docstring + Pydantic 字段，见模块头）
// ---------------------------------------------------------------------------

fn tool(name: &str, description: &str, schema: serde_json::Value) -> McpListedTool {
    McpListedTool {
        name: name.to_string(),
        description: description.to_string(),
        input_schema: schema,
    }
}

fn budget_prop(default: &str) -> serde_json::Value {
    serde_json::json!({
        "type": "string",
        "enum": ["small", "normal", "large"],
        "default": default,
    })
}

/// core 7 工具 manifest（来源：`qwen_mm_plugins_core/{readers,visualizers,producers}`）。
pub fn qwen_mm_core_tools() -> Vec<McpListedTool> {
    vec![
        tool(
            "read_image",
            // 出处: readers/image.py handle docstring + ReadImageArgs.
            "Read an image at model-optimized dynamic resolution (small~512, normal~1024, large~1448). Returns a resized image plus a dimension summary.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "image_path": {"type": "string"},
                    "budget": budget_prop("normal"),
                },
                "required": ["image_path"],
            }),
        ),
        tool(
            "media_info",
            // 出处: readers/media_info.py handle docstring + MediaInfoArgs.
            "Read full metadata of a video/audio file via ffprobe (container, duration, streams, fps, rotation, VFR flag). Header-only: fast even on huge files. Run BEFORE read_video and before any clip/edit.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "raw": {"type": "boolean", "default": false},
                },
                "required": ["path"],
            }),
        ),
        tool(
            "read_video",
            // 出处: readers/video.py handle docstring + ReadVideoArgs.
            "Extract video frames with dynamic resolution and FPS (fps=0 auto-selects by duration). Run media_info first for codec/fps/rotation/VFR. Use start_time/end_time to window.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "video_path": {"type": "string"},
                    "fps": {"type": "number", "default": 0},
                    "max_frames": {"type": "integer"},
                    "budget": budget_prop("normal"),
                    "start_time": {"type": ["number", "string"]},
                    "end_time": {"type": ["number", "string"]},
                },
                "required": ["video_path"],
            }),
        ),
        tool(
            "visualize",
            // 出处: visualizers/visualize.py handle docstring + VisualizeArgs.
            "Render any supported file for model inspection: PDF/SVG pages, Office docs (needs LibreOffice), CSV/XLSX tables+charts, syntax-highlighted code, DrawIO, SRT/VTT subtitles, NIfTI volumes (local, read-only, not for diagnosis), GIS, notebooks, LaTeX. Auto-detects type.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {"type": "string"},
                    "pages": {"type": "string"},
                    "budget": budget_prop("large"),
                    "max_pages": {"type": "integer", "default": 20},
                },
                "required": ["file_path"],
            }),
        ),
        tool(
            "save_view",
            // 出处: producers/save_view.py handle docstring + SaveViewArgs.
            "Materialize document pages (pages='1-5') or video frames (times=[seconds]) as standalone image FILES and return their paths. Feed the paths to crop/draw_bbox/grounding/ocr. Writes files.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {"type": "string"},
                    "pages": {"type": "string"},
                    "times": {"type": "array"},
                    "dpi": {"type": "integer"},
                    "budget": budget_prop("normal"),
                    "output_dir": {"type": "string"},
                },
                "required": ["file_path"],
            }),
        ),
        tool(
            "crop",
            // 出处: producers/crop.py handle docstring + CropArgs.
            "Crop a rectangle from an image (0-1000 normalized coordinates, same as grounding output). Saves to disk and returns a preview. Writes files.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "image_path": {"type": "string"},
                    "box": {"type": "array", "minItems": 4, "maxItems": 4},
                    "output_path": {"type": "string"},
                },
                "required": ["image_path", "box"],
            }),
        ),
        tool(
            "draw_bbox",
            // 出处: producers/draw_bbox.py handle docstring + DrawBboxArgs.
            "Draw bounding boxes on an image (0-1000 normalized coordinates). Saves the annotated result to disk and returns a preview. Writes files.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "image_path": {"type": "string"},
                    "bboxes": {"type": "array"},
                    "output_path": {"type": "string"},
                },
                "required": ["image_path", "bboxes"],
            }),
        ),
    ]
}

/// search 3 工具 manifest（来源：`qwen_mm_plugins_search/tools/*.py`）。
///
/// 后端选择走环境变量（`QWEN_MM_SEARCH_BACKEND` 未设时按
/// SERPER→TAVILY→EXA→SERPLY 顺序取第一个有 key 的；`image_search` 恒用
/// Serper Lens）。`image_search` 默认不公开上传（`allow_public_upload=false`）。
pub fn qwen_mm_search_tools() -> Vec<McpListedTool> {
    vec![
        tool(
            "web_search",
            // 出处: tools/web_search.py WebSearchArgs.
            "Search the web for facts (multi-query). Backend auto-selected from configured API keys; pin via QWEN_MM_SEARCH_BACKEND. Confirm-before-commit: never answer from appearance alone.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "queries": {"type": "array", "minItems": 1},
                    "api_key": {"type": "string"},
                },
                "required": ["queries"],
            }),
        ),
        tool(
            "web_extractor",
            // 出处: tools/web_extractor.py WebExtractorArgs.
            "Read web page(s) in depth toward a stated goal.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "urls": {"type": "array", "minItems": 1},
                    "goal": {"type": "string"},
                    "api_key": {"type": "string"},
                },
                "required": ["urls", "goal"],
            }),
        ),
        tool(
            "image_search",
            // 出处: tools/image_search.py ImageSearchArgs.
            "Reverse-image search a local frame/photo to identify an entity (Serper Lens). Optional bbox uses 0-1000 coordinates after EXIF orientation, matching read_image/crop. Private by default: needs allow_public_upload=true to upload.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "image_path": {"type": "string"},
                    "bbox": {"type": "array", "minItems": 4, "maxItems": 4},
                    "api_key": {"type": "string"},
                    "allow_public_upload": {"type": "boolean", "default": false},
                },
                "required": ["image_path"],
            }),
        ),
    ]
}

// ---------------------------------------------------------------------------
// 注册胶水（fail-closed：探测不到就不注册，只报告）
// ---------------------------------------------------------------------------

/// 单能力注册报告。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QwenRegisterReport {
    pub server: String,
    pub registered_tools: usize,
    /// `None` = 已注册；`Some` = 未注册的原因 + 指引。
    pub skipped: Option<QwenResolveError>,
}

/// manifest → `McpToolDef` 列表（transport 记 Local 启动方式，供状态展示；
/// 写文件的工具标 Medium，其余只读标 Low —— 家规"高危需显式标注"的落地）。
fn to_defs(
    launch: &QwenLaunch,
    tools: Vec<McpListedTool>,
    file_writing: &[&str],
) -> Vec<McpToolDef> {
    tools
        .into_iter()
        .map(|t| {
            let risk_level = if file_writing.contains(&t.name.as_str()) {
                RiskLevel::Medium
            } else {
                RiskLevel::Low
            };
            McpToolDef {
                name: t.name,
                description: t.description,
                input_schema: t.input_schema,
                transport: McpTransport::Local {
                    command: launch.command.clone(),
                    args: launch.args.clone(),
                },
                server_name: launch.server_name.clone(),
                schema_version: None,
                required_permission: None,
                risk_level,
            }
        })
        .collect()
}

/// 把 manifest 转成 `McpToolDef` 并走会话式注册进 `reg`，返回工具数。
fn register_launch(
    reg: &mut McpRegistry,
    launch: &QwenLaunch,
    timeout_ms: u64,
    tools: Vec<McpListedTool>,
    file_writing: &[&str],
) -> usize {
    let defs = to_defs(launch, tools, file_writing);
    let n = defs.len();
    let args: Vec<&str> = launch.args.iter().map(|s| s.as_str()).collect();
    reg.register_stdio_session(
        &launch.server_name,
        &launch.command,
        &args,
        timeout_ms,
        defs,
    );
    n
}

/// 注册 core（需启动探测通过）。
pub fn register_qwen_mm_core(reg: &mut McpRegistry) -> QwenRegisterReport {
    match resolve_core_launch() {
        Ok(launch) => {
            let n = register_launch(
                reg,
                &launch,
                CORE_TIMEOUT_MS,
                qwen_mm_core_tools(),
                &["save_view", "crop", "draw_bbox"],
            );
            QwenRegisterReport {
                server: launch.server_name,
                registered_tools: n,
                skipped: None,
            }
        }
        Err(e) => QwenRegisterReport {
            server: QWEN_MM_CORE_PLUGIN.to_string(),
            registered_tools: 0,
            skipped: Some(e),
        },
    }
}

/// 注册 search（需启动探测通过 **且** 至少一个后端 key 存在）。
pub fn register_qwen_mm_search(reg: &mut McpRegistry) -> QwenRegisterReport {
    let creds = search_credential_status();
    if !creds.iter().any(|(_, present)| *present) {
        return QwenRegisterReport {
            server: QWEN_MM_SEARCH_PLUGIN.to_string(),
            registered_tools: 0,
            skipped: Some(QwenResolveError {
                server: QWEN_MM_SEARCH_PLUGIN.to_string(),
                reason: "no search backend key (need one of SERPER_API_KEY, TAVILY_API_KEY, EXA_API_KEY, SERPLY_API_KEY)".to_string(),
                hint: "export SERPER_API_KEY=... (or TAVILY/EXA/SERPLY) and retry".to_string(),
            }),
        };
    }
    match resolve_search_launch() {
        Ok(launch) => {
            let n = register_launch(reg, &launch, SEARCH_TIMEOUT_MS, qwen_mm_search_tools(), &[]);
            QwenRegisterReport {
                server: launch.server_name,
                registered_tools: n,
                skipped: None,
            }
        }
        Err(e) => QwenRegisterReport {
            server: QWEN_MM_SEARCH_PLUGIN.to_string(),
            registered_tools: 0,
            skipped: Some(e),
        },
    }
}

/// 一键注册全部（core + search），永不 hard-fail：逐项报告。
pub fn register_qwen_mm_all(reg: &mut McpRegistry) -> Vec<QwenRegisterReport> {
    vec![register_qwen_mm_core(reg), register_qwen_mm_search(reg)]
}

/// 一键注册进全局表（`all_native_tools()` 的来源；`entry/interactive.rs` 调用）。
///
/// 语义与实例方法一致：`register_stdio_session_global` 内经 `try_lock`
/// 镜像写入全局；调用方若已持全局锁，镜像跳过——此时返回的 reports 里
/// `skipped == None` 但全局未写入，调用方应在释锁后重试（不丢注册，
/// manifest 数据在 reports 之外无状态）。
pub fn register_qwen_mm_all_global() -> Vec<QwenRegisterReport> {
    let mut reports = Vec::with_capacity(2);
    match resolve_core_launch() {
        Ok(launch) => {
            let defs = to_defs(
                &launch,
                qwen_mm_core_tools(),
                &["save_view", "crop", "draw_bbox"],
            );
            let args: Vec<&str> = launch.args.iter().map(|s| s.as_str()).collect();
            register_stdio_session_global(
                &launch.server_name,
                &launch.command,
                &args,
                CORE_TIMEOUT_MS,
                defs,
            );
            reports.push(QwenRegisterReport {
                server: launch.server_name,
                registered_tools: 7,
                skipped: None,
            });
        }
        Err(e) => reports.push(QwenRegisterReport {
            server: QWEN_MM_CORE_PLUGIN.to_string(),
            registered_tools: 0,
            skipped: Some(e),
        }),
    }
    match resolve_search_launch() {
        Ok(launch) => {
            if search_credential_status().iter().any(|(_, p)| *p) {
                let defs = to_defs(&launch, qwen_mm_search_tools(), &[]);
                let args: Vec<&str> = launch.args.iter().map(|s| s.as_str()).collect();
                register_stdio_session_global(
                    &launch.server_name,
                    &launch.command,
                    &args,
                    SEARCH_TIMEOUT_MS,
                    defs,
                );
                reports.push(QwenRegisterReport {
                    server: launch.server_name,
                    registered_tools: 3,
                    skipped: None,
                });
            } else {
                reports.push(QwenRegisterReport {
                    server: QWEN_MM_SEARCH_PLUGIN.to_string(),
                    registered_tools: 0,
                    skipped: Some(QwenResolveError {
                        server: QWEN_MM_SEARCH_PLUGIN.to_string(),
                        reason: "no search backend key (need one of SERPER_API_KEY, TAVILY_API_KEY, EXA_API_KEY, SERPLY_API_KEY)".to_string(),
                        hint: "export SERPER_API_KEY=... (or TAVILY/EXA/SERPLY) and retry".to_string(),
                    }),
                });
            }
        }
        Err(e) => reports.push(QwenRegisterReport {
            server: QWEN_MM_SEARCH_PLUGIN.to_string(),
            registered_tools: 0,
            skipped: Some(e),
        }),
    }
    reports
}

// ---------------------------------------------------------------------------
// 测试（纯结构断言，不依赖本机状态；环境相关用隔离 env 覆盖）
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_release_tag_format_matches_upstream() {
        assert_eq!(release_tag("core", "1.1.0"), "qwen-mm-plugins-core-v1.1.0");
        assert_eq!(
            release_tag("search", "1.1.0"),
            "qwen-mm-plugins-search-v1.1.0"
        );
    }

    #[test]
    fn test_core_manifest_seven_tools_unique() {
        let tools = qwen_mm_core_tools();
        assert_eq!(tools.len(), 7);
        let mut names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), 7);
        for want in [
            "read_image",
            "media_info",
            "read_video",
            "visualize",
            "save_view",
            "crop",
            "draw_bbox",
        ] {
            assert!(names.contains(&want), "missing {want}");
        }
    }

    #[test]
    fn test_core_schemas_require_path_fields() {
        for t in qwen_mm_core_tools() {
            let req = t
                .input_schema
                .get("required")
                .and_then(|v| v.as_array())
                .expect("each tool has required");
            assert!(!req.is_empty(), "{} must require ≥1 field", t.name);
            assert_eq!(
                t.input_schema.get("type").and_then(|v| v.as_str()),
                Some("object"),
                "{} schema must be object",
                t.name
            );
        }
    }

    #[test]
    fn test_search_manifest_three_tools() {
        let tools = qwen_mm_search_tools();
        assert_eq!(tools.len(), 3);
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"web_search"));
        assert!(names.contains(&"web_extractor"));
        assert!(names.contains(&"image_search"));
    }

    #[test]
    fn test_path_lookup_missing_binary_is_none() {
        assert!(path_lookup("definitely-not-a-real-binary-nt-xyz").is_none());
    }

    #[test]
    fn test_resolve_without_anything_reports_install_hint() {
        // 注入空环境 → 三路探测全灭，必须回可操作的 hint。零全局变更
        // （改 PATH 会和同进程并行测试竞态，见 `ResolveEnv` 文档）。
        let dir = std::env::temp_dir().join(format!(
            "nt_qwen_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).expect("test temp dir");
        let env = ResolveEnv {
            path_dirs: Some(vec![dir.clone()]),
            checkout: Some(None),
        };
        let e = resolve_launch_with(QWEN_MM_CORE_PLUGIN, "core", QWEN_MM_CORE_VERSION, &env)
            .expect_err("must fail with empty env");
        assert!(e.hint.contains("qwen-mm-plugins-core-v1.1.0"));
        assert!(e.to_string().contains("unavailable"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_resolve_source_checkout_layout() {
        // 伪造最小 checkout 树 → 必须走 SourceCheckout（同样零全局变更）。
        let base = std::env::temp_dir().join(format!(
            "nt_qwen_checkout_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let pkg = base
            .join("src")
            .join("capabilities")
            .join("core")
            .join("qwen_mm_plugins_core");
        std::fs::create_dir_all(&pkg).expect("fake checkout tree");
        std::fs::write(pkg.join("__main__.py"), "# fake\n").expect("fake main");
        let env = ResolveEnv {
            path_dirs: Some(vec![base.join("no-such-bin-dir")]),
            checkout: Some(Some(base.clone())),
        };
        let l = resolve_launch_with(QWEN_MM_CORE_PLUGIN, "core", QWEN_MM_CORE_VERSION, &env)
            .expect("checkout resolves");
        assert_eq!(l.via, LaunchVia::SourceCheckout);
        assert!(l.args.iter().any(|a| a.contains("qwen_mm_plugins_core")));
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn test_register_core_with_fake_launch_registers_seven() {
        let mut reg = McpRegistry::new();
        let launch = QwenLaunch {
            server_name: QWEN_MM_CORE_PLUGIN.to_string(),
            command: "true".to_string(),
            args: Vec::new(),
            via: LaunchVia::Path,
        };
        // 只注册、不执行：into NativeTool 是纯构造，不 spawn。
        let n = register_launch(
            &mut reg,
            &launch,
            1000,
            qwen_mm_core_tools(),
            &["save_view", "crop", "draw_bbox"],
        );
        assert_eq!(n, 7);
        assert_eq!(reg.tool_count(), 7);
        let natives = reg.as_native_tools();
        assert_eq!(natives.len(), 7);
        let mut ids: Vec<String> = natives.iter().map(|t| t.id().to_string()).collect();
        ids.sort();
        assert!(ids.contains(&"read_image".to_string()));
        assert!(ids.contains(&"media_info".to_string()));
        // 写文件工具标 Medium，只读标 Low。
        let risks: Vec<(String, RiskLevel)> = reg
            .recommend_tools("")
            .into_iter()
            .map(|d| (d.name.clone(), d.risk_level))
            .collect();
        let level = |name: &str| risks.iter().find(|(n, _)| n == name).map(|(_, r)| *r);
        assert_eq!(level("crop"), Some(RiskLevel::Medium));
        assert_eq!(level("read_image"), Some(RiskLevel::Low));
    }

    #[test]
    fn test_search_gated_without_keys() {
        // 存 → 清 → 验 → 恢复：并行 harness 下不污染他测试。
        let keys = [
            "SERPER_API_KEY",
            "TAVILY_API_KEY",
            "EXA_API_KEY",
            "SERPLY_API_KEY",
        ];
        let saved: Vec<(String, Option<std::ffi::OsString>)> = keys
            .iter()
            .map(|k| (k.to_string(), std::env::var_os(k)))
            .collect();
        for k in keys {
            std::env::remove_var(k);
        }
        let status = search_credential_status();
        assert_eq!(status.len(), 4);
        assert!(status.iter().all(|(_, present)| !present));
        let mut reg = McpRegistry::new();
        let rep = register_qwen_mm_search(&mut reg);
        assert_eq!(rep.registered_tools, 0);
        assert!(rep.skipped.is_some());
        for (k, v) in saved {
            if let Some(val) = v {
                std::env::set_var(k, val);
            }
        }
    }

    #[test]
    fn test_system_dep_status_shape() {
        // 只断言形状（label 是能力描述，不是二进制名；present 位随本机变，不测）。
        let deps = core_system_dep_status();
        let labels: Vec<&str> = deps.iter().map(|d| d.label.as_str()).collect();
        assert_eq!(
            labels,
            [
                "read_video / media_info (video & audio)",
                "visualize: Office / DrawIO (LibreOffice)",
                "visualize: LaTeX (.tex)",
                "visualize: 3D best-quality render (Blender)",
            ]
        );
    }

    /// 活测试（`#[ignore]`：只在显式调用时跑）：Rust 会话客户端直连真服务器。
    ///
    /// 前置（无 key 路径，全走 `scripts/ops/nt_qwen_mm_setup.sh setup`）：
    /// PATH 含 `qwen-mm-plugins-core` 垫片（或设 `QWEN_MM_PLUGINS_CHECKOUT`），
    /// 且 `ffmpeg` 在 PATH。跑法：
    /// `NT_QWEN_MM_LIVE=1 cargo test -p neotrix --lib -- --ignored live_qwen_core_keyless --nocapture`
    /// 缺前置即 loud-fail（opt-in 测试不静默过，这是故意的）。
    #[test]
    #[ignore]
    fn live_qwen_core_keyless() {
        use crate::nt_mcp_stdio_session::McpStdioSession;

        assert_eq!(
            std::env::var("NT_QWEN_MM_LIVE").as_deref(),
            Ok("1"),
            "live test needs NT_QWEN_MM_LIVE=1 (opt-in)"
        );
        let launch = resolve_core_launch().expect(
            "qwen-mm core not provisioned: run `bash scripts/ops/nt_qwen_mm_setup.sh setup` \
             and `export PATH=\"$HOME/.neotrix/qwen-mm/bin:$PATH\"`",
        );
        eprintln!(
            "live via {:?}: {} {:?}",
            launch.via, launch.command, launch.args
        );
        let session = McpStdioSession::new(launch.command.clone(), launch.args.clone())
            .with_timeout_ms(CORE_TIMEOUT_MS);

        // 1) 清单：恰好 7 工具（与 manifest 逐名一致，E2E-3 的 Rust 版）。
        let listed = session.list_tools().expect("tools/list");
        let mut names: Vec<&str> = listed.iter().map(|t| t.name.as_str()).collect();
        names.sort_unstable();
        assert_eq!(
            names,
            [
                "crop",
                "draw_bbox",
                "media_info",
                "read_image",
                "read_video",
                "save_view",
                "visualize"
            ]
        );

        // 2) 夹具：ffmpeg 现场造 png＋mp4（SYSTEM_DEPS 声明的依赖，缺即 loud-fail）。
        let dir = std::env::temp_dir().join(format!("nt_live_{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("fixture dir");
        let png = dir.join("t.png");
        let mp4 = dir.join("t.mp4");
        // 输出路径必须拼在 ffmpeg 参数最后。
        let mk = |args: &[&str], out: &std::path::Path| -> Vec<String> {
            let mut v: Vec<String> = args.iter().map(|s| s.to_string()).collect();
            v.push(out.to_string_lossy().to_string());
            v
        };
        let run2 = |args: &[&str], out: &std::path::Path| {
            let full = mk(args, out);
            let st = std::process::Command::new("ffmpeg")
                .args(&full)
                .output()
                .expect("live test needs ffmpeg on PATH (see SYSTEM_DEPS)");
            assert!(st.status.success(), "ffmpeg fixture failed");
        };
        run2(
            &[
                "-y",
                "-v",
                "error",
                "-f",
                "lavfi",
                "-i",
                "testsrc=duration=1:size=160x120:rate=5",
                "-frames:v",
                "1",
            ],
            &png,
        );
        run2(
            &[
                "-y",
                "-v",
                "error",
                "-f",
                "lavfi",
                "-i",
                "testsrc=duration=2:size=160x120:rate=5",
                "-pix_fmt",
                "yuv420p",
            ],
            &mp4,
        );

        // 3) media_info → 真元数据。
        let mi = session
            .call_tool("media_info", &serde_json::json!({"path": mp4}))
            .expect("media_info");
        assert!(!mi.is_error, "media_info error: {}", mi.text);
        assert!(mi.text.contains("Video stream"), "unexpected: {}", mi.text);
        eprintln!(
            "media_info:\n{}",
            mi.text.lines().take(4).collect::<Vec<_>>().join("\n")
        );

        // 4) read_image → 真落盘（image 块变文件，文本只留路径）。
        let ri = session
            .call_tool(
                "read_image",
                &serde_json::json!({"image_path": png, "budget": "small"}),
            )
            .expect("read_image");
        assert!(!ri.is_error, "read_image error: {}", ri.text);
        assert_eq!(ri.saved_images.len(), 1);
        let img_bytes = std::fs::read(&ri.saved_images[0]).expect("saved image readable");
        assert!(!img_bytes.is_empty());
        eprintln!("read_image saved: {}", ri.saved_images[0].display());

        // 5) read_video → 真帧。
        let rv = session
            .call_tool(
                "read_video",
                &serde_json::json!({"video_path": mp4, "fps": 1, "budget": "small"}),
            )
            .expect("read_video");
        assert!(!rv.is_error, "read_video error: {}", rv.text);
        assert!(
            rv.text.contains("frames @"),
            "unexpected: {}",
            rv.text.lines().next().unwrap_or("")
        );

        let _ = std::fs::remove_dir_all(&dir);
        eprintln!("live_qwen_core_keyless: ALL GREEN (no keys used)");
    }
}
